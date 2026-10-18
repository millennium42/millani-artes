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

DB-012 adicionou cinco provas SQLite físicas: rejeição sem escrita, erro de constraint após DDL/gravações, COMMIT rejeitado por FK deferred, panic unwind e recuperação após exit86 com hot journal DELETE. Root confirmou fmt/clippy, Rust44/44, release31/31 e Cargo/OSV; harness final SHA-256 `7bf3b01606b6a1d99ced6dc38280f75f1b7c8de6aec859847392a0d4254b32f4`. CI entrega `38400843616` passou 23/23; remote harness31/31, cinco casos PASS. Review E2 sem P0/P1/P2, limitada ao diff. Isso prova infraestrutura nos cenários exercitados, não INV-FIN-006/007 em consumidores nem power loss/WAL/recovery global. CI documental final e nota73 pendentes.


DB-013 implementa append de evento na transação emprestada; testes locais e CI de entrega no SHA final passaram, incluindo rejeição de inserção ignorada. Não prova que escrita financeira e evento sejam atômicos juntos; DB-014 permanece responsável. CI documental e nota74 pendentes; ver [spec](../../planning/specs/DB-013.md) e [handoff](../../planning/handoffs/DB-013.md).


DB-014 | E3 e CI de entrega `38438041786` no SHA `af1c5aabbefa785b8d26b7351cddf538ad68769b` confirmam cinco testes SQLite físicos; auditoria e operação são atômicas nos cenários exercitados. Produto permanece E0. Ver [handoff](../../planning/handoffs/DB-014.md).

FIN-008 primary E3 adiciona um consumidor concreto: criação de Account e `audit_event` usam a mesma `execute_write_transaction`; os testes físicos exercitam sucesso/reopen e falhas nos inserts sem persistência parcial, inclusive contexto descartado após erro. Isso não fecha INV-FIN-006/007 globalmente nem prova projeção financeira/INV-FIN-001; consumers e estados adicionais seguem necessários. Gitleaks staged, validators e source CI pendentes; detalhes no [handoff FIN-008](../../planning/handoffs/FIN-008.md).
## FIN-008 — evidência do executor (E3)
FIN-008 testa gravação atômica de account e audit_event, incluindo reopen e rollback. Source CI 38765481430 no SHA eea53ff05f2df34c5ff8e9aa759b70dc0a25ab57; não generaliza a outras operações.
## FIN-017 — Entry e audit event (contrato E2, execução pendente)
O caso de uso valida `Entry` antes de solicitar IDs (movimento e evento) e o instante UTC em ms; entrada inválida não chama IDs, relógio ou store. O adapter exige kind Entry e valida novamente antes de tomar a conexão. Em seguida, `execute_write_transaction` usa `BEGIN IMMEDIATE` para inserir com parâmetros em `main.movements` e anexar `audit_event` (`movement_entry_create` / `movement`, ID da movimentação) na mesma transação. Exige uma linha afetada; erro, zero linhas ou erro de auditoria aborta e deixa o adapter sem conexão reutilizável. Sucesso devolve a conexão ao adapter e o caso de uso só então retorna `Movement`. Migrations 002–005 permanecem imutáveis; sem IPC/UI. Estes são critérios, ainda sem prova GREEN.


FIN-017 | E3 local:6unit+3SQLite/4falhas,criaçãoEntry+audit atômicas/reopen/rollback/poison/inputinválido sem efeitos;Rust121+8docs/debug+release/fmt/clippy/TS/securityE4 PASS;coveragecrítica app43/43linhas3/3fn eadapter51/51linhas5/5fn,controle0,rejeita95. Review independente concluído;CI source/final/liberação pendentes,in_progress/passesfalse. Sem jornadaUIIPC/globalRust/setup/E2E;apósFIN017METApausará,nenhum próximoID. Ver spec/handoffFIN017;custonão registrado.
