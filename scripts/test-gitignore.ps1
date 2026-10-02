$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$ignored = @(
    'local.db', 'local.db-wal', 'local.db-shm', 'local.sqlite', 'local.sqlite-journal',
    'local.sqlite3', 'local.sqlite3-wal', 'local.backup', 'local.bak', 'backups/local.zip',
    '.env', '.env.local', 'backup.key', 'backup.pem', 'backup.p12', 'backup.pfx',
    'debug.log', 'node_modules/example/index.js', 'target/example.exe', 'dist/index.html',
    'coverage/index.html', 'artifacts/setup.exe', 'release/setup.exe', '.vscode/settings.json',
    '.idea/workspace.xml', 'Thumbs.db', 'Desktop.ini', '.DS_Store'
)
foreach ($path in $ignored) {
    & git -C $root check-ignore --no-index --quiet -- $path
    if ($LASTEXITCODE -ne 0) { throw "Artefato sensível/local não ignorado: $path" }
}
$preserved = @('package-lock.json', 'Cargo.lock', 'README.md', 'scripts/check-project-map.ps1')
foreach ($path in $preserved) {
    & git -C $root check-ignore --no-index --quiet -- $path
    if ($LASTEXITCODE -ne 1) { throw "Fonte/lockfile indevidamente ignorado: $path" }
}
Write-Output "PASS: $($ignored.Count) artefatos ignorados; $($preserved.Count) fontes/lockfiles preservados."
$global:LASTEXITCODE = 0
