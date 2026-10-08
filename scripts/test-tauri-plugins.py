"""Synthetic behavioral checks; no plugin installation or product compilation."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_tauri_plugins import GateError, check

BASE = {
    "src-tauri/Cargo.toml": '[dependencies]\ntauri = "=2.12.1"\n',
    "src-tauri/Cargo.lock": 'version = 4\n[[package]]\nname = "tauri"\nversion = "2.12.1"\n',
    "package.json": '{"dependencies":{"react":"19.3.0"}}',
    "package-lock.json": '{"packages":{}}',
    "src-tauri/tauri.conf.json": '{"app":{"security":{"capabilities":[{"permissions":[]}]}}}',
    "src-tauri/src/main.rs": "fn main() { tauri::Builder::default(); }",
    "docs/security/TAURI_PLUGIN_REGISTER.json": "{}",
}
ADR = "docs/architecture/adr/ADR-023-dialog.md"
ACCEPTED = "# ADR-023 — dialog\nStatus: accepted\nPlugin-package: tauri-plugin-dialog\nPlugin-package: @tauri-apps/plugin-dialog\n"
CASES = []
def case(label, changes, allowed=False):
    CASES.append((label, {**BASE, **changes}, allowed))

case("empty", {}, True)
case("missing-source", {"src-tauri/src/main.rs": None})
case("unapproved", {"src-tauri/Cargo.toml": '[dependencies]\ntauri-plugin-dialog = "2"\n'})
case("alias-target", {"src-tauri/Cargo.toml": '[target."cfg(windows)".dependencies]\nchooser = { package = "tauri-plugin-dialog", version = "2" }\n'})
case("build", {"src-tauri/Cargo.toml": '[build-dependencies]\ntauri-plugin-dialog = "2"\n'})
case("dev", {"src-tauri/Cargo.toml": '[dev-dependencies]\ntauri-plugin-dialog = "2"\n'})
case("rust-transitive", {"src-tauri/Cargo.lock": '[[package]]\nname="tauri-plugin-dialog"\nversion="2"\n'})
case("npm-direct", {"package.json": '{"dependencies":{"@tauri-apps/plugin-dialog":"2"}}'})
case("npm-alias", {"package.json": '{"optionalDependencies":{"chooser":"npm:@tauri-apps/plugin-dialog@2"}}'})
case("npm-thirdparty", {"package.json": '{"devDependencies":{"@vendor/plugin-dialog":"2"}}'})
case("npm-transitive", {"package-lock.json": '{"packages":{"node_modules/@tauri-apps/plugin-dialog":{"version":"2"}}}'})
case("config", {"src-tauri/tauri.conf.json": '{"plugins":{"dialog":{}}}'})
case("grant", {"src-tauri/tauri.conf.json": '{"app":{"security":{"capabilities":[{"permissions":["dialog:allow-open"]}]}}}'})
case("registration", {"src-tauri/src/main.rs": "fn main(){ b.plugin(tauri_plugin_dialog::init()); }"})
case("dynamic", {"src-tauri/src/main.rs": "fn main(){ b.plugin(something); }"})
case("batch", {"src-tauri/src/main.rs": "fn main(){ b.plugins(list); }"})
case("missing", {"package.json": None})
case("bad-json", {"package.json": "{"})
case("duplicate", {"docs/security/TAURI_PLUGIN_REGISTER.json": '{"tauri-plugin-dialog":"a","tauri-plugin-dialog":"b"}'})
approved = {"src-tauri/Cargo.toml": '[dependencies]\nchooser={package="tauri-plugin-dialog", version="2"}\n',
            "src-tauri/src/main.rs": "fn main(){ b.plugin(tauri_plugin_dialog::init()); }",
            "docs/security/TAURI_PLUGIN_REGISTER.json": json.dumps({"tauri-plugin-dialog": ADR}), ADR: ACCEPTED}
case("approved-specific", approved, True)
for label, text in [("generic", "# ADR-023\nStatus: accepted\n"), ("pending", ACCEPTED.replace("accepted", "pending")), ("mismatch", ACCEPTED.replace("tauri-plugin-dialog", "tauri-plugin-shell"))]:
    case(label, {**approved, ADR: text})
case("missing-adr", {**approved, ADR: None})
case("traversal", {**approved, "docs/security/TAURI_PLUGIN_REGISTER.json": '{"tauri-plugin-dialog":"../../outside.md"}'})
case("unused-invalid", {"docs/security/TAURI_PLUGIN_REGISTER.json": '{"tauri-plugin-dialog":"docs/architecture/adr/ADR-023-dialog.md"}'})
case("npm-approved", {**approved, "package.json": '{"dependencies":{"@tauri-apps/plugin-dialog":"2"}}', "docs/security/TAURI_PLUGIN_REGISTER.json": json.dumps({"tauri-plugin-dialog": ADR, "@tauri-apps/plugin-dialog": ADR})}, True)
case("invalid-register", {"docs/security/TAURI_PLUGIN_REGISTER.json": "[]"})
case("bad-toml", {"src-tauri/Cargo.toml": "[bad"})
case("core-grant", {"src-tauri/tauri.conf.json": '{"app":{"security":{"capabilities":[{"permissions":["core:window:allow-set-title"]}]}}}'}, True)

case("approved-whitespace", {**approved, "src-tauri/src/main.rs": "fn main(){ b.plugin (tauri_plugin_dialog::init()); }"}, True)
case("workspace-root", {"Cargo.toml": '[workspace.dependencies]\ntauri-plugin-dialog="2"\n'})
case("capability-file", {"src-tauri/capabilities/additional.json": '{"permissions":["dialog:allow-open"]}'})
case("bad-identity", {"docs/security/TAURI_PLUGIN_REGISTER.json": json.dumps({"":"docs/architecture/adr/ADR-023-dialog.md"}), ADR: "# ADR-023\nStatus: accepted\nPlugin-package: \n"})

def guard(path, parent):
    if not path.is_relative_to(parent) or path == parent:
        raise RuntimeError("Fixture target invalid")
    for ancestor in (path, *path.parents):
        if ancestor.exists() and (ancestor.is_symlink() or ancestor.is_junction()):
            raise RuntimeError("Fixture reparse path")

def main():
    parent = Path(__file__).resolve().parent.parent
    artifacts = parent / "artifacts"
    guard(artifacts, parent)
    artifacts.mkdir(exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix="sec005-", dir=artifacts)).resolve()
    guard(root, artifacts.resolve())
    try:
        for i, (label, files, allowed) in enumerate(CASES):
            fixture = root / str(i)
            fixture.mkdir()
            for name, content in files.items():
                if content is None:
                    continue
                target = fixture / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(content, encoding="utf-8")
            try:
                check(fixture)
            except GateError as error:
                assert not allowed, label
                if label in {"unapproved", "alias-target", "build", "dev", "rust-transitive", "npm-direct", "npm-alias", "npm-thirdparty", "npm-transitive", "config", "grant", "registration", "workspace-root", "capability-file"}:
                    assert str(error) == "PLUGIN_WITHOUT_SPECIFIC_ADR", (label, str(error))
            else:
                assert allowed, label
        for i, expected in [(0, 0), (2, 1)]:
            fixture = root / str(i)
            scripts = fixture / "scripts"
            scripts.mkdir()
            shutil.copyfile(Path(__file__).with_name("check_tauri_plugins.py"), scripts / "check_tauri_plugins.py")
            result = subprocess.run([sys.executable, "-B", str(scripts / "check_tauri_plugins.py")], capture_output=True, text=True)
            assert result.returncode == expected, "CLI exit code"
            if expected:
                assert result.stderr.strip() == "FAIL: PLUGIN_WITHOUT_SPECIFIC_ADR"
        print(json.dumps({"fixtures": len(CASES), "cliCases": 2, "status": "pass", "python": sys.version.split()[0]}))
    finally:
        guard(root, artifacts.resolve())
        shutil.rmtree(root)
    assert not root.exists()

if __name__ == "__main__":
    main()
