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

FIN-004 CI final run38504766831/job115659617683 no SHA9e2068b: 60 testes release ordinários+2 doctests, zero falhas/1 WebView2 ignored; probe startup normal-return, saldo e junction passaram. Coverage Rust global não roda na CI; evidência global é medição local E3.

CAT-001 standalone teve 13/13 testes e coverage focal 8/8 linhas, mas não confirma doctests nem suíte Cargo primária. Root integrou sources; fmt --check passou. Clippy, suite completa, doctests, audit/OSV e CI permanecem pendentes.

CAT-001 primário: fmt/clippy PASS; Rust61 ordinários+4 compile_fail doctests PASS, 0 falhas/1 WebView ignored; guard doctest E0277 real. Audit422/0/2 e OSV542/3/0 PASS. CI ainda pendente.

CAT-001 CI38507827612: release61 ordinários+4 compile_fail doctests PASS, zero falhas/1 WebView ignored; Category named test, quatro guard doctests e junction passaram. SQLite3.53.2. Coverage Rust foi medida localmente, não executada na CI.

FIN-010: suíte standalone focal 13/13 reportada na worktree, mas não constitui execução Cargo primária. Fmt, clippy, suíte completa/doctests e CI primários aguardam integração.

FIN-010 primário: fmt/clippy PASS; Rust62 ordinários+4 compile_fail doctests PASS, 0 falhas/1 WebView ignored; audit422/0/2 e OSV542/3/0 PASS. Gitleaks staged, review final, validators e CI pendentes. Estado histórico antes da entrega CI final.

FIN-010 executor E3 concluído: delivery CI 62 testes ordinários+4 compile-fail doctests PASS, 1 WebView2 ignored. Nenhuma cobertura Rust global alegada; liberação documental pendente.


## FIN-006 — evidência SQLite de migration
O harness real executou upgrade de banco vazio v1→v2→v3, ledger `[1,2,3]`, upgrade de v2 populado preservando audit, fechamento/reabertura, integridade, ausência de seed e round-trip de quatro account_type keys, ID BLOB16, nome Unicode com U+0000 inicial e signed i64 extremos. Root reportou 43 PASS/0 falhas/0 ignorados no SQLite 3.53.2. Execução focal não substitui gates Cargo/Rust completos nem CI no SHA integrado; validação de consumer permanece futura.

FIN-006 coverage focal primária E3: harness compilado em 1.600s; 43 PASS/0 ignorados, 1.68s teste, SQLite 3.53.2. LLVM focado em database.rs passou gates críticos; Gitleaks staged, validators e CI integrada ainda pendentes.


FIN-006 entrega CI E3: 64 testes ordinários+4 compile-fail doctests PASS, 0 falhas/1 WebView ignored; dois casos FIN-006 nomeados passaram, SQLite3.53.2. Coverage Rust não é executada pela CI.

CAT-002 executor done/E3/passes:true após CI exata 38616439285/job116032611143 SUCCESS 23/0 no SHA `3a94855181576ed354a73831dca3e1964641e79d`. Focal fresh formatado: 17 PASS; release 67 testes ordinários +4 compile-fail doctests PASS/1 WebView ignored; junction guard e category tests PASS; fmt/clippy PASS. Frontend checks mediram apenas scaffold (2 arquivos/3 testes/100%). Valida contrato explícito, não consumer, persistência, ativação ou jornadas; publicação final pendente.

## FIN-011 — validação de Context
FIN-011 executor `done`/E3: três testes Context exercitam aceitação exata `home`/`factory` e rejeição de vazio, whitespace, uppercase, chaves espaçadas/desconhecidas e labels Casa/Fábrica; erro estático. Source CI38621842911/SHA `809626e89dabbcce158fb466f63401c0a7b02c99` passou70 testes ordinários+4 compile-fail doctests/1 WebView ignored; junction PASS, SQLite3.53.2. Não adicionar consumidor nem inferir validação fora do domínio. Publicação documental ainda pendente.

FIN-007 primary E3: 2 testes novos do harness (16 invalid+1 duplicate oracle) passaram dentro de 45 SQL PASS; suite Cargo72 ordinary+4 compile-fail doctests PASS/1 WebView ignored. Staged Gitleaks, validators e source CI ainda pendentes.

Fechamento FIN-007: source CI `38628675813`/job `116074330758` SUCCESS23/0 no SHA `a58ad20b7664db5f7b5fb85a3cb7b0b341a46978`; 72 testes ordinários+4 compile-fail doctests PASS/1 WebView ignored, Gitleaks187/0, quatro validators PASS e junction smoke PASS. A versão SQLite não consta no log.

FIN-008 primary E3: Cargo82 testes ordinários (27 library,7 binário,48 SQLite)+4 compile-fail doctests PASS/1 WebView ignored; fmt e clippy all-targets/locked/keep-going/`-D warnings` PASS. Casos físicos verificam criação/reopen dos quatro campos e audit_event e rollback diante de falha nos inserts; unit tests cobrem validação/portas e erros sanitizados. Harness usa migrations reais v1/v2/v3. Não demonstra startup/UI/IPC, resultado financeiro nem INV-FIN-001. Gitleaks staged, quatro validators e CI primary seguem pendentes.
## FIN-008 — evidência do executor (E3)
FIN-008 source CI executou 82 testes ordinários+4 compile-fail PASS/1 WebView ignored; inclui sucesso/reopen/rollback SQL e junction smoke. Source SHA eea53ff05f2df34c5ff8e9aa759b70dc0a25ab57; sem UI/IPC.

## CAT-003 — integracao primaria 16/10/2026

Primario: debug/release86 ordinarios+8doctests PASS/1WebView ignored;50SQLite incluem2CAT003 com banco vazio/upgradev3 populado/constraints/NUL/ledger/integridade/reopen. Fonte: planning/handoffs/CAT-003.md e planning/specs/CAT-003.md, SHA7573c65899416cf180bb25545f6cfae6335bb00e; E3 local, CI de entrega em execucao, item ainda aberto.


CAT-003 executor done/E3/passestrue apos CI38907719490/job116995418974/attempt1/SHA7573c65899416cf180bb25545f6cfae6335bb00e SUCCESS23/0. Evidencia e limites em planning/handoffs/CAT-003.md;CI documental final/nota87/liberacao pendentes. FIN-012/UI-001 continuam abertos.


## FIN-012 — integração primária em 16/10/2026

Os dois testes de metadados e quatro guards compile-fail passaram na suíte integrada e no CI source 38907719490. Cobrem extremos i64, zero/negativo, contextos, espécies, IDs e associações; não comprovam execução financeira. Fontes e evidência: planning/specs/FIN-012.md e planning/handoffs/FIN-012.md; source 7573c65, main 7cad038, produção idêntica. Item ainda aberto até fechamento documental.

FIN-012 executor done/E3/passes true após os checks e CI do código integrado, com fonte em planning/handoffs/FIN-012.md. CI documental final, nota88 e liberação pendentes. UI-001 aguarda liberação. Sem comprovação de consumidor, regras financeiras, coverage global ou instalador.


## UI-001 — verificação primária em 16/10/2026

Fonte: planning/handoffs/UI-001.md, source7573c65/main59a3789, E3 local.

```json
{
  "SSR": "3 testes de estrutura/bootstrap, sem tratar como prova visual",
  "nativo": "execução real, capturas externas1026x752/313x752, teclado Tab/ShiftTab observado",
  "saída": "código não registrado(null);janela e processos próprios encerrados"
}
```


UI-001 executor concluído: E3 de execução e inspeção nativa, checks do código idêntico e CI source38907719490 SUCCESS23/0; review integrado/visual E2 sem P0/P1/P2. Documentos e matrizes atualizados antes de passes true. Reserva retida até CI documental final no novo SHA, nota89/preservação88notas/Gitlimpo/WTarquivada e liberação do root. Código de saída nativo não registrado(null); sem claim exit0, WCAG numérico, mínimo de produto, jornada financeira, Rustglobal ou setup. Próximo após liberação: reconciliar elegibilidade FIN-005 no roadmap.


## FIN-009 — leitura de contas, E3 local em 17/10/2026

Testes cobrem 24 contas, 4 tipos enum, 6 valores i64. Unicode, nomes repetidos, falhas. Unitários: 4. SQLite: 7. Cobertura app: 9/9 linhas. Adapter: 88/88 linhas, 7/7 funções. Controle: 0/9 linhas, 0/3 funções. Debug e release/custom-protocol passaram97 ordinários+8doctests, com1native ignored; medição instrumentada include-ignored passou98+8, nenhum ignored.


FIN-009 LLVM23.1.1 global local E3:459/470linhas=97,659574%,46/49funções=93,877551%; global90PASS. Críticos conservadores create_account50/50linhas4/4funções, list_accounts9/9e3/3, accounts88/88e7/7, database276/280(98,5714%)e25/25:95PASS. Produção somente src-tauri/src;8arquivos reportados,lib.rs/ports.rs semlinhasexecutáveis N/A; testes/deps/gerados/tooling excluídos. Main normal incluído0/3linhas0/1função(não executado); infraestrutura startup6/10linhas1/3funções. Instrumentado98ordinários+8doctests include-ignored,0ignored. Statements não medidos,branches stableN/A;regiões não statements,sem vetorcompleto/finançasUI. Relatório e logs preservados externamente, FIN-009-global-rust-20261017.json e FIN-009-global-rust-closeout-20261017.json; perfilWebView2 próprio removido sem processo ativo após196entradas semreparse. Runner originalexit1 porcleanup pendente, métricas/testesexit0; closeout separado confirmoucleanup,sem reclassificar exit. Cache instrumentado ignorado preservado. Isto não é coverage executada na CI nem instalador.


FIN-009 executor done/E3/passestrue após CI39009285553/job117328105513/attempt1/SHA83e682303354c000d2319f512f8321882901aa35 SUCCESS23/0,97ordinários+8doctestsPASS/1ignored. Evidência local global459/470linhas46/49funções ecríticos95PASS não é coverage CI; statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4 atualizados; CI documental final/nota90/preservação89/liberação ainda pendentes. Sem UI/IPC/receita/setup; fonte planning/handoffs/FIN-009.md.


## FIN-013 — validação pura de entrada, E3 local em 17/10/2026

Cinco testes focais passam: 1/123456/MAX e 0/-1/MIN nos dois contextos, quatro campos preservados, Exit/Transfer positivos rejeitados e erro real com Display/Debug/source estáticos. Debug e release/custom-protocol: 102 ordinários (38lib+7bin+57SQLite)+8doctests PASS, 1WebViewignored. RED original antes BUILD; controle final remove somente o novo método na WT própria, obtém E0599 e restaura bytes exatos.


FIN-013 executor done/E3/passestrue após CI39021912847/job117365252558/attempt1/SHAa3dee6febce169d4dbac3779ce6a67c998e506ab SUCCESS23/0,102ordinários+8doctestsPASS/1ignored. Crítico domainmod24/24linhas5/5funções/controle rejeita95 é E3 local,não coverageCI/globalatual;statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4/validators atualizados;CIfinaldocumental/nota91/preservação90/liberação pendentes. SemUIIPC/persistência/receita/setup. Fonte planning/handoffs/FIN-013.md.


## FIN-014 — validação pura de saída, E3 local em 17/10/2026

Cinco testes transpostos do padrão de entrada passaram sem correção GPT: 1/123456/MAX e0/-1/MIN em Home/Factory, quatro campos preservados, Entry/Transfer positivos rejeitados e erro real estático/sourceNone. RED root antes BUILD confirmou só E0599 validate_exit ausente. Debug e release/custom-protocol:107ordinários(43lib+7bin+57SQLite)+8doctests PASS/1WebViewignored.


FIN-014 | CI source aprovado / fechamento funcional E3 | SHA c27b4eaf6c7c8462b0301b3489218ee927a83a89; run39026610317/job117378726603/attempt1 success,23 passos aprovados/0 ignorados,7 marcadores sourceSha exatos. Rust remoto107 ordinários(43lib+7bin+57SQLite)+8doctests PASS/1WebViewignored; artifact frontend único scaffold. PE AMD64/custom-protocol8.536.064 bytes SHA256 5c46f6e9944536be390c2937aebc1d4bb80aa1ece7b215124573fbef227eae51; não é instalador. Local fmt/clippy/debug/release/quality/build/scans/E4 aprovados; crítico domainmod29/29linhas6/6funções e controle24/29linhas5/6funções rejeita95. Revisão independente Sol6medium semP0/P1/P2. Quatro validadores415arquivos/28fontes/92itens PASS; stagedGitleaks14arquivos/703969bytes/0achados/framedSHA2562a94609b975ca6224b734c68833cd0a216a0ab410796d9f1ce31c6e6b2933602. E4 exata até23/10/2026, sem exceção nova. Evidência focal preservada em operacao/FIN-014-focal-evidence-20261017,9arquivos com hashes; WT arquivada recuperavelmente, scratch próprio removido. Sem código ou teste corrigido por GPT; Laguna local P-QA/P-DOM, integrador reproduziu gates. Custo não registrado. Esta entrega é validação pura de saída; não comprova jornada UI/IPC/persistência/receita, global Rust atual ou setup.exe. CI final/nota92 preservando91/97branches/checkpoint/Gitlimpo/liberação ainda pendentes; reserva ATIVA e próximo ID bloqueado até esse fechamento.


FIN-015 | VERIFY local | E3 | Migration005 main.movements STRICT e registro v5 adicionam sete campos que preservam Movement/MovementKind raw i64, context separado da conta, shape por kind e três FKs NO ACTION. Executor e migrations001–004 inalterados. Três testes físicos PASS: cadeia vazia1..5 e30roundtrips(3kinds×2contextos×5i64), schema/FKs/índicePK, reopen/integridade; upgradev4 com contas/categoria/audit e snapshots exatos; colisão com sentinel, rollback/ledger4 preservados, remoção apenas da fixture e retryv5. Probes shape/FK/delete preservam os dados. Nenhum banco da usuária utilizado.
Debug e release/custom-protocol:110 testes ordinários(43lib+7bin+60SQLite)+8doctests PASS;1WebViewignored. Fmt/clippy-Dwarnings, TSquality(format/lint/typecheck/coverage)/build PASS;frontend3testes2arquivos scaffold100%,sem finanças. npm120/0,Cargo422/0vuln/2E4,OSV542/3E4/0fora,plugin/SAST45PASS. E4 exatas nos hashes aprovados,expiram23/10/2026;nenhuma exceção nova ou threshold reduzido.
LLVM23.1.1 focal database.rs276/280linhas(98,5714%)/25/25funções(100%) passa95. Controle omite TODO harness SQL e mede0/280linhas0/25funções,rejeitando95; calibra o gate,não atribui cobertura aos três testes novos. Statements não medidos;branches stableN/A;regiões não são statements. Não comprova Rustglobal atual.
Review independente P-REV Sol6medium read-only4paths/spec semP0/P1/P2,não executou checks. LocaisQwen/Laguna/Muse propuseram/testaram;root rejeitou drafts e corrigiu resíduos focais após falhas. RodadaLaguna longa foi interrompida com evidência preservada: processo computava/contador avançava,mas contexto31,5K/32,8K em autotest-retry;erroi32/ToSql. Sem mudança no executor;servidor próprio45340 encerrado pelo controlador/porta11434fechada. Receiptsoriginais e cinco arquivos do job interrompido preservados em operacao/FIN-015-interrupted-correction-evidence-20261017.
Fontes/recibos FIN-015-main-integration/rust-gates/other-gates/verification-facts/GREEN-root/root-residue-20261017.json e artifacts/fin015-coverage/receipt-coverage.json. Custo não registrado. Validators/stagedscan/CI source/final/nota93preservando92/liberação pendentes,passesfalse. Não comprova adapters/consumidores/UIIPC/receita/transferência válida/setup.exe/E2E.


FIN-015 | CI source aprovado / executor concluído | E3 | SHA23459892a5682aef1d8c512f53badf5a3466f5b2,run39061456176/job117480113685/attempt1 success23/0skip;7marcadores sourceSha exatos. Rust remoto110ordinários(43lib+7bin+60SQLite)+8doctestsPASS/1WebViewignored, incluindo3testesfísicosFIN015. Únicoartifactfrontend scaffold. PE AMD64/customprotocol8.536.064B SHA256743847089a72cb894cd7042a262c2f69148ccbc49e65be457695c464874e371a,não é instalador. Quatrovalidadores420arquivos/28fontes/93itens e stagedGitleaks15arquivos/763917B/0achados/framedSHA256abb4bac21f06430e4cc4a54dddeaa45bb75f083c44e0502e2b6dd47af9287472PASS. Gateslocais/coveragecrítica/review descritos no VERIFY e recibosoperacionais;E4exatasaté23out. Evidênciafocal10arquivos preservados comhashes;originaisdojobinterrompido preservadosseparadamente;WTarquivada/scratchpróprioremovido. Executor done/E3/passestrue apósdocs/matrizes/log/handoff eCI source;CI documental final/nota93preservando92/97branches/checkpoint/mainlimpa/liberação ainda pendentes,reservaATIVA. SemjornadaUIIPC/receita/transferênciavalidada/Rustglobalatual/setup.exe/E2E. Custo não registrado.


FIN-016 | VERIFY local | E3 | Dois testes físicos novos da migration005 imutável:39linhas inválidas(38matrix+id15) e7probesPK/FK/NOACTION,com errosConstraintViolation eextendedPRIMARYKEY/FOREIGNKEY nas probes pertinentes. Cadeia real1..5 por open_database/executor; snapshots7camposmovements/4accounts,ledger/autocommit/integridade/FKcheck e reopen preservados. Seeds0/-5 continuam aceitos conforme rawschema. Nenhuma produção/migration/manifest/lock alterada;identidade byteexata do primary registrada.
RootRED compila e executa2testes;controleCHECK temporário só no testenovo daWT fazid15seraceito e primeiroteste falha/segundopassa. Runner restaura todosbytes econfirmahashesprodução,sem alteração SQLaplicada. GREENnormal2PASS/0warnings. Debug e releasecustomprotocol112ordinários(43lib+7bin+62SQLite)+8doctestsPASS/1WebViewignored. Fmt/clippy-Dwarnings/TSquality(format/lint/typecheck/coverage)/buildPASS;frontend3testes2arquivos scaffold100%,semfinanças.
npm120/0,Cargo422/0vuln/2E4,OSV542/3E4/0fora,plugin/SAST45PASS. E4 nos inputs aprovados até23out,semnovaexceção/thresholdreduzido. Cobertura críticaE3FIN015database276/280linhas25/25funções econtrole0/280/0/25 reusados porhashprodução exato;controle omiteTODOharnessSQL,não atribui métrica aos testesnovos. Statementsnão medidos/branchesstableN/A;semglobalatual/CIcoverage.
P-REVSol6medium read-only2paths/spec semP0/P1/P2,nãoexecutouchecks. LocaisLaguna/Muse produziram candidatos;integrador rejeitou bypass,corrigiu contexto faltante003 e resíduos de API/oráculos apósfalhas. Último jobboundedwrite semautotestretry declarouchecksnot_run;rootexecutouRED/GREENseparadamente. Recibosoriginais/candidatosrejeitados/jobserver interrompido preservados;executorinstalado intacto. Servidorpróprio58064 encerradopelocontrolador,11434fechada.
Fontes/recibos FIN-016-RED-root/GREEN-root/root-residue/main-integration/rust-gates/other-gates/verification-facts-20261017.json emoperacao. Custo não registrado. Validators/stagedscan/CI source/final/nota94preservando93/liberação pendentes;in_progress/passesfalse. SemjornadaUIIPC/receita/setup.exe/E2E.


FIN-016 | CI source aprovado / executor concluído | E3 | SHAf31c8286091af4285bd189a5b5348661fe4181a1,run39079462454/job117533270499/attempt1success23/0skip,7sourceShamarcadores exatos. Rust remoto112ordinários(43lib+7bin+62SQLite)+8doctestsPASS/1WebViewignored,incluindo2testesFIN016. Únicoartifactfrontend scaffold. PE AMD64/customprotocol8.536.064B SHA2567147d0a64af88ec112a8292a6f3d3dd44589678da4f779939a7c10eafe191f4b,nãoinstalador. Quatrovalidadores424arquivos/28fontes/94itens e stagedGitleaks13arquivos/762500B/0achados/framedSHA2560b1846d7fbc92b5d1f6c1a192ba1b4464fcea4891043d6aee9abf81bb1627086PASS. Local112+8/gates/securityE4/review aprovados;produçãoexataFIN015permaneceinalterada,coveragecrítica reutilizada porhash semclaimglobalatual. Evidênciafocal4arquivoshashverified e originaisrejeitados/interrompidospreservados;WTarquivada. Docs/matrizes/log/handoff atualizados antesdone/E3/passestrue. CIfinal/nota94preservando93/97branches/checkpoint/mainlimpa/liberaçãopendentes,reservaATIVA. SemjornadaUIIPC/receita/setup.exe/E2E. Custonão registrado.
## FIN-017 — persistência de Entry (planejado, E2)
Aceite futuro requer unitários de sucesso/limites/ausência de efeitos em inválidos/falhas de ID-relógio-store e conversão de duração; SQLite físico deve usar cadeia real de migrations 001–005, seed por `create_account`, reopen, auditoria exata e preservação de saldo inicial. Cobrir rejeição anterior ao consumo da conexão e rollback atômico para conta inexistente, ID duplicado, abort de audit e inserção ignorada/zero linhas. Nenhum resultado GREEN está registrado.


FIN-017 | E3 local:6unit+3SQLite/4falhas,criaçãoEntry+audit atômicas/reopen/rollback/poison/inputinválido sem efeitos;Rust121+8docs/debug+release/fmt/clippy/TS/securityE4 PASS;coveragecrítica app43/43linhas3/3fn eadapter51/51linhas5/5fn,controle0,rejeita95. Review independente concluído;CI source/final/liberação pendentes,in_progress/passesfalse. Sem jornadaUIIPC/globalRust/setup/E2E;apósFIN017METApausará,nenhum próximoID. Ver spec/handoffFIN017;custonão registrado.


FIN-017 | done/E3: source5a09ca2ec7117be7e39605bfe25848e7fc39acaf CI39090756071/job117566803340/attempt1success23/0skip/7marcadores exatos;121testesRust+8doctests. Entry+audit atômicos/6unit+3SQLite4falhas,critical app43/43linhas3/3fn+adapter51/51linhas5/5fn100%,controle0. Gateslocais/review/validators/stagedscan aprovados;docs antespassestrue. CI documental/nota95/liberação pendentes;WTarquivada,evidênciapreservada,locaisencerrados. DepoispausarMETA por decisão humana,nenhumpróximoID. ProdutoUIIPC/receitaFIN005/globalRust/setup.exe/E2E pendentes;custonão registrado.


CAT-004 E3 local:6unit e3SQLite físicos com helpers existentes cobrem nomes exatos/repetidos, validação antes de portas, IDs/Clockchecked/erros opacos, criação/auditoria/reopen/reuso; três faults reais duplicata/auditABORT/INSERTIGNORE e snapshots completos/poison. RED APIsausentes e guardinativa comprovados;130ordinários+8doctests debug/releasePASS/1WebViewignored. Nenhuma jornadaUIIPC/E2E provada; CI pendente; planning/handoffs/CAT-004.md.


CAT-004 executor done/E3/passestrue após CI39094721622 no SHAcbf40d8392e66a38146ebfd6abb61cd42766d47e success23/0 e130Rust+8docs; detalhes e limites no handoff CAT-004. CI documental final/nota96/liberação pendentes. MVP02 criação backend parcial; semUIIPC/E2E/setup. MetaE2Eativa.


CAT-005 E3 local: 8 unitários + 3 SQLite reais cobrem validação antes de efeitos, um ID, Clock checked, nome literal, ativa/inativa, NotFound/reuso/reopen, snapshots completos e rollback/poison na mesma instância por audit duplicado/ABORT/UPDATE IGNORE. Rust 141+8 doctests em debug/release PASS, um WebView ignored; quality/build/security PASS. Revisão independente sem P0/P1/P2. CI e E2E pendentes; handoff CAT-005.


CAT-005 executor done/E3/passestrue após CI39098498169 no SHAaef71f2308dbfa1d11f4960568fc0b041d1e7221 success23/0 e141Rust+8doctests, incluindo8unit+3SQLite. Gates locais, cobertura crítica >=95 e controle, revisão independente e validators/stagedscan aprovados; detalhes no handoff CAT-005. CI documental final/nota97/liberação pendentes. MVP02 renomeação backend parcial; UIIPC/desativação/E2E/setup pendentes. Meta E2E ativa.


CAT-006 E3:7unit+3SQLite reais cobrem umID/Clockchecked/erros opacos, ativa/inativa com nome literal/reopen/snapshots completos, ausência/reuso e3faults com rollback/poison mesma instância. Rust151+8docs debug/release,quality/build/securityE4/review PASS. CI/E2E pendentes; handoffCAT006.


CAT-006 done/E3/passestrue após CI39128570976 no SHA25311c0fb01d066e39d557e3779287d680553c7f,23success/0skip e151Rust+8docs;7unit+3SQLite e gateslocais/review/validators/stagedscan PASS. Fonte: handoffCAT006. CI documental final/nota98/liberação pendentes. MetaE2Eativa; UIIPC/histórico consumidor/setup/E2E pendentes.


CAT-007 E3:2SQLite físicos provam migration6vazia/upgrade,nullableBLOB16/FKNOACTION,probes inválidos/snapshots/reopen e vínculo de movimento real após rename/deactivate com audit exato. Atribuição feita sópela fixture SQL no campo real. Rust153+8docs/debug/release/quality/security/review PASS;CI/E2E pendentes; handoffCAT007.


CAT-007 done/E3/passestrue após CI39131643684/SHA143190b374d186da75f2a44bdc85cb0bfad58219success23/0skip e153Rust+8docs,2SQLite novos. Migration6/FKhistórica física/reopen/rename/deactivate e gates/review/validators/stagedscan PASS. CI documental final/nota99/liberação pendentes;MetaE2Eativa. FixtureSQL não comprova atribuiçãoUIIPC/globalRust/setup/E2E. Fonte:handoffCAT007.


FIN-018 E3:6unit reutilizados e3SQLite1..6/contextos/i64MAX/1/reopen/categoriaNULL/audit/snapshots;invalidkindEntry/Transfer/amount e4faults contaFK/IDduplicado/auditABORT/INSERTIGNORE rollback/poison mesma instância. Rust162+8docs/debug/release,quality/security/review PASS;CI/E2E pendentes;handoffFIN018.


FIN-018 done/E3/passestrue após CI39134383058/SHA4f7e7703793ac8ef5632935ade79badd3f9d44fasuccess23/0skip e162Rust+8docs,6unit+3SQLite. Saída+audit atômicos/rollback4faults/poison e gates/review/validators/stagedscan PASS. CI documental final/nota100/liberação pendentes;MetaE2Eativa. Sem saldo/receitaFIN005/UIIPC/globalRust/setup/E2E. Fonte:handoffFIN018.


FIN-019 E3:5unit reais amboscontextos/positivos1/123456/MAX/zero/-1/MIN,Entry/Exit rejeitados,metadados/erro opaco;RED5E0599/GREEN5. Rust167+8docs/debug/release,quality/security/reviewPASS. Não prova transferência financeira;CI pendente;handoffFIN019.


FIN-019 done/E3/passestrue após CI39136146173/SHA7ac497730b507c5576b05bff8f285169215b21cbsuccess23/0skip e167Rust+8docs,5unit. Validação pura Transferpositivo e gates/review/validators/stagedscan PASS. CI documental final/nota101/liberação pendentes;MetaE2Eativa. FIN020/021/022/UIIPC/globalRust/setup/E2E pendentes. Fonte:handoffFIN019.


FIN-020 E3:RED6pass/1fail porcontasiguais antesguard;GREEN7. Doisnovostestes:iguais0/FF econtasdistintas somenteúltimobyte ambasdireções,1/MAX,amboscontextos/metadados;cincoanteriores preservados. Rust169+8docs/debug+release,quality/security/reviewPASS;CIpendente;handoffFIN020.


FIN-020 done/E3/passestrue após CI39137975946/SHA78beb2f62b024899d923b136251d3907b937b82asuccess23/0skip/7marcadores,Rust169+8docs/7transferunit(2novos). Contasiguaisrejeitadas/contasdistintas/metadados preservados,gates/review/validators/stagedscanPASS. CI final/nota102/liberaçãopendentes;MetaE2Eativa;FIN021/022/023/UIIPC/globalRust/setup/E2Ependentes. Fonte:handoffFIN020.


FIN-021 E3 REDAPIausente/GREEN7unit+3SQLite/5falhas(sourceFK,destFK,duplicate,auditABORT,IGNORE),snapshot/reopen/poison/reuso/errosopacos;Rust179+8docs/debug+release/quality/security/reviewPASS. NãofechaFIN022/023/UIIPC;CIpendente;handoffFIN021.


FIN-021 done/E3/passestrue após CI39142344305/SHA1ba38ca0cca75a313431cc6ab5ae89ef50db88f3success23/0skip/7marcadores,Rust179+8docs/7unit+3SQLite. Gates/review/validators/stagedscanPASS;CI final/nota103/liberaçãopendentes;MetaE2Eativa;FIN022neutralidade/FIN023matrizrollback/FIN024saldos/receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;fonte:handoffFIN021.


FIN-022 VERIFY E3/INV-FIN003:contribuiçãorealporMovement Entry+/Exit-/Transfer0validada,5unit/RED-GREEN/metadados/limites/contextos/neutralidadecomposição;184Rust+8docs/debug+release/quality/securityE4/LLVMdomaincriticalcontrole/P-REV PASS. In_progress/passesfalse atéCI;relatórioagregado/filtrocontextoFIN027,saldoFIN024,matrizrollbackFIN023,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN022.


FIN-022 done/E3/passestrue após CI39144045457/SHAe42ec5ca331be38c7e0c5d9dfb6cca4b3bd12b3bsuccess23/0skip/7marcadores,Rust184+8docs/5unit. ContribuiçãoEntry+/Exit-/Transfer0validada/gates/review/validators/stagedscanPASS;CI final/nota104/liberaçãopendentes;relatórioagregado/filtrocontextoFIN027,saldoFIN024,matrizrollbackFIN023,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN022.


FIN-023 VERIFY E3:3falhas adicionais físicas auditPK/auditIGNORE/COMMITFK, snapshots/reopen/poison/novo adapter;RED2mutantes/GREEN1,produção inalterada. Rust185+8docs/debug+release/quality/securityE4/coveragefamília/P-REV PASS;in_progress/passesfalseatéCI;saldoFIN024/relatórioFIN027/receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN023.


FIN-023 done/E3/passestrue após CI39147317365/SHAac7659754b5dc5733c82ce369c5b6c4053610d3bsuccess23/0skip/7marcadores,Rust185+8docs/novoSQLite3casos. Produçãoinalterada/RED2mutantes/gates/review/validators/stagedscanPASS;CI final/nota105/liberaçãopendentes;saldoFIN024/relatórioFIN027/receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN023.


FIN-024 VERIFY E3:saldo real porAccount comi128checked/erroopaco/5unit+1SQLite persistência/list_accounts/saldos-1179/7669. Rust191+8docs/debug+release/quality/securityE4/LLVMdomaincriticalcontrole/P-REV PASS. In_progress/passesfalseatéCI;prova de edição/derivaçãoFIN025,leitor público/extratoFIN026,resultadoFIN027,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN024.


FIN-024 done/E3/passestrue após CI39175166911/SHAd6c3ba969dd9daeb9f3b02f881518a8d3dfabadfsuccess23/0skip/7marcadores,Rust191+8docs/5unit+1SQLite. SaldoAccount real i128checked/gates/review/validators/stagedscanPASS;CI final/nota106/liberaçãopendentes;prova de edição/derivaçãoFIN025,leitor público/extratoFIN026,resultadoFIN027,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN024.


FIN-025 VERIFY E3:1unit replayreal2contas/prefixos/reverse/vazio/repetição;2mutantesRED/GREENreal/produçãozero. Rust192+8docs/debug+release/quality/securityE4/LLVMdomaincriticalcontrole/P-REV PASS. In_progress/passesfalseatéCI;edição canônicaFIN028/prova semsaldo diretoFIN030,leitor público/extratoFIN026,resultadoFIN027,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN025.


FIN-025 done/E3/passestrue após CI39176957384/SHAc408c42a3a06135b6792262d7d330479ac9cf1f1success23/0skip/7marcadores,Rust192+8docs/1unitnovo replay/2mutantesRED eproduçãozero. Gates/review/validators/stagedscanPASS;CI final/nota107/liberaçãopendentes;edição canônicaFIN028/prova semsaldo diretoFIN030,leitor público/extratoFIN026,resultadoFIN027,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN025.


FIN-026 VERIFY E3:leitorcanônico real port+adapterstrict/erroopaco/1call/read-only e saldo pelo leitorpúblico. 3unit+3SQLnovos+1SQLsaldoPASS/Rust198+8docs/debug+release/quality/securityE4/LLVMapp+adaptercriticalcontrole/P-REV PASS. Writers/domínio/migrations/deps preservados. In_progress/passesfalseatéCI;UIIPC/cronologia/resultadoFIN027/receitaFIN005/ediçãoFIN028/030/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN026.


FIN-026 done/E3/passestrue após CI39180075013/SHA6b8b77b7840579b8a4d6c50a1c4c8de0d15f71f1success23/0skip/7marcadores,Rust198+8docs/3unit+3SQLnovos+SQLsaldo viareaderreal. Gates/review/validators/stagedscanPASS,writerspreservados;CI final/nota108/liberaçãopendentes;UIIPC/cronologia/resultadoFIN027/receitaFIN005/ediçãoFIN028/030/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN026.


FIN-027 VERIFY E3:resultado líquido realporcontexto viaport/helperdomínio,Transfer0/validaantesfiltro/i128checked/erroopaco. 5unit+2SQLnovos+1SQLsaldoPASS/Rust205+8docs/debug+release/quality/securityE4/LLVMnewappcriticalcontrole/P-REV PASS. In_progress/passesfalseatéCI;provaFIN005/ediçãoFIN028/030/UIIPC/cronologia/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN027.


FIN-027 done/E3/passestrue após CI39181782882/SHAd438ab051db80153f5f2c505a56f710f1f0d0d41success23/0skip/7marcadores,Rust205+8docs/5unit+2SQLnovos+SQLsaldo chamandoresultadoreal. Gates/review/validators/stagedscanPASS;CI final/nota109/liberaçãopendentes;provaFIN005/ediçãoFIN028/030/UIIPC/cronologia/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN027.


FIN-005 VERIFY E3:1SQLreal/openingsMIN/-1/0/1/MAX semEntry/report0,2entradasreais/report1e2;REDmutanteAccountStore/GREENreal. Rust206+8docs/debug+release/quality/securityE4/LLVMcalcsourceidêntica-controle/P-REV PASS. Produçãointacta/depformalFIN004preservada/consumerFIN026/027real. In_progress/passesfalseatéCI;ediçãoFIN028/030/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN005.


FIN-005 done/E3/passestrue após CI39184150887/SHAfd0f63a5decdefe4ea1715aee67e3a975cd4a192success23/0skip/7marcadores,Rust206+8docs/1SQLinitial+mutanteRED/GREENreal. OpeningsnãoEntry/report0,2entradasreais/report1e2,consumidoresFIN026/027reais;gates/review/validators/stagedscanPASS/produçãointacta. CI final/nota110/liberaçãopendentes;ediçãoFIN028/030/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN005.


FIN-028 VERIFY E3: edição canônica real/identidade estável/UPDATE+audit atômicos/rollback e preservação de dados. 6unit+4SQL/Rust216+8docs/quality/security/LLVM app+adapter>=95 com controle/review PASS. In_progress/passesfalse até CI. FIN-029/030, UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte: handoff FIN-028.


FIN-028 done/E3/passestrue após CI39187261583/SHA15020e5ea781b2ac58edb501c1abebb459a4493e success23/0skip/7marcadores;Rust216+8docs/6unit+4SQL. Edição canônica real/ID estável/UPDATE+audit atômicos/rollback/dados preservados;gates/review/validators/stagedscan PASS. CI final/nota111/liberação pendentes;FIN-029/030, UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;Meta E2E ativa;fonte:handoff FIN-028.


FIN-029 VERIFY E3:1SQL real/sete edições/audits exatos e imutáveis/rollback físico,2mutantes RED/GREEN;produção intacta. Rust217+8docs/quality/security/critical existente com controle/review PASS. In_progress/passesfalse até CI;FIN-030, UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte: handoff FIN-029.


FIN-029 done/E3/passestrue após CI39189813304/SHA7ba07364c37a1624fa349f04897e59c981ae6abb success23/0skip/7marcadores,Rust217+8docs/1SQLauditnovo/2mutantesRED/GREENreal/produçãointacta. Gates/review/validators/stagedscan PASS;CI final/nota112/liberação pendentes;FIN-030, UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff FIN-029.


FIN-030 VERIFY E3:reuso de testes reais puro/SQLite confirma saldo derivado após edição,aberturas/contas/ledger intactos;2mutantesRED/GREEN. Rust217+8docs/quality/security/critical existente com controle/review PASS;produção/testes intactos. CI documental pendente,sem Rust remoto atual;UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff FIN-030.


FIN-030 done/E3/passestrue após CI documental39191779815/SHAb05b88280ef9b39b23ed4cc61df5eb210b4a9764success16/7skip/5marcadores. E3 localRust217+8docs/reuso2testes/2mutantesRED/GREEN/gates/review;produção/testes intactos;sem Rust remoto no novoSHA. CI final/nota113/liberação pendentes;UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff FIN-030.


REC-001 VERIFY E3:FixedRecurrence pura/nome obrigatório/valorpositivo/dia1..31/erroexistente/semDebugDisplay;RED/GREEN4unit+2doctestsprivacidade. Rust221+10docs/quality/security/LLVMdomain79/79 e10/10 com controle/review PASS;domínioanteriorpreservado. CIs pendentes;REC-002..015 (variável,estados,calendário de meses curtos,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-001.


REC-001 done/E3/passestrue após CI39193231687/SHAd52b3d3a6a8342867cc85dba78221d4becdea51asuccess23/0skip/7marcadores,Rust221+10docs/4unit+2doctestsnovos. FixedRecurrencepura/gates/review/validators/stagedscanPASS;CI final/nota114/liberação pendentes;REC-002..015 (variável,estados,calendário de meses curtos,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-001.


REC-002 VERIFY E3:VariableRecurrence pura/semvalor/nomeobrigatório/dia1..31/erroexistente/semDebugDisplay;RED/GREEN3unit+3doctests. Rust224+13docs/quality/security/LLVMdomain84/84 e11/11 com controle/review PASS;FixedRecurrence e domínio anterior preservados. CIs pendentes;REC-003..015 (estados,valorporocorrência,calendário de meses curtos,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-002.


REC-002 done/E3/passestrue após CI39195265289/SHAbb69d0f8c584a5e766b07ffa5faefe1ccf5916aasuccess23/0skip/7marcadores,Rust224+13docs/3unit+3doctestsnovos. VariableRecurrencepura/gates/review/validators/stagedscanPASS;CI final/nota115/liberação pendentes;REC-003..015 (estados,valorporocorrência,calendário de meses curtos,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-002.


REC-003 VERIFY E3:AwaitingValue inicial de VariableRecurrence validada por método real,semvalor/efeitos;RED/GREEN3unit+1doc. Rust227+14docs/quality/security/LLVMdomain88/88 e12/12 com controle porfunções/review PASS;domínioanteriorpreservado. CIs pendentes;REC-004..015 (demais estados,valorporocorrência,calendário,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-003.


REC-003 done/E3/passestrue após CI39197268222/SHA3aa07d06f19515688d5a09c1f21de12b1d5cac93success23/0skip/7marcadores,Rust227+14docs/3unit+1doctestnovo. AwaitingValuevalidado/gates/review/validators/stagedscanPASS;CI final/nota116/liberação pendentes;REC-004..015 (demais estados,valorporocorrência,calendário,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-003.


REC-004 VERIFY E3:Payablepositivo/Statevalidate/Fixedinitialreal/Awaitingpreservado/semefeitos;RED/GREEN4unit+2docs. P2doctestomitidorestaurado/verificado;Rust231+16docs/quality/security/LLVMdomain102/102 e14/14comcontrole/review PASS. CIs pendentes;REC-005..015 (demais estados,valorporocorrência,calendário,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-004.


REC-004 done/E3/passestrue após CI39199353402/SHAfd2787d91af0082b7fcb0628ecabb00cdece0ac9success23/0skip/7marcadores,Rust231+16docs/4unit+2doctestsnovos. Payablevalidado/gates/review/validators/stagedscanPASS;CI final/nota117/liberação pendentes;REC-005..015 (demais estados,valorporocorrência,calendário,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-004.


REC-005 VERIFY E3:Paid e mark_paid puros,apenasPayable+Exitválida devalorigual/IDnãonulo;domínio/docs anteriores preservados. RED/GREEN4unit,Rust235+16docs/quality/security/LLVMdomain123/123 e15/15 comcontrole/review PASS. CIs pendentes;REC-006..015 (valor por ocorrência,calendário,geração,persistência,pagamento transacional,conta/contexto,saída única e idempotência persistida),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-005.


REC-005 done/E3/passestrue após CI39201697719/SHA1656a218b2bc0f74735cafc7e56bb9d8bacab5c8success23/0skip/7marcadores,Rust235+16docs/4unitnovos. Paid/mark_paid puros/gates/review/validators/stagedscanPASS;CI final/nota118/liberação pendentes;REC-006..015 (valor por ocorrência,calendário,geração,persistência,pagamento transacional,conta/contexto,saída única e idempotência persistida),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-005.


REC-006 VERIFY E3:Overduepositivo/mark_overdue puro comClockinjetado e now>due_at;mark_paidreal aceita vencido. RED/GREEN4unit/Rust239+16docs/quality/security/LLVMdomain142/142 e16/16 comcontrole/review PASS. Fixture1ns corrigida100ns após probeWindows,originais/falha preservados,produçãointacta. CIs pendentes;REC-007..015 (calendário,fuso,geração,valor por ocorrência,persistência,pagamento transacional,conta/contexto,saída única e idempotência persistida),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-006.


REC-006 done/E3/passestrue após CI39204073999/SHA5f145b619b43eb7213357235d12e8e680824f4c5success23/0skip/7marcadores,Rust239+16docs/4unitnovos. Overdue/Clockdeadline/mark_paid/gates/review/validators/stagedscanPASS;fixture100ns/probe/falhaoriginais registrados;CI final/nota119/liberação pendentes;REC-007..015 (calendário,fuso,geração,valor por ocorrência,persistência,pagamento transacional,conta/contexto,saída única e idempotência persistida),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-006.


REC-007 VERIFY E3:migration7recurrences/obligationsSTRICT,templates/estados/FK/UNIQUE/contextosnapshot;5SQLitevazio+upgrade+constraints+reopen+rollbacksegundaDDL/retry/2mutantesPASS. Rust244+16docs/quality/security/LLVMdatabase276/280 e25/25 comcontroleTODOSSQL/review PASS. CIs pendentes;REC-008..015 (calendário/fuso,geração,valor por ocorrência,pagamento transacional,Exit/valor/contexto correspondentes,saída única e idempotência do caso real),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-007.


REC-007 done/E3/passestrue após CI39206760143/SHA0717d00ae478ffc2dedee6d9ad24f923084d01c8success23/0skip/7marcadores,Rust244+16docs/5SQLnovos. Migration7/constraints/upgrade/rollback/retry/2mutantes/gates/review/validators/stagedscanPASS;CI final/nota120/liberação pendentes;REC-008..015 (calendário/fuso,geração,valor por ocorrência,pagamento transacional,Exit/valor/contexto correspondentes,saída única e idempotência do caso real),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-007.


REC-008 VERIFY E3:geraçãofixaPayable/calendárioBRTUTC-3fixo(defaultintegrador)+configobligationauditSQLatômicos/sem movimentos. RED3stages/GREEN4domain6app5SQL+2docs;Rust259+18docs/quality/security/LLVM3críticos100% comcontrole/review PASS. CorreçõesDATA locais efixmecânicoClippy registrados/originais preservados. CIs pendentes;REC-009..015 (variável,valor por ocorrência,vencer,pagamento transacional,saída única e idempotência do caso real),política de fuso fixa sujeita a resposta humana,UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-008.


REC-008 done/E3/passestrue após CI39212810372/SHA128cd0ab765b87786bb38945d8f7c6a081b81733success23/0skip/7marcadores,Rust259+18docs/10unit5SQL2docsnovos. Geraçãofixa/calendárioBRTfixo/configobligationauditTx/rollback+poison/gates/review/validators/stagedscanPASS;CI final/nota121/liberação pendentes;REC-009..015 (variável,valor por ocorrência,vencer,pagamento transacional,saída única e idempotência do caso real),política de fuso fixa sujeita a resposta humana,UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-008.


REC-009 VERIFY E3:geração variável AwaitingValue/NULL e auditoria em Tx compartilhada com Fixed,sem movimentos;9unit+5SQL novos,Fixed regressões273+18docs/gates/security/LLVM3críticos100% com controle negativo/review PASS. CI fonte/final pendentes;REC-010..015 (informar valor por ocorrência,pagamento transacional,saída única e idempotência de sucesso),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;calendário reaproveita default do integrador último dia/fim do dia UTC-3 fixo,sem IANA/DST histórico/aceite humano. Fonte:handoff REC-009.


REC-009 done/E3/passestrue após CI39220204263/SHA5504bae8b2938761f0b245fe330be7e303af504bsuccess23/0skip/7marcadores,Rust273+18docs/9unit+5SQLnovos. Geração variável AwaitingValue/NULL/Tx compartilhada/Fixed regressões/rollback+poison/gates/review/validators/stagedscan PASS;CI final/nota122/liberação pendentes;REC-010..015 (informar valor por ocorrência,pagamento transacional,saída única e idempotência de sucesso),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;calendário reaproveita default do integrador último dia/fim do dia UTC-3 fixo,sem IANA/DST histórico/aceite humano;MetaE2Eativa;fonte:handoff REC-009.


REC-010 VERIFY E3:informar valor em uma ocorrência AwaitingValue→Payable positivo/metadata/templateNULL preservados/únicoUPDATEguardado+auditTx/rollback+poison/sem movimentos;9unit+5SQLnovos287+18docs/gates/security/LLVM3críticos100% comcontrole negativo/reviewP2corrigido PASS. Correçõesprodução integrador registradas;CIs pendentes;REC-011..015 (vencimento persistido,pagamento transacional,saída única e idempotência de sucesso),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;não há edição de valor já informado neste item. Fonte:handoff REC-010.


REC-010 | CORRECT/VERIFY staged | E3 | Publicação abortou antescommit emdiffcheck:espaçosfimdelinha dentrostringSQL. P-INT removeu somente whitespace,sequênciatokens de todoarquivo idêntica/únicoUPDATEguardado/params intactos. Originais10relatórios/hashes preservados emoperacao/REC-010-before-staged-correction-20261019 além33focais. Rustfmt/clippy/debug/release eLLVM refeitos apóstrimPASS,287+18docs/9-36-67linhas100% comcontrole0;7outroschecks anteriores válidos semrerun. Perfis anteriores preservados eLLVM nova pastaafter-format;nenhumthresholdalterado. E2P-REV anterior/P2resolvido,semnovachamada porwhitespace. CIfonte/final/nota123/liberação pendentes;MetaE2Eativa.


REC-010 done/E3/passestrue após CI39231570957/SHA2e7d8fb14a8ca4fde22b54d817a3e71439f68095success23/0skip/7marcadores,Rust287+18docs/9unit+5SQLnovos. Informar valor de uma ocorrência/Payable/únicoUPDATEguardado+audit/rollback+poison/gates/LLVM/reviewP2corrigido/validators/stagedscanPASS;CI final/nota123/liberação pendentes;REC-011..015 (vencimento persistido,pagamento transacional,saída única e idempotência de sucesso),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;não há edição de valor já informado neste item;MetaE2Eativa;fonte:handoff REC-010.


REC-011 VERIFY E3:Payable→Overdue estritamente apósClock/100ns/epochi128ano1,appClock1ID1Store/portaTimeexata/UPDATEguardado+auditTx/rollbackpoison/semalterarvalores;10unit+5SQLnovos302+18docs/gates/security/LLVM3críticos100% comcontrole negativo/review PASS. Rootfixtures/importsfmtcorreçõesregistradas;decisãohumanacalendárioemBUSINESS_RULES;CIs pendentes;REC-012..015 (pagamento transacional,saída única e idempotência),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;implementação de calendário atual usaUTC-3fixo,semprovaIANA/DSThistórico,oaceitehumanofoiparapolíticaúltimodia/fimdiaBrasília. Fonte:handoff REC-011.


REC-011 done/E3/passestrue apósCI39241758403/SHAc9db2729c3812feddacd2dabd8e82032432c3716success23/0skip/7marcadores,Rust302+18docs/10unit+5SQLnovos. Vencerobrigação/Clock100ns/epochi128ano1/guardasSQL/auditTx/rollbackpoison/gates/LLVM/review/validators/stagedscanPASS;decisãohumanacalendárioemBUSINESS_RULES;CI final/nota124/liberação pendentes;REC-012..015 (pagamento transacional,saída única e idempotência),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;implementação de calendário atual usaUTC-3fixo,semprovaIANA/DSThistórico,oaceitehumanofoiparapolíticaúltimodia/fimdiaBrasília;MetaE2Eativa;fonte:handoff REC-011.


REC-012 VERIFY E3:pagarPayable/Overdue→Exit+Paidlink+auditnumaTx,centavos/contexto preservados/2IDs1Clock1Store/guardas/rollbackpoison/semsegundaExit;6unit+5SQLnovos313+18docs/gates/security/LLVM2críticos100% comcontrole negativo/review PASS. CorreçõesexecutáveisROOT emAPParm/preIOvalidationsregistradas/SQLlocalintacto;CIs pendentes;REC-013..015 exigem provas próprias desaídaúnica/rollback/concorrência/repetição,idempotência de sucesso nãoprometida;UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;calendárioaprovadoúltimodia/fimdiaBrasília registrado,implementaçãoUTC-3fixosemprovaIANA/DSThistórico. Fonte:handoff REC-012.


REC-012 done/E3/passestrue apósCI39248473269/SHAb4ee66f594ef2c723df3f595c7d14aa35d82cee3success23/0skip/7marcadores,Rust313+18docs/6unit+5SQLnovos. PagarrecorrênciaExit+Paidlink+auditTx/guardas/rollbackpoison/replaybásico/gates/LLVM/review/validators/stagedscanPASS;correçõesexecutáveisROOTregistradas;CI final/nota125/liberação pendentes;REC-013..015 exigem provas próprias desaídaúnica/rollback/concorrência/repetição,idempotência de sucesso nãoprometida;UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;calendárioaprovadoúltimodia/fimdiaBrasília registrado,implementaçãoUTC-3fixosemprovaIANA/DSThistórico;MetaE2Eativa;fonte:handoff REC-012.


REC-013 VERIFY E3:1SQLnovo/8combinações16pagamentos/umaExit por obrigação em2meses/saldo eresultado exatos/reopen/histórico incremental preservado. Review1P2 corrigido e3mutantes finais rejeitados/GREEN6SQL/Rust314+18docs/gates/security PASS; produção/migrations imutáveis, coverageREC012 histórica referenciada sem nova medição. CIs/nota126/liberação pendentes;MetaE2Eativa;REC014/015/UIIPC/globalRust/instalador/E2Ependentes;fonte:handoff REC-013.


REC-013 done/E3/passestrue apósCI39252069599/SHA1434f282bd06445994baca20333207888ac030fbsuccess23/0skip/7marcadores,Rust314+18docs/1SQLnovo. UmaExit por obrigação/2meses/8combinações16pagamentos/saldo e resultado reais/reopen/históricoincremental/3mutantes/reviewP2corrigido/gatesPASS;produção e7migrations intactas,coveragehistórica referenciada;CI final/nota126/liberação pendentes;REC-014/015 ainda exigem provas próprias de rollback completo/concorrência/repetição; sucesso idempotente não prometido. UI/IPC, cobertura Rust global atual, instalador Windows/instalação/atualização/backup-restauração/hash/assinatura e E2E oficial pendentes. Política mensal último dia/fimdiaBrasília aceita e registrada; implementação UTC-3 fixa sem prova IANA/DST histórica;MetaE2Eativa;fonte:handoff REC-013.


REC-014 VERIFY E3:1SQLnovo/64cenários físicos8combinações×8falhasinclusiveCOMMITdeferred/snapshot+reopen/projeções reais preservadas/poison+freshválido;2mutantes parciais rejeitados/GREEN7SQL/Rust315+18docs/gates/security/reviewPASS. Produção/7migrations intactas;coverageREC012histórica referenciada. CIs/nota127/liberação pendentes;REC015/UIIPC/globalRust/instalador/E2Ependentes;Metaativa;fonte:handoffREC-014.


REC-014 done/E3/passestrue apósCI39255062132/SHA921465015075a33f16368761f39bcf53c1a56c1fsuccess23/0skip/7marcadores,Rust315+18docs/1SQLnovo. Rollbackpagamento64cenários/snapshot+reopen/leitores+projeções/poison+freshválido/FKdeferredCOMMIT/2mutantes/review/gatesPASS;produção e7migrations intactas,coveragehistórica referenciada;CI final/nota127/liberação pendentes;REC-015 ainda exige provas próprias de concorrência/repetição;sem retry automático/sucesso idempotente prometido. Não prova powerloss/WAL/panic-abort/falha física deROLLBACK/recuperação global. UI/IPC,cobertura Rust global atual e instalador Windows/instalação/atualização/backup-restauração/hash/assinatura/E2E oficial pendentes. Política mensal último dia/fimdiaBrasília aceita;implementaçãoUTC-3fixa sem provaIANA/DSThistórica;Metaativa;fonte:handoffREC-014.


REC-015 VERIFY E3:2SQLnovos/8seriais+8corridas/Paid0UUID/staleUUIDnovoereusadosemsegundaExit/1vencedor+loserpoison/outramêspagável/PaidFK-audit-payload8col/reopen/histórico/projeções exatos;3REDs atuais verificados/GREEN9SQL/P2corrigidopeloroot/Rust317+18docs/gates/security/reviewPASS. Parsercorrigidosemrerunválido,rawpreservado;skillsadaptadas aplicadas. Produção/7migrations intactas,coveragehistórica referenciada;CIs/nota128/liberação pendentes;UIIPC/globalRust/instalador/E2Ependentes;Metaativa;fonte:handoffREC-015.


REC-015 done/E3/passestrue apósCI39261345263/SHAdfc4c405366cce24ee28be8db8bc147d1f3d4652success23/0skip/7marcadores,Rust317+18docs/2SQLnovos. Repetição8seriais+8corridas/Paid0UUID/staleguard/1vencedor/loserpoison/outramêspagável/payload8col-audit-PaidFK/reopen/históricoprojeções/3REDs/gatesPASS;P2darevisãocorrigidopeloroot/0pendentes;parserreclassificadosemrerunválido eoriginais preservados,skillsadaptadasaplicadas;produção e7migrations intactas,coveragehistórica referenciada;CI final/nota128/liberação pendentes;Repetição retorna erro,sem sucesso idempotente anterior/chave deintenção/retryautomático. Perdedor concorrente pode falhar noBEGINIMMEDIATE ou guarda apóscommit;guarda éprovada independentemente na sequência serial comUUIDsnovos. Não prova vencedor fixo/processos separados/distribuição/powerloss/WAL/panic-abort/falha físicaROLLBACK/recuperação global. UI/IPC,coberturaRustglobal e instaladorWindows/instalação/atualização/backuprestauração/hashassinatura/E2E oficial pendentes. Política mensal último dia/fimdiaBrasília aceita;implementaçãoUTC-3fixa semprovaIANA/DSThistórica;Metaativa;fonte:handoffREC-015.


PRD-001 VERIFY E3:MVP07 representação Product id/nome/descrição/preçosugerido/ativo;1unit+2compile_fail/RED/GREEN/Rust318+20docs/gates/security/reviewPASS,domínioanterior e7migrations preservados. LLVMdomain>=95 com controle;tipo sem linhas executáveis/sem percentual próprio. Validações/CRUD/SQLite/UI e INV-SAL005 pendentes. CIs/nota129/liberação pendentes;MetaE2Eativa;fonte:handoff PRD-001.


PRD-001 done/E3/passestrue apósCI39271100208/SHAc092545f872849245405f65b4529e68e3d5ec60dsuccess23/0skip/7marcadores,Rust318+20docs/1unit+2doctestsnovos. Product template puro/RED/GREEN/LLVMdomaincomcontrole/gates/security/reviewPASS;domínio anterior e7migrations preservados;CIfinal/nota129/liberação pendentes;PRD-002..007 (validações,migration,criar,alterar,desativar),vendas/snapshots e INV-SAL-005 ainda pendentes. Representação não é aceite de entrada inválida por operação. UI/IPC,cobertura Rust global e release Windows/instalação/atualização/backup-restauração/hashassinatura/E2E oficial pendentes. Product tem zero linhas executáveis;percentual próprio não aplicável;stable branches não medidos;sem coverage global ou remotoRust.;Metaativa;fonte:handoff PRD-001.


PRD-002 VERIFY E3:MVP07 Product.validate_name puro/nome obrigatório conforme trim/erro existente/sem normalizarcampos nem acoplarpreçoativo;2unitnovos+PRD001/doctests preservados/RED/GREEN3/Rust320+20docs/gates/security/reviewPASS. LLVMdomain>=95 com controle sem2novos rejeitado;domínio anterior e7migrations intactos. CIs/nota130/liberação pendentes;MetaE2Eativa;fonte:handoff PRD-002.


PRD-002 done/E3/passestrue apósCI39277622649/SHAeda023c04e073daa00f981c6210bba554398247bsuccess23/0skip/7marcadores,Rust320+20docs/2unitnovos. Product.validate_name puro/Unicodeblank/semnormalização-preçoativo/RED/GREEN3/LLVMdomaincomcontrole/gates/security/reviewPASS;domínio anterior e7migrations preservados;CIfinal/nota130/liberação pendentes;PRD-003..007 (preço,migration,criar,alterar,desativar),vendas/snapshots e INV-SAL-005 pendentes. validate_name não valida produto inteiro nem autoriza operação financeira;whitespace segue str::trim,não é sanitização geral. UI/IPC,cobertura Rust global e release Windows/instalação/atualização/backup-restauração/hashassinatura/E2E oficial pendentes;stable branches/statements/global/CI Rustcoverage não medidos.;Metaativa;fonte:handoff PRD-002.


PRD-003 VERIFY E3:MVP07 preço sugerido não negativo/zero válido conforme humano19out;validate_suggested_price puro/2unitnovos/RED/GREEN5/Rust322+20docs/gates/security/reviewPASS. LLVMdomain>=95controle omitindo5Productrejeitado;domínio anterior e7migrations intactos. CIs/nota131/liberação pendentes;Metaativa;fonte:handoffPRD-003.


PRD-003 done/E3/passestrue apósCI39288285719/SHAc8c11fb388b7ea7d4ab86768fb947842f53fc91asuccess23/0skip/7marcadores,Rust322+20docs/2unitnovos. Preço zero válido/negativosrejeitados/validaçãopura/RED/GREEN5/LLVMdomaincontrole5Product/gates/security/reviewPASS;domínio anterior e7migrations intactos;CIfinal/nota131/liberação pendentes;PRD-004..007(migration/criar/alterar/desativar),vendas/snapshots/INV-SAL005,UI/IPC,cobertura Rustglobal,releaseWindows/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. Stablebranches/statements/global/CI Rustcoverage não medidos;validação focal não valida produto inteiro.;Metaativa;fonte:handoffPRD-003.


PRD-004 VERIFY E3:MVP07 migration8ProductSTRICT5campos/zero válido/negativosrejeitados/2constantesRustzeroexec/5SQLitefísicosREDGREEN+2mutantes/gates/security/reviewPASS. LLVMdatabase>=95controle semTODOSSQLite rejeitado;6SQLanteriores+v1vazia/executores/domíniointactos. CIs/nota132/liberaçãopendentes;Metaativa;fonte:handoffPRD-004.


PRD-004 done/E3/passestrue apósCI39292809030/SHAc93e947e0a1b4d2fdea9793aa02a25276022fef5success23/0skip/7marcadores,Rust327+20docs/5SQLnovos. Migration8productsSTRICT/zero válido/2constantesRustzeroexec/REDGREEN+2mutantes/LLVMdatabasecomcontrole/gates/security/reviewPASS;executores/domínio/6SQLanteriores+v1vaziaintactos;CIfinal/nota132/liberaçãopendentes;PRD005..007(criar/alterar/desativar),vendas/snapshots/INVSAL005,UIIPC,coberturaRustglobal,releaseWindows/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. SQLname<>vazio não replicaUnicodeblank do domínio. STRICT permite coerções sem perda;sem percentualSQL/globalRust/CI/statements/branchesstable.;Metaativa;fonte:handoffPRD-004.


PRD-005 VERIFY E3:MVP07 criaçãoProductativo/validaçãonomepreçoantesportas/SQL+auditIMMEDIATEparametrizado/rows1/poisoner-freshrecovery/7unit4SQLnovosREDGREEN/gates/security/reviewPASS. LLVM2arquivos>=95controleomitenovosrejeitado;7SQLmigrationsanteriores+v1vazia/domíniointactos. CIs/nota133/liberaçãopendentes;Metaativa;fonte:handoffPRD-005.


PRD-005 done/E3/passestrue apósCI39298191906/SHA0ecf5a59905791a2eae452716c0b8c2220207246success23/0skip/7marcadores,Rust338+20docs/7unit4SQLnovos. CriaçãoProductativo/validaçõesantesportas/SQL+auditIMMEDIATE/rows1/poisoner-freshrecovery/REDGREEN/LLVM2arquivoscomcontrole/gates/security/reviewPASS;7migrationsSQL+v1vazia/domainintactos;CIfinal/nota133/liberaçãopendentes;PRD006/007(editar/desativar),vendas/snapshots/INVSAL005,UIIPC,coberturaRustglobal,releaseWindows/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. UUIDversão/uniqueness pertencem ao adapterUuidGenerator;zeroaudit/entity/timeinválidos são rejeitados porappend_audit_event e revertidosSQL. Stablebranches/statements/global/CI Rustcoverage não medidos.;Metaativa;fonte:handoffPRD-005.


PRD-006 E3 local em23/10/2026: 9unit edit_product_tests.rs e5SQLite físicos product_editing.rs cobrem campos exatos, ambos estados ativos, limites monetários/tempo, erros, ausência, auditoria, rollback, conexão inutilizada e recuperação. Debug/release customprotocol:202lib+7bin+143SQL=352ordinários,20doctests,1WebViewignored. Formatação/clippy/TS/build/npm/plugins/SAST aprovados; Cargo/OSV bloqueados por exceções expiradas. Sem prova E2E; CIs novos pendentes. Fonte: planning/handoffs/PRD-006.md.


PRD-006 VERIFY E3 local aprovado23out: ediçãoProduct/9unit5SQL/352Rust+20docs/LLVM2arquivos100%controle rejeitado/gates/security/reviewPASS. Vite8.3.3/Tokio1.53.3;E4somente2exceçõesWindows/hashescorrentes/antes06novUTC conforme aceite da proposta c86bf6753ee0bb8d579e74e699e4bab108222835. ProduçãoGPTfallback0saídaslocaisusadas;2P2docs corrigidos. CIs/nota134/liberação pendentes/passesfalse;INVSAL005SAL008/E2Ependentes;fonte:handoffPRD-006.


PRD-006 done/E3/passestrue apósCI39998764928/SHA46ddaf405b7dc4a4ab7b7b0683d87a6fd47d9c0esuccess23/0skip/7marcadores,Rust352+20docs/9unit5SQLnovos. EdiçãoProduct/validaçõesantesportas/IDativopreservados/UPDATE+auditIMMEDIATE/rows1/NotFound/poisoner-freshrecovery/REDGREEN/LLVMapp+adaptercomcontrole/gates/security/reviewPASS;PRD005/migrations/domainintactos;CIfinal/nota134/liberaçãopendentes;PRD007(desativar),vendas/snapshotsreais/INVSAL005provaSAL008,UIIPC,coberturaRustglobal,releaseWindows/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. Metadataeditpreservaativo inclusiveinativo,padrãoCategoryrename;nenhumasnapshotfictíciausada. Stablebranches/statements/global/CI Rustcoverage não medidos.;Metaativa;fonte:handoffPRD-006.


PRD-007 VERIFY E3: desativação Product/7 unit+5 SQL/364 Rust+20 doctests/LLVM dois arquivos 100% e controle rejeitado/gates de qualidade e segurança PASS; P-REV E2 sem achados. Uma inferência local P-DAT/121,752s usada sem correção de comportamento; app por reuso determinístico de Category. CIs/nota135/liberação pendentes; passesfalse; E2E oficial pendente; fonte: handoff PRD-007.


PRD-007 done/E3/passestrue após CI 40004684967/SHA b861d473dc1b4fb26771ea7d600923a5d1b397fc, success23/0 skip/7 marcadores; Rust364+20 docs/7 unit+5 SQL novos. Desativação preserva metadados, aceita já inativo com nova auditoria; UPDATE+audit IMMEDIATE/rows1/NotFound/rollback/poisoner/fresh recovery; criação/edição anteriores preservadas. RED/GREEN/LLVM app+adapter com controle/gates/security/review PASS. CI final/nota135/liberação pendentes; Meta ativa. Snapshots reais de venda/INV-SAL-005 dependem de SAL-008; jornadas UI/IPC, cobertura Rust global e instalador Windows com instalação/atualização/backup/restauração/hash/assinatura/E2E oficial pendentes. LLVM local mede somente linhas/funções dos dois arquivos inteiros; branches/statements/global/CI Rust coverage não medidos. Fonte: handoff PRD-007.


CUS-001 VERIFY E3: Customer mínimo puro id/nome/telefone opcional, dados sintéticos exatos/sem Debug ou Display; 1 unit+2 compile_fail novos, Rust365+22docs/gates/security/P-REV PASS. LLVM domain95 com controle, tipo sem percentual próprio. CUS002..005/UI/IPC/E2E pendentes; CI/nota136/liberação pendentes; passesfalse/Meta ativa; fonte: handoff CUS-001.


CUS-001 done/E3/passestrue após CI 40013227015/SHA d498cc325cbc5c7af2957b232419b2d70ea4bf15, success23/0skip/7 marcadores, Rust365+22docs/1unit+2doctests novos. Customer mínimo puro id/nome/telefone opcional/sem Debug ou Display; RED/GREEN/LLVM domain com controle/gates/security/review PASS. Domínio anterior/migrations/deps/exceções preservados; CI final/nota136/liberação pendentes; Meta ativa. Representação não decide aceite de entrada: CUS-002/003 validam nome/telefone e CUS-004/005 implementam persistência/criação. Formato/tamanho/normalização/unicidade do telefone não decididos aqui. Vendas/INV-SAL-002, UI/IPC, cobertura Rust global e instalador/instalação/atualização/backup/restauração/hash/assinatura/E2E oficial pendentes. Customer tem zero linhas executáveis; percentual próprio não aplicável. LLVM local cobre somente domain/mod.rs inteiro; branches/statements/global/CI Rust coverage não medidos. Fonte: handoff CUS-001.


CUS-002 VERIFY E3: Customer.validate_name por reuso literal Product, vazio/whitespace inválidos, conteúdo bruto preservado/telefone independente. RED/GREEN3Customer/2unit novos/Rust367+22docs/LLVMdomain95 e controle/gates/security/P-REV PASS.0inferências/runtime não iniciado. CUS003..005/UI/IPC/E2E pendentes; CI/nota137/liberação pendentes; passesfalse/Meta ativa; fonte: handoff CUS-002.


CUS-002 done/E3/passestrue após CI 40015925340/SHA 6c17d78f192d91623107470e8403151b20b03529,success23/0skip/7 marcadores,Rust367+22docs/2unit novos. Customer.validate_name puro por reuso Product, blank rejeitado/texto bruto e campos preservados/telefone independente. RED/GREEN/LLVM domain com controle/gates/security/review PASS.0inferências; domínio anterior/migrations/deps/exceções preservados. CI final/nota137/liberação pendentes; Meta ativa. Não valida Customer inteiro nem formato/tamanho/normalização/unicidade de telefone. CUS-003/004/005, vendas/INV-SAL-002, UI/IPC, Rust global, instalador/instalação/atualização/backup/restauração/hash/assinatura/E2E oficial pendentes. LLVM local cobre domain/mod.rs inteiro; controle sem domínio prova instrumentação, não causalidade isolada do método. Percentual isolado/branches/statements/global/CI Rust coverage não medidos. Fonte: handoff CUS-002.


CUS-003 VERIFY E3: None válido/Some máscara ASCII(DD)9XXXX-XXXX/DDD67/inicial9, sem mutação/nome/IO. Regra humana no produto/fontes Anatel preservadas. RED/GREEN7Customer/4unitnovos/Rust371+22docs/LLVMdomain95controle/gates/security/P-REV PASS. Laguna timeout603,471s/0output usado; P-INT fallback; runtime encerrado. CUS004005/UI/IPC/E2E pendentes; CI/nota138/liberação pendentes; passesfalse/Metaativa; fonte: handoff CUS-003.


CUS-003 done/E3/passestrue apósCI40048716627/SHAb94b80800dc6ad82e9be36da5271b4d34b8000c8,success23/0skip/7marcadores,Rust371+22docs/4unitnovos. Customer.validate_phone opcional, formato exato/DDD67/primeiro9/ASCII/sem mutação/panic/nome; regra humana registrada/fontes Anatel. RED/GREEN/LLVMdomaincontrole/gates/security/review PASS; métodos anteriores/migrations/deps/exceções preservados. CIfinal/nota138/liberação pendentes; Meta ativa. Validação sintática e de DDD; não confirma existência, atividade, titularidade ou WhatsApp. Some vazio ou fora da máscara é inválido; telefone ausente permanece opcional. CUS004005/persistência/UI/IPC/vendas/INV-SAL-002/Rustglobal/instalador/instalação/atualização/backup/restauração/hashassinatura/E2Eoficial pendentes. LLVM local somente domain/mod.rs inteiro; controle sem domínio prova instrumentação, não causalidade isolada; sem percentual isolado/branches/statements/global/CI Rustcoverage. Fonte: handoff CUS-003.


CUS-004 VERIFY E3: migration009customersSTRICT/ID16/nome/phoneNULLoumaskASCII15DDD67,duasconstantes9/zerofunções. RED/GREEN5SQLite/vazio-upgrade8histórico-rollback-colisão/2mutantesrejeitados/Rust376+22docs/LLVMdatabase95controleTODOSSQL/gates/security/P-REV PASS;7SQLanterioresintactos.0inferências/runtime não iniciado. CUS005/UIIPC/globalRust/instalador/E2Ependentes;CI/nota139/liberação pendentes;passesfalse/Metaativa;fonte:handoff CUS-004.


CUS-004 done/E3/passestrue apósCI40051436747/SHAc37983da2d3c588807bd0e1f6b312471455ae680,success23/0skip/7marcadores,Rust376+22docs/5SQLnovos. Migration009customersSTRICT/phoneopcionalmaskDDDaprovados/2constv9/zerofunções;vazio/upgrade8histórico/rollback9/colisão/constraints/mutantes/LLVMdatabasecontrole/gates/security/review PASS;7SQLanterioresintactos. CIfinal/nota139/liberação pendentes;Meta ativa. SQLname<>empty segue Product/NULsafe, não implementa trimUnicode do domínioCUS002. Phoneassegura sintaxe/DDD, não linha ativa ou titularidade. Constantes/SQLsempercentualexecutável próprio; LLVMdatabase inteiro mede executores existentes, controle sem TODOSSQLite prova instrumentação. SemRustglobal/CI/SQLpercent/statements/branchesstable. CUS005/criação/clienteemvendapendente/UIIPC/vendas/instalador/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. Nove versões significam oitoSQLfísicos maisv1vazia. Fonte:handoff CUS-004.


CUS-005 VERIFY E3: criarCustomer name/phoneNULLSomebrutos/validarantesportas/2IDsClockchecked/INSERT+customer_createauditIMMEDIATE/rows1/invaliddiretoreusável/rollback-poisoned-freshrecovery. RED/GREEN7unit4SQL/Rust387+22docs/LLVM2arquivos95controlegates/security/P-REV PASS.8SQLanterioresintactos;0inferências/runtime. CIs/nota140/liberação pendentes;MVP09UI/clienteemvendapendente/SMK005/globalRust/instalador/E2Ependentes;passesfalse/Metaativa;fonte:handoff CUS-005.


CUS-005 done/E3/passestrue apósCI40054116789/SHAa3020fb565cc0125d7d55f771e7df2d885648d74,success23/0skip/7marcadores,Rust387+22docs/7unit4SQLnovos. CreateCustomer validaçõesantesportas/camposNoneSome/2IDsClockchecked/INSERT+auditIMMEDIATE/rows1/rollback/poisoned-freshrecovery/opacos;REDGREEN/LLVM2arquivoscontrole/gates/security/review PASS.8SQL/domínio/produtoanterioresintactos;0inferências. CIfinal/nota140/liberação pendentes;Meta ativa. Não criaUI/IPC/listagem/edição/desativaçãodecliente/CRM/estoque/venda. ClienteemvendapendenteINV-SAL-002/jornadaMVP09/UI009/SMK005, coberturaRustglobal,instalador/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. UUIDversão/unicidade pertence ao fornecedor;MAXms/overflow testados no helper,semalegarSystemTimeMAXdoplatforma. LLVMlocal sódoisarquivosinteiros comcontroleomitenovos7unit4SQL;sembranches/statements/global/CI Rustcoverage. Custo não registrado. Fonte:handoff CUS-005.


SAL-001 VERIFY E3: SaleItem puro7campos/quantidadeinteira/centavos/valorespróprios; decisão humana R$ ou percentual registrada. RED/GREEN1unit2compile_fail/gates/security/P-REV PASS;Rust388+24docs/LLVMdomain95controle. Validação/cálculo/SQL/imutabilidade operacional/UI/IPC/E2E pendentes;CI/nota141/liberação pendentes;passesfalse/Metaativa;fonte:handoff SAL-001.


SAL-001 done/E3/passestrue apósCI40147508621/SHA77468ca1aabd84daa88768fd1c03925be8683c6a,success23/0skip/7marcadores,Rust388+24docs/1unit2docsnovos. SaleItem7campos próprios/quantidadeinteira/centavos/semDebugDisplay;RED/GREEN/LLVMdomain95controle/gates/security/reviewPASS;domínio/migrations/deps/exceções preservados;CIfinal/nota141/liberação pendentes;Metaativa. Representação não prova validação, conversão percentual, cálculo, persistência ou imutabilidade operacional SAL-008/INV-SAL-005. SAL-002/003/004 e demais consumidores seguem seus IDs. Precisão percentual/arredondamento não definidos neste tipo puro. UI/IPC, cobertura Rust global e release Windows com instalação/atualização/artefato/hash/assinatura/backup/restauração/E2E oficial pendentes. SaleItem tem zero linhas executáveis e não recebe percentual próprio. LLVM local mede somente domain/mod.rs inteiro; branches/statements/global/CI Rust coverage não medidos. Fonte:handoff SAL-001.


SAL-002 VERIFY E3: validate_quantity inteiroi64>0, limitesMIN/-1/0 e1/2/MAX,7campos preservados;RED/GREEN3módulo2unitnovos/Rust390+24docs/LLVMdomain95controle/gates/security/P-REV PASS.0inferências. Monetários/cálculo/SQL/snapshot operacional/UI/IPC/E2E pendentes;CI/nota142/liberação pendentes;passesfalse/Metaativa;fonte:handoff SAL-002.


SAL-002 done/E3/passestrue apósCI40148872782/SHAa06c4c81037d92b184c7c5f2579824cea10dec43,success23/0skip/7marcadores,Rust390+24docs/2unitnovos. validate_quantity inteiroi64>0/limites/preservação7campos;RED/GREEN/LLVMdomain95controle/gates/security/reviewPASS;0inferências;domínio/migrations/deps/exceções preservados;CIfinal/nota142/liberação pendentes;Metaativa. Não valida item inteiro, campos monetários ou descrição nem calcula/converte desconto percentual. SAL-003/004 e demais consumidores, snapshot operacional SAL-008/INV-SAL-005, UI/IPC, Rust global, instalador Windows/instalação/atualização/artefato/hash/assinatura/backup/restauração/E2E oficial pendentes. LLVM local cobre domain/mod.rs inteiro; controle sem domínio prova instrumentação, não causalidade isolada do método. Percentual isolado/branches/statements/global/CI Rust coverage não medidos. Fonte:handoff SAL-002.


SAL-003 VERIFY E3: descontoR$ ou0..100%/2casas/meio centavo para cima;5unit17fixturesDecimal/10001pb×7brutos/2docsprivacidade. RED/GREEN5/Rust395+26docs/LLVMdomain95controle/gates/security/P-REV PASS.1Laguna130,6s/correção determinística/CLIrefresh/runtimeoff. Parser/cálculo líquido/SQL/snapshot operacional/UI/IPC/E2E pendentes;CI/nota143/liberação pendentes;passesfalse;objetivoE2E íntegro;Metaformalblocked;fonte:handoffSAL-003.


SAL-003 done/E3/passestrue apósCI40152206878/SHA6ab5cc31645be236d4988f486db0794007178945,success23/0skip/7marcadores,Rust395+26docs/5unit2docsnovos. validate_discount R$/%/limites/centavos/meio para cima;RED/GREEN/LLVMdomain95controle/gates/security/reviewPASS;1Laguna130,6s/correção/root/runtimeoff;domínio/migrations/deps/exceções preservados;CIfinal/nota143/liberação pendentes;objetivoE2E íntegro. Entrada textual commais2casas deve ser rejeitada no futuro consumidor; não há parser/UI implementado. Cálculo bruto/líquido e validação de snapshot SAL-004,criação/persistência/snapshot operacionalSAL-008/INV-SAL-005,UI/IPC,Rustglobal,releaseWindows instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. LLVM local mede domain/mod.rs inteiro;controle sem domínio prova instrumentação,não causalidade isolada;sem percentuais isolados/branches/statements/global/CI Rustcoverage. Meta formal observada blocked após resposta humana; decisão de produto resolvida e execução autorizada,sem declarar retomada formal inexistente. Fonte:handoffSAL-003.


SAL-004 VERIFY E3: calculate_total i128/líquido0..i64MAX/descontoaplicado1vez/7campospreservados;5unitnovos/GREEN8/Rust400+26docs/LLVMdomain95controle/gates/security/P-REV PASS. Testes24out reutilizados porigualdade hashes25out. LagunaPDOM117,032s/QwenPTRCconferido/runtimesoff. CI/nota144/liberação pendentes;passesfalse;uma tarefa manualsemretomarMeta;SAL005nãoiniciar;fonte:handoffSAL-004.


SAL-004 done/E3/passestrue apósCI40168424623/SHAc63029e20d31a263835b9f2e273d933df2895dcf,success23/0skip/7marcadores,Rust400+26docs/5unitnovos. Total i128/resultadolíquidoi64/desconto1vez/preservação7campos;RED/GREEN/LLVMdomain95controle/gates/security/reviewPASS;Laguna117,032s+Qwenconferido/runtimesoff;domínio/migrations/deps/exceções preservados;CIfinal/nota144/liberação pendentes;única tarefa manual/Metaformalblocked. Não cria nem persiste item/venda, não prova snapshot operacionalSAL008/INV-SAL-005,UI/IPC,coberturaRustglobal ou releaseWindows instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial. LLVMmede domain/mod.rs inteiro;controle semdomínio prova instrumentação,não causalidade isolada;sempercentuaisisolados/branches/statements/global/CI Rustcoverage. Fechamento manual de uma única tarefa autorizado25out,sem retomar Meta formalblocked;SAL-005 não iniciar. Fonte:handoffSAL-004.


SAL-005 VERIFY E3: factorylivre validado/productNone/descriçãopreservada,desconto/total corretos/helperwide/APIi64 compatível;8unitnovos/9Decimal100digits/REDGREEN/Rust408+26docs/LLVMdomain95controle/gates/security/P-REV PASS. Laguna603,293s timeout0diff/rootfallback/errodainferênciagenéricacorrigido/oldcallersintactos/runtimeoff. CI/nota145/liberação pendentes;passesfalse;Metaativa;INV-SAL006 completo eUIE2E pendentes;fonte:handoffSAL-005.


SAL-005 done/E3/passestrue apósCI40194173733/SHA1f9a943c230cbbe7409f56728449ef140321cd72,success23/0skip/7marcadores,Rust408+26docs/8unitnovos. Factorylivre/productNone/descriçãopreservada/desconto-totalvalidos/helperwideprivado/APIi64mantida;RED/GREEN/LLVMdomain95controle/gates/security/reviewPASS;Laguna603,293stimeout/rootfallback/correçãodecompatibilidade/oldtestsintactos/runtimeoff;domínio/migrations/deps/exceções preservados;CIfinal/nota145/liberação pendentes;Metaativa. Factory puro e product_idNone não provam criação/persistência de venda ou INV-SAL006 completo: SAL006/013 próprios;imutabilidade operacionalSAL008,UI/IPC,coberturaRustglobal,releaseWindows instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. Exigência de descrição-rótulo é escolha técnica explícita da spec por reuso do padrão de nomes,não decisão humana nova atribuída. LLVM local cobre domain/mod.rs inteiro;controle semdomínio prova instrumentação,não causalidade isolada;sem percentuaisisolados/branches/statements/global/CI Rustcoverage. Metaformal retomadaactive confirmada;manter atéE2E,sempre encerrar locais antespausa/interrupção. Fonte:handoffSAL-005.


SAL-006 VERIFY E3 parcial INV-SAL006/MVP08: catálogo/estado lógico preservados no factory público em2SQLite;controles reais COUNT/ROWS rejeitados,produtoNone;produção intacta. 410Rust+26docs/2SQLnovos/gates/security/LLVMdomain95controle/P-REV PASS;CI/nota146/liberação pendentes;passesfalse. Prova parcial INV-SAL006/MVP08 na fronteira factory/catálogo: factory puro não recebe store/conexão/caminho. Persistência de item na venda/ausência de efeitos no consumidor futuro continua SAL013/SMK005, não satisfeita aqui. UI/IPC, Rustglobal, instalador/instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. LLVMdomain inteiro inalterado; controle0 confirma instrumentação, não causalidade isolada nem percentual dos novos testes/global/CI/branches/statements. Meta formal observada blocked, retomada depende controle humano do app; heartbeat autoriza este ID, sem reativação simulada. Custo não registrado. Fonte:handoff SAL-006.


SAL-006 done/E3/passestrue apósCI40229763924/SHA7d2c301e99110172de5c201e208dfc4dce5ffd02,success23/0skip/7marcadores,410Rust+26docs/2SQLnovos. Não interferência factory/catálogo/estado lógico,produtoNone,2controlesCOUNT/ROWS rejeitados;produção intacta;CIfinal/nota146/liberação pendentes. Prova parcial INV-SAL006/MVP08 na fronteira factory/catálogo: factory puro não recebe store/conexão/caminho. Persistência de item na venda/ausência de efeitos no consumidor futuro continua SAL013/SMK005, não satisfeita aqui. UI/IPC, Rustglobal, instalador/instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. LLVMdomain inteiro inalterado; controle0 confirma instrumentação, não causalidade isolada nem percentual dos novos testes/global/CI/branches/statements. Meta formal observada blocked, retomada depende controle humano do app; heartbeat autoriza este ID, sem reativação simulada. Custo não registrado. Fonte:handoff SAL-006.


SAL-007 VERIFY E3: create_product_item puro exige template ativo/nome e preço sugerido válidos; copia descrição exata inclusive vazia em String própria,product_idSome,id próprio e preço de lançamento explícito. Reusa validação/cálculo monetário via create_item privado extraído literalmente do factory livre; API e comportamento de create_free_item preservados. Zero/ambas modalidades/tiesUP/bruto largo e erros opacos;template inalterado em sucesso/erro. Seis unitários novos; domínio não relacionado, testes antigos, migrations/deps/exceções preservados. RED/GREEN6novos/416Rust26docs/gates/security/LLVMdomain95controle/P-REV PASS;CI/nota147/liberação pendentes;passesfalse. Preço de lançamento explícito é interpretação anunciada de sugerido, pergunta opcional sem resposta não atribuída a decisão humana; default de catálogo e edição na UI são consumidores futuros. Copiar campos próprios não prova snapshot operacional: edição real/persistência/INV-SAL005 dependem SAL008 e consumidores seguintes. UI/IPC,Rustglobal,instalador/instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. LLVM local somente domain/mod.rs inteiro;controle0 prova instrumentação,não causalidade isolada/percentual de função nova/global/CI/branches/statements. Meta ativa até critério oficial;locais encerrar antespausa/interrupção. Custo não registrado. Fonte:handoff SAL-007.


SAL-007 done/E3/passestrue apósCI40262115383/SHAd078d15a462911e1c210316bf0e3d924a6a4cde3,success23/0skip/7marcadores,416Rust+26docs/6unitnovos. create_product_item puro exige template ativo/nome e preço sugerido válidos; copia descrição exata inclusive vazia em String própria,product_idSome,id próprio e preço de lançamento explícito. Reusa validação/cálculo monetário via create_item privado extraído literalmente do factory livre; API e comportamento de create_free_item preservados. Zero/ambas modalidades/tiesUP/bruto largo e erros opacos;template inalterado em sucesso/erro. Seis unitários novos; domínio não relacionado, testes antigos, migrations/deps/exceções preservados. CIfinal/nota147/liberação pendentes. Preço de lançamento explícito é interpretação anunciada de sugerido, pergunta opcional sem resposta não atribuída a decisão humana; default de catálogo e edição na UI são consumidores futuros. Copiar campos próprios não prova snapshot operacional: edição real/persistência/INV-SAL005 dependem SAL008 e consumidores seguintes. UI/IPC,Rustglobal,instalador/instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. LLVM local somente domain/mod.rs inteiro;controle0 prova instrumentação,não causalidade isolada/percentual de função nova/global/CI/branches/statements. Meta ativa até critério oficial;locais encerrar antespausa/interrupção. Custo não registrado. Fonte:handoff SAL-007.


SAL-009 VERIFY E3: Sale representa id opaco16,statusOpen/Finalized eVec<SaleItem> próprios. Dois unitários verificam identidade/ordem/7campos de dois itens criados pelas factories reais (produto+livre),independência do template apósdrop,IDs0/MAX/ambosestados/coleçãovazia. Dois compile_fail privacidadeSale semDebug/Display. Sem métodos/algoritmo/derive/totalcache/campos futuros/IO/SQL/deps;domínio não relacionado/testes antigos/migrations/deps/exceções preservados. RED/GREEN2unit2docsnovos/418Rust28docs/gates/security/LLVMdomain95controle/P-REV PASS;0inferências;CI/nota149/liberação pendentes;passesfalse. Representação não aprova finalização vazia/transições nem prova validação/cálculo/pagamentos/pendência/persistência: SAL010..020 próprios. INV-SAL005/SAL008 exigeSAL012 e leitura após edição/desativação/reabertura;continua blocked/passesfalse. UI/IPC,Rustglobal,instalador/instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. Sale/SaleStatus têm zero linhas executáveis/sem percentual próprio;LLVMdomain inteiro comcontrole0 não écausalidadeisolada/global/CI/branches/statements. USER_JOURNEYS jornada3 já prevêajustepreçoSAL007,semnovoaceitehumano. Metaativa;custonão registrado. Fonte:handoff SAL-009.


SAL-009 done/E3/passestrue apósCI40265642848/SHAfdbcda5f3ffd20650be8b1e805e3e8600460903b,success23/0skip/7marcadores,418Rust+28docs/2unit2docsnovos. Sale representa id opaco16,statusOpen/Finalized eVec<SaleItem> próprios. Dois unitários verificam identidade/ordem/7campos de dois itens criados pelas factories reais (produto+livre),independência do template apósdrop,IDs0/MAX/ambosestados/coleçãovazia. Dois compile_fail privacidadeSale semDebug/Display. Sem métodos/algoritmo/derive/totalcache/campos futuros/IO/SQL/deps;domínio não relacionado/testes antigos/migrations/deps/exceções preservados. CIfinal/nota149/liberação pendentes. Representação não aprova finalização vazia/transições nem prova validação/cálculo/pagamentos/pendência/persistência: SAL010..020 próprios. INV-SAL005/SAL008 exigeSAL012 e leitura após edição/desativação/reabertura;continua blocked/passesfalse. UI/IPC,Rustglobal,instalador/instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. Sale/SaleStatus têm zero linhas executáveis/sem percentual próprio;LLVMdomain inteiro comcontrole0 não écausalidadeisolada/global/CI/branches/statements. USER_JOURNEYS jornada3 já prevêajustepreçoSAL007,semnovoaceitehumano. Metaativa;custonão registrado. Fonte:handoff SAL-009.


SAL-010 VERIFY E3: Migration nova010/main.salesSTRICT comidBLOB16NOTNULLPK eestadoTEXTNOTNULL limitado às chavesopen/finalized,semdefaults/FK/índiceextra/campos futuros. Rustsomente2constsde versão10/includeSQL;executor/domínio/migrations002..009/testes antigos/deps/exceções intactos. CincoSQLite reais: shapeexato/roundtripestadosIDs0MAX/reopen;INSERTUPDATEconstraintsNULL/types/IDs/estados/NUL/Unicode/duplicata semmutação;upgradev9semeadopreserva8tabelas/schema/ledger;rollbackapósDDLporfalhaledger+retry;colisãonome/versãorepetida. Controles SQL sóemcópias aceitamID15/paid e oráculos rejeitam com marcadores0/1,exit101. RED/GREEN5SQLnovos/2controles/423Rust28docs/gates/security/LLVMdatabase95controle/P-REV PASS;Laguna61.216sSQLusadosemcorreçãocomportamento/runtimeoff;CI/nota150/liberação pendentes;passesfalse. Schema/constraints/upgrade/rollback não provam CRUD de venda,persistência de itens,finalização/recebimentos/INV-SAL005/SAL008. SAL011/012 e demais consumidores próprios;UI/IPC,Rustglobal,instalador/instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. FixturesSQLde dados antigos são preparação de migration,não fluxo deprodução. DDLnãoaprova finalização vazia/transições. SQL/constantes sem percentual próprio;LLVMlocal somente database.rsinteiro,controle0 prova instrumentação,não causalidadeisolada/global/CI/branches/statements. Metaativa atéE2E;locais encerrar antespausa/interrupção;custonão registrado. Fonte:handoff SAL-010.


SAL-010 done/E3/passestrue apósCI40328240584/SHAcb14cd05a51b6cee689c93c215de70062a5b5870,success23/0skip/7marcadores,423Rust+28docs/5SQLnovos. Migration nova010/main.salesSTRICT comidBLOB16NOTNULLPK eestadoTEXTNOTNULL limitado às chavesopen/finalized,semdefaults/FK/índiceextra/campos futuros. Rustsomente2constsde versão10/includeSQL;executor/domínio/migrations002..009/testes antigos/deps/exceções intactos. CincoSQLite reais: shapeexato/roundtripestadosIDs0MAX/reopen;INSERTUPDATEconstraintsNULL/types/IDs/estados/NUL/Unicode/duplicata semmutação;upgradev9semeadopreserva8tabelas/schema/ledger;rollbackapósDDLporfalhaledger+retry;colisãonome/versãorepetida. Controles SQL sóemcópias aceitamID15/paid e oráculos rejeitam com marcadores0/1,exit101. CIfinal/nota150/liberação pendentes. Schema/constraints/upgrade/rollback não provam CRUD de venda,persistência de itens,finalização/recebimentos/INV-SAL005/SAL008. SAL011/012 e demais consumidores próprios;UI/IPC,Rustglobal,instalador/instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. FixturesSQLde dados antigos são preparação de migration,não fluxo deprodução. DDLnãoaprova finalização vazia/transições. SQL/constantes sem percentual próprio;LLVMlocal somente database.rsinteiro,controle0 prova instrumentação,não causalidadeisolada/global/CI/branches/statements. Metaativa atéE2E;locais encerrar antespausa/interrupção;custonão registrado. Fonte:handoff SAL-010.


SAL-011 VERIFY E3: Migration011/main.sale_itemsSTRICT9campos de snapshot+venda/posição,2FKsNOACTION eUNIQUEporvenda/posição,semcampos futuros. CHECKmonetárionomeado garanteq*p=d+t exato porquocientes/restos/CASEsemoverflow/REAL,brutoslargosválidospreservados. SeteSQLitevazio/shape/order/NULLSome/texto/roundtrip/reopen,matrizINSERTUPDATEinválida,oráculoindependentei1282413casos,FKeparentmetadata,upgrade9tabelas/schema,ledgerfaultrollbackretry,nomes/versõescolididos. ControlesSQLcópiasID15/remoçãoCHECKmonetário rejeitados101. Rustsó2consts/hook;executor/domínio/migrations002..010/testesantigos/deps/exceçõespreservados. GREEN7SQLnovos/i128oracle2413/2controles/430Rust28docs/gates/security/LLVMdatabase95controle/P-REV PASS. Laguna117.4s+105.581scorreçãolocal/primeirarejeiçãopreservada/runtimeoff;Cargo19aliasesobsoletosquarentenadoscanônicosretidos/retomadasóetapafalhapendentes. CI/nota151/liberaçãopendentes;passesfalse. Schema/SQLrawfixtures não provam fluxo deprodução de adicionar/ler item:SAL012/013 próprios;SAL008/INV-SAL005 segueblockedatéSAL012,critério apósreabertura VIAAPLICAÇÃOintacto. Finalização/recebimentos/UI/IPC/Rustglobal/instalador/instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. DDLnãoaprova workflow/transições/finalizaçãovazia. SQL/constssempercentualpróprio;LLVMsódatabase.rsinteiro,controle0instrumentação,nãocausalidadeisolada/global/CI/branches/statements. Metaativa;locais encerrar antespausa/interrupção;custonãoregistrado. Fonte:handoff SAL-011.


SAL-011 done/E3/passestrue apósCI40339661441/SHAee87c01937a309ef27018de81e62972f161f5739,success23/0skip/7marcadoresexatos,430Rust+28docs/7SQLnovos. Migration011/main.sale_itemsSTRICT9campos de snapshot+venda/posição,2FKsNOACTION eUNIQUEporvenda/posição,semcampos futuros. CHECKmonetárionomeado garanteq*p=d+t exato porquocientes/restos/CASEsemoverflow/REAL,brutoslargosválidospreservados. SeteSQLitevazio/shape/order/NULLSome/texto/roundtrip/reopen,matrizINSERTUPDATEinválida,oráculoindependentei1282413casos,FKeparentmetadata,upgrade9tabelas/schema,ledgerfaultrollbackretry,nomes/versõescolididos. ControlesSQLcópiasID15/remoçãoCHECKmonetário rejeitados101. Rustsó2consts/hook;executor/domínio/migrations002..010/testesantigos/deps/exceçõespreservados. CIfinal/nota151/liberaçãopendentes. Schema/SQLrawfixtures não provam fluxo deprodução de adicionar/ler item:SAL012/013 próprios;SAL008/INV-SAL005 segueblockedatéSAL012,critério apósreabertura VIAAPLICAÇÃOintacto. Finalização/recebimentos/UI/IPC/Rustglobal/instalador/instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. DDLnãoaprova workflow/transições/finalizaçãovazia. SQL/constssempercentualpróprio;LLVMsódatabase.rsinteiro,controle0instrumentação,nãocausalidadeisolada/global/CI/branches/statements. Metaativa;locais encerrar antespausa/interrupção;custonãoregistrado. Fonte:handoff SAL-011.


SAL-012 | VERIFY E3 | Adição via aplicação real: None inicia venda aberta com primeiro item; Some exige existente aberta. Produto atual validado e snapshot próprio/preço escolhido/desconto R$ ou%/ordem checked/header+item+audit na mesma transação IMMEDIATE. Leitura por ID em snapshot DEFERRED decodifica todos os itens ordenados sem catálogo vivo/resultado parcial. Dinheiro largo/zero/meio-centavo reutiliza domínio. Rejeições sem efeitos/reuso; falhas SQL/audit/rows/IDs/lock/query-only/collisions rollback e poisoner/fresh recovery. Sem alteração de caixa/catálogo/migrations/domínio/deps/testes anteriores. Gates locais aprovados;passesfalse até CI remoto exato. SAL-008/INV-SAL-005 histórico após edição/desativação/reabertura continua seu ID próprio; testes presentes provam produtor/leitor e snapshot sem catálogo, não fecham aquele critério. Finalização/recebimentos/UI/IPC/Rustglobal/releaseWindows instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. LLVM local somente linhas/funções dos2arquivos inteiros, sem global/CI/statements/branches. Meta ativa; encerrar locais antespausa/interrupção; custo não registrado. Fonte:handoff SAL-012.


SAL-012 | CI fonte aprovado / critérios satisfeitos | E3 | SHA1817edc0611f7fc54af61c5737ed12ece08e9b43,run40377784110/job121765450893/attempt1:success23/0skip/7marcadores exatos;445Rust257lib7bin181SQL+28docs/5unit10SQLnovos. Adição via aplicação real: None inicia venda aberta com primeiro item; Some exige existente aberta. Produto atual validado e snapshot próprio/preço escolhido/desconto R$ ou%/ordem checked/header+item+audit na mesma transação IMMEDIATE. Leitura por ID em snapshot DEFERRED decodifica todos os itens ordenados sem catálogo vivo/resultado parcial. Dinheiro largo/zero/meio-centavo reutiliza domínio. Rejeições sem efeitos/reuso; falhas SQL/audit/rows/IDs/lock/query-only/collisions rollback e poisoner/fresh recovery. Sem alteração de caixa/catálogo/migrations/domínio/deps/testes anteriores. Gates locais/review/coverage95controle/stagedscan PASS. DocsantesdoneE3passestrue;CIfinal/nota152/liberação pendentes. SAL-008/INV-SAL-005 histórico após edição/desativação/reabertura continua seu ID próprio; testes presentes provam produtor/leitor e snapshot sem catálogo, não fecham aquele critério. Finalização/recebimentos/UI/IPC/Rustglobal/releaseWindows instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. LLVM local somente linhas/funções dos2arquivos inteiros, sem global/CI/statements/branches. Meta ativa; encerrar locais antespausa/interrupção; custo não registrado. Fonte:handoff SAL-012.
