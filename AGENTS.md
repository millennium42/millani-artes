# Regras para agentes

Leia nesta ordem: `README.md`, este arquivo, `00-MAPA-DO-PROJETO.md`, regras de produto, ADRs aceitos, o work item e sua spec. Autoridade: decisão humana registrada > `docs/product/` > ADR aceito > spec > este arquivo > demais textos.

Trabalhe em uma tarefa por vez. Antes de editar: status, branch, HEAD, CI, testes e arquivos relacionados. Siga `SPEC → RECON → RED → BUILD → TEST → SECURITY → REVIEW → CORRECT → VERIFY → CI → LEARN`. Não invente evidência; E3 requer execução, e CI verde exige run remoto no SHA atual. Para delegação, siga `governance/MULTI_AGENT_POLICY.md` e escolha uma persona de `governance/AGENT_PERSONAS.md`; para contexto e chamadas, siga `governance/TOKEN_ECONOMY_POLICY.md`.

Use o mínimo necessário: reutilize antes de criar, prefira stdlib/plataforma e não adicione dependência sem ADR/tool register. Teste invariantes explicitamente; cobertura não substitui comportamento. Não reduza thresholds. Para Rust execute fmt, clippy, test e audit; para TypeScript execute format/lint/typecheck/test/coverage quando existirem. Segurança proporcional ao diff é obrigatória.

Migrations são versionadas, aditivas quando possível, nunca editadas após aplicadas, testadas em banco vazio e upgrade. Commits são atômicos. Atualize spec, matrizes, `planning/PROGRESS_LOG.md` append-only e handoff antes de `passes: true`; não inicie a próxima tarefa.

Formato de atualização: `ID | estado | evidência E0–E4 | checks | riscos | próximo ID`. Ausências devem ser `não registrado`.


Para retomar com os trabalhadores locais preparados por decisão humana de 15/10/2026, leia também [roteamento e personas locais](governance/LOCAL_MODEL_ROUTING.md). GPT participa das revisões exigidas e dos casos extremos; o ciclo, a hierarquia e os gates acima permanecem obrigatórios.
