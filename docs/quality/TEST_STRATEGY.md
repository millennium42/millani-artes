# Estratégia de testes

Unitários cobrem cálculos, estados, validações e invariantes. Integração cobre SQLite, migrations, constraints, transações e backup/restore com banco temporário limpo. Componentes cobrem formulários, erros e acessibilidade. Smoke/E2E cobrem fluxo de produção progressivo.

Teste mockado não prova integração. Toda regra de `INVARIANTS.md` terá ao menos um teste explícito, mapeado em `docs/traceability/INVARIANT_TEST_MATRIX.md`.
