# Modelo transacional

Toda escrita financeira composta executa numa transação SQLite. Validações que dependem do estado atual, gravações e evento técnico pertencem à mesma unidade. Falha cancela a unidade inteira. A UI não persiste por etapas.

Idempotência e prevenção de duplicação serão especificadas por caso de uso; não são presumidas por botão, relógio ou interface.
