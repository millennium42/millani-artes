"""Pinned OSV lockfile gate for the existing Windows job; stdlib only."""
from contextlib import contextmanager
import argparse
from datetime import datetime, timezone, date
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import time
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parent.parent
VERSION = "2.6.0"
CACHE = ROOT / "artifacts/tools/osv-scanner" / VERSION
SCRATCH = ROOT / "artifacts/sec009-check"
EXE_NAME = "osv-scanner_windows_amd64.exe"
SOURCE_COMMIT = "e840a6e8adb14b7777c78e26cfbf6e2abc1d1fc6"
PINS = {
    EXE_NAME: (58692096, "e0ed7644118b717b028c249ee9d3515024e55e8510747ca08906eb96765354d6"),
    "LICENSE": (11358, "cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30"),
    "osv-scanner_SHA256SUMS": (554, "29f6fbc8bdd02d977df4b0987705d046233c36b70469a7021c623e8c155c9ddc"),
}
LOCKS = ("package-lock.json", "src-tauri/Cargo.lock")
EXCEPTIONS = ROOT / "docs/security/OSV_WINDOWS_EXCEPTION.json"
APPROVAL = ROOT / "docs/security/OSV_WINDOWS_APPROVAL.json"


class GateError(Exception):
    def __init__(self, code, findings=0, ids=()):
        super().__init__(code)
        self.code, self.findings, self.ids = code, findings, sorted(set(ids))[:20]


def require(condition, code):
    if not condition:
        raise GateError(code)


def safe_path(path):
    for parent in [path, *path.parents]:
        require(not parent.is_symlink() and not parent.is_junction(), "OSV_REPARSE_DENIED")


def process_env():
    return {k: v for k, v in os.environ.items()
            if not k.upper().startswith(("GIT_", "OSV_", "GITLEAKS_"))
            and k.upper() not in {"GH_TOKEN", "GITHUB_TOKEN"}}


def invoke(args, cwd=ROOT):
    try:
        return subprocess.run(args, cwd=cwd, env=process_env(), capture_output=True, timeout=150)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise GateError("OSV_PROCESS_FAILED") from error


def validate_files(files):
    require(set(files) == set(PINS), "OSV_CACHE_INVALID")
    for name, (size, digest) in PINS.items():
        require(len(files[name]) == size and hashlib.sha256(files[name]).hexdigest() == digest,
                "OSV_INTEGRITY_INVALID")
    exe = files[EXE_NAME]
    offset = struct.unpack_from("<I", exe, 0x3c)[0]
    require(exe[:2] == b"MZ" and exe[offset:offset + 4] == b"PE\0\0"
            and struct.unpack_from("<H", exe, offset + 4)[0] == 0x8664
            and struct.unpack_from("<H", exe, offset + 24)[0] == 0x20b, "OSV_BINARY_INVALID")
    require(f"{PINS[EXE_NAME][1]}  {EXE_NAME}" in files["osv-scanner_SHA256SUMS"].decode().splitlines(),
            "OSV_CHECKSUM_INVALID")


def scanner():
    safe_path(CACHE)
    if not CACHE.exists():
        files = {}
        for name, (size, _) in PINS.items():
            url = (f"https://raw.githubusercontent.com/google/osv-scanner/{SOURCE_COMMIT}/LICENSE"
                   if name == "LICENSE" else f"https://github.com/google/osv-scanner/releases/download/v{VERSION}/{name}")
            request = urllib.request.Request(url, headers={"User-Agent": "MillaniArtes-OSV"})
            with urllib.request.urlopen(request, timeout=30) as response:
                files[name] = response.read(size + 1)
        validate_files(files)
        CACHE.mkdir(parents=True, exist_ok=False)
        for name, content in files.items():
            (CACHE / name).write_bytes(content)
    validate_cache()
    exe = CACHE / EXE_NAME
    result = invoke([str(exe), "--version"])
    lines = [line for line in result.stdout.splitlines() if line.startswith(b"osv-scanner version:")]
    require(result.returncode == 0 and lines == [b"osv-scanner version: 2.6.0"], "OSV_VERSION_INVALID")
    return exe


def validate_cache():
    safe_path(CACHE)
    require(CACHE.is_dir(), "OSV_CACHE_INVALID")
    paths = list(CACHE.iterdir())
    for path in paths:
        safe_path(path)
    require({p.name for p in paths} == set(PINS) and all(p.is_file() for p in paths), "OSV_CACHE_INVALID")
    validate_files({p.name: p.read_bytes() for p in paths})
def object_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "OSV_DUPLICATE_KEY")
        result[key] = value
    return result


def package_key(package):
    require(isinstance(package, dict), "OSV_PACKAGE_INVALID")
    key = tuple(package.get(k) for k in ("ecosystem", "name", "version"))
    require(all(isinstance(v, str) and v for v in key), "OSV_PACKAGE_INVALID")
    return key


def inputs(source):
    safe_path(source)
    source = source.resolve()
    require(source.is_relative_to(ROOT), "OSV_SOURCE_INVALID")
    blobs, expected = {}, {}
    for name in LOCKS:
        path = source / name
        safe_path(path)
        require(path.is_file() and 0 < path.stat().st_size <= 4000000, "OSV_LOCK_MISSING_OR_INVALID")
        require(not (path.parent / "osv-scanner.toml").exists(), "OSV_SOURCE_CONFIG_DENIED")
        blobs[name] = path.read_bytes()
    npm = json.loads(blobs[LOCKS[0]], object_pairs_hook=object_pairs)
    require(isinstance(npm, dict) and npm.get("lockfileVersion") == 3
            and isinstance(npm.get("packages"), dict) and "" in npm["packages"], "OSV_NPM_INVALID")
    packages = set()
    for path, value in npm["packages"].items():
        if not path:
            continue
        require(path.startswith("node_modules/") and isinstance(value, dict) and not value.get("link"),
                "OSV_NPM_INVALID")
        name = value.get("name", path.rsplit("node_modules/", 1)[-1])
        packages.add(package_key({"ecosystem": "npm", "name": name, "version": value.get("version")}))
    require(packages, "OSV_NPM_EMPTY")
    expected[LOCKS[0]] = packages
    cargo = tomllib.loads(blobs[LOCKS[1]].decode("utf-8"))
    require(cargo.get("version") == 4 and isinstance(cargo.get("package"), list), "OSV_CARGO_INVALID")
    packages = set()
    for value in cargo["package"]:
        require(isinstance(value, dict), "OSV_CARGO_INVALID")
        if "source" not in value:
            require(value.get("name") == npm["packages"][""].get("name")
                    and value.get("version") == npm["packages"][""].get("version"), "OSV_LOCAL_PACKAGE_UNSUPPORTED")
            packages.add(package_key({"ecosystem": "crates.io", "name": value.get("name"), "version": value.get("version")}))
            continue
        require(value["source"] == "registry+https://github.com/rust-lang/crates.io-index", "OSV_CARGO_SOURCE_UNSUPPORTED")
        packages.add(package_key({"ecosystem": "crates.io", "name": value.get("name"), "version": value.get("version")}))
    require(packages, "OSV_CARGO_EMPTY")
    expected[LOCKS[1]] = packages
    return source, blobs, expected


@contextmanager
def scratch():
    safe_path(SCRATCH)
    require(not SCRATCH.exists(), "OSV_SCRATCH_EXISTS")
    SCRATCH.mkdir(parents=True)
    try:
        yield SCRATCH
    finally:
        require(SCRATCH.resolve().is_relative_to(ROOT / "artifacts") and SCRATCH.name == "sec009-check",
                "OSV_CLEANUP_DENIED")
        for path in [SCRATCH, *SCRATCH.rglob("*")]:
            safe_path(path)
        shutil.rmtree(SCRATCH)


def approval_digest(config):
    scope = {key: config[key] for key in ("target", "expiresBeforeUTC", "inputSHA256", "exceptions")}
    return hashlib.sha256(json.dumps(scope, sort_keys=True, ensure_ascii=False,
                                     separators=(",", ":")).encode("utf-8")).hexdigest()


def load_exceptions(blobs):
    safe_path(EXCEPTIONS)
    config = json.loads(EXCEPTIONS.read_bytes(), object_pairs_hook=object_pairs)
    require(isinstance(config, dict) and type(config.get("approved")) is bool, "OSV_EXCEPTION_INVALID")
    if not config["approved"]:
        return set(), None
    require(config.get("target") == "x86_64-pc-windows-msvc"
            and isinstance(config.get("approvalRecord"), str)
            and re.fullmatch(r"sha256:[0-9a-f]{64}", config["approvalRecord"]),
            "OSV_EXCEPTION_UNAPPROVED")
    safe_path(APPROVAL)
    require(APPROVAL.is_file() and 0 < APPROVAL.stat().st_size <= 16384, "OSV_EXCEPTION_APPROVAL_INVALID")
    record_bytes = APPROVAL.read_bytes()
    require("sha256:" + hashlib.sha256(record_bytes).hexdigest() == config["approvalRecord"],
            "OSV_EXCEPTION_RECORD_CHANGED")
    record = json.loads(record_bytes, object_pairs_hook=object_pairs)
    require(isinstance(record, dict)
            and set(record) == {"approvedScopeSHA256", "humanDecision", "decisionReference"},
            "OSV_EXCEPTION_APPROVAL_INVALID")
    require(all(isinstance(value, str) and value.strip() for value in record.values()),
            "OSV_EXCEPTION_UNAPPROVED")
    require(record["approvedScopeSHA256"] == approval_digest(config), "OSV_EXCEPTION_APPROVAL_MISMATCH")
    expires = date.fromisoformat(config["expiresBeforeUTC"])
    require(datetime.now(timezone.utc).date() < expires, "OSV_EXCEPTION_EXPIRED")
    require(set(config["inputSHA256"]) == {"src-tauri/Cargo.lock", "src-tauri/Cargo.toml"}, "OSV_EXCEPTION_SCOPE_INVALID")
    for name, digest in config["inputSHA256"].items():
        path = ROOT / name
        safe_path(path)
        content = blobs[name] if name in blobs else path.read_bytes()
        require(hashlib.sha256(content).hexdigest() == digest, "OSV_EXCEPTION_SCOPE_CHANGED")
    require(isinstance(config.get("exceptions"), list) and config["exceptions"], "OSV_EXCEPTION_INVALID")
    allowed = set()
    for entry in config["exceptions"]:
        key = package_key(entry)
        require(key[0] == "crates.io" and isinstance(entry.get("ids"), list) and entry["ids"], "OSV_EXCEPTION_INVALID")
        for id in entry["ids"]:
            require(isinstance(id, str) and re.fullmatch(r"[A-Za-z0-9_-]{1,100}", id), "OSV_EXCEPTION_INVALID")
            allowed.add((*key, id))
    return allowed, expires.isoformat()


def report_path(path):
    require(isinstance(path, str) and Path(path).is_absolute(), "OSV_REPORT_SOURCE_INVALID")
    # Compare report paths lexically: an untrusted report must not trigger filesystem/UNC traversal.
    return os.path.normcase(os.path.normpath(path))


def check_report(result, expected, allowed=()):
    require(result.returncode in {0, 1}, "OSV_SCAN_FAILED")
    require(len(result.stdout) <= 16000000, "OSV_REPORT_INVALID")
    try:
        report = json.loads(result.stdout, object_pairs_hook=object_pairs)
    except (ValueError, UnicodeError) as error:
        raise GateError("OSV_REPORT_INVALID") from error
    require(isinstance(report, dict) and isinstance(report.get("results"), list), "OSV_REPORT_INVALID")
    require(not report.get("experimental_generic_findings") and not report.get("image_metadata"), "OSV_UNEXPECTED_FINDING")
    seen, ids, findings, excepted = set(), set(), 0, 0
    for entry in report["results"]:
        require(isinstance(entry, dict) and isinstance(entry.get("source"), dict), "OSV_REPORT_INVALID")
        path = report_path(entry["source"].get("path"))
        require(entry["source"].get("type") == "lockfile" and path in expected and path not in seen,
                "OSV_REPORT_SOURCE_INVALID")
        require(not entry.get("experimental_pes"), "OSV_REPORT_SUPPRESSION_DENIED")
        seen.add(path)
        require(isinstance(entry.get("packages"), list), "OSV_REPORT_INVALID")
        packages = set()
        for item in entry["packages"]:
            require(isinstance(item, dict), "OSV_REPORT_INVALID")
            key = package_key(item.get("package"))
            packages.add(key)
            vulnerabilities, groups = item.get("vulnerabilities", []), item.get("groups", [])
            require(isinstance(vulnerabilities, list) and isinstance(groups, list), "OSV_REPORT_INVALID")
            require(not item.get("license_violations"), "OSV_UNEXPECTED_FINDING")
            require(not groups or vulnerabilities, "OSV_REPORT_INVALID")
            for vuln in vulnerabilities:
                require(isinstance(vuln, dict) and isinstance(vuln.get("id"), str), "OSV_REPORT_INVALID")
                if (*key, vuln["id"]) in allowed:
                    excepted += 1
                    continue
                findings += 1
                ids.add(vuln["id"] if re.fullmatch(r"[A-Za-z0-9_-]{1,100}", vuln["id"]) else "UNRECOGNIZED-ID")
        require(packages == expected[path], "OSV_INVENTORY_INCOMPLETE")
    require(seen == set(expected), "OSV_INVENTORY_INCOMPLETE")
    if findings:
        raise GateError("OSV_FINDINGS", findings, ids)
    require(result.returncode == 0 or excepted > 0, "OSV_SCAN_FAILED")
    return {"packages": sum(len(v) for v in expected.values()), "findings": excepted, "excepted": excepted, "unexcepted": 0}


def scan(source, exe):
    source, blobs, inventory = inputs(source)
    allowed, expiry = load_exceptions(blobs)
    start = time.perf_counter()
    with scratch() as directory:
        expected = {}
        for name, content in blobs.items():
            path = directory / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
            expected[report_path(str(path))] = inventory[name]
        config = directory / "empty.toml"
        config.write_bytes(b"")
        args = [str(exe), "scan", "source", "--format=json", "--all-packages", "--no-ignore",
                "--no-resolve", "--no-call-analysis=all", "--config", str(config)]
        for name in LOCKS:
            args.extend(["--lockfile", ":" + str(directory / name)])
        summary = check_report(invoke(args, directory), expected, allowed)
        validate_cache()
    require(all((source / name).read_bytes() == content for name, content in blobs.items()), "OSV_INPUT_CHANGED")
    sha = invoke(["git", "-C", str(ROOT), "rev-parse", "HEAD"])
    require(sha.returncode == 0, "OSV_SOURCE_INVALID")
    # ponytail: only pinned lockfile inventory and current OSV data; no proof of exploitability or all future vulnerabilities.
    return {"version": VERSION, "binarySHA256": PINS[EXE_NAME][1], "sourceSha": sha.stdout.decode().strip(),
            "lockfiles": {name: len(inventory[name]) for name in LOCKS}, **summary, "exceptionExpiry": expiry,
            "lockSHA256": {name: hashlib.sha256(blobs[name]).hexdigest() for name in LOCKS},
            "seconds": round(time.perf_counter() - start, 3)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, default=ROOT)
    args = parser.parse_args()
    try:
        require(sys.platform == "win32" and sys.version_info >= (3, 12), "OSV_PLATFORM_UNSUPPORTED")
        evidence = scan(args.source.absolute(), scanner())
        print("OSV_EVIDENCE " + json.dumps(evidence, sort_keys=True))
        return 0
    except GateError as error:
        print(json.dumps({"status": "fail", "code": error.code, "findings": error.findings, "ids": error.ids}), file=sys.stderr)
        return 1
    except Exception:
        print(json.dumps({"status": "fail", "code": "OSV_INPUT_OR_INSTALL_FAILED"}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
