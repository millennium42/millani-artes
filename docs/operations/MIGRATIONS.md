# Migrations

Migrations são versionadas, aditivas onde possível e imutáveis depois de aplicadas. Todo conjunto prova banco vazio, upgrade do snapshot anterior, constraints, índices justificados e rollback/estratégia declarada. Valores monetários são inteiros de centavos (ADR-013). Nunca dependa de banco manual para CI.

DB-003 executa o SQL interno, estático e revisado do chamador numa única transação SQLite: só devolve a conexão após commit confirmado; em qualquer erro, tenta rollback quando aplicável e descarta a conexão consumida. Isso não registra versão nem valida SQL externo; seleção/registro pertencem a DB-004. Executor concluído E3; CI de entrega 38197556421 e CI final documental 38198511557 passaram. Veja [contrato e evidência](../../planning/specs/DB-003.md), a API [Transaction do rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html) e [transações SQLite](https://www.sqlite.org/lang_transaction.html).

DB-004 valida `main.schema_migration` via `main.sqlite_schema`/`PRAGMA main.table_xinfo` e aplica só a próxima versão suportada, registrando-a na mesma transação. Executor `done`/E3/`passes=true`; CI de entrega 38201472821 passou 23/23 e CI final 38299901151/job114980000210 passou 16 success/7 skipped/0 artifact no SHA 94e37d38f5df375ffb9492e3f80d272f2f44b331. Nota 65 publicada nas refs de evidência `eafc15283e4eb5e6906002a04ef229cf903474ff`; root confirmou preservação das 64 notas anteriores e refs finais, liberando a reserva de 12 paths. Limites: sem auth do ledger, sandbox, fault injection, C SQLite audit ou coverage Rust global. Veja [spec/handoff](../../planning/specs/DB-004.md).


## DB-005 — migration versionada vazia em arquivo

Dois novos testes reais no harness comprovam arquivo sem objetos/version 0, aplicação de versão 1 com SQL vazio, ledger `[1]`, shape/PK/NOT NULL/hidden/CHECK, autocommit, integridade, reabertura, isolamento entre arquivos e rejeição por PK/CHECK de duplicata, zero e negativo. E3 local: Rust 31/31 (6 lib/7 bin/18 harness), release 18/18, SQLite 3.53.2, fmt/clippy; 16 casos anteriores preservados. RED controlado no caller (0 versus 1), não falha de produção; review E2 sem P0/P1/P2. CI de entrega 38303443795/job114991660147 passou 23/23, 0 skips, release harness 18/18 (SQLite 3.53.2); staged scan e validators da entrega passaram. CI documental final 38304515870/job114995197572 no SHA 1d92fdd185cf24aececd1fa5b5708d9d0efeab03 passou (16 etapas success/7 skipped/0 artifacts); nota 66 publicada nas refs 0c75692168c5c220c364733567b5dc28a01a5739, preservadas as 65 notas anteriores e refs, e reserva de 11 paths liberada. O teste é infraestrutura de harness: não prova migration de produto, startup ou bootstrap financeiro. [Spec](../../planning/specs/DB-005.md) e [handoff](../../planning/handoffs/DB-005.md).


## DB-006 — snapshot lógico de upgrade para teste

DB-006 materializa uma fixture SQL trusted e sintética em arquivos SQLite de teste por `execute_migration`, sem exercer a API versionada nem representar migration de produto. RED controlado com fixture comentada: compilou em 8.05s e falhou no oráculo (0 passados/1 falho/18 filtrados; 0 objetos encontrados versus 2 esperados). GREEN preencheu DDL e sentinelas; um teste novo, sem mudança do oráculo, e formatação/alias local `SnapshotColumn`. E3 root: fmt 2.992s, clippy 4.673s, Rust 32/32 (6 lib/7 bin/19 harness), release 19/19, SQLite 3.53.2; 18 testes anteriores preservados. Review E2 sem P0/P1/P2. Executor done/E3/passes=true/humanGateRequired=false; CI de entrega 38307470576/job115004892529 no SHA 0b8f2a7d0dc8d236d91498cc20e28b358c62dcce passou 23/23, release19/19 SQLite3.53.2. DB-006 fechado: CI final 38308867688/job115009442508 no SHA dd60e198c7ae93bebc41ac873442e5be8193a748, 16 success/7 skipped/0 artifacts; nota67 pública nas refs 69f8ea41a260c442944713b9cff6552f39cbd4a2, 66 notas anteriores preservadas e reserva12 liberada pelo root. Não prova plano/startup de produto, migrations financeiras, backup/restore, setup, auth/concurrency global, fault injection física ou coverage Rust global. [Spec](../../planning/specs/DB-006.md) e [handoff](../../planning/handoffs/DB-006.md).


## DB-007 — upgrade versionado do snapshot

[Spec](../../planning/specs/DB-007.md)/[handoff](../../planning/handoffs/DB-007.md): RED controlado ledger `[1]` vs `[1,2]`; GREEN caller versionado. Root E3 Rust33/33, release20/20 SQLite3.53.2, 19 antigos preservados; review sem P0/P1/P2. CI entrega e documental passaram; executor `done`/E3/`passes=true`. DB-007 fechado: CI final 38320496225/job115047171038 no SHA 323e336b53c74147c4b70bdec44391dfad29fe72, 16 success/7 skipped/0 artifacts; nota68 pública refs f9786502025de68b33275fff766f0428cc862bd1, 67 notas preservadas e reserva11 liberada.


## DB-008 — evidência de rollback

[Spec](../../planning/specs/DB-008.md)/[handoff](../../planning/handoffs/DB-008.md): tarefa documental sem código; cinco fixtures existentes PASS em stdout debug/release. Root fmt/clippy/Rust33/33/release20/20; review E2 sem achados. CI documental entrega 38322109825/job115052349852 no SHA 08a214888bf581d40e74a143bfdfdc48dea0878a passou 16 success/7 skipped/0 artifact (docs apenas). Executor `done`/E3/`passes=true`; DB-008 fechado: CI final38322839469/job115054651101 no SHA dfbe60893150c222b17987518f65a161d3e7ef77, 16/7/0; nota69 refs054bfc0f6bce94a795158a89ded50d20d71bd989, 68 notas preservadas/reserva10 liberada. Sem prova de falha física de rollback.


## DB-009 — ativar foreign keys ao abrir

[Spec](../../planning/specs/DB-009.md)/[handoff](../../planning/handoffs/DB-009.md): baseline ON, controle OFF fail-closed; E3 local 34/34 e release21/21; CI entrega 38324569231/job115060146427 passou 23/23. Executor `done`/E3/`passes=true`. DB-009 fechado: CI final 38325659041/job115063600279 no SHA 17f5397efc996091bfb4271a5169d4eede4249fe (16/7/0); nota70 refs 097a4650bac277404275272876296cbd5e682dbb, 69 notas preservadas e reserva12 liberada pelo root.


## DB-010 — provar enforcement de foreign keys (fechado)

[Spec](../../planning/specs/DB-010.md)/[handoff](../../planning/handoffs/DB-010.md): executor E3 fechado. Baseline SHA `083ac68e073667db3cba474722962b89ca7cdaa5`; CI `38387475775` SUCCESS 16/7/0; nota71 pública `dc2a872282bfaf9fb035cef0a0c336e6bdc8ff19`, 70 notas preservadas e reserva11 liberada. DB-010 limita-se a schema sintético; não cobre schemas/processos de produto nem migrations financeiras.

## DB-011 — transação de escrita

`execute_write_transaction` usa BEGIN IMMEDIATE e compartilha o executor interno com migrations, mantendo migrations DEFERRED. E3 local: fmt/clippy, Rust39/39 e release26/26; quatro testes novos passaram. Review independente E2 sem P0/P1/P2. CI entrega `38394130462`/job `115284940750` passou 23/23; release remoto26/26. Gitleaks staged13 paths/0 findings e quatro validators PASS. Teste de COMMIT com FK deferred verifica falha no commit, sem resultado/conexão; isso não prova rollback de operação composta, DB-012 continua pendente. CI final/nota72 ainda pendentes; ver [spec](../../planning/specs/DB-011.md) e [handoff](../../planning/handoffs/DB-011.md).

## DB-013 — migration do evento técnico (entrega E3)

Migration 002 foi aplicada no RED e é imutável; cinco testes focais, suíte Rust49/49 e release harness36/36 passaram localmente em SQLite3.53.2. Banco vazio e upgrade com probes, constraints e triggers foram exercitados. Cargo/OSV passaram após preservação do advisory RustSec fora do lockfile; review E2 sem P0/P1/P2. CI de entrega 38429912850 no SHA 63ef7f6004f3e697f365f34f681c499726f2f867 passou (harness36/36, cinco db013_* PASS); fechamento documental/nota74 pendentes; ver [spec](../../planning/specs/DB-013.md) e [handoff](../../planning/handoffs/DB-013.md). Não comprova auditoria na mesma operação financeira.


## FIN-006 — migration inicial de contas (E3 local)
A migration imutável `003-accounts.sql` adiciona tabela STRICT com somente `id` BLOB16, `name` TEXT, `account_type` TEXT e `initial_balance_cents` INTEGER; registro e `include_str!` usam o executor versionado existente. Harness SQLite físico executou banco vazio v1→v2→v3/ledger `[1,2,3]` e upgrade v2 com audit preservado, reopen/integrity, sem seed, NUL no nome e limites signed i64; 43 PASS, SQLite 3.53.2. Não prova startup, consumer, UI, regra FIN-005 ou política comercial de sinal. Full gates e CI integrados pendentes; ver [spec](../../planning/specs/FIN-006.md) e [handoff](../../planning/handoffs/FIN-006.md).

FIN-006 focal E3: database.rs 276/280 linhas (98.57142857%) e 25/25 funções (100%); gates críticos ADR-019 passam. Métricas de statements não disponíveis; sem vetor coverage completo/global. Gitleaks staged, validators e CI integrada pendentes.


FIN-006 CI E3: migration 003 e executor passaram 43 testes SQLite dentro da suite release64+4; SQLite3.53.2. Isso prova somente a persistência exercitada pelo harness, não startup ou consumer. Liberação documental ainda pendente.
