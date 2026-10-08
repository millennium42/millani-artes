# Gitleaks local — SEC-006

Gitleaks é uma ferramenta local de detecção de segredos. A instalação SEC-006 fornece o executável usado pelo gate SEC-007 abaixo. O [registro](../../governance/TOOL_REGISTER.md), a [spec](../../planning/specs/SEC-006.md) e o [handoff](../../planning/handoffs/SEC-006.md) delimitam a evidência.

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

## SEC-007 — gate do histórico Git
[Spec](../../planning/specs/SEC-007.md) e [handoff](../../planning/handoffs/SEC-007.md):15 checks locais passaram e149 commits disponíveis varridos com zeroachados no SHA42bcfc648467f6ba835e5250fc279204b0a6a4f0. Requisito globalSEC007 continua parcial: demais scanners/auditorias ainda possuem executores próprios; logs/chaves/runtime/finanças não comprovados por este scanner.
Wrapper Windows/Python≥3.12 fixa8.30.1 e os digests acima, baixa somente se cache ausente, valida ZIP antes gravar/executar e recusa cache adulterado. Cache criado pelo wrapper contém três arquivos extraídos; ZIP da instalação SEC006 pode permanecer. SHA256/NotSigned/mesmo fornecedor não autenticam independentemente o código. Sem setupaction/cachejob/cloudscanner extra.
Na raiz, com assertions habilitadas:

```powershell
rtk proxy python -B scripts/test-gitleaks.py
if ($LASTEXITCODE -ne 0) { throw 'Gitleaks guard checks failed.' }
rtk proxy python -B scripts/check_gitleaks.py
if ($LASTEXITCODE -ne 0) { throw 'Gitleaks scan failed.' }
```

A etapa obrigatória no CI existente executa ambos antes do build, checkout fetch-depth0. Scan git --all dos refs disponíveis, defaults explícitos, inline allow desativado, .gitleaksignore na raiz recusado, envGIT_/GITLEAKS_ removido; shallow/empty/nonGit/report inválido/erro bloqueiam. Report JSON em memória redacted100, saída pública só códigos/contagens/versão/hash/SHA/tempo. Não adicionar ignore/allowlist/baseline para ocultar achados. O teste cria e remove somente artifacts/sec007-tests; gate usa artifacts/sec007-check, ambos precisam estar ausentes e sem junction/symlink nos ancestrais. Cleanup verifica contenção/árvore; Git readonly usa chmod limitado à fixture. Se recusar preexistência, verificar conteúdo e ownership antes intervenção, não apagar artifacts inteiro.

Os15 casos distinguem7 integrações Git-gate de8 fronteiras sintéticas/CLI real; download fresco será exercitado pelo runner sem cache. Primeiro RED demonstrou falha do stub permissivo; tentativa com cleanup incompleto descartada e corrigida. Histórico local149commits zeroachados, sem garantia de detectar todo segredo ou objetos/refs inacessíveis. Rawreport/token/fixtures não são publicados. Revisão e CI exato pendentes, passesfalse/reserva12ATIVA; segurança global permanece parcial.

SEC007 CORRECT: P2 de cache extra reproduzido (RED8.977s) e corrigido antes execução. Cache aceita exatamente os três arquivos obrigatórios e, opcionalmente, somente o ZIP oficial validado de SEC006.16casos passaram/9.135s;scan149commits zeroachados/1.494s. Re-review/checks/diffscan/CI exato pendentes;passesfalse/reserva12ATIVA/METAativa/SEC008nãoiniciado.

SEC007 executor done/passestrue/E3:16checks/root,reviewP2corrigido/re-reviewsemachados,entrega2597cf4047b870ee30e9bca8f029f9884269c6cf/[CI37768499862](https://github.com/millennium42/millani-artes/actions/runs/37768499862) success20etapas/1coverageartifact. Gate7 obrigatório/103commits públicos disponíveis/0achados;local pré-entrega149inclui notas. Requisito globalSEC007 continua parcial, sem E4 novo. CI final/nota48preserva47/refsGitclean/liberação pendentes/reserva12ATIVA/METAativa/SEC008nãoiniciado.
