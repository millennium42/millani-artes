"""Pinned Gitleaks guard for the existing Windows job; stdlib only."""
from contextlib import contextmanager
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import time
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parent.parent
VERSION = "8.30.1"
ZIP_NAME = f"gitleaks_{VERSION}_windows_x64.zip"
ZIP_SIZE = 8438883
ZIP_SHA256 = "d29144deff3a68aa93ced33dddf84b7fdc26070add4aa0f4513094c8332afc4e"
EXE_SHA256 = "17157e2ee8b76fc8b1d8bee607a250e34b8a8023c8bc81822d4b5ee4d78fcb7c"
LICENSE_SHA256 = "e3884b252b3bfc045e55be43a34d1e80da070bc6f804ac95bf4660e97d62ebc6"
CACHE = ROOT / "artifacts/tools/gitleaks" / VERSION
SCRATCH = ROOT / "artifacts/sec007-check"


class GateError(Exception):
    def __init__(self, code, findings=0):
        super().__init__(code)
        self.code = code
        self.findings = findings


def safe_path(path):
    for parent in [path, *path.parents]:
        if parent.exists() and (parent.is_symlink() or parent.is_junction()):
            raise GateError("GITLEAKS_REPARSE_DENIED")


def process_env():
    return {k: v for k, v in os.environ.items()
            if not k.upper().startswith(("GIT_", "GITLEAKS_"))
            and k.upper() not in {"GH_TOKEN", "GITHUB_TOKEN"}}


def invoke(args, cwd=ROOT):
    try:
        return subprocess.run(args, cwd=cwd, env=process_env(), capture_output=True, timeout=150)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise GateError("GITLEAKS_PROCESS_FAILED") from error


def validate_files(files):
    if set(files) != {"gitleaks.exe", "LICENSE", "README.md"}:
        raise GateError("GITLEAKS_PACKAGE_INVALID")
    exe = files["gitleaks.exe"]
    if len(exe) != 22575104 or hashlib.sha256(exe).hexdigest() != EXE_SHA256:
        raise GateError("GITLEAKS_BINARY_INVALID")
    if hashlib.sha256(files["LICENSE"]).hexdigest() != LICENSE_SHA256:
        raise GateError("GITLEAKS_LICENSE_INVALID")
    offset = struct.unpack_from("<I", exe, 0x3C)[0]
    if (exe[:2] != b"MZ" or exe[offset:offset + 4] != b"PE\0\0"
            or struct.unpack_from("<H", exe, offset + 4)[0] != 0x8664
            or struct.unpack_from("<H", exe, offset + 24)[0] != 0x20B):
        raise GateError("GITLEAKS_BINARY_INVALID")


def unpack(payload):
    if len(payload) != ZIP_SIZE or hashlib.sha256(payload).hexdigest() != ZIP_SHA256:
        raise GateError("GITLEAKS_ARCHIVE_INVALID")
    with zipfile.ZipFile(io.BytesIO(payload)) as archive:
        members = archive.infolist()
        if (len(members) != 3 or {m.filename for m in members} != {"gitleaks.exe", "LICENSE", "README.md"}
                or any(m.is_dir() or m.file_size > 64000000 for m in members)):
            raise GateError("GITLEAKS_PACKAGE_INVALID")
        files = {m.filename: archive.read(m) for m in members}
    validate_files(files)
    return files


def scanner():
    safe_path(CACHE)
    if not CACHE.exists():
        url = f"https://github.com/gitleaks/gitleaks/releases/download/v{VERSION}/{ZIP_NAME}"
        request = urllib.request.Request(url, headers={"User-Agent": "MillaniArtes-Gitleaks"})
        with urllib.request.urlopen(request, timeout=30) as response:
            files = unpack(response.read(ZIP_SIZE + 1))
        CACHE.mkdir(parents=True, exist_ok=False)
        for name, content in files.items():
            (CACHE / name).write_bytes(content)
    paths = list(CACHE.iterdir())
    for path in paths:
        safe_path(path)
    expected = {"gitleaks.exe", "LICENSE", "README.md"}
    names = {path.name for path in paths}
    if names not in (expected, expected | {ZIP_NAME}) or any(not path.is_file() for path in paths):
        raise GateError("GITLEAKS_PACKAGE_INVALID")
    if ZIP_NAME in names:
        # SEC-006 retained the official ZIP; permit it only after the pinned integrity check.
        unpack((CACHE / ZIP_NAME).read_bytes())
    validate_files({path.name: path.read_bytes() for path in paths if path.name in expected})
    exe = CACHE / "gitleaks.exe"
    version = invoke([str(exe), "version"])
    if version.returncode != 0 or version.stdout.strip() != VERSION.encode():
        raise GateError("GITLEAKS_VERSION_INVALID")
    return exe


@contextmanager
def scratch():
    safe_path(SCRATCH)
    if SCRATCH.exists():
        raise GateError("GITLEAKS_SCRATCH_EXISTS")
    SCRATCH.mkdir(parents=True)
    try:
        yield SCRATCH
    finally:
        if not SCRATCH.resolve().is_relative_to(ROOT / "artifacts") or SCRATCH.name != "sec007-check":
            raise GateError("GITLEAKS_CLEANUP_DENIED")
        safe_path(SCRATCH)
        for path in SCRATCH.rglob("*"):
            safe_path(path)
        shutil.rmtree(SCRATCH)


def git_value(source, *args):
    result = invoke(["git", "-C", str(source), *args])
    if result.returncode != 0:
        raise GateError("GITLEAKS_SOURCE_INVALID")
    return result.stdout.decode("utf-8").strip()


def source_state(source):
    safe_path(source)
    source = source.resolve()
    if not source.is_relative_to(ROOT) or not (source / ".git").exists():
        raise GateError("GITLEAKS_SOURCE_INVALID")
    if Path(git_value(source, "rev-parse", "--show-toplevel")).resolve() != source:
        raise GateError("GITLEAKS_SOURCE_INVALID")
    if git_value(source, "rev-parse", "--is-shallow-repository") != "false":
        raise GateError("GITLEAKS_HISTORY_INCOMPLETE")
    if (source / ".gitleaksignore").exists():
        # Upstream also loads this file independently of --gitleaks-ignore-path.
        raise GateError("GITLEAKS_IGNORE_DENIED")
    commits = int(git_value(source, "rev-list", "--count", "--all"))
    if commits < 1:
        raise GateError("GITLEAKS_HISTORY_EMPTY")
    return source, commits, git_value(source, "rev-parse", "HEAD")


def check_report(result):
    if result.returncode not in {0, 1}:
        raise GateError("GITLEAKS_SCAN_FAILED")
    try:
        report = json.loads(result.stdout)
    except (ValueError, UnicodeError) as error:
        raise GateError("GITLEAKS_REPORT_INVALID") from error
    if not isinstance(report, list) or any(not isinstance(f, dict) or f.get("Secret") != "REDACTED" for f in report):
        raise GateError("GITLEAKS_REPORT_INVALID")
    if result.returncode == 1 and report:
        raise GateError("GITLEAKS_FINDINGS", len(report))
    if result.returncode != 0 or report:
        raise GateError("GITLEAKS_SCAN_FAILED")


def scan(source, exe):
    source, commits, sha = source_state(source)
    with scratch() as directory:
        config = directory / "default.toml"
        config.write_text("[extend]\nuseDefault = true\n", encoding="utf-8")
        ignore = directory / "empty.ignore"
        ignore.write_bytes(b"")
        start = time.perf_counter()
        result = invoke([str(exe), "git", str(source), "--log-opts=--all",
                         "--config", str(config), "--gitleaks-ignore-path", str(ignore),
                         "--ignore-gitleaks-allow", "--redact=100", "--no-banner", "--no-color",
                         "--exit-code", "1", "--timeout", "120", "--report-format", "json", "--report-path", "-"])
        check_report(result)
        seconds = round(time.perf_counter() - start, 3)
    # ponytail: rule-based scan of available refs; no claim about unreachable refs or all possible secrets.
    return {"version": VERSION, "binarySHA256": EXE_SHA256, "sourceSha": sha,
            "commits": commits, "findings": 0, "seconds": seconds}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, default=ROOT)
    args = parser.parse_args()
    try:
        if sys.platform != "win32" or sys.version_info < (3, 12):
            raise GateError("GITLEAKS_PLATFORM_UNSUPPORTED")
        evidence = scan(args.source.absolute(), scanner())
        print("GITLEAKS_EVIDENCE " + json.dumps(evidence, sort_keys=True))
        return 0
    except GateError as error:
        print(json.dumps({"status": "fail", "code": error.code, "findings": error.findings}), file=sys.stderr)
        return 1
    except Exception:
        # The tool boundary must not echo raw process/config/report exceptions.
        print(json.dumps({"status": "fail", "code": "GITLEAKS_INPUT_OR_INSTALL_FAILED"}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
