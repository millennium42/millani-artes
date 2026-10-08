"""SEC-005: static plugin declarations require a specific accepted ADR."""
import json
from pathlib import Path
import re
import sys
import tomllib

class GateError(Exception):
    pass

def require(condition, code):
    if not condition:
        raise GateError(code)

def object_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "DUPLICATE_JSON_KEY")
        result[key] = value
    return result

def read_json(path):
    value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=object_pairs)
    require(isinstance(value, dict), "JSON_OBJECT_REQUIRED")
    return value

def plugin_package(name):
    return isinstance(name, str) and bool(re.fullmatch(
        r"(?:tauri[-_]plugin[-_][a-z0-9_-]+|@[^/]+/plugin-[a-z0-9-]+)", name))

def check(root):
    root = Path(root).resolve()
    try:
        register = read_json(root / "docs/security/TAURI_PLUGIN_REGISTER.json")
        for package, relative in register.items():
            require(plugin_package(package), "PLUGIN_PACKAGE_INVALID")
            require(isinstance(relative, str) and bool(re.fullmatch(
                r"docs/architecture/adr/ADR-[0-9]{3}-[a-z0-9-]+\.md", relative)), "ADR_PATH_INVALID")
            path = (root / relative).resolve()
            require(path.is_relative_to(root / "docs/architecture/adr"), "ADR_PATH_ESCAPE")
            text = path.read_text(encoding="utf-8")
            require(re.search(r"(?m)^Status: accepted\s*$", text) is not None, "ADR_NOT_ACCEPTED")
            require(re.search(r"(?m)^Plugin-package: " + re.escape(package) + r"\s*$", text) is not None, "ADR_PLUGIN_MISMATCH")
        detected = set()

        def add(name):
            if plugin_package(name):
                detected.add(name)

        def cargo_tables(table):
            require(isinstance(table, dict), "CARGO_TABLE_INVALID")
            for key, value in table.items():
                if key in ("dependencies", "build-dependencies", "dev-dependencies"):
                    require(isinstance(value, dict), "CARGO_DEPENDENCIES_INVALID")
                    for alias, declaration in value.items():
                        require(isinstance(declaration, (str, dict)), "CARGO_DEPENDENCY_INVALID")
                        add(alias)
                        if isinstance(declaration, dict):
                            add(declaration.get("package", alias))
                elif isinstance(value, dict):
                    cargo_tables(value)

        cargo_tables(tomllib.loads((root / "src-tauri/Cargo.toml").read_text(encoding="utf-8")))
        if (root / "Cargo.toml").exists():
            cargo_tables(tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8")))
        lock = tomllib.loads((root / "src-tauri/Cargo.lock").read_text(encoding="utf-8"))
        require(isinstance(lock.get("package"), list), "CARGO_LOCK_INVALID")
        for package in lock["package"]:
            require(isinstance(package, dict) and isinstance(package.get("name"), str), "CARGO_LOCK_PACKAGE_INVALID")
            add(package["name"])
        npm = read_json(root / "package.json")
        for section in ("dependencies", "devDependencies", "optionalDependencies", "peerDependencies"):
            declarations = npm.get(section, {})
            require(isinstance(declarations, dict), "NPM_DEPENDENCIES_INVALID")
            for name, version in declarations.items():
                add(name)
                require(isinstance(version, str), "NPM_DEPENDENCY_INVALID")
                if version.startswith("npm:"):
                    identity = version[4:]
                    add(("@" + identity.split("@")[1]) if identity.startswith("@") else identity.split("@")[0])
        lock = read_json(root / "package-lock.json")
        require(isinstance(lock.get("packages"), dict), "NPM_LOCK_INVALID")
        for location, package in lock["packages"].items():
            require(isinstance(package, dict), "NPM_LOCK_PACKAGE_INVALID")
            add(location.rsplit("node_modules/", 1)[-1])
            add(package.get("name"))

        def capability(value):
            require(isinstance(value, dict), "CAPABILITY_INVALID")
            permissions = value.get("permissions", [])
            require(isinstance(permissions, list), "PERMISSIONS_INVALID")
            for permission in permissions:
                name = permission.get("identifier") if isinstance(permission, dict) else permission
                require(isinstance(name, str), "PERMISSION_INVALID")
                prefix = name.split(":", 1)[0]
                if ":" in name and prefix != "core":
                    detected.add("tauri-plugin-" + prefix)

        config = read_json(root / "src-tauri/tauri.conf.json")
        plugins = config.get("plugins", {})
        require(isinstance(plugins, dict), "PLUGIN_CONFIG_INVALID")
        detected.update("tauri-plugin-" + name for name in plugins)
        for value in config.get("app", {}).get("security", {}).get("capabilities", []):
            if isinstance(value, str):
                path = root / "src-tauri/capabilities" / (value + ".json")
                require(path.resolve().is_relative_to(root / "src-tauri/capabilities"), "CAPABILITY_PATH_ESCAPE")
                capability(read_json(path))
            else:
                capability(value)
        for path in (root / "src-tauri/capabilities").glob("*"):
            if path.suffix == ".json":
                capability(read_json(path))
            elif path.suffix == ".toml":
                capability(tomllib.loads(path.read_text(encoding="utf-8")))

        sources = list((root / "src-tauri/src").rglob("*.rs"))
        require(sources, "RUST_SOURCE_MISSING")
        build = root / "src-tauri/build.rs"
        if build.exists():
            sources.append(build)
        for path in sources:
            text = path.read_text(encoding="utf-8")
            for call in re.finditer(r"\.plugins?\s*\(", text):
                rest = text[call.end():]
                name = re.match(r"\s*tauri_plugin_([a-z0-9_]+)\s*::", rest)
                require(name is not None and re.fullmatch(r"\.plugin\s*\(", call.group()) is not None, "PLUGIN_REGISTRATION_UNIDENTIFIED")
                detected.add("tauri-plugin-" + name[1].replace("_", "-"))
        require(detected <= register.keys(), "PLUGIN_WITHOUT_SPECIFIC_ADR")
        return {"plugins": len(detected), "adrRecords": len(register), "python": sys.version.split()[0]}
    except GateError:
        raise
    except (OSError, ValueError, TypeError, KeyError, AttributeError, IndexError):
        raise GateError("GATE_INPUT_INVALID") from None

def main():
    try:
        result = check(Path(__file__).resolve().parent.parent)
    except GateError as error:
        print("FAIL: " + str(error), file=sys.stderr)
        return 1
    print(json.dumps({"status": "pass", **result}))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
