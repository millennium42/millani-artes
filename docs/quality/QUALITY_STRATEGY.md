# Estratégia de qualidade

Qualidade é comportamento provado, não volume de código. Cada work item declara aceites, testes, cobertura, segurança e evidência. P0/P1 bloqueiam; release estável busca P2=0. Review compara diff à spec e procura requisito ausente.

## ARC-005 — lint arquitetural

Reutilizar [Biome](../../biome.json) e [harness SAST](../../scripts/test-sast.py) antes de npm quality na etapa existente. [Spec/handoff](../../planning/handoffs/ARC-005.md) delimitam45casos/42CLI/3PS reais, incluindo SQL em componente/helper/entrypoint, driver/subpath/require, IPC literal, controles limpos e indisponibilidade da regra. Falha permanece interrompendo downstream; workflow e thresholds90/90/90/85 intactos.

Root executou format/lint/typecheck/coverage em4.134s:3testes frontend,100%/4linhas5statements1função2branches. Sem mudança Rust, sua suite local não repetida neste item; build nativo e audits continuam exigidos no CI existente. Coverage do scaffold não mede guard Python/Grit ou finanças. Review/CI/nota61 pendentes; limites de padrões SQL constam na spec.
