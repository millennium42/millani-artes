"""npm gate invariants and real CLI against a synthetic loopback registry."""
from copy import deepcopy
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import gzip
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import threading
import time
from types import SimpleNamespace
from unittest.mock import patch
import check_npm_audit as gate

if sys.flags.optimize:
    raise SystemExit("NPM_AUDIT_TESTS_BLOCKED OPTIMIZE_DENIED")

FIXTURES = gate.ROOT / "artifacts/sec010-tests"


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
    packages = {"node_modules/fixture-package": {"version": "1.0.0"}}

    def report(severity=None):
        counts = {level: int(level == severity) for level in gate.SEVERITIES}
        counts["total"] = sum(counts.values())
        return {"auditReportVersion": 2, "vulnerabilities": {
            "fixture-package": {"name": "fixture-package", "severity": severity,
                                "nodes": ["node_modules/fixture-package"]}} if severity else {},
                "metadata": {"vulnerabilities": counts, "dependencies": {"total": 1}}}

    def response(value, code=0):
        return SimpleNamespace(returncode=code, stdout=json.dumps(value).encode(), stderr=b"")

    case("scanner-error", lambda: gate.check_report(response({}, 127), packages), "NPM_SCAN_FAILED")
    for severity in (None, "info", "low", "moderate", "high", "critical"):
        case("report-" + str(severity), lambda s=severity: gate.check_report(
            response(report(s), int(s in ("high", "critical"))), packages),
            "NPM_HIGH_CRITICAL_FOUND" if severity in ("high", "critical") else None)
    case("high-with-exit-zero", lambda: gate.check_report(response(report("high")), packages),
         "NPM_HIGH_CRITICAL_FOUND")
    case("nonzero-clean", lambda: gate.check_report(response(report(), 1), packages), "NPM_SCAN_FAILED")
    unknown = report()
    unknown["metadata"]["vulnerabilities"]["unknown"] = 1
    case("unknown-summary-severity", lambda: gate.check_report(response(unknown), packages), "NPM_REPORT_INVALID")
    for name, mutate in [
            ("missing-metadata", lambda v: v.pop("metadata")),
            ("missing-dependencies", lambda v: v["metadata"].pop("dependencies")),
            ("partial-dependencies", lambda v: v["metadata"]["dependencies"].update(total=0)),
            ("error-with-success", lambda v: v.update(error={"code": "fixture"})),
            ("fake-clean-summary", lambda v: v["metadata"]["vulnerabilities"].update(high=0, total=0)),
            ("unknown-severity", lambda v: v["vulnerabilities"]["fixture-package"].update(severity="unknown")),
            ("unknown-node", lambda v: v["vulnerabilities"]["fixture-package"].update(nodes=["node_modules/other"])),
            ("bool-count", lambda v: v["metadata"]["vulnerabilities"].update(high=True))]:
        value = report("high")
        mutate(value)
        case(name, lambda v=value: gate.check_report(response(v), packages), "NPM_REPORT_INVALID")
    for name, blob, code in [
            ("malformed-json", b"{", "NPM_JSON_INVALID"),
            ("duplicate-key", b'{"auditReportVersion":2,"auditReportVersion":2}', "NPM_DUPLICATE_KEY"),
            ("nan-json", b'{"x":NaN}', "NPM_JSON_INVALID"),
            ("empty-report", b"", "NPM_REPORT_INVALID")]:
        case(name, lambda b=blob: gate.check_report(SimpleNamespace(returncode=0, stdout=b), packages), code)

    with patch.dict(os.environ, {"npm_config_audit_level": "none", "NPM_CONFIG_OMIT": "dev",
                                "NODE_OPTIONS": "--require=secret", "NODE_ENV": "production",
                                "NODE_AUTH_TOKEN": "fixture-token", "NPM_TOKEN": "fixture-token"}):
        env = gate.process_env()
        assert not any(k.upper().startswith("NPM_CONFIG_") for k in env)
        assert not any(k in env for k in ("NODE_OPTIONS", "NODE_ENV", "NODE_AUTH_TOKEN", "NPM_TOKEN"))
    cases.append("environment-isolation")
    with patch.object(gate.subprocess, "run", side_effect=subprocess.TimeoutExpired("fixture", 120)):
        case("timeout", lambda: gate.invoke(["fixture"]), "NPM_PROCESS_FAILED")
    with patch.object(gate.subprocess, "run", side_effect=OSError("fixture")):
        case("process-error", lambda: gate.invoke(["fixture"]), "NPM_PROCESS_FAILED")
    with patch.object(gate.shutil, "which", return_value=None):
        case("missing-node", gate.runtime, "NPM_NODE_MISSING")
    node, cli = gate.runtime()
    with patch.object(gate, "invoke", return_value=SimpleNamespace(returncode=0, stdout=b"v24.19.0")):
        case("wrong-node-version", gate.runtime, "NPM_NODE_VERSION_INVALID")
    with patch.object(gate, "invoke", side_effect=[
            SimpleNamespace(returncode=0, stdout=b"v24.21.0"),
            SimpleNamespace(returncode=0, stdout=b"11.17.0")]):
        case("wrong-npm-version", gate.runtime, "NPM_VERSION_INVALID")

    gate.safe_path(FIXTURES)
    assert not FIXTURES.exists()
    FIXTURES.mkdir(parents=True)
    source = FIXTURES / "source"
    source.mkdir()
    manifest = {"name": "fixture-app", "version": "1.0.0", "private": True,
                "devDependencies": {"fixture-package": "1.0.0"},
                "optionalDependencies": {"fixture-optional": "1.0.0"},
                "scripts": {"prepare": "node -e \"require('fs').writeFileSync('SCRIPT_RAN','bad')\""}}
    lock = {"name": manifest["name"], "version": manifest["version"], "lockfileVersion": 3,
            "packages": {"": manifest,
                         "node_modules/fixture-package": {"version": "1.0.0", "dev": True,
                             "resolved": gate.REGISTRY + "fixture-package/-/fixture-package-1.0.0.tgz"},
                         "node_modules/fixture-optional": {"version": "1.0.0", "optional": True,
                             "resolved": gate.REGISTRY + "fixture-optional/-/fixture-optional-1.0.0.tgz"}}}

    def write_inputs(value=lock):
        (source / "package.json").write_text(json.dumps(manifest), encoding="utf-8")
        (source / "package-lock.json").write_text(json.dumps(value), encoding="utf-8")

    state = {"severity": None, "requests": [], "payloads": []}

    class Registry(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def send_json(self, value, status=200):
            blob = json.dumps(value).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(blob)))
            self.end_headers()
            self.wfile.write(blob)

        def do_POST(self):
            state["requests"].append(self.path)
            payload = self.rfile.read(int(self.headers["Content-Length"]))
            if self.headers.get("Content-Encoding") == "gzip":
                payload = gzip.decompress(payload)
            state["payloads"].append(json.loads(payload))
            if self.path != "/-/npm/v1/security/advisories/bulk":
                self.send_json({"error": "unexpected fixture endpoint"}, 500)
                return
            severity = state["severity"]
            value = {"fixture-package": [{"id": 999999999, "name": "fixture-package",
                     "title": "Synthetic Millani fixture", "url": "https://example.invalid/fixture",
                     "severity": severity, "vulnerable_versions": "<2.0.0",
                     "cwe": [], "cvss": {"score": 0, "vectorString": None}}]} if severity else {}
            self.send_json(value)

        def do_GET(self):
            state["requests"].append(self.path)
            name = self.path.lstrip("/")
            if name not in ("fixture-package", "fixture-optional"):
                self.send_json({"error": "unexpected fixture endpoint"}, 500)
                return
            self.send_json({"name": name, "versions": {
                version: {"name": name, "version": version} for version in ("1.0.0", "2.0.0")},
                "dist-tags": {"latest": "2.0.0"}})

    server = None
    try:
        write_inputs()
        (source / ".npmrc").write_text("audit=false\nomit=dev\nregistry=http://127.0.0.1:1\n", encoding="utf-8")
        original = gate.inputs(source)[0]
        server = ThreadingHTTPServer(("127.0.0.1", 0), Registry)
        worker = threading.Thread(target=server.serve_forever, daemon=True)
        worker.start()
        base_invoke = gate.invoke

        def loopback(args, cwd=gate.ROOT):
            if "audit" not in args:
                return base_invoke(args, cwd)
            assert str(node) == args[0] and str(cli) == args[1]
            assert "--audit-level=high" in args and "--package-lock-only" in args and "--ignore-scripts" in args
            assert all("--include=" + kind in args for kind in ("dev", "optional", "peer"))
            assert "--registry=" + gate.REGISTRY in args
            args = [("--registry=http://127.0.0.1:" + str(server.server_port) + "/")
                    if a.startswith("--registry=") else a for a in args]
            result = base_invoke(args, cwd)
            assert not (cwd / "SCRIPT_RAN").exists()
            return result

        with patch.object(gate, "invoke", side_effect=loopback), patch.dict(os.environ, {
                "npm_config_registry": "http://127.0.0.1:1", "npm_config_audit": "false",
                "npm_config_omit": "dev", "NODE_ENV": "production"}):
            for severity in (None, "moderate", "high", "critical"):
                state["severity"] = severity
                case("real-cli-" + str(severity), lambda: gate.scan(source),
                     "NPM_HIGH_CRITICAL_FOUND" if severity in ("high", "critical") else None)
                assert not gate.SCRATCH.exists()
        assert len(state["payloads"]) == 4
        assert all(p == {"fixture-package": ["1.0.0"], "fixture-optional": ["1.0.0"]}
                   for p in state["payloads"]), state["payloads"]
        assert gate.inputs(source)[0] == original and not (source / "SCRIPT_RAN").exists()
        cases.append("real-cli-inventory-env-npmrc-no-scripts-lock-preserved")
        (source / "package-lock.json").unlink()
        case("missing-lock", lambda: gate.inputs(source), "NPM_INPUT_INVALID")
        write_inputs()
        broken = deepcopy(lock)
        broken["packages"][""]["name"] = "different"
        (source / "package-lock.json").write_text(json.dumps(broken), encoding="utf-8")
        case("manifest-lock-mismatch", lambda: gate.inputs(source), "NPM_MANIFEST_LOCK_MISMATCH")
        write_inputs()
        broken = deepcopy(lock)
        broken["packages"]["node_modules/fixture-package"]["resolved"] = "file:///outside"
        (source / "package-lock.json").write_text(json.dumps(broken), encoding="utf-8")
        case("non-registry-package", lambda: gate.inputs(source), "NPM_PACKAGE_INVALID")
        write_inputs()
        (source / "package-lock.json").write_text("{", encoding="utf-8")
        case("malformed-lock", lambda: gate.inputs(source), "NPM_JSON_INVALID")
        write_inputs()
        target, link = FIXTURES / "target", FIXTURES / "link"
        target.mkdir()
        (target / "sentinel").write_text("preserve", encoding="utf-8")
        result = subprocess.run(["rtk", "proxy", "cmd", "/c", "mklink", "/J", str(link), str(target)], capture_output=True)
        assert result.returncode == 0 and link.is_junction()
        try:
            case("real-junction-input", lambda: gate.inputs(link), "NPM_REPARSE_DENIED")
            assert (target / "sentinel").read_text() == "preserve"
        finally:
            assert link.absolute().is_relative_to(FIXTURES) and link.is_junction() and link.resolve() == target.resolve()
            link.rmdir()
        with gate.scratch():
            case("existing-scratch", lambda: gate.scan(source), "NPM_SCRATCH_EXISTS")
        assert not gate.SCRATCH.exists()
    finally:
        if server:
            server.shutdown()
            server.server_close()
            worker.join(timeout=5)
        assert FIXTURES.resolve().is_relative_to(gate.ROOT / "artifacts")
        for path in [FIXTURES, *FIXTURES.rglob("*")]:
            gate.safe_path(path)
        shutil.rmtree(FIXTURES)
    print("NPM_AUDIT_TESTS " + json.dumps({"cases": len(cases), "status": "pass",
                                         "realCLI": 4, "elapsedSeconds": round(time.perf_counter() - start, 3)}))


if __name__ == "__main__":
    main()
