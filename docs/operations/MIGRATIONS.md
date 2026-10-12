# Migrations

Migrations são versionadas, aditivas onde possível e imutáveis depois de aplicadas. Todo conjunto prova banco vazio, upgrade do snapshot anterior, constraints, índices justificados e rollback/estratégia declarada. Valores monetários são inteiros de centavos (ADR-013). Nunca dependa de banco manual para CI.

DB-003 executa o SQL interno, estático e revisado do chamador numa única transação SQLite: só devolve a conexão após commit confirmado; em qualquer erro, tenta rollback quando aplicável e descarta a conexão consumida. Isso não registra versão nem valida SQL externo; seleção/registro pertencem a DB-004. Veja [contrato e evidência](../../planning/specs/DB-003.md), a API [Transaction do rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html) e [transações SQLite](https://www.sqlite.org/lang_transaction.html).
