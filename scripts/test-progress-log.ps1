$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$checker = Join-Path $PSScriptRoot 'check-progress-log.ps1'
$taskRoot = Join-Path ([IO.Path]::GetTempPath()) ('millani-pln004-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path (Join-Path $taskRoot 'planning') -Force | Out-Null
$logPath = Join-Path $taskRoot 'planning/PROGRESS_LOG.md'
$caseCount = 0
$baseline = [Text.Encoding]::UTF8.GetBytes("Histórico sintético á`nLinha antiga`n")
function Invoke-FixtureGit([string[]]$Arguments) {
    $output = @(& git -C $taskRoot @Arguments 2>&1)
    if ($LASTEXITCODE -ne 0) { throw 'Operação Git da fixture falhou; conteúdo omitido.' }
    return $output
}
function Stage-Bytes([byte[]]$Bytes) {
    [IO.File]::WriteAllBytes($logPath, $Bytes)
    Invoke-FixtureGit @('add','--','planning/PROGRESS_LOG.md') | Out-Null
}
function Assert-ProgressCase([bool]$Expected, [string]$Label, [string]$Base = $script:baseSha, [string]$Target = 'INDEX') {
    $passed=$true; $message=''; $output=@()
    try { $output=@(& $checker -Root $taskRoot -BaseRef $Base -TargetRef $Target *>&1) }
    catch { $passed=$false; $message=$_.Exception.Message }
    if (($output | Out-String).Contains('SYNTHETIC_SECRET_MARKER') -or $message.Contains('SYNTHETIC_SECRET_MARKER')) { throw 'Check expôs conteúdo sintético.' }
    if ($passed -ne $Expected) { throw "Resultado inesperado na fixture: $Label; $message" }
    $script:caseCount++
}
try {
    Invoke-FixtureGit @('init','--quiet') | Out-Null
    Invoke-FixtureGit @('config','user.name','Fixture') | Out-Null
    Invoke-FixtureGit @('config','user.email','fixture@example.invalid') | Out-Null
    Invoke-FixtureGit @('config','core.autocrlf','false') | Out-Null
    Stage-Bytes $baseline
    Invoke-FixtureGit @('commit','--quiet','-m','base') | Out-Null
    $script:baseSha=@(Invoke-FixtureGit @('rev-parse','HEAD'))[0]
    Assert-ProgressCase $true 'índice idêntico'
    Assert-ProgressCase $true 'commit idêntico' $baseSha 'HEAD'
    $append=[byte[]]($baseline + [Text.Encoding]::UTF8.GetBytes("Novo registro`n"))
    Stage-Bytes $append
    Assert-ProgressCase $true 'append staged'
    [IO.File]::WriteAllBytes($logPath,[Text.Encoding]::UTF8.GetBytes('Unstaged diferente'))
    Assert-ProgressCase $true 'unstaged não é alvo INDEX'
    Invoke-FixtureGit @('commit','--quiet','-m','append') | Out-Null
    $appendSha=@(Invoke-FixtureGit @('rev-parse','HEAD'))[0]
    Assert-ProgressCase $true 'push before→head' $baseSha $appendSha
    Assert-ProgressCase $true 'PR base→commit candidato' $baseSha 'HEAD'
    Invoke-FixtureGit @('branch','SYNTHETIC_SECRET_MARKER',$appendSha) | Out-Null
    Assert-ProgressCase $true 'ref válida não ecoa nome recebido' $baseSha 'SYNTHETIC_SECRET_MARKER'
    foreach ($mutation in @(
        @{Label='reescrita';Bytes=[Text.Encoding]::UTF8.GetBytes("Histórico alterado`nLinha antiga`nSYNTHETIC_SECRET_MARKER")},
        @{Label='truncamento';Bytes=[Text.Encoding]::UTF8.GetBytes("Histórico sintético á`n")},
        @{Label='inserção inicial';Bytes=[Text.Encoding]::UTF8.GetBytes("Novo`nHistórico sintético á`nLinha antiga`n")},
        @{Label='inserção intermediária';Bytes=[Text.Encoding]::UTF8.GetBytes("Histórico sintético á`nInserção`nLinha antiga`n")},
        @{Label='CRLF no blob';Bytes=[Text.Encoding]::UTF8.GetBytes("Histórico sintético á`r`nLinha antiga`r`n")},
        @{Label='BOM';Bytes=[byte[]](@(0xEF,0xBB,0xBF)+$baseline)},
        @{Label='vazio';Bytes=[byte[]]@()}
    )) {
        Stage-Bytes $mutation.Bytes
        Assert-ProgressCase $false $mutation.Label
    }
    Invoke-FixtureGit @('commit','--quiet','-m','empty candidate') | Out-Null
    $emptySha=@(Invoke-FixtureGit @('rev-parse','HEAD'))[0]
    Assert-ProgressCase $false 'commit truncado' $baseSha 'HEAD'
    Assert-ProgressCase $false 'base vazia' $emptySha 'HEAD'
    Assert-ProgressCase $false 'base posterior ao alvo' $appendSha $baseSha
    Stage-Bytes $baseline
    foreach ($ref in @('missing-ref','0000000000000000000000000000000000000000','--SYNTHETIC_SECRET_MARKER',' ')) {
        Assert-ProgressCase $false 'base inválida' $ref
    }
    Assert-ProgressCase $false 'target inválido' $baseSha 'SYNTHETIC_SECRET_MARKER-missing'
    Invoke-FixtureGit @('rm','--quiet','--cached','--','planning/PROGRESS_LOG.md') | Out-Null
    Assert-ProgressCase $false 'blob ausente no índice'
    Invoke-FixtureGit @('commit','--quiet','-m','log absent') | Out-Null
    $missingSha=@(Invoke-FixtureGit @('rev-parse','HEAD'))[0]
    Assert-ProgressCase $false 'blob ausente no alvo' $baseSha $missingSha
    Assert-ProgressCase $false 'blob ausente na base' $missingSha $missingSha
    Stage-Bytes $append
    Invoke-FixtureGit @('config','core.autocrlf','true') | Out-Null
    [IO.File]::WriteAllText($logPath,[Text.Encoding]::UTF8.GetString($append).Replace("`n","`r`n"),[Text.UTF8Encoding]::new($false))
    Invoke-FixtureGit @('add','--','planning/PROGRESS_LOG.md') | Out-Null
    Assert-ProgressCase $true 'checkout CRLF normalizado por Git no índice'
    Write-Output "PASS: $caseCount fixtures de blobs/prefixo/refs/índice/commits/erros sanitizados; semântica do log não validada."
} finally {
    $resolved=(Resolve-Path -LiteralPath $taskRoot).Path
    $tempBase=[IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    if ($resolved -ne $taskRoot -or -not $resolved.StartsWith($tempBase,[StringComparison]::OrdinalIgnoreCase)) { throw 'Recusada limpeza fora da fixture temporária.' }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
