$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$checker = Join-Path $PSScriptRoot 'check-evidence-states.ps1'
$taskRoot = Join-Path ([IO.Path]::GetTempPath()) ('millani-gov002-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $taskRoot | Out-Null
$fixture = Join-Path $taskRoot 'items.json'
$caseCount = 0
function Assert-EvidenceCase([string]$Json, [bool]$Expected, [string]$Label) {
    Set-Content -LiteralPath $fixture -Value $Json -Encoding utf8
    $accepted = $true
    try { & $checker -Root $taskRoot | Out-Null }
    catch {
        if ($_.Exception.Message.Contains('SYNTHETIC_SECRET_MARKER')) { throw 'Check expôs conteúdo da fixture.' }
        $accepted = $false
    }
    if ($accepted -ne $Expected) { throw "Resultado inesperado na fixture: $Label" }
    $script:caseCount++
}
try {
    $valid = @('E0', 'E1', 'E2', 'E3', 'E4') | ForEach-Object { @{ evidence = $_; passes = $false } }
    Assert-EvidenceCase ($valid | ConvertTo-Json) $true 'E0–E4/false'
    Assert-EvidenceCase '{"evidence":"E0","passes":false}' $true 'objeto único'
    Assert-EvidenceCase '[{"evidence":"E3","passes":true},{"evidence":"E4","passes":true}]' $true 'E3/E4/true apenas sintaxe'
    foreach ($level in @('E5', 'e3', '', $null, 3, @{}, @())) {
        Assert-EvidenceCase (@{ evidence = $level; passes = $false } | ConvertTo-Json -Depth 4) $false 'nível inválido'
    }
    foreach ($passed in @('true', 0, $null)) {
        Assert-EvidenceCase (@{ evidence = 'E3'; passes = $passed } | ConvertTo-Json) $false 'passes não booleano'
    }
    foreach ($level in @('E0', 'E1', 'E2')) {
        Assert-EvidenceCase (@{ evidence = $level; passes = $true } | ConvertTo-Json) $false 'nível insuficiente'
    }
    foreach ($json in @('{}', '{"passes":false}', '{"evidence":"E3"}', '[]', 'null', '3', '[[{"evidence":"E3","passes":false}]]', '[{"evidence":"E3","passes":false},{"evidence":"E5","passes":false}]', '{"SYNTHETIC_SECRET_MARKER":')) {
        Assert-EvidenceCase $json $false 'estrutura/JSON inválido'
    }
    Remove-Item -LiteralPath $fixture
    $failed = $false
    try { & $checker -Root $taskRoot | Out-Null } catch { $failed = $true }
    if (-not $failed) { throw 'Check aceitou diretório sem JSON.' }
    Write-Output "PASS: $caseCount fixtures de estados/tipos/gates/JSON e diretório vazio; conteúdo de erro sanitizado."
} finally {
    $resolved = (Resolve-Path -LiteralPath $taskRoot).Path
    $tempBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    if ($resolved -ne $taskRoot -or -not $resolved.StartsWith($tempBase, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Recusada limpeza fora da fixture temporária.'
    }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
