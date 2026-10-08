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
