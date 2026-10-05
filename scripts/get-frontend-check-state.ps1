param(
    [string]$BaseRef = '',
    [string]$TargetRef = 'HEAD',
    [switch]$Force
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-SelectionGit {
    param([string[]]$GitArguments)
    $result = @(& git @GitArguments 2>$null)
    if ($LASTEXITCODE -ne 0) {
        throw 'Histórico Git indisponível para selecionar checks.'
    }
    return $result
}

try {
    $targetCommit = @(Invoke-SelectionGit -GitArguments @('rev-parse', '--verify', "$TargetRef^{commit}"))[0]
    if ($Force) {
        Write-Output 'true'
        return
    }
    if ([string]::IsNullOrWhiteSpace($BaseRef)) {
        throw 'Base obrigatória.'
    }
    if ($BaseRef -match '^(0{40}|0{64})$') {
        $changedPaths = @(Invoke-SelectionGit -GitArguments @('-c', 'core.quotepath=false', 'ls-tree', '-r', '--name-only', $targetCommit))
    } else {
        $baseCommit = @(Invoke-SelectionGit -GitArguments @('rev-parse', '--verify', "$BaseRef^{commit}"))[0]
        $changedPaths = @(Invoke-SelectionGit -GitArguments @('-c', 'core.quotepath=false', 'diff', '--name-only', '--no-renames', $baseCommit, $targetCommit, '--'))
    }
    $documentPath = '^(?:(?:README|AGENTS|00-MAPA-DO-PROJETO|CONTRIBUTING|SECURITY)\.md|(?:docs|governance|planning)/.+\.md|planning/work-items/[^/]+\.json|planning/(?:WORK_ITEM\.schema|WORK_ITEM_TEMPLATE)\.json)$'
    $required = @($changedPaths | Where-Object { $_ -notmatch $documentPath }).Count -gt 0
    Write-Output $required.ToString().ToLowerInvariant()
} catch {
    throw 'Não foi possível selecionar checks; CI deve falhar, nunca assumir documentação somente.'
}
