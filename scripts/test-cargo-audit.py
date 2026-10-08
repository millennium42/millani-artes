"""Cargo invariants and real CLI against a synthetic offline Git advisory DB."""
from copy import deepcopy
from datetime import datetime, timezone, timedelta
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import time
from types import SimpleNamespace
from unittest.mock import patch
import check_cargo_audit as gate

if sys.flags.optimize:
    raise SystemExit("CARGO_AUDIT_TESTS_BLOCKED OPTIMIZE_DENIED")
FIXTURES = gate.ROOT / "artifacts/sec011-tests"


def remove_readonly(func, path, error):
    file = Path(path)
    assert file.absolute().is_relative_to(FIXTURES) and file.is_file()
    gate.safe_path(file)
    os.chmod(file, stat.S_IWRITE)
    func(path)


def main():
    start = time.perf_counter()
    cases = []

    def case(name, action, code=None):
        try:
            action()
        except gate.GateError as error:
            assert str(error) == code, (name, str(error), code)
        else:
            assert code is None, name + " unexpectedly accepted"
        cases.append(name)

    optimized = subprocess.run([sys.executable, "-O", "-B", str(Path(__file__).resolve())], capture_output=True)
    assert optimized.returncode == 1 and b"OPTIMIZE_DENIED" in optimized.stderr
    cases.append("optimized-tests-denied")
    packages = {("fixture-crate", "1.0.0", "registry+https://github.com/rust-lang/crates.io-index")}
    clean = {"database": {"advisory-count": 1, "last-commit": "1" * 40,
                         "last-updated": "2026-10-08T00:00:00Z"},
             "lockfile": {"dependency-count": 1},
             "settings": {"target_arch": [], "target_os": [], "severity": None, "ignore": [],
                          "informational_warnings": ["unmaintained", "unsound", "notice"]},
             "vulnerabilities": {"found": False, "count": 0, "list": []}, "warnings": {}}

    def response(value, code=0):
        return SimpleNamespace(returncode=code, stdout=json.dumps(value).encode())

    case("scanner-error", lambda: gate.check_report(response({}, 2), packages), "CARGO_SCAN_FAILED")
    case("clean-report", lambda: gate.check_report(response(clean), packages))
    case("nonzero-clean", lambda: gate.check_report(response(clean, 1), packages), "CARGO_EXIT_REPORT_MISMATCH")
    value = deepcopy(clean)
    value["vulnerabilities"] = {"found": True, "count": 1, "list": [{}]}
    case("vulnerability-even-exit0", lambda: gate.check_report(response(value), packages), "CARGO_VULNERABILITIES_FOUND")
    warning = deepcopy(clean)
    warning["warnings"] = {"unmaintained": [{"kind": "unmaintained",
        "package": dict(zip(("name", "version", "source"), next(iter(packages)))),
        "advisory": {"id": "RUSTSEC-2026-0001"}}]}
    case("warning-denied", lambda: gate.check_report(response(warning, 1), packages), "CARGO_WARNINGS_UNAPPROVED")
    case("warning-zero-exit", lambda: gate.check_report(response(warning), packages), "CARGO_EXIT_REPORT_MISMATCH")
    allowed = {(*next(iter(packages)), "unmaintained", "RUSTSEC-2026-0001")}
    case("synthetic-approved-warning", lambda: gate.check_report(response(warning, 1), packages, allowed))
    for name, mutate, code in [
            ("missing-database", lambda v: v.pop("database"), "CARGO_REPORT_INVALID"),
            ("partial-inventory", lambda v: v["lockfile"].update({"dependency-count": 0}), "CARGO_REPORT_INVALID"),
            ("false-count", lambda v: v["vulnerabilities"].update(count=1), "CARGO_REPORT_INVALID"),
            ("false-found", lambda v: v["vulnerabilities"].update(found=True), "CARGO_REPORT_INVALID"),
            ("unknown-warning-kind", lambda v: v["warnings"].update(unknown=[]), "CARGO_REPORT_INVALID"),
            ("suppressed-ignore", lambda v: v["settings"].update(ignore=["RUSTSEC-2026-0001"]), "CARGO_SETTINGS_INVALID"),
            ("suppressed-severity", lambda v: v["settings"].update(severity="high"), "CARGO_SETTINGS_INVALID"),
            ("target-filter", lambda v: v["settings"].update(target_os=["windows"]), "CARGO_SETTINGS_INVALID"),
            ("missing-setting", lambda v: v["settings"].pop("severity"), "CARGO_SETTINGS_INVALID"),
            ("missing-warning-category", lambda v: v["settings"].update(informational_warnings=[]), "CARGO_SETTINGS_INVALID"),
            ("bad-db-commit", lambda v: v["database"].update({"last-commit": "invalid"}), "CARGO_DATABASE_INVALID")]:
        value = deepcopy(clean)
        mutate(value)
        case(name, lambda v=value: gate.check_report(response(v), packages), code)
    for name, blob, code in [("duplicate-json", b'{"x":1,"x":2}', "CARGO_DUPLICATE_KEY"),
                             ("malformed-json", b"{", "CARGO_JSON_INVALID"),
                             ("nan-json", b'{"x":NaN}', "CARGO_JSON_INVALID"),
                             ("empty-report", b"", "CARGO_REPORT_INVALID")]:
        case(name, lambda b=blob: gate.check_report(SimpleNamespace(returncode=0, stdout=b), packages), code)
    with patch.object(gate.subprocess, "run", side_effect=subprocess.TimeoutExpired("fixture", 150)):
        case("timeout", lambda: gate.invoke(["fixture"]), "CARGO_PROCESS_FAILED")
    with patch.dict(os.environ, {"CARGO_HOME": "fixture", "GIT_CONFIG_COUNT": "1", "RUSTUP_TOOLCHAIN": "bad"}):
        assert not any(k.upper().startswith(("CARGO_", "RUSTUP_", "GIT_")) for k in gate.process_env())
    cases.append("environment-isolation")
    gate.safe_path(FIXTURES)
    assert not FIXTURES.exists()
    FIXTURES.mkdir(parents=True)
    source, db = FIXTURES / "source", FIXTURES / "db"
    (source / "src-tauri").mkdir(parents=True)
    db.mkdir()
    cfgpath, recordpath = FIXTURES / "exception.json", FIXTURES / "approval.json"
    cfgpath.write_text('{"approved":false}', encoding="utf-8")
    exe = gate.scanner()

    def git(*args):
        result = subprocess.run(["rtk", "proxy", "git", "-c", "user.name=Millani Fixture",
            "-c", "user.email=fixture@example.invalid", "-c", "commit.gpgSign=false",
            "-c", "core.hooksPath=NUL", *args], cwd=db, capture_output=True, timeout=30)
        assert result.returncode == 0, args
        return result.stdout

    def inputs(version="1.0.0"):
        (source / "src-tauri/Cargo.toml").write_text('[package]\nname="fixture-app"\nversion="1.0.0"\n', encoding="utf-8")
        (source / "src-tauri/Cargo.lock").write_text('version=4\n[[package]]\nname="fixture-app"\nversion="1.0.0"\n'
            '[[package]]\nname="fixture-crate"\nversion="' + version + '"\n'
            'source="registry+https://github.com/rust-lang/crates.io-index"\nchecksum="' + "0" * 64 + '"\n', encoding="utf-8")

    def advisory(kind=None):
        directory = db / "crates/fixture-crate"
        directory.mkdir(parents=True, exist_ok=True)
        (directory / "RUSTSEC-2026-0001.md").write_text(chr(96) * 3
            + 'toml\n[advisory]\nid="RUSTSEC-2026-0001"\npackage="fixture-crate"\ndate="2026-10-08"\n'
            + ('informational="' + kind + '"\n' if kind else "")
            + '[versions]\npatched=[">=2.0.0"]\n' + chr(96) * 3
            + '\n\n# Synthetic advisory\n\nOwn fixture only.\n', encoding="utf-8")
        git("add", "crates")
        git("commit", "-m", "synthetic advisory")

    def approve(config, decision="SYNTHETIC TEST, NOT HUMAN APPROVAL", reference="fixture://sec011"):
        record = {"approvedScopeSHA256": gate.scope_digest(config),
                  "humanDecision": decision, "decisionReference": reference}
        content = (json.dumps(record) + "\n").encode()
        recordpath.write_bytes(content)
        config["approvalRecord"] = "sha256:" + hashlib.sha256(content).hexdigest()
        cfgpath.write_text(json.dumps(config), encoding="utf-8")

    try:
        git("init")
        git("remote", "add", "origin", gate.DB_URL)
        advisory()
        inputs("2.0.0")
        base = gate.invoke

        def offline(args, cwd=gate.ROOT):
            if len(args) < 2 or args[1] != "audit":
                return base(args, cwd)
            assert args[0] == str(exe) and "--deny" in args and "warnings" in args and "--json" in args
            assert "--ignore" not in args and "--target-os" not in args and "--no-yanked" not in args
            assert (cwd / ".cargo/audit.toml").read_text() == gate.CONFIG
            result = base([*args, "--no-fetch", "--no-yanked"], cwd)
            # Offline CLI omits Git metadata; annotate only these two fixture fields.
            if result.stdout:
                report = json.loads(result.stdout)
                assert report["database"]["last-commit"] is None and report["database"]["last-updated"] is None
                report["database"]["last-commit"] = git("rev-parse", "HEAD").decode().strip()
                report["database"]["last-updated"] = git("show", "-s", "--format=%cI", "HEAD").decode().strip()
                result = SimpleNamespace(returncode=result.returncode, stdout=json.dumps(report).encode())
            return result

        with patch.object(gate, "DB", db), patch.object(gate, "EXCEPTIONS", cfgpath), \
                patch.object(gate, "APPROVAL", recordpath), patch.object(gate, "invoke", side_effect=offline):
            case("real-cli-clean", lambda: gate.scan(source))
            inputs()
            case("real-cli-vulnerability", lambda: gate.scan(source), "CARGO_VULNERABILITIES_FOUND")
            advisory("unmaintained")
            case("real-cli-warning", lambda: gate.scan(source), "CARGO_WARNINGS_UNAPPROVED")
            (source / ".cargo").mkdir()
            (source / ".cargo/audit.toml").write_text('[advisories]\nignore=["RUSTSEC-2026-0001"]\n', encoding="utf-8")
            with patch.dict(os.environ, {"CARGO_HOME": str(FIXTURES / "evil")}):
                case("real-cli-source-ignore-ineffective", lambda: gate.scan(source), "CARGO_WARNINGS_UNAPPROVED")
            blobs, _ = gate.inputs(source)
            config = {"approved": True, "approvalRecord": None, "gate": "SEC-011/cargo-audit/0.22.2",
                "target": "x86_64-pc-windows-msvc",
                "expiresBeforeUTC": (datetime.now(timezone.utc).date() + timedelta(days=2)).isoformat(),
                "inputSHA256": {name: hashlib.sha256(blob).hexdigest() for name, blob in blobs.items()},
                "exceptions": [{"name": "fixture-crate", "version": "1.0.0", "kind": "unmaintained", "id": "RUSTSEC-2026-0001"}]}
            approve(config)
            case("synthetic-decision-denied", lambda: gate.load_exceptions(blobs), "CARGO_APPROVAL_INVALID")
            reference = "Codex user reply to SEC-011 proposal " + "1" * 40
            approve(config, "Rejeito a exceção SEC-011", reference)
            case("rejection-denied", lambda: gate.load_exceptions(blobs), "CARGO_APPROVAL_INVALID")
            approve(config, gate.APPROVAL_TEXT)
            case("fixture-reference-denied", lambda: gate.load_exceptions(blobs), "CARGO_APPROVAL_INVALID")
            # Fixture approval is available only through this explicit test patch.
            with patch.object(gate, "APPROVAL_TEXT", "SYNTHETIC TEST, NOT HUMAN APPROVAL"), \
                    patch.object(gate, "REFERENCE_RE", r"fixture://sec011"):
                approve(config)
                case("real-cli-synthetic-approval", lambda: gate.scan(source))
                recordpath.write_text('{"approvedScopeSHA256":null}', encoding="utf-8")
                case("record-tampered", lambda: gate.load_exceptions(blobs), "CARGO_APPROVAL_RECORD_CHANGED")
                approve(config)
                for name, mutate, code in [
                    ("gate-binding", lambda v: v.update(gate="SEC-009/OSV"), "CARGO_EXCEPTION_UNAPPROVED"),
                    ("scope-changed", lambda v: v["exceptions"][0].update(version="9.0.0"), "CARGO_APPROVAL_SCOPE_CHANGED"),
                    ("expiry", lambda v: v.update(expiresBeforeUTC="2000-01-01"), "CARGO_EXCEPTION_EXPIRED"),
                    ("input-binding", lambda v: v["inputSHA256"].update({"src-tauri/Cargo.lock": "0" * 64}), "CARGO_EXCEPTION_INPUT_CHANGED")]:
                    value = deepcopy(config)
                    mutate(value)
                    if name in ("expiry", "input-binding"):
                        approve(value)
                    else:
                        cfgpath.write_text(json.dumps(value), encoding="utf-8")
                    case(name, lambda: gate.load_exceptions(blobs), code)
                    approve(config)
                advisory("unsound")
                case("other-warning-kind", lambda: gate.scan(source), "CARGO_WARNINGS_UNAPPROVED")
            case("fixture-approval-denied-after-unpatch", lambda: gate.load_exceptions(blobs), "CARGO_APPROVAL_INVALID")
            config["approved"] = False
            cfgpath.write_text(json.dumps(config), encoding="utf-8")
            with patch.object(gate, "EXE", FIXTURES / "missing.exe"):
                case("missing-binary", gate.scanner, "CARGO_BINARY_INVALID")
            with patch.object(gate, "invoke", return_value=SimpleNamespace(returncode=0, stdout=b"cargo-audit 0.22.1")):
                case("wrong-version", gate.scanner, "CARGO_VERSION_INVALID")
            (source / "src-tauri/Cargo.lock").unlink()
            case("missing-input", lambda: gate.inputs(source), "CARGO_INPUT_INVALID")
            inputs()
            dirty = db / "unexpected.md"
            dirty.write_text("own fixture", encoding="utf-8")
            try:
                case("dirty-db-denied", gate.validate_db, "CARGO_DATABASE_DIRTY")
            finally:
                dirty.unlink()
            with gate.scratch():
                case("scratch-exists", lambda: gate.scan(source), "CARGO_SCRATCH_EXISTS")
            target, link = FIXTURES / "target", FIXTURES / "link"
            target.mkdir()
            (target / "sentinel").write_text("preserve", encoding="utf-8")
            result = subprocess.run(["rtk", "proxy", "cmd", "/c", "mklink", "/J", str(link), str(target)], capture_output=True)
            assert result.returncode == 0 and link.is_junction()
            try:
                case("real-junction", lambda: gate.inputs(link), "CARGO_REPARSE_DENIED")
                assert (target / "sentinel").read_text() == "preserve"
            finally:
                assert link.absolute().is_relative_to(FIXTURES) and link.is_junction() and link.resolve() == target.resolve()
                link.rmdir()
        assert not gate.SCRATCH.exists()
    finally:
        assert FIXTURES.resolve().is_relative_to(gate.ROOT / "artifacts")
        for path in [FIXTURES, *FIXTURES.rglob("*")]:
            gate.safe_path(path)
        shutil.rmtree(FIXTURES, onexc=remove_readonly)
    print("CARGO_AUDIT_TESTS " + json.dumps({"cases": len(cases), "realCLI": 6, "status": "pass",
                                           "seconds": round(time.perf_counter() - start, 3),
                                           "fixtureRemoved": not FIXTURES.exists()}))


if __name__ == "__main__":
    main()
