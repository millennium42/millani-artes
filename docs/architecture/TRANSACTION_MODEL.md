# Modelo transacional

Toda escrita financeira composta executa numa transação SQLite. Validações que dependem do estado atual, gravações e evento técnico pertencem à mesma unidade. Falha cancela a unidade inteira. A UI não persiste por etapas.

Idempotência e prevenção de duplicação serão especificadas por caso de uso; não são presumidas por botão, relógio ou interface.

## Fronteira da unidade — ARC-004

O caso de uso na aplicação possui a unidade de escrita, do início ao resultado. Ele recebe a intenção da UI, valida a entrada e coordena os repositórios; a infraestrutura fornece a transação SQLite. Domínio permanece puro. Repositórios usados pela operação compartilham a mesma conexão/contexto transacional e não fazem commits independentes. Uma operação composta não pode ser dividida entre chamadas de persistência da UI.

Validações puras de formato/tipo podem ocorrer antes de abrir a unidade. Validações que leem estado atual — saldo devido, quantidade restante, obrigação já paga ou referências ativas — ocorrem dentro dela, junto das gravações e do audit_event técnico. Uma validação feita antes da abertura não substitui a verificação do estado que autoriza a escrita. DB-011 implementa `execute_write_transaction` com `BEGIN IMMEDIATE`: consome `Connection`, empresta `&Transaction` e devolve `(Connection, T)` somente após commit confirmado. Erros descartam o contexto; o callback confiável não controla transação nem produz efeitos externos. Nenhuma recuperação global é prometida, e não há retry automático.

O caso de uso devolve sucesso e seu resultado somente após commit confirmado. Nenhuma escrita financeira, etapa intermediária ou audit_event pode ser confirmada por fora para contornar uma falha. Diagnóstico sanitizado é distinto do audit_event financeiro: falha do logger não altera resultado nem rollback, e falha do audit_event cancela também as escritas financeiras.

| Situação | Resultado exigido da fronteira | Prova futura |
|---|---|---|
| Abertura falha | Não executar validações dependentes do banco, gravações ou auditoria; retornar falha | DB-011/012 |
| Validação do estado rejeita a operação | Cancelar a unidade, sem escrita ou audit_event confirmado | DB-012 e consumidor |
| Uma gravação falha após outra ter executado | Cancelar todas as gravações da unidade; nenhuma parte confirmada | DB-012 e consumidor |
| audit_event falha | Cancelar também as gravações financeiras | DB-014 |
| Todas as etapas e commit têm sucesso | Confirmar a unidade inteira e então devolver sucesso | DB-011/014 e consumidor |
| Commit falha | Não devolver sucesso; cancelar se a unidade ainda estiver ativa e verificar o estado conforme o adapter. Não repetir automaticamente a intenção | DB-011/012 |
| Cancelamento falha ou estado final é incerto | Não alegar restauração; impedir reutilização do contexto incerto até recuperação verificada. Não repetir a operação às cegas | DB-011/012 e recuperação |
| Pânico antes de commit confirmado | Não converter em sucesso; especificar cancelamento no unwind e recuperação após abort, verificando o comportamento real sem presumir execução de Drop | DB-011/012 |

Erros de domínio e de infraestrutura permanecem distintos; [Result/DomainError](../../src-tauri/src/domain/mod.rs) não são conversão automática de erro SQLite. A [política de logs](../security/LOGGING_POLICY.md) proíbe publicar SQL, parâmetros, dados financeiros, payloads ou erros externos brutos. Este documento não adiciona mapper, sink ou novo código de erro.

## Limites e verificação

[INV-FIN-006/007](../product/INVARIANTS.md) continuam sem prova de produto. DB-011 define a transação real; DB-012 prova rollback genérico; DB-013 cria audit_event; DB-014 prova auditoria na mesma unidade. Os consumidores precisam exercitar seus próprios estados e pontos de falha: por exemplo FIN-021/023, REC-012/014, SAL-020/022, RCV-006/011 e RET-010/013. Reutilizar a [matriz de invariantes](../traceability/INVARIANT_TEST_MATRIX.md) e a [estratégia de testes](../quality/TEST_STRATEGY.md), sem tratar mock como prova SQLite.

Transação não garante idempotência, não é retry automático e não protege efeito externo ou troca de arquivo. Backup/restore segue seu protocolo de validação, staging, preservação e recuperação; rollback SQL não restaura arquivo substituído. Nenhum evento externo é autorizado por este contrato.

[Spec](../../planning/specs/DB-011.md) e [handoff](../../planning/handoffs/DB-011.md) registram a API e os limites E3. Root executou os quatro testes de harness e a CI de entrega no SHA `803ca4cf85ac56b135e9ce10509c7d97b1a8f6aa`; review independente E2 sem achados impeditivos. Os testes exercitam begin/commit, visibilidade, BUSY e sanitização; não provam rollback genérico composto, pânico/abort, auditoria ou invariantes financeiras. DB-012 continua responsável pelo rollback genérico; DB-013/014 por audit_event e mesma unidade. Fechamento DB-011: CI final no SHA `0c0660cf5abe9b1e51a5067c49c11e7364069764`, nota72 publicada e reserva liberada.

DB-012 adicionou cinco provas SQLite físicas: rejeição sem escrita, erro de constraint após DDL/gravações, COMMIT rejeitado por FK deferred, panic unwind e recuperação após exit86 com hot journal DELETE. Root confirmou fmt/clippy, Rust44/44, release31/31 e Cargo/OSV; harness final SHA-256 `7bf3b01606b6a1d99ced6dc38280f75f1b7c8de6aec859847392a0d4254b32f4`. CI entrega `38400843616` passou 23/23; remote harness31/31, cinco casos PASS. Review E2 sem P0/P1/P2, limitada ao diff. Isso prova infraestrutura nos cenários exercitados, não INV-FIN-006/007 em consumidores nem power loss/WAL/recovery global. Este registro antecede o CI documental final; a nota Git no SHA final confirma a liberação.

SAL-022 acrescenta um caso E3 do consumidor: finalização file-backed de venda parcial com FK de conta deferred, falha no commit, erro sanitizado e snapshot completo preservado após reopen. Não amplia a prova para outros consumidores nem altera o estado global E0/parcial de INV-FIN-006/007; CI e SHA final constam no recibo Git.


DB-013 implementa append de evento na transação emprestada; testes locais e CI de entrega no SHA final passaram, incluindo rejeição de inserção ignorada. Não prova que escrita financeira e evento sejam atômicos juntos; DB-014 permanece responsável. CI documental e nota74 pendentes; ver [spec](../../planning/specs/DB-013.md) e [handoff](../../planning/handoffs/DB-013.md).


DB-014 | E3 e CI de entrega `38438041786` no SHA `af1c5aabbefa785b8d26b7351cddf538ad68769b` confirmam cinco testes SQLite físicos; auditoria e operação são atômicas nos cenários exercitados. Produto permanece E0. Ver [handoff](../../planning/handoffs/DB-014.md).

FIN-008 primary E3 adiciona um consumidor concreto: criação de Account e `audit_event` usam a mesma `execute_write_transaction`; os testes físicos exercitam sucesso/reopen e falhas nos inserts sem persistência parcial, inclusive contexto descartado após erro. Isso não fecha INV-FIN-006/007 globalmente nem prova projeção financeira/INV-FIN-001; consumers e estados adicionais seguem necessários. Gitleaks staged, validators e source CI pendentes; detalhes no [handoff FIN-008](../../planning/handoffs/FIN-008.md).
## FIN-008 — evidência do executor (E3)
FIN-008 testa gravação atômica de account e audit_event, incluindo reopen e rollback. Source CI 38765481430 no SHA eea53ff05f2df34c5ff8e9aa759b70dc0a25ab57; não generaliza a outras operações.
## FIN-017 — Entry e audit event (contrato E2, execução pendente)
O caso de uso valida `Entry` antes de solicitar IDs (movimento e evento) e o instante UTC em ms; entrada inválida não chama IDs, relógio ou store. O adapter exige kind Entry e valida novamente antes de tomar a conexão. Em seguida, `execute_write_transaction` usa `BEGIN IMMEDIATE` para inserir com parâmetros em `main.movements` e anexar `audit_event` (`movement_entry_create` / `movement`, ID da movimentação) na mesma transação. Exige uma linha afetada; erro, zero linhas ou erro de auditoria aborta e deixa o adapter sem conexão reutilizável. Sucesso devolve a conexão ao adapter e o caso de uso só então retorna `Movement`. Migrations 002–005 permanecem imutáveis; sem IPC/UI. Estes são critérios, ainda sem prova GREEN.


FIN-017 | E3 local:6unit+3SQLite/4falhas,criaçãoEntry+audit atômicas/reopen/rollback/poison/inputinválido sem efeitos;Rust121+8docs/debug+release/fmt/clippy/TS/securityE4 PASS;coveragecrítica app43/43linhas3/3fn eadapter51/51linhas5/5fn,controle0,rejeita95. Review independente concluído;CI source/final/liberação pendentes,in_progress/passesfalse. Sem jornadaUIIPC/globalRust/setup/E2E;apósFIN017METApausará,nenhum próximoID. Ver spec/handoffFIN017;custonão registrado.


FIN-017 | done/E3: source5a09ca2ec7117be7e39605bfe25848e7fc39acaf CI39090756071/job117566803340/attempt1success23/0skip/7marcadores exatos;121testesRust+8doctests. Entry+audit atômicos/6unit+3SQLite4falhas,critical app43/43linhas3/3fn+adapter51/51linhas5/5fn100%,controle0. Gateslocais/review/validators/stagedscan aprovados;docs antespassestrue. CI documental/nota95/liberação pendentes;WTarquivada,evidênciapreservada,locaisencerrados. DepoispausarMETA por decisão humana,nenhumpróximoID. ProdutoUIIPC/receitaFIN005/globalRust/setup.exe/E2E pendentes;custonão registrado.


REC-014 VERIFY E3:1SQLnovo/64cenários físicos8combinações×8falhasinclusiveCOMMITdeferred/snapshot+reopen/projeções reais preservadas/poison+freshválido;2mutantes parciais rejeitados/GREEN7SQL/Rust315+18docs/gates/security/reviewPASS. Produção/7migrations intactas;coverageREC012histórica referenciada. CIs/nota127/liberação pendentes;REC015/UIIPC/globalRust/instalador/E2Ependentes;Metaativa;fonte:handoffREC-014.


REC-014 done/E3/passestrue apósCI39255062132/SHA921465015075a33f16368761f39bcf53c1a56c1fsuccess23/0skip/7marcadores,Rust315+18docs/1SQLnovo. Rollbackpagamento64cenários/snapshot+reopen/leitores+projeções/poison+freshválido/FKdeferredCOMMIT/2mutantes/review/gatesPASS;produção e7migrations intactas,coveragehistórica referenciada;CI final/nota127/liberação pendentes;REC-015 ainda exige provas próprias de concorrência/repetição;sem retry automático/sucesso idempotente prometido. Não prova powerloss/WAL/panic-abort/falha física deROLLBACK/recuperação global. UI/IPC,cobertura Rust global atual e instalador Windows/instalação/atualização/backup-restauração/hash/assinatura/E2E oficial pendentes. Política mensal último dia/fimdiaBrasília aceita;implementaçãoUTC-3fixa sem provaIANA/DSThistórica;Metaativa;fonte:handoffREC-014.


REC-015 VERIFY E3:2SQLnovos/8seriais+8corridas/Paid0UUID/staleUUIDnovoereusadosemsegundaExit/1vencedor+loserpoison/outramêspagável/PaidFK-audit-payload8col/reopen/histórico/projeções exatos;3REDs atuais verificados/GREEN9SQL/P2corrigidopeloroot/Rust317+18docs/gates/security/reviewPASS. Parsercorrigidosemrerunválido,rawpreservado;skillsadaptadas aplicadas. Produção/7migrations intactas,coveragehistórica referenciada;CIs/nota128/liberação pendentes;UIIPC/globalRust/instalador/E2Ependentes;Metaativa;fonte:handoffREC-015.


REC-015 done/E3/passestrue apósCI39261345263/SHAdfc4c405366cce24ee28be8db8bc147d1f3d4652success23/0skip/7marcadores,Rust317+18docs/2SQLnovos. Repetição8seriais+8corridas/Paid0UUID/staleguard/1vencedor/loserpoison/outramêspagável/payload8col-audit-PaidFK/reopen/históricoprojeções/3REDs/gatesPASS;P2darevisãocorrigidopeloroot/0pendentes;parserreclassificadosemrerunválido eoriginais preservados,skillsadaptadasaplicadas;produção e7migrations intactas,coveragehistórica referenciada;CI final/nota128/liberação pendentes;Repetição retorna erro,sem sucesso idempotente anterior/chave deintenção/retryautomático. Perdedor concorrente pode falhar noBEGINIMMEDIATE ou guarda apóscommit;guarda éprovada independentemente na sequência serial comUUIDsnovos. Não prova vencedor fixo/processos separados/distribuição/powerloss/WAL/panic-abort/falha físicaROLLBACK/recuperação global. UI/IPC,coberturaRustglobal e instaladorWindows/instalação/atualização/backuprestauração/hashassinatura/E2E oficial pendentes. Política mensal último dia/fimdiaBrasília aceita;implementaçãoUTC-3fixa semprovaIANA/DSThistórica;Metaativa;fonte:handoffREC-015.

## RCV-006 — registro do recibo dentro da transação (E3 focal concluído)

`persist_receivable_receipt_in_transaction` recebe uma `Transaction` emprestada, lê principal e soma dos recibos dentro da unidade, aplica `Receivable::remaining_due_cents` e grava `main.receipts` com `audit_event` estático na mesma transação. O helper não controla commit nem `Movement`. Sete testes SQLite focais cobrem a migration, upgrade, campos, devido, rollback de auditoria e rejeição quando o INSERT é ignorado. Rust 1.99/LLVM 23.1.1 mediu o helper com 68/68 linhas e 65/70 regiões; o controle sem os testes RCV-006 teve 0/68 linhas. A cobertura de branches está indisponível neste toolchain stable e não se infere cobertura global. CI fonte e CI push main passaram nos SHAs registrados no handoff e recibo de evidência. RCV-007 agora coordena a entrada financeira na unidade transacional; o helper RCV-006 continua sem provar saldo projetado, aumento observado de conta, INV-FIN-004 ou a operação composta completa.

## RCV-007 — recebimento e entrada na mesma transação (E3 liberado; saldo projetado e rollback consumidor permanecem separados)

O caso de uso prepara um `MovementKind::Entry` com a mesma conta, valor em centavos e contexto de `ReceivableReceipt`. O adapter verifica essa paridade antes de consumir a conexão e executa `persist_receivable_receipt_in_transaction` e `persist_entry_in_transaction` em uma única `execute_write_transaction` (`BEGIN IMMEDIATE`), mantendo os dois `audit_event` estáticos. A resposta de sucesso só é devolvida depois do commit.

Cinco testes SQLite focais exercitam persistência válida com reopen, divergências em chamadas diretas ao adapter, valor zero, falha no INSERT do recibo e falha no segundo evento de auditoria; após falhas, o estado reaberto não contém escrita parcial. Coverage local LLVM 23.1.1: caso de uso 55/55 linhas, 3/3 funções, 65/66 regiões; adapter composto 27/27 linhas, 31/32 regiões; closure 21/21 linhas, 22/22 regiões; constructor 5/5 linhas, 3/3 regiões. Controles sem testes do item: zero linhas nos alvos; branches indisponíveis e sem alegação global.

CI source run 41121374610/job 124288115793 passou no head exato a0b39d96eeec0fc8523284cb6cb9462c0ec5cdf9; CI push main run 41123334768/job 124295322775 passou no SHA exato fa17c836e64da7088b555efa25e29a3fdd638a38. O fechamento documental, seu CI exato e o recibo refs/notes/evidence ficam ligados ao SHA final do item; a nota preserva 172 mappings anteriores e adiciona somente RCV-007.

Scans do CI main: Cargo Audit 422 dependências, zero vulnerabilidades e dois avisos aprovados; OSV 542 pacotes, três findings excetuados e zero fora; Gitleaks 385 commits/zero; npm 120 dependências/zero; SAST 45 casos. As exceções não foram alteradas e expiram em 2026-11-06. Os runs anteriores 41118326841 e 41120643470 permanecem documentados como timeout e falha de fixture antes do Cargo Audit real, respectivamente; o run source 41121374610 passou após correção do fixture.

A prova não demonstra a projeção do saldo por conta (RCV-008), o rollback dedicado do consumidor (RCV-011), INV-FIN-004 completa, fluxo de produção, UI/IPC, smoke, E2E, cobertura global ou recuperação após falha física.

## RCV-011 — commit falho no consumidor de recebimento (E3 candidato)

O teste `rcv011_deferred_commit_failure_rolls_back_receipt_entry_and_events` chama o consumidor e adapter reais. Um trigger temporário no `audit_event` final adiciona uma FK deferred sintética inválida; o erro só aparece no COMMIT, após recibo, primeira auditoria, Entry e auditoria final. Ao reabrir o arquivo, todos os snapshots observados permanecem iguais; schema v13, autocommit, `integrity_check` e `foreign_key_check` permanecem válidos. O RED separou temporariamente recibo/auditoria e entrada/auditoria em commits distintos; a prova detectou as linhas parciais, e o arquivo de produção foi restaurado ao blob da base. Isso demonstra uma falha de COMMIT para este consumidor no SQLite file-backed; não prova rollback físico, recuperação WAL/power loss, outros consumidores ou invariantes globais. Sem mudança de produção/schema/migration/dependência. Candidato E3; P-REV final read-only nos 11 paths staged confirmou o índice INV-FIN-006 com RCV-011 e não encontrou P0/P1/P2/P3; o revisor não executou checks; CI source/final e nota de liberação pendentes.
