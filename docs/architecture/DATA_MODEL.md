# Modelo de dados inicial

Entidades previstas: `account`, `category`, `financial_movement`, `recurrence`, `product`, `sale`, `sale_item`, `customer`, `receivable`, `receipt`, `return`, `refund`, `audit_event`, `schema_migration` e metadados de backup.

Dinheiro será inteiro em centavos (ADR-013); valores agregados são consultas/projeções. Chaves, constraints, índices e migrations concretos ficam para `DB-*`, depois de uma spec e testes de banco vazio/upgrade.


DB-013 implementa `audit_event` técnico com IDs opacos de 16 bytes, instante Unix ms UTC e tags estáticas de operação/entidade. Não armazena payload financeiro nem prova invariantes financeiras; detalhes e limites em [spec](../../planning/specs/DB-013.md).

FIN-001 executor done/E3 após CI `38444911363` no SHA `31af12adcf8bebe617539d9e5dde286a64674deb`; `Account` contém somente `id: [u8; 16]` e `name: String`, com teste de preservação exata. Não define validação, tipo, saldo inicial, schema SQLite ou persistência. CI final documental e liberação pendentes; invariantes financeiras E0. [Spec e handoff](../../planning/specs/FIN-001.md).

FIN-003 VERIFY local E3: Account agora inclui account_type: AccountType com Cash, BankPix, Card e Other; representação pública em memória sem derive/default/tag numérica/conversor. Tipo não é contexto Casa/Fábrica; sem persistência/schema/serde definido. Hashes e limites no [handoff](../../planning/handoffs/FIN-003.md); CI/Gitleaks staged/validators finais pendentes.

CI de entrega FIN-003 (02c86f9): AccountType e campo compilaram e os testes de tipo/name passaram. Contrato continua apenas representação em memória; sem schema, serde, persistência ou semântica de saldo. FIN-004 futuro trata saldo inicial; contexto permanece separado.

FIN-004 VERIFY local E3: Account armazena initial_balance_cents como i64, centavos sem conversão/cálculo. Valores negativos/zero são representáveis, não uma regra comercial aprovada. Sem schema/persistência; FIN-005 avalia classificação do saldo inicial.


FIN-004 — medição de qualidade: campo initial_balance_cents permanece i64 em memória, sem cálculo ou classificação financeira. A medição Rust global E3 alcançou 301/312 linhas e 30/33 funções; não prova persistência, consumidores financeiros ou setup.

FIN-004 CI final de entrega no SHA 9e2068b passou; initial_balance_cents continua apenas valor i64 em memória. Sem consumidor real de receita, persistência ou cálculo demonstrado; FIN-005 permanece sem prova.

CAT-001 integração: Category representa somente id [u8;16], name String e is_active bool. O booleano não demonstra workflow de ativação/desativação, histórico, schema ou persistência. Integração primária ainda aguarda suíte e CI.

CAT-001 integração primária VERIFY: Category mantém somente ID, nome e bool; 61 testes+4 compile_fail doctests passaram, sem provar fluxo de ativação, persistência, histórico ou schema.

CAT-001 CI final c4def86 confirma Category como representação em memória (ID, nome, bool) e testes associados; não prova workflow, schema, persistência ou história de categorias. MVP permanece parcial.

FIN-010 representa contexto somente como enum Home/Factory no domínio; não decide campo em Account, serialização ou persistência. Evidência local standalone não confirma integração/testes primários.

FIN-010 local primário: Context Home/Factory compilou e suíte62+4 doctests passou. Enum continua somente classificação; sem consumer, Account.context, conversão externa ou persistência.

FIN-010 E3: Context Home/Factory permanece enum classificatório puro. CI run38512646302 passou; não prova consumer, persistência ou conversão financeira.


## FIN-006 — persistência inicial de contas (E3 local)
A migration `003-accounts.sql` registra quatro campos: ID opaco BLOB16, nome TEXT, chave técnica `account_type` (`cash`, `bank_pix`, `card`, `other`) e saldo inicial em centavos INTEGER. Testes físicos confirmaram round-trip, inclusive nome Unicode com prefixo NUL e extremos i64, vazio sem seed e upgrade com audit preservado; 43 PASS em SQLite 3.53.2. As chaves não são ordinais e o inteiro não aprova regra financeira. Sem consumer/startup/UI demonstrado; FIN-005 receita continua sem prova.

FIN-006 coverage focal E3 confirma gates ADR-019 na produção database.rs: 276/280 linhas e 25/25 funções; statements não medidos, sem vetor/global claim. O teste físico não demonstra app consumer, startup ou receita.


FIN-006 CI final 38570028370 passou testes da migration e do executor. O registro inicial de conta não demonstra consumo pela aplicação, startup, UI ou regra de receita FIN-005.

CAT-002 executor done/E3/passes:true: CI 38616439285 no SHA `3a94855181576ed354a73831dca3e1964641e79d` passou 67 testes +4 compile-fail doctests; 1 WebView ignored; guard/category tests PASS. `validate_name` permanece validação explícita em memória. Não prova consumer, persistência, ativação, FIN-005 ou setup; publicação final pendente.

## FIN-011 — conversão interna de contexto (E3 focal WT)
`Context` continua enum classificatório `Home`/`Factory`, separado de conta (ADR-008). Conversão interna `TryFrom<&str>` aceita exatamente `home`/`factory`; não estabelece storage key, contrato externo, contexto em Account ou persistência. Executor FIN-011 `done`/E3: primary source CI 38621842911 no SHA `809626e89dabbcce158fb466f63401c0a7b02c99` passou 70 testes+4 doctests/1 WebView ignored; três testes Context/junction passaram. Sem inferir consumo financeiro. CI documental e liberação ACTIVE24 pendentes.

FIN-007 primary E3 exercitou constraints da migration 003 imutável: 16 classes de linha inválida e ID duplicado; 45 SQL tests PASS no primary. Nenhuma mudança de schema ou regra de produto; source CI ainda pendente.

FIN-007 source CI `38628675813` no SHA `a58ad20b7664db5f7b5fb85a3cb7b0b341a46978` passou; registro acima descreve os testes do executor, sem inferir regra/consumer. Publicação documental permanece pendente.

## FIN-008 — gravação de conta e auditoria na aplicação (E3 primary)
O caso de uso valida nome, injeta IDs/relógio e usa adapter SQLite parametrizado para persistir os quatro campos de Account junto de um audit_event na mesma transação. Cargo executou 48 testes SQL; casos FIN-008 cobrem reopen e rollback de account/audit. Nenhuma migration/schema foi alterada. Isso prova o caminho focal exercitado, não startup, UI/IPC, projeção de saldo/receita, INV-FIN-001 ou consumer posterior; Gitleaks staged, validators e CI ainda pendentes.
## FIN-008 — evidência do executor (E3)
FIN-008 confirma a persistência dos quatro campos Account sem inferir receita do saldo inicial. Source CI 38765481430 no SHA eea53ff05f2df34c5ff8e9aa759b70dc0a25ab57; INV-FIN-001 segue sem prova.

## CAT-003 — integracao primaria 16/10/2026

Tabela main.categories STRICT v4 com idBLOB16/nameTEXTnaovazio NUL-safe/is_activeINTEGER0|1; constantes registradas e executor/migrations001-003 preservados. Fonte: planning/handoffs/CAT-003.md e planning/specs/CAT-003.md, SHA7573c65899416cf180bb25545f6cfae6335bb00e; E3 local, CI de entrega em execucao, item ainda aberto.


CAT-003 executor done/E3/passestrue apos CI38907719490/job116995418974/attempt1/SHA7573c65899416cf180bb25545f6cfae6335bb00e SUCCESS23/0. Evidencia e limites em planning/handoffs/CAT-003.md;CI documental final/nota87/liberacao pendentes. FIN-012/UI-001 continuam abertos.


## FIN-012 — integração primária em 16/10/2026

MovementKind contém a conta em Entry/Exit e dois endpoints em Transfer; Movement mantém identidade opaca, kind, amount_cents i64 e Context. Contrato de metadados sem schema ou regra financeira. Fontes e evidência: planning/specs/FIN-012.md e planning/handoffs/FIN-012.md; source 7573c65, main 7cad038, produção idêntica. Item ainda aberto até fechamento documental.

FIN-012 executor done/E3/passes true após os checks e CI do código integrado, com fonte em planning/handoffs/FIN-012.md. CI documental final, nota88 e liberação pendentes. UI-001 aguarda liberação. Sem comprovação de consumidor, regras financeiras, coverage global ou instalador.


## UI-001 — verificação primária em 16/10/2026

Fonte: planning/handoffs/UI-001.md, source7573c65/main59a3789, E3 local.

```json
{
  "escopo": "shell estática sem dados/operação financeira, persistência ou novo IPC"
}
```


UI-001 executor concluído: E3 de execução e inspeção nativa, checks do código idêntico e CI source38907719490 SUCCESS23/0; review integrado/visual E2 sem P0/P1/P2. Documentos e matrizes atualizados antes de passes true. Reserva retida até CI documental final no novo SHA, nota89/preservação88notas/Gitlimpo/WTarquivada e liberação do root. Código de saída nativo não registrado(null); sem claim exit0, WCAG numérico, mínimo de produto, jornada financeira, Rustglobal ou setup. Próximo após liberação: reconciliar elegibilidade FIN-005 no roadmap.


## FIN-009 — leitura de contas, E3 local em 17/10/2026

O modelo de dados mantém estrutura existente sem alterações de schema. A tabela accounts mantém os campos id, name, account_type, initial_balance_cents. Nenhuma nova coluna adicionada. Não há migration nova; persistência de criação permanece FIN-008.


FIN-009 executor done/E3/passestrue após CI39009285553/job117328105513/attempt1/SHA83e682303354c000d2319f512f8321882901aa35 SUCCESS23/0,97ordinários+8doctestsPASS/1ignored. Evidência local global459/470linhas46/49funções ecríticos95PASS não é coverage CI; statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4 atualizados; CI documental final/nota90/preservação89/liberação ainda pendentes. Sem UI/IPC/receita/setup; fonte planning/handoffs/FIN-009.md.


## FIN-013 — validação pura de entrada, E3 local em 17/10/2026

Movement e MovementKind conservam seus campos e construção bruta, inclusive valores negativos usados em fixtures de FIN-012. A validação explícita validate_entry(&self) aceita apenas Entry com amount_cents > 0. Não altera schema, saldo, histórico, contexto ou conta; não executa migrations.


FIN-013 executor done/E3/passestrue após CI39021912847/job117365252558/attempt1/SHAa3dee6febce169d4dbac3779ce6a67c998e506ab SUCCESS23/0,102ordinários+8doctestsPASS/1ignored. Crítico domainmod24/24linhas5/5funções/controle rejeita95 é E3 local,não coverageCI/globalatual;statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4/validators atualizados;CIfinaldocumental/nota91/preservação90/liberação pendentes. SemUIIPC/persistência/receita/setup. Fonte planning/handoffs/FIN-013.md.


## FIN-014 — validação pura de saída, E3 local em 17/10/2026

Movement e seus campos permanecem brutos, sem mudança de schema ou construção automática. validate_exit(&self) valida explicitamente Exit com amount_cents > 0. Não altera conta/contexto/saldo/histórico nem executa migrations. validate_entry de FIN-013 e os traits existentes permanecem iguais.


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
`main.movements` da migration 005 já existente armazena Entry com `kind='entry'`, ID opaco, `amount_cents`, `context` (`home`/`factory`) e `account_id`; endpoints de transferência ficam nulos conforme o schema. FIN-017 não altera migrations 002–005 nem infere saldo/projeção financeira. Contrato e gates pendentes: [spec](../../planning/specs/FIN-017.md) e [handoff](../../planning/handoffs/FIN-017.md).


FIN-017 | E3 local:6unit+3SQLite/4falhas,criaçãoEntry+audit atômicas/reopen/rollback/poison/inputinválido sem efeitos;Rust121+8docs/debug+release/fmt/clippy/TS/securityE4 PASS;coveragecrítica app43/43linhas3/3fn eadapter51/51linhas5/5fn,controle0,rejeita95. Review independente concluído;CI source/final/liberação pendentes,in_progress/passesfalse. Sem jornadaUIIPC/globalRust/setup/E2E;apósFIN017METApausará,nenhum próximoID. Ver spec/handoffFIN017;custonão registrado.


FIN-017 | done/E3: source5a09ca2ec7117be7e39605bfe25848e7fc39acaf CI39090756071/job117566803340/attempt1success23/0skip/7marcadores exatos;121testesRust+8doctests. Entry+audit atômicos/6unit+3SQLite4falhas,critical app43/43linhas3/3fn+adapter51/51linhas5/5fn100%,controle0. Gateslocais/review/validators/stagedscan aprovados;docs antespassestrue. CI documental/nota95/liberação pendentes;WTarquivada,evidênciapreservada,locaisencerrados. DepoispausarMETA por decisão humana,nenhumpróximoID. ProdutoUIIPC/receitaFIN005/globalRust/setup.exe/E2E pendentes;custonão registrado.


CAT-007: movements.category_id BLOB nullable/NULLoublob16/FKcategories.id NOACTION adicionada por migration6; preserva v5 e referência a categoria renomeada/inativa. Domínio e APIs permanecem anteriores; atribuição testada por fixture SQL, UIIPC/política inativa não comprovadas. E3 física; handoffCAT007.


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


REC-012 done/E3/passestrue apósCI39248473269/SHAb4ee66f594ef2c723df3f595c7d14aa35d82cee3success23/0skip/7marcadores,Rust313+18docs/6unit+5SQLnovos. PagarrecorrênciaExit+Paidlink+auditTx/guardas/rollbackpoison/replaybásico/gates/LLVM/review/validators/stagedscanPASS;correçõesexecutáveisROOTregistradas;CI final/nota125/liberação pendentes;REC-013..015 exigem provas próprias desaídaúnica/rollback/concorrência/repetição,idempotência de sucesso nãoprometida;UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;calendárioaprovadoúltimodia/fimdiaBrasília registrado,implementaçãoUTC-3fixosemprovaIANA/DSThistórico;MetaE2Eativa;fonte:handoff REC-012.


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


PRD-006 E3 local: edição altera somente nome, descrição e preço sugerido; preserva id/ativo, inclusive inativo. Zero aceito, negativos rejeitados; Product e um audit são atômicos, ausência não escreve. Sem migration nova: SQL002..008 e versão1 vazia preservadas. Snapshot real de venda/INV-SAL-005 depende de SAL-008. Item não liberado; segurança/CI pendentes. Fonte: planning/handoffs/PRD-006.md.


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
