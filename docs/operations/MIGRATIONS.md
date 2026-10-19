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

FIN-007 primary E3 testa constraints de `003-accounts.sql` sem alterar a migration ou o executor versionado. Harness SQL45 PASS, incluindo as duas novas provas de rejeição/preservação e reabertura. Source CI permanece pendente; não há migration nova.

Source CI `38628675813` no SHA `a58ad20b7664db5f7b5fb85a3cb7b0b341a46978` passou release72+4 doctests/1 ignored, usando migrations reais; versão SQLite não registrada no log. Sem migration nova ou mudança do executor.


## CAT-003 — integracao primaria 16/10/2026

v4 categories exercitada em arquivo fisico vazio->v4 e upgradev3 populado,com constraints/ledger/autocommit/integridade/reopen;accounts e audit_event preservados. Migrations001-003/executor inalterados;nenhum banco da usuaria usado. Fonte: planning/handoffs/CAT-003.md e planning/specs/CAT-003.md, SHA7573c65899416cf180bb25545f6cfae6335bb00e; E3 local, CI de entrega em execucao, item ainda aberto.


CAT-003 executor done/E3/passestrue apos CI38907719490/job116995418974/attempt1/SHA7573c65899416cf180bb25545f6cfae6335bb00e SUCCESS23/0. Evidencia e limites em planning/handoffs/CAT-003.md;CI documental final/nota87/liberacao pendentes. FIN-012/UI-001 continuam abertos.


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


CAT-007 migration006-movement-categories.sql/versão6 aditiva ALTER: nullable category_id/FKNOACTION,sem seed/default/índice extra. Migrations1..5 hash idêntico; testes físicos vazio1..6 eupgradev5 provam preservação/constraints/FKs/reopen. Sem down migration: rollback de falha pelo executor transacional existente. E3 local; CI pendente; handoffCAT007.


CAT-007 done/E3/passestrue após CI39131643684/SHA143190b374d186da75f2a44bdc85cb0bfad58219success23/0skip e153Rust+8docs,2SQLite novos. Migration6/FKhistórica física/reopen/rename/deactivate e gates/review/validators/stagedscan PASS. CI documental final/nota99/liberação pendentes;MetaE2Eativa. FixtureSQL não comprova atribuiçãoUIIPC/globalRust/setup/E2E. Fonte:handoffCAT007.


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
