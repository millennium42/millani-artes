"""Real Git fixtures for the Gitleaks gate; synthetic values never leave this process."""
import importlib.util
import json
import os
from pathlib import Path
import random
import shutil
import stat
import string
import subprocess
import sys
import time
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parent.parent
GATE = ROOT / "scripts/check_gitleaks.py"
FIXTURES = ROOT / "artifacts/sec007-tests"


def safe(path):
    for parent in [path, *path.parents]:
        if parent.exists() and (parent.is_symlink() or parent.is_junction()):
            raise RuntimeError("Fixture reparse point denied.")


def remove_readonly(func, name, error):
    path = Path(name)
    if not isinstance(error, PermissionError) or not path.resolve().is_relative_to(FIXTURES):
        raise error
    safe(path)
    path.chmod(path.stat().st_mode | stat.S_IWRITE)
    func(name)


def main():
    if not __debug__:
        raise RuntimeError("Assertions must be enabled.")
    safe(FIXTURES)
    if FIXTURES.exists():
        raise RuntimeError("Fixture directory already exists.")
    FIXTURES.mkdir(parents=True)
    results = []
    fake = "gh" + "p_" + "".join(random.Random(7007).choices(string.ascii_letters + string.digits, k=36))
    env = {k: v for k, v in os.environ.items() if not k.upper().startswith(("GIT_", "GITLEAKS_"))}
    repo = FIXTURES / "repo"
    repo.mkdir()

    def git(*args):
        result = subprocess.run(["git", "-C", str(repo), *args], env=env, capture_output=True, timeout=30)
        assert result.returncode == 0, "Fixture Git setup failed."

    def invoke(name, source=repo, expected_code=None, extra_env=None):
        start = time.perf_counter()
        result = subprocess.run([sys.executable, "-B", str(GATE), "--source", str(source)], env=env | (extra_env or {}),
                                capture_output=True, text=True, encoding="utf-8", timeout=180)
        assert fake not in result.stdout + result.stderr, "Synthetic value escaped gate output."
        if expected_code:
            assert result.returncode == 1 and json.loads(result.stderr)["code"] == expected_code, "Wrong rejection code."
        else:
            assert result.returncode == 0 and json.loads(result.stdout.split(" ", 1)[1])["findings"] == 0, "Clean fixture rejected."
        results.append({"name": name, "exit": result.returncode, "seconds": round(time.perf_counter() - start, 3)})
        return result

    try:
        git("init")
        git("config", "user.name", "Gate Fixture")
        git("config", "user.email", "gate-fixture@example.invalid")
        git("config", "core.hooksPath", str(repo / ".git/disabled-hooks"))
        (repo / "clean.txt").write_text("Installation gate fixture.\n", encoding="utf-8")
        git("add", "clean.txt")
        git("commit", "-m", "clean synthetic fixture")
        invoke("clean")
        (repo / "secret.txt").write_text("token = " + fake + " # gitleaks:allow\n", encoding="utf-8")
        git("add", "secret.txt")
        git("commit", "-m", "synthetic leak fixture")
        git("rm", "secret.txt")
        git("commit", "-m", "remove synthetic value from HEAD")
        result = invoke("historical-inline-allow-rejected", expected_code="GITLEAKS_FINDINGS")
        assert json.loads(result.stderr)["findings"] == 1, "Historical finding missing."
        hostile = repo / ".gitleaks.toml"
        hostile.write_text('[allowlist]\nregexes = [".*"]\n', encoding="utf-8")
        invoke("config-env-cannot-suppress", expected_code="GITLEAKS_FINDINGS",
               extra_env={"GITLEAKS_CONFIG": str(hostile), "GITLEAKS_CONFIG_TOML": hostile.read_text(encoding="utf-8")})
        hostile.unlink()
        ignore = repo / ".gitleaksignore"
        ignore.write_bytes(b"")
        invoke("source-ignore-denied", expected_code="GITLEAKS_IGNORE_DENIED")
        ignore.unlink()
        shallow = FIXTURES / "shallow"
        git("clone", "--depth", "1", repo.as_uri(), str(shallow))
        invoke("shallow-denied", shallow, "GITLEAKS_HISTORY_INCOMPLETE")
        empty = FIXTURES / "empty"
        git("init", str(empty))
        invoke("empty-denied", empty, "GITLEAKS_HISTORY_EMPTY")
        non_git = FIXTURES / "non-git"
        non_git.mkdir()
        invoke("non-git-denied", non_git, "GITLEAKS_SOURCE_INVALID")

        # Boundary negatives complement real Git/CLI cases; they do not prove network behavior.
        spec = importlib.util.spec_from_file_location("gitleaks_gate", GATE)
        gate = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(gate)

        def rejected(name, code, action):
            try:
                action()
            except gate.GateError as error:
                assert error.code == code, "Wrong boundary rejection."
            else:
                raise AssertionError("Unsafe boundary accepted.")
            results.append({"name": name, "layer": "boundary", "code": code})

        invalid = subprocess.run([str(gate.CACHE / "gitleaks.exe"), "--millani-invalid-flag"],
                                 env=env, capture_output=True, timeout=30)
        assert invalid.returncode != 0
        rejected("real-cli-error", "GITLEAKS_SCAN_FAILED", lambda: gate.check_report(invalid))
        rejected("unredacted-report", "GITLEAKS_REPORT_INVALID",
                 lambda: gate.check_report(SimpleNamespace(returncode=1, stdout=json.dumps([{"Secret": fake}]))))
        rejected("malformed-report", "GITLEAKS_REPORT_INVALID",
                 lambda: gate.check_report(SimpleNamespace(returncode=0, stdout=b"not json")))
        rejected("partial-scan", "GITLEAKS_SCAN_FAILED",
                 lambda: gate.check_report(SimpleNamespace(returncode=1, stdout=b"[]")))
        rejected("missing-process", "GITLEAKS_PROCESS_FAILED", lambda: gate.invoke([str(FIXTURES / "absent.exe")]))
        payload = (gate.CACHE / gate.ZIP_NAME).read_bytes() if (gate.CACHE / gate.ZIP_NAME).exists() else None
        if payload is not None:
            gate.unpack(payload)
        rejected("archive-integrity", "GITLEAKS_ARCHIVE_INVALID", lambda: gate.unpack(b"invalid archive"))
        files = {name: (gate.CACHE / name).read_bytes() for name in ["gitleaks.exe", "LICENSE", "README.md"]}
        rejected("binary-integrity", "GITLEAKS_BINARY_INVALID", lambda: gate.validate_files(files | {"gitleaks.exe": b"invalid"}))
        rejected("license-integrity", "GITLEAKS_LICENSE_INVALID", lambda: gate.validate_files(files | {"LICENSE": b"invalid"}))
        original_cache = gate.CACHE
        gate.CACHE = FIXTURES / "extra-cache"
        gate.CACHE.mkdir()
        try:
            for name, content in files.items():
                (gate.CACHE / name).write_bytes(content)
            (gate.CACHE / "unexpected.dll").write_bytes(b"fixture, never loaded")
            rejected("cache-extra-file", "GITLEAKS_PACKAGE_INVALID", gate.scanner)
        finally:
            gate.CACHE = original_cache
    finally:
        assert FIXTURES.resolve().is_relative_to(ROOT / "artifacts") and FIXTURES.name == "sec007-tests"
        safe(FIXTURES)
        for path in FIXTURES.rglob("*"):
            safe(path)
        shutil.rmtree(FIXTURES, onexc=remove_readonly)
    assert not FIXTURES.exists()
    print(json.dumps({"cases": len(results), "results": results, "status": "pass"}))


if __name__ == "__main__":
    main()
