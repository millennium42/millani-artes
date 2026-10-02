param([string]$Root = (Join-Path (Split-Path $PSScriptRoot -Parent) 'planning/work-items'))

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not (Test-Path -LiteralPath $Root -PathType Container)) { throw 'Diretório de work items ausente.' }
$files = @(Get-ChildItem -LiteralPath $Root -Filter '*.json' -File)
if (-not $files.Count) { throw 'Nenhum JSON de work item encontrado.' }
$taskCount = 0
foreach ($file in $files) {
    try { $items = @(Get-Content -LiteralPath $file.FullName -Raw -Encoding utf8 | ConvertFrom-Json) }
    catch { throw "JSON inválido em $($file.Name)." }
    if (-not $items.Count) { throw "Sem work items em $($file.Name)." }
    foreach ($item in $items) {
        if ($item -isnot [pscustomobject]) { throw "Work item inválido em $($file.Name)." }
        $level = $item.PSObject.Properties['evidence']
        $passed = $item.PSObject.Properties['passes']
        if (-not $level -or $level.Value -isnot [string] -or $level.Value -cnotin @('E0', 'E1', 'E2', 'E3', 'E4')) {
            throw "Nível de evidência inválido em $($file.Name)."
        }
        if (-not $passed -or $passed.Value -isnot [bool]) { throw "passes deve ser booleano em $($file.Name)." }
        if ($passed.Value -and $level.Value -cnotin @('E3', 'E4')) {
            throw "passes=true exige declaração E3/E4 em $($file.Name); fatos/gates requerem comprovante separado."
        }
        $taskCount++
    }
}
Write-Output "Declarações de evidência válidas: $taskCount work items; fatos/gates não comprovados por este check."
