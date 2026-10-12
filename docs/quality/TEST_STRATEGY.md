# Estratégia de testes

Unitários cobrem cálculos, estados, validações e invariantes. Integração cobre SQLite, migrations, constraints, transações e backup/restore com banco temporário limpo. Componentes cobrem formulários, erros e acessibilidade. Smoke/E2E cobrem fluxo de produção progressivo.

Teste mockado não prova integração. Toda regra de `INVARIANTS.md` terá ao menos um teste explícito, mapeado em `docs/traceability/INVARIANT_TEST_MATRIX.md`.

Para a [fronteira transacional](../architecture/TRANSACTION_MODEL.md), testes unitários de orquestração não bastam. DB-011/012/014 usarão banco temporário real para verificar sucesso completo, rejeição da validação, falha entre gravações, falha de auditoria, commit e cancelamento; conferir estado completo e audit_event, não apenas retorno do método. Cada consumidor repetirá as falhas relevantes às suas invariantes. Pânico, erro de cancelamento e estado incerto exigem prova do comportamento do executor; não presumir cancelamento por mock ou por Drop. ARC-004 é definição documental: seus validators não executam transação e não fecham INV-FIN-006/007.

## DB-001 — harness SQLite em memória

`src-tauri/tests/sqlite_harness.rs` usa conexão SQLite real `open_in_memory`, dados sintéticos e parâmetros SQL. O teste verifica schema inicialmente vazio, round-trip de `O'Brien`, isolamento entre duas conexões simultâneas e conexão limpa após descarte; registra `sqlite_version()` (SQLite 3.53.2 nesta execução). Isso comprova apenas o harness Rust local. Não comprova armazenamento em arquivo, permissões, migrations, locks entre processos, backup/restore, invariantes financeiras ou segurança global contra SQL injection. Root executou 14 testes Rust e teste release dedicado, todos passaram. CI Documentation 38095838774/job114341423695 passou 23/23 etapas no SHA `315fd0feabdb3bc55223441154a9bbdc615c2fd2`; release harness passou 1 teste com SQLite 3.53.2. O aceite E4 das exceções Cargo/OSV tem escopo limitado ao alvo Windows e prazo registrados na spec; não altera a cobertura funcional desse harness nem os seus limites. Cobertura reportada é somente do scaffold (4 linhas), não cobertura Rust global.
## DB-002 — arquivo SQLite local

Testes de arquivo real cobrem criação, reabertura, round-trip e rejeições com sentinelas. E3 local: 16 testes Rust, release harness 3/3, fmt/clippy; review E2 sem achados. Gates pós-aceite Cargo/OSV passaram; tentativas concorrentes de fixtures/gates colidiram em scratch e retries locais em série passaram. Coverage Rust global não registrada. CI 38103942047 validou a entrega pública (23/23); CI do fechamento documental pendente. Testes não provam startup, containment, migrations, finanças ou backup/restore. Veja [spec/handoff](../../planning/specs/DB-002.md).

## DB-003 — executor transacional de migrations

O harness de arquivo real cobre DDL em banco vazio, upgrade com `O'Brien` parametrizado e reabertura, SQL vazio, rollback DDL+DML, início negado por transação externa, commit `BUSY` com reader/timeout zero e erros opacos. E3 local root: debug harness 9/9, suíte Rust 22/22, release harness 9/9, SQLite 3.53.2; fmt/clippy passaram após correção. Review E2 sem P0/P1/P2. CI de entrega 38197556421 passou 23/23; CI final documental 38198511557/job114646455371 passou 16 success/7 skipped. Coverage Rust global não registrada. Veja [spec/handoff](../../planning/specs/DB-003.md) e [handoff](../../planning/handoffs/DB-003.md).

## DB-004 — ledger e execução versionada

Arquivo SQLite real cobre schema vazio e ledger vazio, versões 1/2 e reabertura com `O'Brien`, metadados/versões inválidos sem mudanças, rollback após falha SQL e falha real no insert do ledger. E3 local root: Rust 29/29 (6 lib/7 bin/16 harness), release harness 16/16, SQLite 3.53.2, fmt/clippy; review incremental sem P0/P1/P2 após corrigir dois P2. CI de entrega 38201472821 passou 23/23; CI final 38299901151/job114980000210 no SHA 94e37d38f5df375ffb9492e3f80d272f2f44b331 passou 16 success/7 skipped/0 artifact. Nota 65 publicada (refs eafc15283e4eb5e6906002a04ef229cf903474ff); 64 notas e refs remotos preservados, reserva liberada. Coverage Rust global não registrada; [spec/handoff](../../planning/specs/DB-004.md).


## DB-005 — migration versionada vazia em arquivo

Dois testes novos cobrem arquivo vazio, aplicação da versão 1 por API versionada com SQL vazio, ledger único/shape/CHECK/autocommit, reabertura/integrity_check, isolamento de dois arquivos e constraints de duplicata/0/-1. E3 root: RED caller controlado 0 vs 1; GREEN dois testes, Rust 31/31 (6 lib/7 bin/18 harness), release 18/18 em SQLite 3.53.2, fmt/clippy; 16 testes anteriores inalterados. Review E2 sem P0/P1/P2. A prova cobre harness/ledger inicial, não migration financeira nem startup. CI entrega 38303443795/job114991660147 passou 23/23 sem skips; release harness 18/18 em SQLite 3.53.2. Scan staged de 10 paths (49.326 bytes/0 findings/11 hashes) e validators da entrega (322 arquivos/28 fontes/67 JSON+evidence) passaram. CI documental final 38304515870/job114995197572 no SHA 1d92fdd185cf24aececd1fa5b5708d9d0efeab03 passou 16 etapas/7 skipped/0 artifacts; nota66 publicada nas refs 0c75692168c5c220c364733567b5dc28a01a5739, preservadas 65 notas/refs e reserva11 liberada; coverage Rust global não registrada. Veja [spec](../../planning/specs/DB-005.md) e [handoff](../../planning/handoffs/DB-005.md).


## DB-006 — snapshot lógico SQLite

Um teste de integração e fixture confiável criam snapshot sintético em dois arquivos físicos via executor existente, verificando estado vazio, ledger/tabelas, sentinelas, shape, autocommit, reabertura, integridade e isolamento. RED da fixture comentada falhou em runtime (0 passados/1 falho/18 filtrados; 0 objetos encontrados versus 2 esperados); GREEN apenas preencheu SQL e manteve o oráculo. E3: fmt/clippy, Rust32/32 (6/7/19), release19/19, SQLite3.53.2; os 18 testes anteriores preservados. Review E2 sem P0/P1/P2. Executor `done`/E3/`passes=true`; CI entrega 38307470576/job115004892529 no SHA 0b8f2a7d0dc8d236d91498cc20e28b358c62dcce passou 23/23 e release19/19 SQLite3.53.2; CI documental/nota67 pendentes; teste não valida migração versionada, migração de produto, startup ou backup/restore. [Spec](../../planning/specs/DB-006.md), [handoff](../../planning/handoffs/DB-006.md).


## DB-007 — upgrade versionado do snapshot

[Spec](../../planning/specs/DB-007.md)/[handoff](../../planning/handoffs/DB-007.md): RED controlado ledger `[1]` contra `[1,2]`; GREEN apenas caller versionado. Root E3: Rust33/33, release20/20 (SQLite3.53.2), 19 testes anteriores preservados; review E2 sem P0/P1/P2. Cargo/OSV e CI entrega passaram; executor `done`/E3/`passes=true`/`humanGateRequired=false`. CI documental/nota68/preservação67 notas+refs/liberação11 paths pendentes/ACTIVE. Prova somente upgrade sintético de teste.
