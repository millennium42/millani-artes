Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$projectRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$fixtureRoot = [IO.Path]::GetFullPath((Join-Path $projectRoot 'artifacts/inf010-selector-fixtures'))
$selectorPath = Join-Path $PSScriptRoot 'get-frontend-check-state.ps1'
$pwshPath = (Get-Command pwsh -ErrorAction Stop).Source
if (Test-Path -LiteralPath $fixtureRoot) { throw 'Fixture já existe; não sobrescrever.' }
if (-not $fixtureRoot.StartsWith($projectRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Fixture fora do workspace.'
}
$null = [IO.Directory]::CreateDirectory($fixtureRoot)
$fixtureCount = 0
$fixtureRevision = 0
function Invoke-FixtureGit {
    param([string[]]$GitArguments)
    $result = @(& git -C $fixtureRoot -c commit.gpgsign=false @GitArguments 2>$null)
    if ($LASTEXITCODE -ne 0) { throw 'Falha Git na fixture sintética.' }
    return $result
}
function Write-FixtureFile {
    param([string]$RelativePath)
    $script:fixtureRevision++
    $file = Join-Path $fixtureRoot $RelativePath
    $null = [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($file))
    [IO.File]::WriteAllText($file, "fixture-$script:fixtureRevision", [Text.UTF8Encoding]::new($false))
}
function Commit-Fixture {
    $null = Invoke-FixtureGit -GitArguments @('add', '-A')
    $null = Invoke-FixtureGit -GitArguments @('commit', '--quiet', '-m', 'synthetic fixture')
    return @(Invoke-FixtureGit -GitArguments @('rev-parse', 'HEAD'))[0]
}
function Assert-Selection {
    param([string]$Base, [string]$Target, [string]$Expected, [switch]$Manual, [switch]$Invalid)
    Push-Location -LiteralPath $fixtureRoot
    try {
        $arguments = @('-NoProfile', '-File', $selectorPath, '-BaseRef', $Base, '-TargetRef', $Target)
        if ($Manual) { $arguments += '-Force' }
        $result = @(& $pwshPath @arguments 2>$null)
        $code = $LASTEXITCODE
        if ($Invalid) {
            if ($code -eq 0) { throw 'Git inválido não bloqueou.' }
        } elseif ($code -ne 0 -or $result.Count -ne 1 -or $result[0] -cne $Expected) {
            throw "Seleção divergente na fixture $script:fixtureCount."
        }
        $script:fixtureCount++
    } finally { Pop-Location }
}
try {
    $null = Invoke-FixtureGit -GitArguments @('init', '--quiet')
    $null = Invoke-FixtureGit -GitArguments @('config', 'user.name', 'Synthetic CI Fixture')
    $null = Invoke-FixtureGit -GitArguments @('config', 'user.email', 'fixture@example.invalid')
    $hooks = Join-Path $fixtureRoot 'empty-hooks'
    $null = [IO.Directory]::CreateDirectory($hooks)
    $null = Invoke-FixtureGit -GitArguments @('config', 'core.hooksPath', $hooks)
    foreach ($file in @('README.md', 'docs/note.md', 'src/App.tsx', 'src/styles.css')) { Write-FixtureFile $file }
    $initial = Commit-Fixture
    Assert-Selection $initial $initial 'false'
    Assert-Selection ('0' * 40) $initial 'true'
    Assert-Selection $initial $initial 'true' -Manual
    foreach ($file in @('README.md', 'docs/note.md', 'planning/work-items/INF-010.json')) {
        $before = @(Invoke-FixtureGit -GitArguments @('rev-parse', 'HEAD'))[0]
        Write-FixtureFile $file
        $after = Commit-Fixture
        Assert-Selection $before $after 'false'
    }
    foreach ($file in @('src/App.tsx', 'src/styles.css', 'package-lock.json', '.node-version', '.gitignore', 'vite.config.ts', '.github/workflows/docs.yml', 'scripts/get-frontend-check-state.ps1', 'custom-config.txt')) {
        $before = @(Invoke-FixtureGit -GitArguments @('rev-parse', 'HEAD'))[0]
        Write-FixtureFile $file
        $after = Commit-Fixture
        Assert-Selection $before $after 'true'
    }
    $before = @(Invoke-FixtureGit -GitArguments @('rev-parse', 'HEAD'))[0]
    Move-Item -LiteralPath (Join-Path $fixtureRoot 'src/App.tsx') -Destination (Join-Path $fixtureRoot 'docs/renamed.md')
    $after = Commit-Fixture
    Assert-Selection $before $after 'true'
    $before = $after
    Remove-Item -LiteralPath (Join-Path $fixtureRoot 'src/styles.css')
    $after = Commit-Fixture
    Assert-Selection $before $after 'true'
    Assert-Selection 'unavailable-ref' $after '' -Invalid
    Assert-Selection $after 'unavailable-ref' '' -Invalid
    Assert-Selection 'unavailable-ref' $after 'true' -Manual
    Write-Output "PASS: $fixtureCount fixtures Git de seleção; source/CSS/desconhecido/rename/delete/manual/zero e falha fechada."
} finally {
    $resolvedFixture = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $fixtureRoot).Path)
    if ($resolvedFixture -ne $fixtureRoot -or -not $resolvedFixture.StartsWith($projectRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Destino de cleanup inválido.'
    }
    Remove-Item -LiteralPath $resolvedFixture -Recurse -Force
}
