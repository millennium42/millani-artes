# Migrations

Migrations são versionadas, aditivas onde possível e imutáveis depois de aplicadas. Todo conjunto prova banco vazio, upgrade do snapshot anterior, constraints, índices justificados e rollback/estratégia declarada. Valores monetários são inteiros de centavos (ADR-013). Nunca dependa de banco manual para CI.
