param([string]$Root = (Split-Path $PSScriptRoot -Parent))

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$files = @(& git -C $Root -c core.quotepath=false ls-files --cached --others --exclude-standard)
if ($LASTEXITCODE -ne 0) { throw 'Não foi possível inventariar o checkout Git.' }
$files = @($files | Sort-Object -Unique)
$map = Get-Content -LiteralPath (Join-Path $Root '00-MAPA-DO-PROJETO.md') -Encoding utf8
$sources = @(
    foreach ($row in $map) {
        if ($row -match '^\|[^|]+\|[^|]*`') {
            foreach ($match in [regex]::Matches($row.Split('|')[2], '`([^`]+)`')) {
                $match.Groups[1].Value
            }
        }
    }
)
if (-not $sources.Count) { throw 'Mapa sem fontes na tabela.' }

$errors = @(
    foreach ($source in $sources) {
        $matchingFiles = @($files | Where-Object {
            $_ -ceq $source -or ($source.EndsWith('/') -and $_.StartsWith($source, [StringComparison]::Ordinal))
        })
        if (-not $matchingFiles.Count -or -not (Test-Path -LiteralPath (Join-Path $Root $source))) {
            "Fonte ausente no checkout: $source"
        }
    }
    foreach ($file in $files) {
        $areas = @($sources | Where-Object {
            $file -ceq $_ -or ($_.EndsWith('/') -and $file.StartsWith($_, [StringComparison]::Ordinal))
        })
        if (-not $areas.Count) { "Arquivo sem área no mapa: $file" }
        if (-not (Test-Path -LiteralPath (Join-Path $Root $file) -PathType Leaf)) {
            "Arquivo do inventário ausente no checkout: $file"
        }
    }
)
if ($errors.Count) { throw ($errors -join [Environment]::NewLine) }
Write-Output "Mapa válido: $($sources.Count) fontes; $($files.Count) arquivos cobertos."
