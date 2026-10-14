# Estratégia de testes

Unitários cobrem cálculos, estados, validações e invariantes. Integração cobre SQLite, migrations, constraints, transações e backup/restore com banco temporário limpo. Componentes cobrem formulários, erros e acessibilidade. Smoke/E2E cobrem fluxo de produção progressivo.

Teste mockado não prova integração. Toda regra de `INVARIANTS.md` terá ao menos um teste explícito, mapeado em `docs/traceability/INVARIANT_TEST_MATRIX.md`.

Para a [fronteira transacional](../architecture/TRANSACTION_MODEL.md), testes unitários de orquestração não bastam. DB-011/012/014 usarão banco temporário real para verificar sucesso completo, rejeição da validação, falha entre gravações, falha de auditoria, commit e cancelamento; conferir estado completo e audit_event, não apenas retorno do método. Cada consumidor repetirá as falhas relevantes às suas invariantes. Pânico, erro de cancelamento e estado incerto exigem prova do comportamento do executor; não presumir cancelamento por mock ou por Drop. ARC-004 é definição documental: seus validators não executam transação e não fecham INV-FIN-006/007.

DB-013 executou cinco focais, release harness36/36 e CI de entrega no SHA final; SQLite3.53.2. A API exige `Ok(1)`, e `RAISE(IGNORE)` verifica rejeição de zero inserções. Prova persistência/constraints isoladas; DB-014 deve provar atomicidade junto às escritas financeiras. CI documental/nota74 pendentes; [handoff](../../planning/handoffs/DB-013.md).

## DB-001 — harness SQLite em memória

`src-tauri/tests/sqlite_harness.rs` usa conexão SQLite real `open_in_memory`, dados sintéticos e parâmetros SQL. O teste verifica schema inicialmente vazio, round-trip de `O'Brien`, isolamento entre duas conexões simultâneas e conexão limpa após descarte; registra `sqlite_version()` (SQLite 3.53.2 nesta execução). Isso comprova apenas o harness Rust local. Não comprova armazenamento em arquivo, permissões, migrations, locks entre processos, backup/restore, invariantes financeiras ou segurança global contra SQL injection. Root executou 14 testes Rust e teste release dedicado, todos passaram. CI Documentation 38095838774/job114341423695 passou 23/23 etapas no SHA `315fd0feabdb3bc55223441154a9bbdc615c2fd2`; release harness passou 1 teste com SQLite 3.53.2. O aceite E4 das exceções Cargo/OSV tem escopo limitado ao alvo Windows e prazo registrados na spec; não altera a cobertura funcional desse harness nem os seus limites. Cobertura reportada é somente do scaffold (4 linhas), não cobertura Rust global.
## DB-002 — arquivo SQLite local

Testes de arquivo real cobrem criação, reabertura, round-trip e rejeições com sentinelas. E3 local: 16 testes Rust, release harness 3/3, fmt/clippy; review E2 sem achados. Gates pós-aceite Cargo/OSV passaram; tentativas concorrentes de fixtures/gates colidiram em scratch e retries locais em série passaram. Coverage Rust global não registrada. CI 38103942047 validou a entrega pública (23/23); DB-002 fechado: CI documental 38104862341/job114368093002 no SHA 1ef3bd833708f46e0e26188e71b3cfb83f679014 passou 16/7/0; nota63 pública. Testes não provam startup, containment, migrations, finanças ou backup/restore. Veja [spec/handoff](../../planning/specs/DB-002.md).

## DB-003 — executor transacional de migrations

O harness de arquivo real cobre DDL em banco vazio, upgrade com `O'Brien` parametrizado e reabertura, SQL vazio, rollback DDL+DML, início negado por transação externa, commit `BUSY` com reader/timeout zero e erros opacos. E3 local root: debug harness 9/9, suíte Rust 22/22, release harness 9/9, SQLite 3.53.2; fmt/clippy passaram após correção. Review E2 sem P0/P1/P2. CI de entrega 38197556421 passou 23/23; CI final documental 38198511557/job114646455371 passou 16 success/7 skipped. Coverage Rust global não registrada. Veja [spec/handoff](../../planning/specs/DB-003.md) e [handoff](../../planning/handoffs/DB-003.md).

## DB-004 — ledger e execução versionada

Arquivo SQLite real cobre schema vazio e ledger vazio, versões 1/2 e reabertura com `O'Brien`, metadados/versões inválidos sem mudanças, rollback após falha SQL e falha real no insert do ledger. E3 local root: Rust 29/29 (6 lib/7 bin/16 harness), release harness 16/16, SQLite 3.53.2, fmt/clippy; review incremental sem P0/P1/P2 após corrigir dois P2. CI de entrega 38201472821 passou 23/23; CI final 38299901151/job114980000210 no SHA 94e37d38f5df375ffb9492e3f80d272f2f44b331 passou 16 success/7 skipped/0 artifact. Nota 65 publicada (refs eafc15283e4eb5e6906002a04ef229cf903474ff); 64 notas e refs remotos preservados, reserva liberada. Coverage Rust global não registrada; [spec/handoff](../../planning/specs/DB-004.md).


## DB-005 — migration versionada vazia em arquivo

Dois testes novos cobrem arquivo vazio, aplicação da versão 1 por API versionada com SQL vazio, ledger único/shape/CHECK/autocommit, reabertura/integrity_check, isolamento de dois arquivos e constraints de duplicata/0/-1. E3 root: RED caller controlado 0 vs 1; GREEN dois testes, Rust 31/31 (6 lib/7 bin/18 harness), release 18/18 em SQLite 3.53.2, fmt/clippy; 16 testes anteriores inalterados. Review E2 sem P0/P1/P2. A prova cobre harness/ledger inicial, não migration financeira nem startup. CI entrega 38303443795/job114991660147 passou 23/23 sem skips; release harness 18/18 em SQLite 3.53.2. Scan staged de 10 paths (49.326 bytes/0 findings/11 hashes) e validators da entrega (322 arquivos/28 fontes/67 JSON+evidence) passaram. CI documental final 38304515870/job114995197572 no SHA 1d92fdd185cf24aececd1fa5b5708d9d0efeab03 passou 16 etapas/7 skipped/0 artifacts; nota66 publicada nas refs 0c75692168c5c220c364733567b5dc28a01a5739, preservadas 65 notas/refs e reserva11 liberada; coverage Rust global não registrada. Veja [spec](../../planning/specs/DB-005.md) e [handoff](../../planning/handoffs/DB-005.md).


## DB-006 — snapshot lógico SQLite

Um teste de integração e fixture confiável criam snapshot sintético em dois arquivos físicos via executor existente, verificando estado vazio, ledger/tabelas, sentinelas, shape, autocommit, reabertura, integridade e isolamento. RED da fixture comentada falhou em runtime (0 passados/1 falho/18 filtrados; 0 objetos encontrados versus 2 esperados); GREEN apenas preencheu SQL e manteve o oráculo. E3: fmt/clippy, Rust32/32 (6/7/19), release19/19, SQLite3.53.2; os 18 testes anteriores preservados. Review E2 sem P0/P1/P2. Executor `done`/E3/`passes=true`; CI entrega 38307470576/job115004892529 no SHA 0b8f2a7d0dc8d236d91498cc20e28b358c62dcce passou 23/23 e release19/19 SQLite3.53.2; CI documental/nota67 pendentes; teste não valida migração versionada, migração de produto, startup ou backup/restore. [Spec](../../planning/specs/DB-006.md), [handoff](../../planning/handoffs/DB-006.md).


## DB-007 — upgrade versionado do snapshot

[Spec](../../planning/specs/DB-007.md)/[handoff](../../planning/handoffs/DB-007.md): RED controlado ledger `[1]` vs `[1,2]`; GREEN caller versionado. Root E3 Rust33/33, release20/20 SQLite3.53.2, 19 antigos preservados; review sem P0/P1/P2. CI entrega e documental passaram; executor `done`/E3/`passes=true`. DB-007 fechado: CI final 38320496225/job115047171038 no SHA 323e336b53c74147c4b70bdec44391dfad29fe72, 16 success/7 skipped/0 artifacts; nota68 pública refs f9786502025de68b33275fff766f0428cc862bd1, 67 notas preservadas e reserva11 liberada.


## DB-008 — evidência de rollback

[Spec](../../planning/specs/DB-008.md)/[handoff](../../planning/handoffs/DB-008.md): tarefa documental sem código; cinco fixtures existentes PASS em stdout debug/release. Root fmt/clippy/Rust33/33/release20/20; review E2 sem achados. CI documental entrega 38322109825/job115052349852 no SHA 08a214888bf581d40e74a143bfdfdc48dea0878a passou 16 success/7 skipped/0 artifact (docs apenas). Executor `done`/E3/`passes=true`; DB-008 fechado: CI final38322839469/job115054651101 no SHA dfbe60893150c222b17987518f65a161d3e7ef77, 16/7/0; nota69 refs054bfc0f6bce94a795158a89ded50d20d71bd989, 68 notas preservadas/reserva10 liberada. Sem prova de falha física de rollback.


## DB-009 — ativar foreign keys ao abrir

[Spec](../../planning/specs/DB-009.md)/[handoff](../../planning/handoffs/DB-009.md): baseline ON, controle OFF fail-closed; E3 local 34/34 e release21/21; CI entrega 38324569231/job115060146427 passou 23/23. Executor `done`/E3/`passes=true`. DB-009 fechado: CI final 38325659041/job115063600279 no SHA 17f5397efc996091bfb4271a5169d4eede4249fe (16/7/0); nota70 refs 097a4650bac277404275272876296cbd5e682dbb, 69 notas preservadas e reserva12 liberada pelo root.


## DB-010 — provar enforcement de foreign keys (fechado)

[Spec](../../planning/specs/DB-010.md)/[handoff](../../planning/handoffs/DB-010.md): executor E3 fechado no baseline SHA `083ac68e073667db3cba474722962b89ca7cdaa5`, CI `38387475775` SUCCESS 16/7/0, nota71 pública `dc2a872282bfaf9fb035cef0a0c336e6bdc8ff19`, 70 notas preservadas/reserva11 liberada. Prova limitada ao schema sintético; coverage Rust global não registrada.

## DB-011 — contrato de transação de escrita

Quatro testes físicos verificam sucesso e resultado pós-commit, visibilidade entre conexões e reopen, BEGIN IMMEDIATE ocupado sem invocar callback, COMMIT falho por FK deferred, e sanitização de erros. O caso de sanitização usa arquivo real e erro real do driver com identificador sintético dentro do erro; não comprova rollback composto. Um marcador local foi renomeado após Gitleaks sinalizar a variável; literal, driver e oráculos não mudaram. Harness final SHA-256 `aec307745a413a46ba209d3a3228dd0ba14505999b309468d3ad4c5c16cc7913`. Root reexecutou fmt/clippy, Rust39/39 e release26/26; quatro casos passaram debug/release. Review independente E2 e confirmação focal não encontraram P0/P1/P2. CI entrega `38394130462` passou 23/23; release remoto26/26, quatro testes nomeados PASS. Gitleaks staged13 paths/0 findings e validators PASS. CI final e nota72 pendentes. Consulte [spec](../../planning/specs/DB-011.md) e [handoff](../../planning/handoffs/DB-011.md).

## DB-012 — rollback genérico

Cinco testes físicos `db012_*` passaram em debug/release. Root confirmou fmt/clippy, Rust44/44 e release31/31 (SQLite3.53.2); harness SHA-256 `7bf3b01606b6a1d99ced6dc38280f75f1b7c8de6aec859847392a0d4254b32f4`. RED mutado detectou snapshot alterado após callback errar; fonte de produção foi restaurada ao hash DB-011. Exit86 verificou hot journal DELETE e recuperação no reopen. Review E2 sem P0/P1/P2, sem conferir hash/testes/scans. CI entrega `38400843616` e harness remoto31/31 confirmados; validators PASS. Gitleaks final de entrega 11 paths/45.749B/0 findings. CI documental final e nota73 pendentes. Ver [spec](../../planning/specs/DB-012.md) e [handoff](../../planning/handoffs/DB-012.md).


## DB-014 — auditoria na unidade

E3 local: cinco casos físicos passaram em debug/release, inclusive erro de audit/SQL, COMMIT deferred e panic unwind; RED detectou commit indevido. CI entrega `38438041786` passou; gates documentais pendem. [Handoff](../../planning/handoffs/DB-014.md).

## FIN-001 — identidade mínima de conta

RED de ausência do tipo falhou com E0432; fmt/clippy passaram. A suíte Cargo completa passou 55 testes ordinários e 2 doctests, sem falhas/ignorados; doctests confirmaram ausência de `Debug`/`Display` via E0277. Release do módulo de domínio direto passou 7/7. CI de entrega `38444911363`/SHA `31af12adcf8bebe617539d9e5dde286a64674deb` passou 23/23 e release SQLite remoto 41/41. Esses testes provam somente a preservação exata dos 16 bytes do ID e do nome Unicode; não provam validação, tipo, saldo ou comportamento financeiro. CI final documental e liberação pendentes. [Spec](../../planning/specs/FIN-001.md) e [handoff](../../planning/handoffs/FIN-001.md).
## FIN-002 — validação explícita do nome da conta
Três testes de domínio cobrem nome vazio, whitespace Unicode (tab/newline/NBSP/em space) e nome Unicode válido com espaços preservados e ID intacto. E3 local do root: suíte debug 58 testes ordinários + 2 doctests sem falhas/ignorados; o teste ignored de smoke WebView2 foi executado localmente. A etapa Rust do CI usa suíte release padrão, portanto não executa o smoke ignored. CI ainda pendente; FIN-008 deve impor a validação antes da persistência porque os campos são públicos.

CI de entrega FIN-002 confirmado: release 57 testes ordinários (10 lib,6 bin,41 SQLite)+2 doctests, 0 falhas/1 ignored; 3 account_name e junction smoke PASS. SQLite41/41, doctests2, bin6. Teste ignored é o smoke WebView2 local, não executado no CI. CI entrega não executou Rust coverage. A falha anterior por wrapper RTK indisponível permanece histórica; o fix pwsh direto passou na entrega. Executor fechado; fechamento documental/liberação pendentes.

## FIN-003 — tipo de conta
RED E0432/E0560/E0609 confirmou enum/campo ausentes. Quatro literais existentes atualizados e account_preserves_each_account_type prova as quatro variantes sem mapping numérico. Debug suite 58 ordinários+2 doctests, 0 falhas/1 ignored; teste do tipo incluído. Comentário de Account reposicionado após enum; correção só de layout, sem rerun de suíte. CI e scanners staged pendentes.

CI entrega38487235749: release58 testes ordinários+2 doctests, 0 falhas/1 ignored; account_type/name e junction tests passaram; SQLite41/41. Rust coverage não executada na CI. Smoke WebView2 ignored. O teste verifica representação do enum em memória e não persistência nem significado financeiro.

## FIN-004 — saldo inicial
RED de campo ausente, cinco literais atualizados a zero e account_preserves_initial_balance_cents verifica i64::MIN, -123456, 0, 1, 123456 e i64::MAX exatos. Suíte local default59 ordinários+2 doctests, 0 falhas/1 ignored; dois compile_fail mantidos. Sem teste de soma/receita/persistência; classificação pertence a FIN-005.


FIN-004 CORRECT E3: startup test isolado corrigido; fmt/clippy PASS e suíte local padrão 60 ordinários+2 doctests/1 WebView2 ignored. Suíte instrumentada com smoke nativo: 61 Rust+2 doctests, 0 falhas/ignored. CI corretiva do SHA atual ainda pendente.
