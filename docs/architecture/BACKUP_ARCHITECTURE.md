# Arquitetura de backup

O banco SQLite ativo é fonte de verdade. Backup local cria uma cópia consistente, criptografa em formato versionado e valida o resultado. Restore trata arquivo como não confiável: valida formato e integridade, preserva o banco corrente, restaura de modo atômico e verifica a cópia restaurada antes de promovê-la.

Algoritmo, formato, KDF e armazenamento de chave ainda exigem decisões/POCs nos work items `BKP-*`; não será inventada criptografia própria.
