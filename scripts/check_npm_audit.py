"""Local npm audit high/critical gate; no install, fix or project npmrc."""
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent.parent
SCRATCH = ROOT / "artifacts/sec010-check"
NPM_VERSION = "11.19.0"
REGISTRY = "https://registry.npmjs.org/"
SEVERITIES = ("info", "low", "moderate", "high", "critical")


class GateError(Exception):
    pass


def require(condition, code):
    if not condition:
        raise GateError(code)


def safe_path(path):
    for parent in [path, *path.parents]:
        require(not parent.is_symlink() and not parent.is_junction(), "NPM_REPARSE_DENIED")


def object_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "NPM_DUPLICATE_KEY")
        result[key] = value
    return result


def read_json(blob):
    try:
        return json.loads(blob, object_pairs_hook=object_pairs,
                          parse_constant=lambda _: require(False, "NPM_JSON_INVALID"))
    except (ValueError, UnicodeError) as error:
        raise GateError("NPM_JSON_INVALID") from error


def process_env():
    return {k: v for k, v in os.environ.items()
            if not k.upper().startswith(("NPM_CONFIG_", "GIT_", "GITLEAKS_", "OSV_"))
            and k.upper() not in {"NODE_OPTIONS", "NODE_PATH", "NODE_ENV",
                                  "NPM_TOKEN", "NODE_AUTH_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"}}


def invoke(args, cwd=ROOT):
    try:
        return subprocess.run(args, cwd=cwd, env=process_env(), capture_output=True, timeout=120)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise GateError("NPM_PROCESS_FAILED") from error


def runtime():
    selected = shutil.which("node.exe")
    require(selected, "NPM_NODE_MISSING")
    node = Path(selected).absolute()
    cli = node.parent / "node_modules/npm/bin/npm-cli.js"
    for path in (node, cli, ROOT / ".node-version"):
        safe_path(path)
        require(path.is_file(), "NPM_RUNTIME_MISSING")
    version = (ROOT / ".node-version").read_text().strip()
    require(version == "24.21.0", "NPM_PIN_CHANGED")
    result = invoke([str(node), "--version"])
    require(result.returncode == 0 and result.stdout.strip() == ("v" + version).encode(),
            "NPM_NODE_VERSION_INVALID")
    result = invoke([str(node), str(cli), "--version"])
    require(result.returncode == 0 and result.stdout.strip() == NPM_VERSION.encode(),
            "NPM_VERSION_INVALID")
    return node, cli


def inputs(source):
    safe_path(source)
    require(source.resolve().is_relative_to(ROOT), "NPM_SOURCE_DENIED")
    blobs = {}
    for name in ("package.json", "package-lock.json"):
        path = source / name
        safe_path(path)
        require(path.is_file() and 0 < path.stat().st_size <= 4000000, "NPM_INPUT_INVALID")
        blobs[name] = path.read_bytes()
    manifest, lock = (read_json(blob) for blob in blobs.values())
    require(isinstance(manifest, dict) and isinstance(lock, dict)
            and lock.get("lockfileVersion") == 3 and isinstance(lock.get("packages"), dict),
            "NPM_INPUT_INVALID")
    packages = lock["packages"]
    require(isinstance(packages.get(""), dict) and len(packages) > 1, "NPM_INPUT_INVALID")
    for field in ("name", "version", "dependencies", "devDependencies", "optionalDependencies", "peerDependencies"):
        require(manifest.get(field) == packages[""].get(field), "NPM_MANIFEST_LOCK_MISMATCH")
    for path, value in packages.items():
        if not path:
            continue
        require(path.startswith("node_modules/") and isinstance(value, dict)
                and not value.get("link") and isinstance(value.get("version"), str)
                and value["version"] and isinstance(value.get("resolved"), str)
                and value["resolved"].startswith(REGISTRY), "NPM_PACKAGE_INVALID")
    return blobs, {path: value for path, value in packages.items() if path}


@contextmanager
def scratch():
    safe_path(SCRATCH)
    require(not SCRATCH.exists(), "NPM_SCRATCH_EXISTS")
    SCRATCH.mkdir(parents=True)
    try:
        yield SCRATCH
    finally:
        require(SCRATCH.resolve().is_relative_to(ROOT / "artifacts")
                and SCRATCH.name == "sec010-check", "NPM_CLEANUP_DENIED")
        for path in [SCRATCH, *SCRATCH.rglob("*")]:
            safe_path(path)
        shutil.rmtree(SCRATCH)


def check_report(result, packages):
    require(result.returncode in (0, 1), "NPM_SCAN_FAILED")
    require(0 < len(result.stdout) <= 4000000, "NPM_REPORT_INVALID")
    report = read_json(result.stdout)
    require(isinstance(report, dict) and "error" not in report
            and report.get("auditReportVersion") == 2
            and isinstance(report.get("vulnerabilities"), dict)
            and isinstance(report.get("metadata"), dict), "NPM_REPORT_INVALID")
    counts = {severity: 0 for severity in SEVERITIES}
    for name, vuln in report["vulnerabilities"].items():
        require(isinstance(vuln, dict) and vuln.get("name") == name
                and isinstance(vuln.get("severity"), str) and vuln["severity"] in counts and isinstance(vuln.get("nodes"), list)
                and vuln["nodes"] and all(isinstance(node, str) and node in packages for node in vuln["nodes"]),
                "NPM_REPORT_INVALID")
        counts[vuln["severity"]] += 1
    counts["total"] = sum(counts.values())
    metadata = report["metadata"]
    require(isinstance(metadata.get("vulnerabilities"), dict)
            and set(metadata["vulnerabilities"]) == {*SEVERITIES, "total"}
            and all(type(metadata["vulnerabilities"].get(k)) is int
                    and metadata["vulnerabilities"][k] == v for k, v in counts.items())
            and isinstance(metadata.get("dependencies"), dict)
            and type(metadata["dependencies"].get("total")) is int
            and metadata["dependencies"]["total"] == len(packages), "NPM_REPORT_INVALID")
    require(not counts["high"] and not counts["critical"], "NPM_HIGH_CRITICAL_FOUND")
    require(result.returncode == 0, "NPM_SCAN_FAILED")
    return counts


def scan(source=ROOT):
    node, cli = runtime()
    blobs, packages = inputs(source)
    with scratch() as work:
        for name, blob in blobs.items():
            (work / name).write_bytes(blob)
        for name in ("user.npmrc", "global.npmrc"):
            (work / name).write_text("", encoding="utf-8")
        args = [str(node), str(cli), "audit", "--package-lock-only", "--ignore-scripts",
                "--audit-level=high", "--json", "--include=dev", "--include=optional",
                "--include=peer", "--registry=" + REGISTRY, "--fetch-retries=0",
                "--fetch-timeout=15000", "--userconfig=" + str(work / "user.npmrc"),
                "--globalconfig=" + str(work / "global.npmrc"), "--cache=" + str(work / "cache"),
                "--logs-dir=" + str(work / "logs")]
        counts = check_report(invoke(args, work), packages)
    after, _ = inputs(source)
    require(after == blobs, "NPM_INPUT_CHANGED")
    return {"node": "24.21.0", "npm": NPM_VERSION, "auditLevel": "high",
            "dependencies": len(packages), "vulnerabilities": counts,
            "inputSHA256": {name: hashlib.sha256(blob).hexdigest() for name, blob in blobs.items()}}


def main():
    start = time.perf_counter()
    try:
        evidence = scan()
        result = invoke(["git", "rev-parse", "HEAD"])
        require(result.returncode == 0 and len(result.stdout.strip()) == 40, "NPM_SOURCE_SHA_INVALID")
        evidence.update({"sourceSha": result.stdout.decode("ascii").strip(),
                         "elapsedSeconds": round(time.perf_counter() - start, 3)})
        print("NPM_AUDIT_EVIDENCE " + json.dumps(evidence, sort_keys=True))
    except (GateError, OSError, ValueError, TypeError, KeyError) as error:
        code = str(error) if isinstance(error, GateError) else "NPM_GATE_FAILED"
        print("NPM_AUDIT_BLOCKED " + code)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
