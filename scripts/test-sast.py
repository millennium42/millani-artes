"""Real Biome fixtures for the current frontend configuration, without executing fixture code."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

import check_npm_audit as npm

if sys.flags.optimize:
    raise SystemExit("SAST_ASSERTIONS_REQUIRED")

ROOT = npm.ROOT
RULE_HTML = "lint/security/noDangerouslySetInnerHtml"
RULE_EVAL = "lint/security/noGlobalEval"
QUALITY_PREFIX = """          python -B scripts/test-sast.py
          if ($LASTEXITCODE -ne 0) { throw 'SAST fixtures failed.' }
          npm run quality
"""


def main():
    started = time.monotonic()
    node, _ = npm.runtime()
    cli = ROOT / "node_modules/@biomejs/biome/bin/biome"
    for path in (cli, ROOT / "biome.json", ROOT / "package.json"):
        npm.safe_path(path)
        assert path.is_file(), "SAST_INPUT_MISSING"
    config_blob = (ROOT / "biome.json").read_bytes()
    manifest_blob = (ROOT / "package.json").read_bytes()
    manifest = npm.read_json(manifest_blob)
    assert manifest["devDependencies"]["@biomejs/biome"] == "2.5.15", "SAST_PIN_CHANGED"
    version = npm.invoke([str(node), str(cli), "--version"])
    assert version.returncode == 0 and version.stdout.strip() == b"Version: 2.5.15", "SAST_VERSION_INVALID"
    env = {k: v for k, v in npm.process_env().items() if not k.upper().startswith("BIOME_")}
    parent = ROOT / "artifacts"
    npm.safe_path(parent)
    parent.mkdir(exist_ok=True)
    scratch = Path(tempfile.mkdtemp(prefix="sec013-sast-", dir=parent))
    npm.safe_path(scratch)
    assert scratch.resolve().is_relative_to(parent.resolve()), "SAST_SCRATCH_DENIED"
    count = 0
    try:
        (scratch / "biome.json").write_bytes(config_blob)
        (scratch / "package.json").write_bytes(manifest_blob)
        (scratch / "src").mkdir()
        # ponytail: pattern checks only; re-evaluate SAST when Rust/SQL/restore inputs exist.
        cases = [
            ("jsx.tsx", 'export const App = () => <div dangerouslySetInnerHTML={{ __html: "fixture" }} />;\n', RULE_HTML),
            ("create.ts", 'import React from "react";\nexport const App = React.createElement("div", { dangerouslySetInnerHTML: { __html: "fixture" } });\n', RULE_HTML),
            ("eval.ts", 'export const value = eval("fixture");\n', RULE_EVAL),
            ("window.ts", 'export const value = window.eval("fixture");\n', RULE_EVAL),
            ("escaped.tsx", 'export const App = () => <p>{"fixture"}</p>;\n', None),
            ("comment.ts", '// eval("fixture"); dangerouslySetInnerHTML\nexport const value = 1;\n', None),
            ("alias.ts", 'const execute = eval;\nexport const value = execute("fixture");\n', RULE_EVAL),
            ("computed.ts", 'const key = "eval";\nexport const value = window[key]("fixture");\n', None),
            ("constructor.ts", 'export const value = new Function("return 1")();\n', None),
            ("parse.ts", 'export const value = ;\n', "parse"),
        ]

        def lint(path):
            return subprocess.run(
                [str(node), str(cli), "lint", "--error-on-warnings", "--reporter=json",
                 "--max-diagnostics=none", "--vcs-enabled=false", "--config-path=" + str(scratch), str(path)],
                cwd=scratch, env=env, capture_output=True, timeout=30)

        for name, source, category in cases:
            path = scratch / "src" / name
            path.write_text(source, encoding="utf-8", newline="\n")
            result = lint(path)
            report = npm.read_json(result.stdout)
            summary = report["summary"]
            assert summary["unchanged"] == 1 and summary["changed"] == 0, "SAST_EMPTY_OR_CHANGED"
            assert summary["diagnosticsNotPrinted"] == 0, "SAST_TRUNCATED"
            diagnostics = report["diagnostics"]
            if category:
                assert result.returncode != 0 and summary["errors"] == 1, name
                assert len(diagnostics) == 1 and diagnostics[0]["category"] == category, name
                assert diagnostics[0]["severity"] == "error", name
            else:
                assert result.returncode == 0 and summary["errors"] == 0, name
                assert not diagnostics, name
            assert path.read_text(encoding="utf-8") == source, "SAST_FIXTURE_CHANGED"
            count += 1

        excluded = scratch / "src-tauri" / "outside.ts"
        excluded.parent.mkdir()
        excluded.write_text('eval("fixture");\n', encoding="utf-8", newline="\n")
        result = lint(excluded)
        assert result.returncode != 0, "SAST_UNMATCHED_PASSED"
        report = npm.read_json(result.stdout)
        assert report["summary"]["unchanged"] == 0, "SAST_SCOPE_CHANGED"
        count += 1
        (scratch / "biome.json").write_text("{", encoding="utf-8")
        assert lint(scratch / "src" / "escaped.tsx").returncode != 0, "SAST_INVALID_CONFIG_PASSED"
        count += 1
    finally:
        for path in (scratch, *scratch.rglob("*")):
            npm.safe_path(path)
        shutil.rmtree(scratch)
    assert not scratch.exists(), "SAST_CLEANUP_FAILED"

    workflow = (ROOT / ".github/workflows/docs.yml").read_text(encoding="utf-8")
    start = workflow.index("      - name: Check frontend quality\n")
    end = workflow.index("      - name: Validate coverage reports\n", start)
    step = workflow[start:end]
    assert "        run: |\n" + QUALITY_PREFIX in step, "SAST_CI_GUARD_MISSING"
    assert step.count("        run:") == 1 and step.count("          python ") == 1, "SAST_CI_SHAPE_CHANGED"
    assert "continue-on-error" not in step, "SAST_CI_BYPASS"
    body = step.split("        run: |\n", 1)[1].strip()
    for exit_code in (0, 1, 2):
        fixture = (
            "$ErrorActionPreference = 'Stop'\n"
            "function python { $global:LASTEXITCODE = " + str(exit_code) + " }\n"
            "function npm { Write-Output 'SAST_DOWNSTREAM'; $global:LASTEXITCODE = 0 }\n"
            + body + "\n")
        result = subprocess.run(["pwsh", "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", fixture],
                                cwd=ROOT, env=env, capture_output=True, timeout=30)
        marker = b"SAST_DOWNSTREAM" in result.stdout
        assert (result.returncode == 0 and marker) if exit_code == 0 else (result.returncode != 0 and not marker), "SAST_FAILURE_NOT_PROPAGATED"
        count += 1
    sha = npm.invoke(["git", "rev-parse", "HEAD"])
    assert sha.returncode == 0, "SAST_SOURCE_SHA_FAILED"
    evidence = {"sourceSha": sha.stdout.decode().strip(), "biome": "2.5.15", "cases": count,
                "biomeCli": 12, "powershell": 3, "configSha256": hashlib.sha256(config_blob).hexdigest(),
                "fixtureVcsDisabledOnly": True, "scratchRemoved": True,
                "limitations": ["computed eval property", "Function constructor", "Rust/SQL/restore"],
                "elapsedSeconds": round(time.monotonic() - started, 3)}
    print("SAST_EVIDENCE " + json.dumps(evidence, separators=(",", ":")))


if __name__ == "__main__":
    try:
        main()
    except (AssertionError, npm.GateError, OSError, ValueError, KeyError, subprocess.TimeoutExpired):
        print("SAST fixture validation failed.", file=sys.stderr)
        sys.exit(1)
