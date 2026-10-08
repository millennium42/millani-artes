"""Pinned local Cargo audit; all-platform lock and explicit warning policy."""
from contextlib import contextmanager
from datetime import datetime, timezone, date
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import tomllib

ROOT = Path(__file__).resolve().parent.parent
SCRATCH = ROOT / "artifacts/sec011-check"
EXE = Path(os.environ["LOCALAPPDATA"]) / "MillaniArtesDev/toolchains/cargo-audit-0.22.2-win-x64/cargo-audit-x86_64-pc-windows-msvc-v0.22.2/cargo-audit.exe"
BIN_SHA = "0157f5ce1ce9fd4fb0a1f7c79af1229771d1f80b6c2613ddb0d9200a8ba73946"
DB = Path.home() / ".cargo/advisory-db"
DB_URL = "https://github.com/RustSec/advisory-db.git"
EXCEPTIONS = ROOT / "docs/security/CARGO_WINDOWS_EXCEPTION.json"
APPROVAL = ROOT / "docs/security/CARGO_WINDOWS_APPROVAL.json"
APPROVAL_TEXT = "Aprovo a exceção SEC-011 no escopo e prazo apresentados"
REFERENCE_RE = r"Codex user reply to SEC-011 proposal [0-9a-f]{40}"
KINDS = {"unmaintained", "unsound", "notice", "yanked"}
CONFIG = '''[advisories]
ignore=[]
informational_warnings=["unmaintained","unsound","notice"]
[database]
fetch=true
stale=false
[output]
deny=["warnings"]
format="json"
quiet=false
show_tree=false
[yanked]
enabled=true
'''


class GateError(Exception):
    def __init__(self, code, ids=()):
        super().__init__(code)
        self.ids = sorted(set(ids))[:20]


def require(condition, code):
    if not condition:
        raise GateError(code)


def safe_path(path):
    for parent in [path, *path.parents]:
        require(not parent.is_symlink() and not parent.is_junction(), "CARGO_REPARSE_DENIED")


def object_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "CARGO_DUPLICATE_KEY")
        result[key] = value
    return result


def read_json(blob):
    try:
        return json.loads(blob, object_pairs_hook=object_pairs,
                          parse_constant=lambda _: require(False, "CARGO_JSON_INVALID"))
    except (ValueError, UnicodeError) as error:
        raise GateError("CARGO_JSON_INVALID") from error


def process_env():
    return {k: v for k, v in os.environ.items()
            if not k.upper().startswith(("CARGO_", "RUST_", "RUSTUP_", "GIT_", "OSV_", "GITLEAKS_"))
            and k.upper() not in {"GH_TOKEN", "GITHUB_TOKEN", "NPM_TOKEN", "NODE_AUTH_TOKEN"}}


def invoke(args, cwd=ROOT):
    try:
        return subprocess.run(args, cwd=cwd, env=process_env(), capture_output=True, timeout=150)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise GateError("CARGO_PROCESS_FAILED") from error


def scanner():
    safe_path(EXE)
    require(EXE.is_file() and EXE.stat().st_size == 15310848
            and hashlib.sha256(EXE.read_bytes()).hexdigest() == BIN_SHA, "CARGO_BINARY_INVALID")
    result = invoke([str(EXE), "--version"])
    require(result.returncode == 0 and result.stdout.strip() == b"cargo-audit 0.22.2", "CARGO_VERSION_INVALID")
    return EXE


def package_key(value):
    require(isinstance(value, dict), "CARGO_PACKAGE_INVALID")
    key = tuple(value.get(k) for k in ("name", "version", "source"))
    require(isinstance(key[0], str) and re.fullmatch(r"[A-Za-z0-9_-]+", key[0])
            and isinstance(key[1], str) and key[1] and key[2] in
            (None, "registry+https://github.com/rust-lang/crates.io-index"), "CARGO_PACKAGE_INVALID")
    return key


def inputs(source):
    safe_path(source)
    require(source.resolve().is_relative_to(ROOT), "CARGO_SOURCE_DENIED")
    blobs = {}
    for name in ("src-tauri/Cargo.lock", "src-tauri/Cargo.toml"):
        path = source / name
        safe_path(path)
        require(path.is_file() and 0 < path.stat().st_size <= 4000000, "CARGO_INPUT_INVALID")
        blobs[name] = path.read_bytes()
    lock, manifest = (tomllib.loads(blob.decode("utf-8")) for blob in blobs.values())
    require(lock.get("version") == 4 and isinstance(lock.get("package"), list)
            and lock["package"] and isinstance(manifest.get("package"), dict), "CARGO_INPUT_INVALID")
    packages = {package_key(p) for p in lock["package"]}
    require(len(packages) == len(lock["package"]), "CARGO_INPUT_INVALID")
    for name, version, source_kind in packages:
        if source_kind is None:
            require(name == manifest["package"].get("name") and version == manifest["package"].get("version"),
                    "CARGO_LOCAL_PACKAGE_UNSUPPORTED")
    return blobs, packages


@contextmanager
def scratch():
    safe_path(SCRATCH)
    require(not SCRATCH.exists(), "CARGO_SCRATCH_EXISTS")
    SCRATCH.mkdir(parents=True)
    try:
        yield SCRATCH
    finally:
        require(SCRATCH.resolve().is_relative_to(ROOT / "artifacts") and SCRATCH.name == "sec011-check",
                "CARGO_CLEANUP_DENIED")
        for path in [SCRATCH, *SCRATCH.rglob("*")]:
            safe_path(path)
        shutil.rmtree(SCRATCH)


def scope_digest(config):
    scope = {k: config[k] for k in ("gate", "target", "expiresBeforeUTC", "inputSHA256", "exceptions")}
    return hashlib.sha256(json.dumps(scope, sort_keys=True, ensure_ascii=False,
                                     separators=(",", ":")).encode()).hexdigest()


def load_exceptions(blobs):
    safe_path(EXCEPTIONS)
    config = read_json(EXCEPTIONS.read_bytes())
    require(isinstance(config, dict) and type(config.get("approved")) is bool, "CARGO_EXCEPTION_INVALID")
    if not config["approved"]:
        return set(), None
    require(set(config) == {"approved", "approvalRecord", "gate", "target", "expiresBeforeUTC",
                            "inputSHA256", "exceptions"}
            and config["gate"] == "SEC-011/cargo-audit/0.22.2"
            and config["target"] == "x86_64-pc-windows-msvc"
            and isinstance(config["approvalRecord"], str)
            and re.fullmatch(r"sha256:[0-9a-f]{64}", config["approvalRecord"]), "CARGO_EXCEPTION_UNAPPROVED")
    safe_path(APPROVAL)
    require(APPROVAL.is_file() and 0 < APPROVAL.stat().st_size <= 16384, "CARGO_APPROVAL_INVALID")
    content = APPROVAL.read_bytes()
    require("sha256:" + hashlib.sha256(content).hexdigest() == config["approvalRecord"],
            "CARGO_APPROVAL_RECORD_CHANGED")
    record = read_json(content)
    require(isinstance(record, dict)
            and set(record) == {"approvedScopeSHA256", "humanDecision", "decisionReference"}
            and all(isinstance(v, str) and v.strip() for v in record.values())
            and record["humanDecision"] == APPROVAL_TEXT
            and re.fullmatch(REFERENCE_RE, record["decisionReference"]), "CARGO_APPROVAL_INVALID")
    require(record["approvedScopeSHA256"] == scope_digest(config), "CARGO_APPROVAL_SCOPE_CHANGED")
    require(re.fullmatch(r"\d{4}-\d{2}-\d{2}", config["expiresBeforeUTC"])
            and datetime.now(timezone.utc).date() < date.fromisoformat(config["expiresBeforeUTC"]),
            "CARGO_EXCEPTION_EXPIRED")
    require(config["inputSHA256"] == {name: hashlib.sha256(blob).hexdigest() for name, blob in blobs.items()},
            "CARGO_EXCEPTION_INPUT_CHANGED")
    require(isinstance(config["exceptions"], list) and config["exceptions"], "CARGO_EXCEPTION_INVALID")
    allowed = set()
    for entry in config["exceptions"]:
        require(isinstance(entry, dict) and set(entry) == {"name", "version", "kind", "id"}
                and entry["kind"] in {"unmaintained", "unsound", "notice"}
                and isinstance(entry["id"], str) and re.fullmatch(r"RUSTSEC-\d{4}-\d{4}", entry["id"]),
                "CARGO_EXCEPTION_INVALID")
        key = (*package_key(entry | {"source": "registry+https://github.com/rust-lang/crates.io-index"}),
               entry["kind"], entry["id"])
        require(key not in allowed, "CARGO_EXCEPTION_INVALID")
        allowed.add(key)
    return allowed, config["expiresBeforeUTC"]


def check_report(result, packages, allowed=()):
    require(result.returncode in (0, 1), "CARGO_SCAN_FAILED")
    require(0 < len(result.stdout) <= 4000000, "CARGO_REPORT_INVALID")
    report = read_json(result.stdout)
    require(isinstance(report, dict) and set(report) == {"database", "lockfile", "settings",
                                                        "vulnerabilities", "warnings"}, "CARGO_REPORT_INVALID")
    require(isinstance(report["lockfile"], dict)
            and type(report["lockfile"].get("dependency-count")) is int
            and report["lockfile"]["dependency-count"] == len(packages), "CARGO_REPORT_INVALID")
    settings = report["settings"]
    require(isinstance(settings, dict)
            and set(settings) == {"target_arch", "target_os", "severity", "ignore", "informational_warnings"}
            and settings.get("ignore") == []
            and settings.get("severity") is None and settings.get("target_arch") == []
            and settings.get("target_os") == []
            and isinstance(settings.get("informational_warnings"), list)
            and set(settings["informational_warnings"]) == {"unmaintained", "unsound", "notice"},
            "CARGO_SETTINGS_INVALID")
    db = report["database"]
    require(isinstance(db, dict) and type(db.get("advisory-count")) is int and db["advisory-count"] > 0
            and isinstance(db.get("last-commit"), str) and re.fullmatch(r"[0-9a-f]{40}", db["last-commit"])
            and isinstance(db.get("last-updated"), str)
            and datetime.fromisoformat(db["last-updated"]).tzinfo is not None, "CARGO_DATABASE_INVALID")
    vulnerabilities = report["vulnerabilities"]
    require(isinstance(vulnerabilities, dict) and isinstance(vulnerabilities.get("list"), list)
            and type(vulnerabilities.get("count")) is int
            and vulnerabilities["count"] == len(vulnerabilities["list"])
            and type(vulnerabilities.get("found")) is bool
            and vulnerabilities["found"] == bool(vulnerabilities["count"]), "CARGO_REPORT_INVALID")
    require(isinstance(report["warnings"], dict) and set(report["warnings"]).issubset(KINDS),
            "CARGO_REPORT_INVALID")
    warnings, ids = [], []
    for kind, entries in report["warnings"].items():
        require(isinstance(entries, list), "CARGO_REPORT_INVALID")
        for entry in entries:
            require(isinstance(entry, dict) and entry.get("kind") == kind
                    and package_key(entry.get("package")) in packages, "CARGO_REPORT_INVALID")
            advisory = entry.get("advisory")
            if kind == "yanked":
                id = "YANKED"
            else:
                require(isinstance(advisory, dict) and isinstance(advisory.get("id"), str)
                        and re.fullmatch(r"RUSTSEC-\d{4}-\d{4}", advisory["id"]), "CARGO_REPORT_INVALID")
                id = advisory["id"]
            key = (*package_key(entry["package"]), kind, id)
            require(key not in warnings, "CARGO_REPORT_INVALID")
            warnings.append(key)
            ids.append(id)
    if vulnerabilities["count"]:
        raise GateError("CARGO_VULNERABILITIES_FOUND")
    require(result.returncode == int(bool(warnings)), "CARGO_EXIT_REPORT_MISMATCH")
    if any(key not in allowed for key in warnings):
        raise GateError("CARGO_WARNINGS_UNAPPROVED", ids)
    return {"dependencies": len(packages), "vulnerabilities": 0, "warnings": len(warnings),
            "excepted": len(warnings), "ids": sorted(set(ids)), "database": db}


def validate_db():
    safe_path(DB)
    safe_path(DB / ".git")
    require(DB.is_dir() and (DB / ".git").is_dir(), "CARGO_DATABASE_MISSING")
    result = invoke(["git", "-C", str(DB), "remote", "get-url", "origin"])
    require(result.returncode == 0 and result.stdout.decode().strip().lower() == DB_URL.lower(),
            "CARGO_DATABASE_ORIGIN_INVALID")
    result = invoke(["git", "-C", str(DB), "status", "--porcelain", "--untracked-files=all"])
    require(result.returncode == 0 and not result.stdout.strip(), "CARGO_DATABASE_DIRTY")
    result = invoke(["git", "-C", str(DB), "rev-parse", "HEAD"])
    require(result.returncode == 0 and re.fullmatch(rb"[0-9a-f]{40}", result.stdout.strip()),
            "CARGO_DATABASE_INVALID")
    return result.stdout.decode().strip()


def scan(source=ROOT):
    exe = scanner()
    blobs, packages = inputs(source)
    allowed, expiry = load_exceptions(blobs)
    validate_db()
    with scratch() as work:
        (work / "Cargo.lock").write_bytes(blobs["src-tauri/Cargo.lock"])
        (work / ".cargo").mkdir()
        (work / ".cargo/audit.toml").write_text(CONFIG, encoding="utf-8")
        args = [str(exe), "audit", "--file", str(work / "Cargo.lock"), "--db", str(DB),
                "--url", DB_URL, "--deny", "warnings", "--json"]
        result = invoke(args, work)
        database_head = validate_db()
        evidence = check_report(result, packages, allowed)
        require(evidence["database"]["last-commit"] == database_head, "CARGO_DATABASE_COMMIT_MISMATCH")
    require(inputs(source)[0] == blobs, "CARGO_INPUT_CHANGED")
    scanner()
    evidence.update({"version": "0.22.2", "binarySHA256": BIN_SHA, "exceptionExpiry": expiry,
                     "inputSHA256": {name: hashlib.sha256(blob).hexdigest() for name, blob in blobs.items()}})
    return evidence


def main():
    start = time.perf_counter()
    try:
        evidence = scan()
        result = invoke(["git", "rev-parse", "HEAD"])
        require(result.returncode == 0 and re.fullmatch(rb"[0-9a-f]{40}", result.stdout.strip()),
                "CARGO_SOURCE_SHA_INVALID")
        evidence.update({"sourceSha": result.stdout.decode().strip(),
                         "elapsedSeconds": round(time.perf_counter() - start, 3)})
        print("CARGO_AUDIT_EVIDENCE " + json.dumps(evidence, sort_keys=True))
    except (GateError, OSError, ValueError, TypeError, KeyError) as error:
        print("CARGO_AUDIT_BLOCKED " + json.dumps({
            "code": str(error) if isinstance(error, GateError) else "CARGO_GATE_FAILED",
            "ids": error.ids if isinstance(error, GateError) else []}, sort_keys=True))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
