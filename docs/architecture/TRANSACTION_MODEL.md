# Modelo transacional

Toda escrita financeira composta executa numa transação SQLite. Validações que dependem do estado atual, gravações e evento técnico pertencem à mesma unidade. Falha cancela a unidade inteira. A UI não persiste por etapas.

Idempotência e prevenção de duplicação serão especificadas por caso de uso; não são presumidas por botão, relógio ou interface.

## Fronteira da unidade — ARC-004

O caso de uso na aplicação possui a unidade de escrita, do início ao resultado. Ele recebe a intenção da UI, valida a entrada e coordena os repositórios; a infraestrutura fornece a transação SQLite. Domínio permanece puro. Repositórios usados pela operação compartilham a mesma conexão/contexto transacional e não fazem commits independentes. Uma operação composta não pode ser dividida entre chamadas de persistência da UI.

Validações puras de formato/tipo podem ocorrer antes de abrir a unidade. Validações que leem estado atual — saldo devido, quantidade restante, obrigação já paga ou referências ativas — ocorrem dentro dela, junto das gravações e do audit_event técnico. Uma validação feita antes da abertura não substitui a verificação do estado que autoriza a escrita. A estratégia SQLite de concorrência e o contrato executável serão definidos em DB-011; não há adapter implementado aqui.

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

[Spec](../../planning/specs/ARC-004.md) e [handoff](../../planning/handoffs/ARC-004.md) registram checks documentais e revisão, com escopo E3 de estrutura/preservação e E2 de contrato. Não existem assinatura transacional, adapter SQLite, migration ou caso de uso financeiro executados nesta etapa.
