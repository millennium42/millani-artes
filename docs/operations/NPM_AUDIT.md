# npm audit — gate local

Use a sessão Node definida em [REPRODUCING](REPRODUCING.md), com Node24.21.0/npm11.19.0 no PATH. O runtime global antigo é preservado; versão diferente bloqueia. Python≥3.12 já disponível. Na raiz:

```powershell
rtk proxy python -B scripts/test-npm-audit.py
rtk proxy python -B scripts/check_npm_audit.py
```

O checker executa a CLI npm instalada, com limiar high, incluindo dev/optional/peer e somente o lock. High/critical, falha de rede/processo, JSON inválido/parcial ou contagens contraditórias retornam1. Info/low/moderate são registrados e podem passar conforme [política](../security/DEPENDENCY_SECURITY.md). [npm oficial](https://docs.npmjs.com/cli/v11/commands/npm-audit/) explica que audit-level muda o código de saída, sem filtrar o relatório.

Somente package.json e package-lock públicos são copiados para artifacts/sec010-check, ausente antes/criado e removido pelo checker com guardas de contenção/reparse. npmrc do projeto não é copiado; user/globalconfigs são vazios explícitos, env de configuração/preload/token removida, cache/logs isolados. Sem install/fix/update/scripts/upload; stderr e relatório bruto ficam em memória. Rede npmjs recebe metadados das dependências públicas, sem dados financeiros. Não é permitido usar a aprovação OSV como exceção npm.

NPM_AUDIT_EVIDENCE contém versões, nível, contagem declarada conferida contra número de entradas do lock, vulnerabilidades, SHA dos inputs/source e duração. Isso não é prova de inventário independente, pacote livre de malware, nem segurança total. Se bloqueado, saída é código estático NPM_AUDIT_BLOCKED; diagnosticar/corrigir antes promover. Não ocultar finding, ignorar erro ou reduzir o limiar.

[Fixtures](../../scripts/test-npm-audit.py) usam servidor HTTP loopback fictício (bulk advisory/packument), CLI real e advisories sintéticos; verificam clean/moderate/high/critical, inventário enviado de dev+optional, npmrc/env inefetivos e scripts não executados. Nenhuma requisição negativa no registry público. artifacts/sec010-tests é scratch próprio; guardas também negam junction real, preservando sentinela. Relatórios simulados testam parser/erro, não substituem audit online.

Execução root em2026-10-08:40casos/4CLIreais/8.926s; audit real120dependências/zero reportadas/1.967s. Hashlockbb9dd4c788dc028f426b4a7d7c3ef3be7ce02b5374ac555455382e9dfb52b284, manifest475d1d3c95ab6a79ae5944eba4c34d100e15fc890a5f21aafac687970d59484a; preservados antes/depois. Integração remota do audit é SEC012, após SEC011. CI atual continua cobrindo gates existentes e build quando seletor exigir. CoveragePython percentual não registrado; limiares do produto inalterados.

Review P2 corrigido: resumo exige exatamente as severidades conhecidas +total, e testes rejeitam Python otimizado (-O/PYTHONOPTIMIZE). RED real mostrou resumo unknown aceito antes; teste negativo agora bloqueia. Rodar em modo normal é obrigatório.

## SEC-010 — entrega comprovada / fechamento

SEC010 executor done/passes:true/E3 após entrega públicaa93b3de4bb2dc13a864c263f73a4c934647c3de9/[CI37815340184](https://github.com/millennium42/millani-artes/actions/runs/37815340184)/job113442423726/attempt1/main/success21etapas/uma job/1artifactcoverage existente. Root conferiu API/logs reais em memória: OSV39casos/3.66s/537pacotes/3registros aprovados3excepted0outros/expiry2026-10-23, Gitleaks8.30.1/111commits/0achados/0.77s, sourceSHAexato. NativePE32+AMD64/custom-protocol/8563200bytes/SHA256edcb99371fd46f252b0c53b642c9d9da39bb1c049f1f0dee8992d5f595780aee, RustCargo1.99/Node24.21/image20260925.250.1; produto/locks/pins/grants/thresholds preservados. Qualityfrontend passouformat/lint/typecheck/coverage; artifact11566902508/retention1dia solicitado(86399s) lido em memória:100%4linhas5statements1function2branches, thresholds90/90/90/85 inalterados. Isso é apenas frontendscaffold atual, nãofinanças/Rust/Python/setup.exe.
Gate npm40casos locais/4CLIreaisloopback/7.799s, auditreal120dependências/0reportadas/1.818s noSHAentrega; inputs{"package-lock.json":"bb9dd4c788dc028f426b4a7d7c3ef3be7ce02b5374ac555455382e9dfb52b284","package.json":"475d1d3c95ab6a79ae5944eba4c34d100e15fc890a5f21aafac687970d59484a"} preservados. Checker/testes npm NÃO executados remotamente; integraçãoCIaudit éSEC012. Nenhuma nova etapa/job/action/upload/cache ou rerun nesta tarefa. Review2P2corrigidos/re-reviewE2semachados; fonte/testes sem mudanças depois das verificações. Doisdelegadoseconômicos (P-QA6-lunalow/P-REV5.6-solmedium),0escalonamentos; custo/cache/tokens/latênciamodellos/retriesevitados não registrado. Contexto sem rerodar scans/testes após mudanças puramente documentais.
passes:true refere-se ao executor/CIentrega observado; LIBERADA ainda exige CI no SHAfinal/nota51 preservando50/publicrefs/Gitlimpo/cleanup e liberação11paths. Reserva11ATIVA até esse recibo externo àárvore.53items50done253tracked/52itens(em24JSONprévios)245tuplas/prefixo234620preservados. META final appfinanceiro/SQLite/backup/setup.exe/release não concluída; próximoSEC011 só após liberação/não iniciado.

## Integração SEC-012

O gate e seus 40 casos passam a ser obrigatórios no workflow existente antes da instalação e do build, inclusive em diff documental; Node fixo é preparado sem condição. [Operação integrada](CI_AUDITS.md) descreve propagação de erros e limites; CI exactSHA ainda pendente neste estado. Checker, lockfiles e política high/critical preservados.

## CI SEC-012 observado

[CI37832138829](https://github.com/millennium42/millani-artes/actions/runs/37832138829) noSHA680e8d2922f8149959df53c62e9b80390651e5c0/attempt1 passou23etapas. Gates reais npm120depszero eCargo417deps0vulns2excepted,40/47/42casos incluindo14PowerShell ebootstrapfrio verdadeiro. Falha inicialRTK local foi reproduzida e corrigida comcmd/git nativos, casos preservados. Reserva16ATIVA atéCIfinal/nota53; não háclaim de domíniofinanceiro/setup.exe. Histórico anterior mantém estado observado àépoca; estado vigente/details no handoffSEC012.
