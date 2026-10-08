# Gitleaks local — SEC-006

Gitleaks é uma ferramenta local de detecção de segredos. Esta instalação prepara SEC-007, que adicionará o gate ao CI. O [registro](../../governance/TOOL_REGISTER.md), a [spec](../../planning/specs/SEC-006.md) e o [handoff](../../planning/handoffs/SEC-006.md) delimitam a evidência.

## Distribuição fixa e integridade
[Release oficial v8.30.1](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1), commit `83d9cd684c87d95d656c1458ef04895a7f1cbd8e`, [licença MIT](https://github.com/gitleaks/gitleaks/blob/83d9cd684c87d95d656c1458ef04895a7f1cbd8e/LICENSE). [ZIP Windows x64](https://github.com/gitleaks/gitleaks/releases/download/v8.30.1/gitleaks_8.30.1_windows_x64.zip) de 8.438.883 bytes e [checksums](https://github.com/gitleaks/gitleaks/releases/download/v8.30.1/gitleaks_8.30.1_checksums.txt) conferidos com o digest da API oficial.
- ZIP SHA256: `d29144deff3a68aa93ced33dddf84b7fdc26070add4aa0f4513094c8332afc4e`.
- checksums.txt SHA256: `061476c21adaf5441516f96f185c1a4706a83cd6329b9b38762271b3d4a52fae`.
- LICENSE SHA256: `e3884b252b3bfc045e55be43a34d1e80da070bc6f804ac95bf4660e97d62ebc6`.
- gitleaks.exe SHA256: `17157e2ee8b76fc8b1d8bee607a250e34b8a8023c8bc81822d4b5ee4d78fcb7c`, 22.575.104 bytes, PE AMD64/PE32+.

Cache dedicado: `artifacts/tools/gitleaks/8.30.1/`, com ZIP, gitleaks.exe, LICENSE e README.md; ignorado pelo Git. Não requer administrador, PATH persistente, serviço, hook ou Docker. Nesta máquina Authenticode retornou NotSigned, sem certificado de signatário/timestamp. Hashes conferem bytes contra a fonte consultada; assinatura/attestation independente, auditoria completa e telemetria não foram comprovadas.

## Reinstalar / verificar
1. Na raiz do projeto, escolher o cache dedicado ausente. Se houver instalação, conferir hash/versão antes reutilizar. Recusar symlinks/junctions no destino e ancestrais; não sobrescrever outro diretório.
2. Baixar o ZIP e o checksum da release fixa, conferir tamanho e os SHA256 acima antes extrair/executar. Conferir também a linha do ZIP no checksum oficial.
3. Abrir o ZIP e exigir exatamente LICENSE, README.md, gitleaks.exe, sem caminhos/diretórios/duplicatas ou links. Extrair somente esses três membros, nunca executar um instalador de origem diferente. Conferir LICENSE e exe pelos hashes acima.
4. Executar pelo path explícito, em PowerShell 7 na raiz:

```powershell
$sec006Exe = Join-Path (Get-Location) 'artifacts/tools/gitleaks/8.30.1/gitleaks.exe'
$sec006Hash = (Get-FileHash -LiteralPath $sec006Exe -Algorithm SHA256).Hash.ToLowerInvariant()
if ($sec006Hash -ne '17157e2ee8b76fc8b1d8bee607a250e34b8a8023c8bc81822d4b5ee4d78fcb7c') { throw 'Gitleaks binary integrity mismatch.' }
rtk proxy $sec006Exe version
if ($LASTEXITCODE -ne 0) { throw 'Gitleaks version command failed.' }
```

Versão esperada: 8.30.1. Download usa GitHub/TLS; execução observa fontes locais. Conexões/telemetria durante scanner não foram medidas. Não introduzir baseline, ignore ou allowlist para esconder achado.

## Probes reproduzíveis sem credencial real
Os probes de instalação foram executados em diretório próprio isolado, sem .gitleaks.toml/.gitleaksignore nem variáveis GITLEAKS_* no subprocesso, usando regras default embutidas. Não enviar valores reais. Para repetir os probes, use uma sessão/diretório temporário controlado com essas ausências e o path absoluto do executável. O valor falso é gerado em memória:

```powershell
'Millani Artes installation probe, no secret.' | rtk proxy $sec006Exe stdin --redact=100 --no-banner --no-color --exit-code 1 --timeout 60
if ($LASTEXITCODE -ne 0) { throw 'Clean probe failed.' }
$sec006Alphabet = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'.ToCharArray()
$sec006Fake = 'gh' + 'p_' + ((Get-Random -InputObject $sec006Alphabet -Count 36) -join '')
("token = " + $sec006Fake) | rtk proxy $sec006Exe stdin --redact=100 --no-banner --no-color --exit-code 1 --timeout 60
if ($LASTEXITCODE -ne 1) { throw 'Synthetic leak was not rejected.' }
$sec006Fake = $null
```

O probe negativo deve detectar github-pat, exit1; não prova que todo segredo seria detectado. Reports de prova, quando necessários, ficam somente em artifacts/, com `--report-format json --report-path <path-controlado> --redact=100`. Conferir ausência do valor sintético em stdout/stderr/report e Secret=REDACTED antes reter resumo; não publicar report bruto. Invocação inválida deve retornar nãozero e jamais ser aceita como scan verde.

## Execução E3 — 2026-10-07
Root/RTK/Windowsx64/Python3.12.10: download ZIP conferido/extraiu apenas três membros (1.218s); exe/licença/hash/PE verificados antes executar. CLI version exit0/8.30.1/1.075s; probe clean exit0/zero achados/0.610s; synthetic-leak exit1/um github-pat/0.580s; invalid-invocation exit126/unknown flag/0.374s através do RTK proxy. Redaction100 verificada em stdout/stderr/JSON, sem valor sintético publicado. Binário preservou hash. Raiz artifacts/sec006-probes removida após checar contenção e reparsepoints; ausência confirmada. Cache/binário/ZIP/licença/reports/probes ignorados pelo artifacts/ existente.

A execução não foi scan global do histórico nem gate remoto. SEC-007 precisa provar bloqueio de achado/falha no workflow e run exato; requisitos de segredos/logs/chaves/runtime/finanças continuam com seus próprios executores. Coverage do produto não alterada. Um scanner não substitui validação/sanitização por construção ou auditoria financeira.

## Remoção
Fechar processos que usem o binário. Validar que o caminho absoluto é exatamente o cache deste projeto, contido em artifacts/tools/gitleaks, e sem reparsepoints; então remover somente esse diretório dedicado. Não usar curinga ou remover artifacts inteiro. Sem serviço/PATH persistente a desfazer; licenças devem acompanhar eventual redistribuição. Nenhuma remoção do cache foi executada nesta tarefa.
