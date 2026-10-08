"""Prepare the existing pinned Cargo audit runtime and official Git database."""
import hashlib
from io import BytesIO
import json
import os
from pathlib import Path
import re
import sys
import urllib.error
import urllib.request
import zipfile
import check_cargo_audit as gate

ZIP = Path(os.environ["LOCALAPPDATA"]) / "MillaniArtesDev/downloads/cargo-audit-0.22.2-win-x64.zip"
ZIP_URL = "https://github.com/rustsec/rustsec/releases/download/cargo-audit/v0.22.2/cargo-audit-x86_64-pc-windows-msvc-v0.22.2.zip"
ZIP_SHA = "0a7316540862c13d954f648917ceacca593747baed6eec180fafa590be2710ab"
ZIP_BYTES = 6192256
PREFIX = "cargo-audit-x86_64-pc-windows-msvc-v0.22.2/"
FILES = {"cargo-audit.exe": 15310848, "CHANGELOG.md": 17948, "LICENSE-APACHE": 11048,
         "LICENSE-MIT": 1096, "README.md": 6038}


def prepare():
    gate.safe_path(ZIP)
    if ZIP.exists():
        gate.require(ZIP.is_file() and ZIP.stat().st_size == ZIP_BYTES, "CI_AUDIT_ZIP_INVALID")
        blob = ZIP.read_bytes()
    else:
        with urllib.request.urlopen(ZIP_URL, timeout=30) as response:
            blob = response.read(ZIP_BYTES + 1)
    gate.require(len(blob) == ZIP_BYTES and hashlib.sha256(blob).hexdigest() == ZIP_SHA,
                 "CI_AUDIT_ZIP_INVALID")
    with zipfile.ZipFile(BytesIO(blob)) as archive:
        gate.require(sorted(archive.namelist()) == sorted([PREFIX, *(PREFIX + name for name in FILES)]),
                     "CI_AUDIT_ZIP_CONTENT_INVALID")
        gate.require(archive.getinfo(PREFIX).is_dir()
                     and all(archive.getinfo(PREFIX + name).file_size == size for name, size in FILES.items()),
                     "CI_AUDIT_ZIP_CONTENT_INVALID")
        payload = {name: archive.read(PREFIX + name) for name in FILES}
    gate.require(hashlib.sha256(payload["cargo-audit.exe"]).hexdigest() == gate.BIN_SHA,
                 "CI_AUDIT_ZIP_CONTENT_INVALID")
    if not ZIP.exists():
        gate.safe_path(ZIP.parent)
        ZIP.parent.mkdir(parents=True, exist_ok=True)
        with ZIP.open("xb") as output:
            output.write(blob)
    directory = gate.EXE.parent
    gate.safe_path(directory)
    installed = not directory.exists()
    if installed:
        directory.mkdir(parents=True)
        for name, content in payload.items():
            path = directory / name
            gate.safe_path(path)
            with path.open("xb") as output:
                output.write(content)
    gate.require(directory.is_dir() and sorted(p.name for p in directory.iterdir()) == sorted(FILES),
                 "CI_AUDIT_RUNTIME_INVALID")
    for name, content in payload.items():
        path = directory / name
        gate.safe_path(path)
        gate.require(path.is_file() and path.stat().st_size == len(content) and path.read_bytes() == content,
                     "CI_AUDIT_RUNTIME_INVALID")
    gate.scanner()
    gate.safe_path(gate.DB)
    cloned = not gate.DB.exists()
    if cloned:
        gate.DB.parent.mkdir(parents=True, exist_ok=True)
        result = gate.invoke(["git", "clone", gate.DB_URL, str(gate.DB)])
        gate.require(result.returncode == 0, "CI_AUDIT_DB_CLONE_FAILED")
    commit = gate.validate_db()
    return {"version": "0.22.2", "zipSHA256": ZIP_SHA, "installed": installed,
            "dbCloned": cloned, "databaseCommit": commit, "licenseFiles": 2}


def main():
    try:
        evidence = prepare()
        result = gate.invoke(["git", "rev-parse", "HEAD"])
        gate.require(result.returncode == 0 and re.fullmatch(rb"[0-9a-f]{40}", result.stdout.strip()),
                     "CI_AUDIT_SOURCE_INVALID")
        evidence["sourceSha"] = result.stdout.decode().strip()
        print("CI_AUDIT_PREPARE " + json.dumps(evidence))
        return 0
    except gate.GateError as error:
        print("CI_AUDIT_PREPARE_BLOCKED " + str(error), file=sys.stderr)
    except (OSError, ValueError, urllib.error.URLError, zipfile.BadZipFile):
        print("CI_AUDIT_PREPARE_BLOCKED CI_AUDIT_PREPARE_FAILED", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
