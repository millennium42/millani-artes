param(
    [string]$Root = (Split-Path $PSScriptRoot -Parent),
    [Parameter(Mandatory)][string]$BaseRef,
    [string]$TargetRef = 'INDEX'
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
function Get-GitBytes([string[]]$GitArguments) {
    $process=[Diagnostics.Process]::new()
    $buffer=[IO.MemoryStream]::new()
    try {
        $process.StartInfo.FileName='git'
        $process.StartInfo.UseShellExecute=$false
        $process.StartInfo.RedirectStandardOutput=$true
        $process.StartInfo.RedirectStandardError=$true
        foreach ($arg in @('-C',$Root)+$GitArguments) { $process.StartInfo.ArgumentList.Add($arg) }
        if (-not $process.Start()) { throw 'Git indisponível.' }
        $copy=$process.StandardOutput.BaseStream.CopyToAsync($buffer)
        $errors=$process.StandardError.ReadToEndAsync()
        $null=$copy.GetAwaiter().GetResult()
        $null=$errors.GetAwaiter().GetResult()
        $process.WaitForExit()
        if ($process.ExitCode -ne 0) { throw 'Git recusou operação.' }
        return ,$buffer.ToArray()
    } catch { throw "Não foi possível ler refs/blobs Git ($($GitArguments[0]), $($_.Exception.GetType().Name)); conteúdo omitido." }
    finally { $process.Dispose(); $buffer.Dispose() }
}
function Resolve-Commit([string]$Ref) {
    if ([string]::IsNullOrWhiteSpace($Ref)) { throw 'Base/alvo explícito ausente.' }
    $bytes=Get-GitBytes @('rev-parse','--verify','--end-of-options',($Ref+'^{commit}'))
    $sha=[Text.Encoding]::UTF8.GetString($bytes).Trim()
    if ($sha -notmatch '^(?:[0-9a-f]{40}|[0-9a-f]{64})$') { throw 'Ref não resolveu para commit.' }
    return $sha
}
$base=Resolve-Commit $BaseRef
$target=if ($TargetRef -ceq 'INDEX') { Resolve-Commit 'HEAD' } else { Resolve-Commit $TargetRef }
$null=Get-GitBytes @('merge-base','--is-ancestor',$base,$target)
$path='planning/PROGRESS_LOG.md'
$before=Get-GitBytes @('cat-file','blob',($base+':'+$path))
$object=if ($TargetRef -ceq 'INDEX') { ':'+$path } else { $target+':'+$path }
$after=Get-GitBytes @('cat-file','blob',$object)
if (-not $before.Length) { throw 'Histórico-base vazio; comparação recusada.' }
if ($after.Length -lt $before.Length) { throw 'Progress log truncado; conteúdo omitido.' }
for ($i=0; $i -lt $before.Length; $i++) {
    if ($before[$i] -ne $after[$i]) { throw 'Histórico do progress log alterado; conteúdo omitido.' }
}
$displayTarget=if ($TargetRef -ceq 'INDEX') { 'INDEX' } else { $target }
Write-Output "PASS: progress log append-only; $($before.Length) bytes históricos preservados; alvo $displayTarget; conteúdo não exibido."
