$ErrorActionPreference = 'Stop'
$checker = Join-Path $PSScriptRoot 'check-project-map.ps1'
$taskRoot = Join-Path ([IO.Path]::GetTempPath()) ('millani-doc002-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $taskRoot | Out-Null
try {
    & git -C $taskRoot init --quiet
    if ($LASTEXITCODE -ne 0) { throw 'Fixture Git não inicializada.' }
    $mapPath = Join-Path $taskRoot '00-MAPA-DO-PROJETO.md'
    $validMap = '| Entrada | `00-MAPA-DO-PROJETO.md`, `README.md` | fontes |'
    Set-Content -LiteralPath $mapPath -Value $validMap -Encoding utf8
    Set-Content -LiteralPath (Join-Path $taskRoot 'README.md') -Value 'Fixture sintética' -Encoding utf8
    & $checker -Root $taskRoot | Out-Null

    Set-Content -LiteralPath $mapPath -Value @($validMap, '| Extra | `inexistente.md` | fonte |') -Encoding utf8
    $failed = $false
    try { & $checker -Root $taskRoot | Out-Null } catch { $failed = $_.Exception.Message.Contains('Fonte ausente') }
    if (-not $failed) { throw 'Check aceitou fonte inexistente.' }

    Set-Content -LiteralPath $mapPath -Value $validMap -Encoding utf8
    $extraPath = Join-Path $taskRoot 'extra.md'
    Set-Content -LiteralPath $extraPath -Value 'Fixture sintética' -Encoding utf8
    $failed = $false
    try { & $checker -Root $taskRoot | Out-Null } catch { $failed = $_.Exception.Message.Contains('Arquivo sem área') }
    if (-not $failed) { throw 'Check aceitou arquivo sem área.' }
    Remove-Item -LiteralPath $extraPath

    & git -C $taskRoot add README.md
    if ($LASTEXITCODE -ne 0) { throw 'Fixture não adicionada ao índice.' }
    & $checker -Root $taskRoot | Out-Null
    Remove-Item -LiteralPath (Join-Path $taskRoot 'README.md')
    $failed = $false
    try { & $checker -Root $taskRoot | Out-Null } catch { $failed = $_.Exception.Message.Contains('ausente') }
    if (-not $failed) { throw 'Check aceitou arquivo versionado ausente.' }
    Write-Output 'PASS: mapa válido, fonte inexistente, arquivo sem área, índice Git e arquivo versionado ausente.'
} finally {
    $resolved = (Resolve-Path -LiteralPath $taskRoot).Path
    $tempBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    if ($resolved -ne $taskRoot -or -not $resolved.StartsWith($tempBase, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Recusada limpeza fora da fixture temporária.'
    }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
