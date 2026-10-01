# Threat model

Assets: histórico e saldos, telefones, SQLite, backups, chaves, configuração e binário. Fronteiras: WebView, core Tauri, plugins, filesystem, SQLite, arquivo selecionado, backup/restore e atualização futura.

| Ameaça | Controle planejado |
|---|---|
| SQL injection/corrupção | consultas parametrizadas, constraints, transações, testes de integração |
| backup adulterado/malformado | formato autenticado, validação antes de restore, cópia preservada, teste negativo |
| path traversal/sobrescrita | paths controlados, allowlist, operações atômicas |
| capability excessiva | deny-by-default e revisão por capability |
| XSS/remoto | CSP, sem conteúdo remoto sem ADR |
| segredo/log sensível | Gitleaks, redaction e política de logs |
| dependência vulnerável | npm/cargo/OSV gates e atualização rastreada |
| duplicação financeira | casos de uso canônicos, transação e testes de idempotência |
