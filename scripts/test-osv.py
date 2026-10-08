"""OSV gate checks with real CLI and a tiny, synthetic offline database."""
from copy import deepcopy
from io import BytesIO
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from types import SimpleNamespace
from unittest.mock import patch
import zipfile
import check_osv as gate

FIXTURES = gate.ROOT / "artifacts/sec009-tests"


def main():
    start = time.perf_counter()
    results = []

    def case(name, action, code=None):
        try:
            action()
        except gate.GateError as error:
            assert code == error.code, (name, error.code, code)
        else:
            assert code is None, name + " unexpectedly accepted."
        results.append({"case": name, "status": "pass", "expectedFailure": code})

    def response(report, exit=0):
        return SimpleNamespace(returncode=exit, stdout=json.dumps(report).encode(), stderr=b"")

    # The original permissive RED stub accepts 127; this check must fail before BUILD.
    case("unexpected-scanner-error", lambda: gate.check_report(response({"results": []}, 127), {}),
         "OSV_SCAN_FAILED")
    gate.safe_path(FIXTURES)
    assert not FIXTURES.exists()
    FIXTURES.mkdir()
    source, cache, db = FIXTURES / "source", FIXTURES / "cache", FIXTURES / "db"
    source.mkdir()
    (source / "src-tauri").mkdir()
    draft = FIXTURES / "draft.json"
    draft.write_text('{"approved":false}', encoding="utf-8")
    original_cache = gate.CACHE
    try:
        exe = gate.scanner()
        cache.mkdir()
        for path in original_cache.iterdir():
            gate.safe_path(path)
            shutil.copyfile(path, cache / path.name)

        def locks(npm="4.17.21", cargo="1.5.5"):
            (source / gate.LOCKS[0]).write_text(json.dumps({
                "name": "fixture-app", "version": "1.0.0", "lockfileVersion": 3,
                "packages": {"": {"name": "fixture-app", "version": "1.0.0"},
                             "node_modules/lodash": {"version": npm,
                                 "resolved": f"https://registry.npmjs.org/lodash/-/lodash-{npm}.tgz"}}}),
                encoding="utf-8")
            (source / gate.LOCKS[1]).write_text(
                'version = 4\n[[package]]\nname = "regex"\nversion = "' + cargo
                + '"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n', encoding="utf-8")

        for ecosystem, package, fixed, id in [
                ("npm", "lodash", "4.17.21", "MILLANI-TEST-NPM-0001"),
                ("crates.io", "regex", "1.5.5", "MILLANI-TEST-CARGO-0001")]:
            directory = db / "osv-scalibr" / ecosystem
            directory.mkdir(parents=True)
            advisory = {"schema_version": "1.6.0", "id": id, "modified": "2026-01-01T00:00:00Z",
                        "affected": [{"package": {"ecosystem": ecosystem, "name": package},
                                      "ranges": [{"type": "SEMVER",
                                                  "events": [{"introduced": "0"}, {"fixed": fixed}]}]}]}
            with zipfile.ZipFile(directory / "all.zip", "w") as archive:
                archive.writestr(id + ".json", json.dumps(advisory))

        base_invoke = gate.invoke
        def offline(args, cwd=gate.ROOT):
            if args[1:3] != ["scan", "source"]:
                return base_invoke(args, cwd)
            assert "--all-packages" in args and "--no-resolve" in args and "--no-call-analysis=all" in args
            assert args.count("--lockfile") == 2 and "--config" in args
            env = gate.process_env() | {"OSV_SCALIBR_LOCAL_DB_CACHE_DIRECTORY": str(db)}
            return subprocess.run([*args, "--offline"], cwd=cwd, env=env, capture_output=True, timeout=30)

        with patch.object(gate, "EXCEPTIONS", draft), patch.object(gate, "invoke", offline):
            locks()
            case("real-cli-offline-clean", lambda: gate.scan(source, exe))
            def post_tamper(args, cwd=gate.ROOT):
                result = offline(args, cwd)
                if args[1:3] == ["scan", "source"]:
                    (cache / "LICENSE").write_bytes(b"post-scan integrity fixture")
                return result
            with patch.object(gate, "CACHE", cache), patch.object(gate, "invoke", post_tamper):
                case("post-scan-cache-tamper", lambda: gate.scan(source, exe), "OSV_INTEGRITY_INVALID")
            shutil.copyfile(original_cache / "LICENSE", cache / "LICENSE")
            locks(npm="4.17.20")
            case("real-cli-offline-npm-finding", lambda: gate.scan(source, exe), "OSV_FINDINGS")
            locks(cargo="1.5.4")
            case("real-cli-offline-cargo-finding", lambda: gate.scan(source, exe), "OSV_FINDINGS")
            locks()
            local_config = source / "osv-scanner.toml"
            local_config.write_text('[[PackageOverrides]]\nignore = true\n', encoding="utf-8")
            case("source-ignore-denied", lambda: gate.scan(source, exe), "OSV_SOURCE_CONFIG_DENIED")
            local_config.unlink()
            (source / gate.LOCKS[1]).unlink()
            case("missing-cargo-lock", lambda: gate.inputs(source), "OSV_LOCK_MISSING_OR_INVALID")
            locks()
            (source / gate.LOCKS[0]).write_text('{"lockfileVersion":3,"lockfileVersion":3}', encoding="utf-8")
            case("duplicate-input-key", lambda: gate.inputs(source), "OSV_DUPLICATE_KEY")
            def malformed_process():
                r = subprocess.run([sys.executable, "-B", str(Path(gate.__file__)), "--source", str(source)],
                                   env=gate.process_env(), capture_output=True, timeout=30)
                assert r.returncode == 1 and json.loads(r.stderr)["code"] == "OSV_INPUT_OR_INSTALL_FAILED"
            (source / gate.LOCKS[0]).write_text("{truncated", encoding="utf-8")
            case("malformed-json-main-fails", malformed_process)
            locks()
            (source / gate.LOCKS[1]).write_text("version = [", encoding="utf-8")
            case("malformed-toml-main-fails", malformed_process)
            locks()
            (source / gate.LOCKS[1]).write_text("version = 4\npackage = []", encoding="utf-8")
            case("empty-cargo-lock", lambda: gate.inputs(source), "OSV_CARGO_EMPTY")
            locks()

        expected = {gate.report_path(str(source / gate.LOCKS[0])): {("npm", "lodash", "4.17.21")},
                    gate.report_path(str(source / gate.LOCKS[1])): {("crates.io", "regex", "1.5.5")}}
        report = {"results": [
            {"source": {"path": path, "type": "lockfile"},
             "packages": [{"package": dict(zip(("ecosystem", "name", "version"), next(iter(keys))))}]}
            for path, keys in expected.items()]}
        case("valid-complete-report", lambda: gate.check_report(response(report), expected))
        case("invalid-report-json", lambda: gate.check_report(
            SimpleNamespace(returncode=0, stdout=b"{truncated"), expected), "OSV_REPORT_INVALID")
        case("duplicate-report-key", lambda: gate.check_report(
            SimpleNamespace(returncode=0, stdout=b'{"results":[],"results":[]}'), expected), "OSV_DUPLICATE_KEY")
        partial = deepcopy(report)
        partial["results"].pop()
        case("partial-lockfile-report", lambda: gate.check_report(response(partial), expected), "OSV_INVENTORY_INCOMPLETE")
        omitted = deepcopy(report)
        omitted["results"][0]["packages"] = []
        case("partial-package-inventory", lambda: gate.check_report(response(omitted), expected), "OSV_INVENTORY_INCOMPLETE")
        vulnerable = deepcopy(report)
        vulnerable["results"][0]["packages"][0]["vulnerabilities"] = [{"id": "MILLANI-TEST-NPM-0001"}]
        case("zero-exit-with-finding", lambda: gate.check_report(response(vulnerable), expected), "OSV_FINDINGS")
        case("nonzero-without-finding", lambda: gate.check_report(response(report, 1), expected), "OSV_SCAN_FAILED")
        with patch.object(gate.subprocess, "run", side_effect=subprocess.TimeoutExpired("fixture", 1)):
            case("process-timeout-boundary", lambda: gate.invoke(["fixture"]), "OSV_PROCESS_FAILED")
        case("missing-process-boundary", lambda: gate.invoke([str(FIXTURES / "missing.exe")]), "OSV_PROCESS_FAILED")
        with patch.dict(os.environ, {"OSV_FIXTURE": "denied", "GIT_FIXTURE": "denied", "GH_TOKEN": "fake-not-a-secret"}):
            assert not any(x in gate.process_env() for x in ["OSV_FIXTURE", "GIT_FIXTURE", "GH_TOKEN"])
        results.append({"case": "suppression-env-removed", "status": "pass", "expectedFailure": None})

        with patch.object(gate, "CACHE", cache):
            (cache / "unexpected.dll").write_bytes(b"never loaded")
            case("extra-cache-file", gate.scanner, "OSV_CACHE_INVALID")
            (cache / "unexpected.dll").unlink()
            (cache / "LICENSE").write_bytes(b"invalid license fixture")
            case("tampered-cache-license", gate.scanner, "OSV_INTEGRITY_INVALID")
            shutil.copyfile(original_cache / "LICENSE", cache / "LICENSE")
            with patch.object(gate, "invoke", return_value=SimpleNamespace(
                    returncode=0, stdout=b"osv-scanner version: 9.9.9\n")):
                case("wrong-scanner-version", gate.scanner, "OSV_VERSION_INVALID")
        missing_cache = FIXTURES / "never-created-cache"
        with patch.object(gate, "CACHE", missing_cache), patch.object(gate.urllib.request, "urlopen",
                side_effect=lambda *a, **k: BytesIO(b"invalid download fixture")):
            case("invalid-download-boundary", gate.scanner, "OSV_INTEGRITY_INVALID")
            assert not missing_cache.exists()

        config = json.loads(gate.EXCEPTIONS.read_bytes())
        proposal = deepcopy(config)
        proposal["approved"] = True
        exception_fixture = FIXTURES / "proposal.json"
        approval_fixture = FIXTURES / "approval.json"
        record = {"approvedScopeSHA256": None, "humanDecision": "synthetic fixture; not human approval",
                  "decisionReference": "synthetic fixture; not an E4 source"}

        def renew_synthetic_approval():
            record["approvedScopeSHA256"] = gate.approval_digest(proposal)
            content = json.dumps(record, sort_keys=True).encode("utf-8")
            approval_fixture.write_bytes(content)
            proposal["approvalRecord"] = "sha256:" + hashlib.sha256(content).hexdigest()

        def write_proposal():
            exception_fixture.write_text(json.dumps(proposal), encoding="utf-8")

        _, blobs, _ = gate.inputs(gate.ROOT)
        with patch.object(gate, "EXCEPTIONS", exception_fixture), patch.object(gate, "APPROVAL", approval_fixture):
            renew_synthetic_approval()
            write_proposal()
            allowed, expiry = gate.load_exceptions(blobs)
            assert len(allowed) == 3 and expiry == proposal["expiresBeforeUTC"]
            results.append({"case": "proposal-synthetic-approval-boundary", "status": "pass", "expectedFailure": None})
            proposal["expiresBeforeUTC"] = "2099-01-01"
            write_proposal()
            case("expiry-change-invalidates-approval", lambda: gate.load_exceptions(blobs),
                 "OSV_EXCEPTION_APPROVAL_MISMATCH")
            proposal["approvalSHA256"] = gate.approval_digest(proposal)
            write_proposal()
            case("recalculated-proposal-keeps-old-record-denied", lambda: gate.load_exceptions(blobs),
                 "OSV_EXCEPTION_APPROVAL_MISMATCH")
            del proposal["approvalSHA256"]
            proposal["expiresBeforeUTC"] = config["expiresBeforeUTC"]
            proposal["exceptions"][0]["ids"].append("MILLANI-TEST-UNAPPROVED-0001")
            write_proposal()
            case("entry-change-invalidates-approval", lambda: gate.load_exceptions(blobs),
                 "OSV_EXCEPTION_APPROVAL_MISMATCH")
            proposal["exceptions"] = deepcopy(config["exceptions"])
            record["humanDecision"] = "altered synthetic decision"
            approval_fixture.write_text(json.dumps(record), encoding="utf-8")
            write_proposal()
            case("altered-record-keeps-old-reference-denied", lambda: gate.load_exceptions(blobs),
                 "OSV_EXCEPTION_RECORD_CHANGED")
            record["humanDecision"] = None
            renew_synthetic_approval()
            write_proposal()
            case("draft-approval-record-denied", lambda: gate.load_exceptions(blobs), "OSV_EXCEPTION_UNAPPROVED")
            record["humanDecision"] = "synthetic fixture; not human approval"
            record["unexpected"] = "invalid schema fixture"
            renew_synthetic_approval()
            write_proposal()
            case("invalid-approval-schema-denied", lambda: gate.load_exceptions(blobs), "OSV_EXCEPTION_APPROVAL_INVALID")
            del record["unexpected"]
            renew_synthetic_approval()
            proposal["approvalRecord"] = None
            write_proposal()
            case("unapproved-exception", lambda: gate.load_exceptions(blobs), "OSV_EXCEPTION_UNAPPROVED")
            proposal["expiresBeforeUTC"] = "2000-01-01"
            renew_synthetic_approval()  # Explicit synthetic renewal tests expiration independently.
            write_proposal()
            case("expired-exception", lambda: gate.load_exceptions(blobs), "OSV_EXCEPTION_EXPIRED")
            proposal["expiresBeforeUTC"] = config["expiresBeforeUTC"]
            proposal["inputSHA256"]["src-tauri/Cargo.lock"] = "0" * 64
            renew_synthetic_approval()  # Explicit synthetic renewal tests input binding independently.
            write_proposal()
            case("changed-exception-scope", lambda: gate.load_exceptions(blobs), "OSV_EXCEPTION_SCOPE_CHANGED")

        allowed = {("npm", "lodash", "4.17.21", "MILLANI-TEST-NPM-0001")}
        assert gate.check_report(response(vulnerable, 1), expected, allowed)["excepted"] == 1
        case("exception-other-package-denied", lambda: gate.check_report(
            response(vulnerable, 1), expected, {("npm", "other", "4.17.21", "MILLANI-TEST-NPM-0001")}), "OSV_FINDINGS")
        results.append({"case": "exception-exact-identity-only", "status": "pass", "expectedFailure": None})
        link = FIXTURES / "link"
        env = os.environ | {"MILLANI_SEC009_LINK": str(link), "MILLANI_SEC009_TARGET": str(source)}
        r = subprocess.run(["pwsh", "-NoProfile", "-Command",
                            "New-Item -ItemType Junction -Path $env:MILLANI_SEC009_LINK -Target $env:MILLANI_SEC009_TARGET | Out-Null"],
                           env=env, capture_output=True, timeout=30)
        assert r.returncode == 0 and link.is_junction()
        try:
            case("own-junction-denied", lambda: gate.inputs(link), "OSV_REPARSE_DENIED")
        finally:
            assert link.absolute().parent == FIXTURES and link.is_junction() and link.resolve() == source.resolve()
            link.rmdir()
    finally:
        assert FIXTURES.resolve().is_relative_to(gate.ROOT / "artifacts") and FIXTURES.name == "sec009-tests"
        for path in [FIXTURES, *FIXTURES.rglob("*")]:
            gate.safe_path(path)
        shutil.rmtree(FIXTURES)
    gate.validate_cache()
    assert not FIXTURES.exists() and not gate.SCRATCH.exists()
    print("OSV_TESTS " + json.dumps({"cases": len(results), "results": results,
          "seconds": round(time.perf_counter() - start, 3), "fixtureRemoved": True,
          "cacheOriginalPreserved": True, "database": "synthetic offline only"}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
