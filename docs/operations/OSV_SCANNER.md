# OSV-Scanner local — SEC-008

OSV-Scanner 2.6.0 foi instalado como ferramenta portátil Windows x64. A instalação prepara SEC-009, responsável por seu gate no CI. [Spec](../../planning/specs/SEC-008.md), [handoff](../../planning/handoffs/SEC-008.md) e [registro](../../governance/TOOL_REGISTER.md) delimitam o escopo; instalar a CLI não comprova ausência de vulnerabilidades.

## Distribuição fixa
[Release oficial](https://github.com/google/osv-scanner/releases/tag/v2.6.0), revisão e840a6e8adb14b7777c78e26cfbf6e2abc1d1fc6. Cache dedicado ignorado pelo Git: artifacts/tools/osv-scanner/2.6.0/.
- osv-scanner_windows_amd64.exe:58.692.096 bytes, SHA256 e0ed7644118b717b028c249ee9d3515024e55e8510747ca08906eb96765354d6, PE AMD64/PE32+.
- osv-scanner_SHA256SUMS:554 bytes, SHA256 29f6fbc8bdd02d977df4b0987705d046233c36b70469a7021c623e8c155c9ddc.
- LICENSE:11.358 bytes, SHA256 cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30, [Apache-2.0 da revisão fixa](https://github.com/google/osv-scanner/blob/e840a6e8adb14b7777c78e26cfbf6e2abc1d1fc6/LICENSE).

A LICENSE principal foi lida e guardada junto do executável; não há NOTICE na raiz da árvore dessa revisão. Licenças transitivas/auditoria completa não foram verificadas. Eventual redistribuição deve acompanhar os termos/avisos aplicáveis; o scanner não é distribuído no aplicativo.
Nesta máquina Authenticode retornou NotSigned, sem signatário ou timestamp. A assinatura do commit consta verificada na API GitHub, mas não assina o executável Windows. A release fornece provenance SLSA; sua verificação criptográfica não foi executada. Hash/API/checksum concordantes conferem integridade contra o mesmo fornecedor, sem autenticação independente.

## Reinstalar e conferir
1. Na raiz do projeto, exigir cache dedicado ausente; recusar symlink/junction no destino e ancestrais. Se houver cache, verificar seus três arquivos e hashes antes reutilizar; não substituir diretório alheio ou corrigir binário adulterado silenciosamente.
2. Baixar somente [exe](https://github.com/google/osv-scanner/releases/download/v2.6.0/osv-scanner_windows_amd64.exe), [checksum](https://github.com/google/osv-scanner/releases/download/v2.6.0/osv-scanner_SHA256SUMS) e [LICENSE](https://raw.githubusercontent.com/google/osv-scanner/e840a6e8adb14b7777c78e26cfbf6e2abc1d1fc6/LICENSE). Conferir tamanhos e hashes acima, inclusive a linha do exe no checksum, antes gravar/executar.
3. Exigir PE32+ AMD64 e somente esses três arquivos regulares no cache. Não requer administrador, PATH persistente, Go, winget, Docker, serviço ou hook. A [documentação oficial](https://google.github.io/osv-scanner/installation/) oferece o binário pronto.
4. Para conferir e executar, em PowerShell7 na raiz:

```powershell
$sec008Cache = Join-Path (Get-Location) 'artifacts/tools/osv-scanner/2.6.0'
$sec008Expected = @{
  'osv-scanner_windows_amd64.exe' = 'e0ed7644118b717b028c249ee9d3515024e55e8510747ca08906eb96765354d6'
  'LICENSE' = 'cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30'
  'osv-scanner_SHA256SUMS' = '29f6fbc8bdd02d977df4b0987705d046233c36b70469a7021c623e8c155c9ddc'
}
function Assert-Sec008Cache {
  $sec008Cursor = Get-Item -LiteralPath $sec008Cache
  while ($null -ne $sec008Cursor) {
    if ($sec008Cursor.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'OSV reparse point denied.' }
    $sec008Cursor = $sec008Cursor.Parent
  }
  $sec008Entries = @(Get-ChildItem -LiteralPath $sec008Cache -Force)
  if ($sec008Entries.Count -ne 3) { throw 'OSV cache must contain exactly three files.' }
  foreach ($sec008Entry in $sec008Entries) {
    if ($sec008Entry.PSIsContainer -or ($sec008Entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -or -not $sec008Expected.ContainsKey($sec008Entry.Name)) { throw 'Unexpected OSV cache member.' }
    if ((Get-FileHash -LiteralPath $sec008Entry.FullName -Algorithm SHA256).Hash.ToLowerInvariant() -ne $sec008Expected[$sec008Entry.Name]) { throw 'OSV integrity mismatch.' }
  }
}
Assert-Sec008Cache
$sec008Exe = Join-Path $sec008Cache 'osv-scanner_windows_amd64.exe'
$sec008Version = @(rtk proxy $sec008Exe --version)
if ($LASTEXITCODE -ne 0) { throw 'OSV version command failed.' }
$sec008VersionLines = @($sec008Version | Where-Object { $_ -match '^osv-scanner version:' })
if ($sec008VersionLines.Count -ne 1 -or $sec008VersionLines[0] -cne 'osv-scanner version: 2.6.0') { throw 'Unexpected OSV version.' }
```

Versão esperada2.6.0. Os mesmos hashes devem permanecer após os probes. Erro/timeout/nonzero inesperado não comprova instalação. Não iniciar scan source/fix/update/mcp nem baixar DB offline para repetir estes probes:

```powershell
rtk proxy $sec008Exe --help
if ($LASTEXITCODE -ne 0) { throw 'OSV help failed.' }
rtk proxy $sec008Exe scan source --help
if ($LASTEXITCODE -ne 0) { throw 'OSV source help failed.' }
rtk proxy $sec008Exe --millani-invalid-flag
if ($LASTEXITCODE -eq 0) { throw 'OSV accepted an invalid flag.' }
Assert-Sec008Cache
```

## Execução E3 — 2026-10-08
Root/RTK/Windowsx64/Python3.12.10:RED da instalação ausente exit1/OSV_NOT_INSTALLED; download de três arquivos2.323s, cada tamanho/hash e PE conferidos antes escrever/executar. version exit0/2.6.0/0.887s;help exit0/0.142s;scan-source-help exit0/0.137s (somente ajuda);invalid-invocation exit127/0.137s. Cwd isolado artifacts/sec008-probes, ambiente subprocesso sem GIT_/OSV_/GITLEAKS_/GH_TOKEN/GITHUB_TOKEN; nenhuma alteração persistente de ambiente. Hash preservado e raiz de probes removida após verificar absoluto/contenção/ancestrais/árvore sem reparse points.
Nenhum scan de dependência, chamada OSV API, DB offline, servidorMCP ou correção automática foi solicitado nesses comandos. Download usa GitHub/TLS; tráfego/telemetria durante a CLI não foi medido. Nenhuma declaração de “zero vulnerabilidades” ou gateOSVremoto. Reports/dados financeiros/credenciais reais não são usados. Coverage do produto e locks permanecem intactos.
Review/checks/documentação/scan de segredos do diff/CI documental/nota49-liberação ainda pendentes;passesfalse/reserva10ATIVA/METAativa.

## Remover
Fechar processos que usem o exe. Verificar caminho absoluto exatamente no cache dedicado deste projeto, contido em artifacts/tools/osv-scanner e sem reparse points, antes remover somente2.6.0. Nunca apagar artifacts inteiro, usar curinga ou remover caminho computado sem confirmação de contenção. Nenhuma remoção do cache foi executada; não há serviço/PATH persistente para desfazer.

## Verificação dos exemplos documentais
Root executou os dois blocos PowerShell reais:positivo+probes passou;em cópia própria extra-file/tampered-license/junction foram recusados exit1 cada,4casos/4.959s.2blocos parseados;cache original não modificado,binário adulterado nunca executado;junction própria apenas para cópia própria,removida com rmdir antes cleanup verificado. Nenhuma prova de scanOSV/rede/telemetria foi inferida. Review/checksdocs/diffsecurity/CI/nota49-liberação pendentes.

## Correção P2 e verificação final dos exemplos
P-REV5.6Solmedium encontrouP2:exemplo aceitava versão inesperada exit0 e não repetia hashes depois dos probes. Root reproduziu em cópia própria com CLIboundary mock versão9.9.9 exit0:guard original aceitou,assert esperado falhou RED1/1.731s (não scan/CLIreal). Correção exige uma linha exata osv-scanner version: 2.6.0 e reutiliza Assert-Sec008Cache antes/depois. Root reexecutou2blocos reais+6casos/9.038s:positivo0;mock versão errada1;alteraçãoLICENSE após último probe1;extra-file1;LICENSE adulterada1;junction1. Todos3hashes originais preservados/cleanup próprio concluído. Negativos nunca executam exe adulterado;mock explícito não substitui probesCLIreais. Apache-2.0 é a licença principal. Re-review/docs/diffsecurity/CIentrega-final/nota49-liberação pendentes;passesfalse/reserva10ATIVA/METAativa/SEC009nãoiniciado.

## SEC-008 — CI de entrega / fechamento documental
Entrega pública c6b4a27bae6ca31e17e80eacd984c5f07163d1d2 / [CI37774382975](https://github.com/millennium42/millani-artes/actions/runs/37774382975), job113301471481/attempt1/main/SHAexato/success19etapas(11success8skip)/zeroartifacts. Root conferiu API dos runs/jobs/steps/artifacts;etapa Scan Git history for secrets obrigatória success antesbuild. Seletor real false baseefb25a13→entrega;nenhum OSV remoto/download/scan integrado,novo job/action/cache/upload,rerun ou CI negativo. Descoberta ghapi indisponível;credentialmanagerGit existente reutilizado somente em memória para API GitHub,sem log/credencial publicado e sem redirects autenticados.
Executor SEC008 done/passestrue/E3 somente instalação local:ausênciaRED/3arquivos oficiais fixos/version2.6.0/4probesCLI/AuthenticodeNotSigned/6casos documentais apósP2/3hashes originais preservados/cleanup concluído/re-reviewsemP0P1P2/checks docs/diffsegredos/CI entrega verde. GatesOSV/dependências sãoSEC009 e posteriores,semclaimzero vulnerabilidades/E4novo. Root4validadores finais/10paths/50JSONanteriores intactos245tuplas/51items48done240files271links;log208642base preservado,entrega213920. Scansegredos diff finalentrega exit0/0achados/0.447s/SHA2564adb72f5e9421b59f32a88a4eac22a2a2e638a7709e2f9eff4f2f3384b95fb05. Reserva10ATIVA até CI do fechamento noSHAexato/nota49preserva48/publicrefs-Gitlimpo/liberação. Recibo final em notas Git evita commit recursivo. Semproduto/manifests/pins/locks/grants/thresholds/workflow/seletor alterado. METAativa:finanças/SQLite/backup/setup.exe/release pendentes. PróximoSEC009←SEC008,INF010;não iniciado neste ciclo.
