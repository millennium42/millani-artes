"""Offline proof of the actual audit step bodies and pinned bootstrap."""
import hashlib
from io import BytesIO
import json
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import time
from types import SimpleNamespace
from unittest.mock import patch
import zipfile
import check_cargo_audit as gate

if sys.flags.optimize:
    raise SystemExit("CI_AUDIT_TESTS_BLOCKED OPTIMIZE_DENIED")
FIXTURES = gate.ROOT / "artifacts/sec012-tests"
NAMES = ["Validate project map against checkout", "Validate work item schema and evidence states",
         "Validate append-only progress log", "Require canonical documentation",
         "Scan Git history for secrets", "Scan lockfiles for vulnerabilities",
         "Test frontend check selection", "Select frontend checks", "Set up pinned Node",
         "Audit npm dependencies", "Audit Cargo dependencies", "Install locked frontend dependencies",
         "Check frontend quality", "Validate coverage reports", "Upload frontend coverage",
         "Prepare pinned Rust", "Build Tauri production", "Verify Tauri executable"]
CHILDREN = {"Audit npm dependencies": ["test-npm-audit.py", "check_npm_audit.py"],
            "Audit Cargo dependencies": ["prepare_cargo_audit.py", "test-ci-audits.py",
                                        "test-cargo-audit.py", "check_cargo_audit.py"]}
MESSAGES = ["npm audit fixtures failed.", "npm audit gate failed.", "Cargo audit preparation failed.",
            "CI audit fixtures failed.", "Cargo audit fixtures failed.", "Cargo audit gate failed."]


def workflow(text):
    # ponytail: validate this workflow's restricted shape; use a YAML parser if its structure grows.
    header = "jobs:\n  validate:\n    runs-on: windows-latest\n    timeout-minutes: 15\n    steps:\n"
    assert text.count(header) == 1 and len(re.findall(r"(?m)^jobs:", text)) == 1, "CI job shape"
    tail = text.split(header)[1]
    blocks = re.split(r"(?m)^      - ", tail)
    assert not blocks[0].strip() and len(blocks) == 20, "CI step count"
    assert re.findall(r"(?m)^  [^ ]", tail) == [], "CI extra job"
    names = [re.match(r"name: ([^\n]+)", block).group(1) for block in blocks[2:]]
    assert names == NAMES, "CI audit ordering"
    assert "continue-on-error" not in text, "CI soft failure"
    steps = dict(zip(names, blocks[2:]))
    node = steps["Set up pinned Node"]
    assert node.strip() == """name: Set up pinned Node
        uses: actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38 # v6
        with:
          node-version-file: .node-version
          cache: ${{ steps.frontend.outputs.required == 'true' && 'npm' || '' }}
          cache-dependency-path: package-lock.json
          package-manager-cache: false""", "CI Node setup"
    assert text.count("if: steps.frontend.outputs.required == 'true'") == 7, "CI build selection"
    assert re.findall(r"uses: (actions/[^\s]+)", text) == [
        "actions/checkout@11d5960a326750d5838078e36cf38b85af677262",
        "actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38",
        "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a"], "CI pinned actions"
    bodies, index = {}, 0
    for name, children in CHILDREN.items():
        body = []
        for child in children:
            body.extend(["python -B scripts/" + child,
                         "if ($LASTEXITCODE -ne 0) { throw '" + MESSAGES[index] + "' }"])
            index += 1
        expected = "name: " + name + "\n        shell: pwsh\n        run: |\n"
        expected += "\n".join("          " + line for line in body)
        assert steps[name].strip() == expected.strip(), "CI mandatory audit body"
        bodies[name] = "\n".join(line[10:] for line in steps[name].split("run: |\n", 1)[1].splitlines()
                                 if line.strip())
    return bodies


def remove_readonly(func, path, error):
    file = Path(path)
    assert file.absolute().is_relative_to(FIXTURES) and file.is_file()
    gate.safe_path(file)
    file.chmod(stat.S_IWRITE)
    func(path)


def main():
    start, cases = time.perf_counter(), []

    def case(name, action, code=None):
        try:
            action()
        except gate.GateError as error:
            assert str(error) == code, (name, str(error), code)
        else:
            assert code is None, name + " unexpectedly accepted"
        cases.append(name)

    text = (gate.ROOT / ".github/workflows/docs.yml").read_text(encoding="utf-8")
    bodies = workflow(text)
    cases.append("actual-workflow")
    import prepare_cargo_audit as prep

    optimized = subprocess.run([sys.executable, "-O", "-B", str(Path(__file__).resolve())], capture_output=True)
    assert optimized.returncode == 1 and b"OPTIMIZE_DENIED" in optimized.stderr
    cases.append("optimized-tests-denied")
    mutations = [text.replace("    timeout-minutes: 15", "    continue-on-error: true\n    timeout-minutes: 15"),
                 text.replace("      - name: Set up pinned Node\n", "      - name: Set up pinned Node\n        if: false\n")]
    for name in CHILDREN:
        for setting in ("if: steps.frontend.outputs.required == 'true'", "if: always()", "continue-on-error: true"):
            mutations.append(text.replace("name: " + name + "\n", "name: " + name + "\n        " + setting + "\n"))
        mutations.append(text.replace("python -B scripts/" + CHILDREN[name][-1], "Write-Output 'success'"))
    mutations.append(text.replace("name: Audit Cargo dependencies", "name: Build Tauri production")
                     .replace("name: Build Tauri production\n        if:", "name: Audit Cargo dependencies\n        if:"))
    for i, mutant in enumerate(mutations):
        try:
            workflow(mutant)
        except (AssertionError, AttributeError):
            cases.append("workflow-bypass-denied-" + str(i))
        else:
            raise AssertionError("workflow bypass accepted")

    blob = prep.ZIP.read_bytes()
    assert len(blob) == prep.ZIP_BYTES and hashlib.sha256(blob).hexdigest() == prep.ZIP_SHA
    gate.safe_path(FIXTURES)
    assert not FIXTURES.exists()
    FIXTURES.mkdir(parents=True)
    try:
        pwsh = shutil.which("pwsh")
        assert pwsh
        gate.safe_path(Path(pwsh).absolute())
        process_cases = 0
        for name, children in CHILDREN.items():
            faults = [(None, 0)] + [(child, code) for child in children for code in (1, 2)]
            for failing, exit_code in faults:
                folder = FIXTURES / ("process-" + str(process_cases))
                (folder / "scripts").mkdir(parents=True)
                for child in children:
                    (folder / "scripts" / child).write_text(
                        "from pathlib import Path\nimport json,sys\n"
                        "name=Path(__file__).name\n"
                        "with Path('order.log').open('a') as f:f.write(name+'\\n')\n"
                        "sys.exit(json.loads(Path('exits.json').read_text()).get(name,0))\n", encoding="utf-8")
                (folder / "exits.json").write_text(json.dumps({failing: exit_code} if failing else {}), encoding="utf-8")
                result = gate.invoke([pwsh, "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
                                      bodies[name] + "\n[IO.File]::WriteAllText('build.marker','build')"], folder)
                order = (folder / "order.log").read_text().splitlines()
                expected = children[:children.index(failing) + 1] if failing else children
                assert order == expected and ((result.returncode == 0) == (failing is None))
                assert (folder / "build.marker").exists() == (failing is None)
                cases.append("powershell-" + str(process_cases))
                process_cases += 1

        repository, db = FIXTURES / "source-db", FIXTURES / "advisory-db"
        repository.mkdir()
        (repository / "README.md").write_text("own offline Git fixture", encoding="utf-8")
        for args in (["git", "init", str(repository)], ["git", "-C", str(repository), "add", "README.md"],
                     ["git", "-C", str(repository), "-c", "user.name=SEC012 fixture",
                      "-c", "user.email=fixture@example.invalid", "commit", "-m", "own fixture"]):
            assert gate.invoke(args).returncode == 0
        archive, executable = FIXTURES / "cargo.zip", FIXTURES / "runtime/cargo-audit.exe"
        original_invoke = gate.invoke
        clones, downloads = [], []

        def offline_invoke(args, cwd=gate.ROOT):
            if args[:2] == ["git", "clone"]:
                assert args == ["git", "clone", gate.DB_URL, str(db)]
                clones.append(args)
                result = original_invoke(["git", "clone", str(repository), str(db)], cwd)
                assert result.returncode == 0
                assert original_invoke(["git", "-C", str(db), "remote", "set-url", "origin", gate.DB_URL]).returncode == 0
                return result
            return original_invoke(args, cwd)

        def offline_download(url, timeout):
            assert url == prep.ZIP_URL and timeout == 30
            downloads.append(url)
            return BytesIO(blob)

        with patch.object(prep, "ZIP", archive), patch.object(gate, "EXE", executable), \
             patch.object(gate, "DB", db), patch.object(gate, "invoke", side_effect=offline_invoke), \
             patch.object(prep.urllib.request, "urlopen", side_effect=offline_download):
            case("cold-bootstrap", prep.prepare)
            assert len(downloads) == len(clones) == 1
            case("warm-bootstrap", prep.prepare)
            assert len(downloads) == len(clones) == 1
            archive.write_bytes(blob[:-1])
            case("short-archive", prep.prepare, "CI_AUDIT_ZIP_INVALID")
            archive.write_bytes(blob[:-1] + bytes([blob[-1] ^ 1]))
            case("corrupt-archive", prep.prepare, "CI_AUDIT_ZIP_INVALID")
            archive.write_bytes(blob + b"x")
            case("oversize-archive", prep.prepare, "CI_AUDIT_ZIP_INVALID")
            archive.write_bytes(blob)
            for member in ("../escape.txt", "unexpected.txt"):
                altered = BytesIO(blob)
                with zipfile.ZipFile(altered, "a") as zipped:
                    zipped.writestr(member, b"own fixture")
                archive.write_bytes(altered.getvalue())
                with patch.object(prep, "ZIP_SHA", hashlib.sha256(altered.getvalue()).hexdigest()), \
                     patch.object(prep, "ZIP_BYTES", len(altered.getvalue())):
                    case("archive-member-" + member, prep.prepare, "CI_AUDIT_ZIP_CONTENT_INVALID")
                assert not (FIXTURES / "escape.txt").exists()
            archive.write_bytes(blob)
            extra = executable.parent / "unexpected.txt"
            extra.write_text("own fixture", encoding="utf-8")
            case("extra-runtime", prep.prepare, "CI_AUDIT_RUNTIME_INVALID")
            extra.unlink()
            license_file = executable.parent / "LICENSE-MIT"
            license_blob = license_file.read_bytes()
            license_file.unlink()
            case("missing-license", prep.prepare, "CI_AUDIT_RUNTIME_INVALID")
            license_file.write_bytes(license_blob)
            binary = executable.read_bytes()
            executable.write_bytes(binary[:-1])
            case("altered-runtime", prep.prepare, "CI_AUDIT_RUNTIME_INVALID")
            executable.write_bytes(binary)
            assert original_invoke(["git", "-C", str(db), "remote", "set-url", "origin", "https://example.invalid/db"]).returncode == 0
            case("wrong-db-origin", prep.prepare, "CARGO_DATABASE_ORIGIN_INVALID")
            assert original_invoke(["git", "-C", str(db), "remote", "set-url", "origin", gate.DB_URL]).returncode == 0
            (db / "unexpected.txt").write_text("own fixture", encoding="utf-8")
            case("dirty-db", prep.prepare, "CARGO_DATABASE_DIRTY")
            (db / "unexpected.txt").unlink()
            with patch.object(gate, "DB", FIXTURES / "missing-db"), \
                 patch.object(gate, "invoke", side_effect=lambda args, cwd=gate.ROOT:
                              SimpleNamespace(returncode=2) if args[:2] == ["git", "clone"]
                              else original_invoke(args, cwd)):
                case("clone-failed", prep.prepare, "CI_AUDIT_DB_CLONE_FAILED")
            link = FIXTURES / "linked-runtime"
            result = subprocess.run(["cmd", "/c", "mklink", "/J", str(link), str(executable.parent)], capture_output=True)
            assert result.returncode == 0 and link.is_junction()
            try:
                with patch.object(gate, "EXE", link / "cargo-audit.exe"):
                    case("runtime-junction", prep.prepare, "CARGO_REPARSE_DENIED")
                assert executable.read_bytes() == binary
            finally:
                assert link.absolute().is_relative_to(FIXTURES) and link.is_junction()
                assert link.resolve() == executable.parent.resolve()
                link.rmdir()
            case("warm-after-negatives", prep.prepare)
        assert not gate.SCRATCH.exists()
    finally:
        assert FIXTURES.absolute().is_relative_to(gate.ROOT / "artifacts")
        gate.safe_path(FIXTURES)
        assert FIXTURES.resolve() == FIXTURES.absolute()
        for path in FIXTURES.rglob("*"):
            gate.safe_path(path)
        shutil.rmtree(FIXTURES, onexc=remove_readonly)
    print("CI_AUDIT_TESTS " + json.dumps({"cases": len(cases), "powershell": process_cases,
          "bootstrap": "cold/warm/offline", "status": "pass", "fixtureRemoved": not FIXTURES.exists(),
          "seconds": round(time.perf_counter() - start, 3)}))


if __name__ == "__main__":
    main()
