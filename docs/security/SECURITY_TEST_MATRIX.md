# Matriz de segurança

| SEC-ID | ameaça | controle | teste/scanner | evidência | status |
|---|---|---|---|---|---|
| SEC-001 | capability excessiva | deny-by-default | inspeção config | work item | todo |
| SEC-002 | injeção | parametrização/constraints | integração SQLite | work item | todo |
| SEC-003 | vazamento | redaction/Gitleaks | scan + teste log | work item | todo |
| SEC-004 | arquivo malicioso | validação/path allowlist | testes negativos | work item | todo |
| SEC-005 | restore destrutivo | staging + integridade | restore inválido | work item | todo |
| SEC-006 | duplicação | transação/idempotência | integração | work item | todo |
| SEC-007 | CVE | scans | CI | work item | todo |
