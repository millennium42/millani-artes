# Modelo de dados inicial

Entidades previstas: `account`, `category`, `financial_movement`, `recurrence`, `product`, `sale`, `sale_item`, `customer`, `receivable`, `receipt`, `return`, `refund`, `audit_event`, `schema_migration` e metadados de backup.

Dinheiro será inteiro em centavos (ADR-013); valores agregados são consultas/projeções. Chaves, constraints, índices e migrations concretos ficam para `DB-*`, depois de uma spec e testes de banco vazio/upgrade.


DB-013 implementa `audit_event` técnico com IDs opacos de 16 bytes, instante Unix ms UTC e tags estáticas de operação/entidade. Não armazena payload financeiro nem prova invariantes financeiras; detalhes e limites em [spec](../../planning/specs/DB-013.md).

FIN-001 E3 local define `Account` no domínio com `id: [u8; 16]` e `name: String`; teste preserva os bytes opacos e o nome Unicode exatamente. Isso não define validação, tipo, saldo inicial, schema SQLite ou persistência. Review E2 sem achados; CI de entrega/final pendente; invariantes financeiras continuam E0. [Spec e handoff](../../planning/specs/FIN-001.md).
