# Estratégia de qualidade

Qualidade é comportamento provado, não volume de código. Cada work item declara aceites, testes, cobertura, segurança e evidência. P0/P1 bloqueiam; release estável busca P2=0. Review compara diff à spec e procura requisito ausente.

## ARC-005 — lint arquitetural

Reutilizar [Biome](../../biome.json) e [harness SAST](../../scripts/test-sast.py) antes de npm quality na etapa existente. [Spec/handoff](../../planning/handoffs/ARC-005.md) delimitam45casos/42CLI/3PS reais, incluindo SQL em componente/helper/entrypoint, driver/subpath/require, IPC literal, controles limpos e indisponibilidade da regra. Falha permanece interrompendo downstream; workflow e thresholds90/90/90/85 intactos.

Root executou format/lint/typecheck/coverage em4.134s:3testes frontend,100%/4linhas5statements1função2branches. Sem mudança Rust, sua suite local não repetida neste item; build nativo e audits continuam exigidos no CI existente. Coverage do scaffold não mede guard Python/Grit ou finanças. Review/CI/nota61 pendentes; limites de padrões SQL constam na spec.


ARC-005 executor done/E3 após [entrega c5872e20](https://github.com/millennium42/millani-artes/commit/c5872e2026291512fd9eb879e3562200d72b7dd7)/[CI37974060505](https://github.com/millennium42/millani-artes/actions/runs/37974060505), tentativa1/23success.45casos SAST/42CLI/3PS, quality/audits/build Windows aprovados; [prova e limites](../../planning/specs/ARC-005.md#ci--learn--entrega-validada). Fechamento preserva fontes; reserva14 até CI final/nota61 pública. Guard de padrões não comprova SQLi/SQLite/finanças; META ativa/DB001 não iniciado.
