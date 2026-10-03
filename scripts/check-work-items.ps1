param(
    [string]$Root = (Join-Path (Split-Path $PSScriptRoot -Parent) 'planning/work-items'),
    [string]$SchemaPath = (Join-Path (Split-Path $PSScriptRoot -Parent) 'planning/WORK_ITEM.schema.json')
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not (Test-Path -LiteralPath $Root -PathType Container)) { throw 'Diretório de work items ausente.' }
if (-not (Test-Path -LiteralPath $SchemaPath -PathType Leaf)) { throw 'Schema de work item ausente.' }
$files = @(Get-ChildItem -LiteralPath $Root -Filter '*.json' -File)
if (-not $files.Count) { throw 'Nenhum JSON de work item encontrado.' }
$ids = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
$taskCount = 0
foreach ($file in $files) {
    try {
        $json = Get-Content -LiteralPath $file.FullName -Raw -Encoding utf8
        if (-not (Test-Json -Json $json -SchemaFile $SchemaPath -ErrorAction Stop)) { throw 'Schema recusou entrada.' }
        $items = @(ConvertFrom-Json -InputObject $json)
    } catch { throw "JSON/schema de work item inválido em $($file.Name); conteúdo omitido." }
    foreach ($item in $items) {
        if (-not $ids.Add($item.id)) { throw "ID de work item duplicado em $($file.Name); conteúdo omitido." }
        $taskCount++
    }
}
Write-Output "Schema válido: $taskCount work items; fatos, aceite humano e CI exigem comprovantes separados."
