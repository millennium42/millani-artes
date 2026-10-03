param([string]$Checker = (Join-Path $PSScriptRoot 'check-work-items.ps1'))

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$taskRoot = Join-Path ([IO.Path]::GetTempPath()) ('millani-pln002-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $taskRoot | Out-Null
$fixture = Join-Path $taskRoot 'items.json'
$caseCount = 0
$valid = @{
    id = 'TST-001'; title = 'Comportamento sintético'; specPath = 'planning/specs/TST-001.md'
    status = 'todo'; dependencies = @(); acceptanceCriteria = @('Critério'); checks = @('Check')
    agentPlan = 'P-INT'; evidence = 'E0'; humanGateRequired = $false; passes = $false; notes = ''
}
function Assert-WorkItemCase([string]$Json, [bool]$Expected, [string]$Label) {
    Set-Content -LiteralPath $fixture -Value $Json -Encoding utf8
    $accepted = $true
    $output = @()
    $message = ''
    try { $output = @(& $Checker -Root $taskRoot *>&1) }
    catch { $message = $_.Exception.Message; $accepted = $false }
    if (($output | Out-String).Contains('SYNTHETIC_SECRET_MARKER') -or $message.Contains('SYNTHETIC_SECRET_MARKER')) {
        throw 'Check expôs conteúdo da fixture.'
    }
    if ($accepted -ne $Expected) { throw "Resultado inesperado na fixture: $Label" }
    $script:caseCount++
}
try {
    Assert-WorkItemCase ($valid | ConvertTo-Json -Depth 6) $true 'objeto único'
    Assert-WorkItemCase (ConvertTo-Json -InputObject @($valid) -Depth 6) $true 'array unitário'
    foreach ($name in $valid.Keys) {
        $item = $valid.Clone(); $item.Remove($name)
        Assert-WorkItemCase ($item | ConvertTo-Json -Depth 6) $false "campo obrigatório ausente: $name"
    }
    foreach ($name in $valid.Keys) {
        $item = $valid.Clone(); $item[$name] = $null
        Assert-WorkItemCase ($item | ConvertTo-Json -Depth 6) $false "campo null: $name"
    }
    foreach ($mutation in @(
        @{id='tst-001'}, @{id='<ID>'}, @{id=('TST-001' + [char]10)}, @{title=' '}, @{specPath='../secret.md'},
        @{specPath=('planning/specs/TST-001.md' + [char]10)},
        @{specPath='planning/specs/../TST-001.md'}, @{status='DONE'}, @{status='unknown'},
        @{dependencies='TST-002'}, @{dependencies=@('TST-002','TST-002')}, @{dependencies=@('invalid')},
        @{acceptanceCriteria=@()}, @{checks=@()}, @{checks=@(' ')}, @{agentPlan=''},
        @{evidence='e3'}, @{evidence='E5'}, @{humanGateRequired='false'}, @{passes='true'}, @{notes=42},
        @{unknown='SYNTHETIC_SECRET_MARKER'}, @{title='SYNTHETIC_SECRET_MARKER';checks=@()}
    )) {
        $item = $valid.Clone(); foreach ($name in $mutation.Keys) { $item[$name] = $mutation[$name] }
        Assert-WorkItemCase ($item | ConvertTo-Json -Depth 6) $false 'valor/tipo inválido'
    }
    foreach ($level in @('E0','E1','E2','E3','E4')) {
        $item = $valid.Clone(); $item.status='done'; $item.passes=$true; $item.evidence=$level
        Assert-WorkItemCase ($item | ConvertTo-Json -Depth 6) ($level -in @('E3','E4')) 'passes exige E3/E4'
    }
    foreach ($state in @('todo','in_progress','blocked')) {
        $item=$valid.Clone(); $item.status=$state; $item.passes=$true; $item.evidence='E3'
        Assert-WorkItemCase ($item | ConvertTo-Json -Depth 6) $false 'passes exige done'
        $item.passes=$false
        Assert-WorkItemCase ($item | ConvertTo-Json -Depth 6) $true 'estado não concluído'
    }
    foreach ($json in @('[]','null','3','[null]','[[{}]]','{}','{"notes":"SYNTHETIC_SECRET_MARKER",','{"evidence":"E3","passes":false}')) {
        Assert-WorkItemCase $json $false 'estrutura/JSON inválido'
    }
    $other=$valid.Clone(); $other.id='TST-002'; $other.specPath='planning/specs/TST-002.md'; $other.dependencies=@('TST-001')
    Assert-WorkItemCase (ConvertTo-Json -InputObject @($valid,$other) -Depth 6) $true 'coleção válida'
    $other.id=$valid.id
    Assert-WorkItemCase (ConvertTo-Json -InputObject @($valid,$other) -Depth 6) $false 'ID repetido em coleção'
    Set-Content -LiteralPath (Join-Path $taskRoot 'other.json') -Value ($valid | ConvertTo-Json -Depth 6) -Encoding utf8
    Assert-WorkItemCase ($valid | ConvertTo-Json -Depth 6) $false 'ID repetido entre arquivos'
    Remove-Item -LiteralPath (Join-Path $taskRoot 'other.json')
    $template = Get-Content (Join-Path (Split-Path $PSScriptRoot -Parent) 'planning/WORK_ITEM_TEMPLATE.json') -Raw | ConvertFrom-Json
    $template.id='TST-003'; $template.specPath='planning/specs/TST-003.md'
    $template.title='Template instanciado'; $template.acceptanceCriteria=@('Critério sintético'); $template.checks=@('Check sintético'); $template.agentPlan='P-INT'
    Assert-WorkItemCase ($template | ConvertTo-Json -Depth 6) $true 'template preenchido'
    foreach ($schemaCase in @('missing','invalid')) {
        $schemaFixture=Join-Path $taskRoot 'schema.txt'
        if ($schemaCase -eq 'invalid') { Set-Content -LiteralPath $schemaFixture -Value '{"SYNTHETIC_SECRET_MARKER":' -Encoding utf8 }
        $failed=$false; $message=''
        try { & $Checker -Root $taskRoot -SchemaPath $schemaFixture | Out-Null }
        catch { $failed=$true; $message=$_.Exception.Message }
        if (-not $failed -or $message.Contains('SYNTHETIC_SECRET_MARKER')) { throw 'Schema ausente/inválido aceito ou exposto.' }
    }
    Remove-Item -LiteralPath $fixture
    $failed=$false; try { & $Checker -Root $taskRoot | Out-Null } catch { $failed=$true }
    if (-not $failed) { throw 'Check aceitou diretório vazio.' }
    $failed=$false; try { & $Checker -Root (Join-Path $taskRoot 'missing') | Out-Null } catch { $failed=$true }
    if (-not $failed) { throw 'Check aceitou diretório ausente.' }
    Write-Output "PASS: $caseCount fixtures de schema/estados/IDs/erros sanitizados/template; diretório vazio/ausente."
} finally {
    $resolved = (Resolve-Path -LiteralPath $taskRoot).Path
    $tempBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    if ($resolved -ne $taskRoot -or -not $resolved.StartsWith($tempBase, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Recusada limpeza fora da fixture temporária.'
    }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
