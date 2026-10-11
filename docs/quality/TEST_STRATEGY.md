# Estratégia de testes

Unitários cobrem cálculos, estados, validações e invariantes. Integração cobre SQLite, migrations, constraints, transações e backup/restore com banco temporário limpo. Componentes cobrem formulários, erros e acessibilidade. Smoke/E2E cobrem fluxo de produção progressivo.

Teste mockado não prova integração. Toda regra de `INVARIANTS.md` terá ao menos um teste explícito, mapeado em `docs/traceability/INVARIANT_TEST_MATRIX.md`.

Para a [fronteira transacional](../architecture/TRANSACTION_MODEL.md), testes unitários de orquestração não bastam. DB-011/012/014 usarão banco temporário real para verificar sucesso completo, rejeição da validação, falha entre gravações, falha de auditoria, commit e cancelamento; conferir estado completo e audit_event, não apenas retorno do método. Cada consumidor repetirá as falhas relevantes às suas invariantes. Pânico, erro de cancelamento e estado incerto exigem prova do comportamento do executor; não presumir cancelamento por mock ou por Drop. ARC-004 é definição documental: seus validators não executam transação e não fecham INV-FIN-006/007.

## DB-001 — harness SQLite em memória

`src-tauri/tests/sqlite_harness.rs` usa conexão SQLite real `open_in_memory`, dados sintéticos e parâmetros SQL. O teste verifica schema inicialmente vazio, round-trip de `O'Brien`, isolamento entre duas conexões simultâneas e conexão limpa após descarte; registra `sqlite_version()` (SQLite 3.53.2 nesta execução). Isso comprova apenas o harness Rust local. Não comprova armazenamento em arquivo, permissões, migrations, locks entre processos, backup/restore, invariantes financeiras ou segurança global contra SQL injection. Root executou 14 testes Rust e teste release dedicado, todos passaram. CI Documentation 38095838774/job114341423695 passou 23/23 etapas no SHA `315fd0feabdb3bc55223441154a9bbdc615c2fd2`; release harness passou 1 teste com SQLite 3.53.2. O aceite E4 das exceções Cargo/OSV tem escopo limitado ao alvo Windows e prazo registrados na spec; não altera a cobertura funcional desse harness nem os seus limites. Cobertura reportada é somente do scaffold (4 linhas), não cobertura Rust global.
## DB-002 — arquivo SQLite local

Testes de arquivo real cobrem criação, reabertura, round-trip e rejeições com sentinelas. E3 local: 16 testes Rust, release harness 3/3, fmt/clippy; review E2 sem achados. Gates pós-aceite Cargo/OSV passaram; tentativas concorrentes de fixtures/gates colidiram em scratch e retries locais em série passaram. Coverage Rust global não registrada. CI 38103942047 validou a entrega pública (23/23); CI do fechamento documental pendente. Testes não provam startup, containment, migrations, finanças ou backup/restore. Veja [spec/handoff](../../planning/specs/DB-002.md).
