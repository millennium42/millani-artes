# Matriz de requisitos

| requisito | regra/módulo | work item inicial | teste futuro | gate |
|---|---|---|---|---|
| contas/saldo | FIN | FIN-001 | unit+integração | coverage |
| movimentos/transferência | FIN | FIN-008 | invariante | coverage |
| recorrência | REC | REC-001 | unit+integração | smoke |
| produto/venda | SAL | PRD-001 | unit+integração | coverage |
| cliente/recebimento | RCV | CUS-001 | integração | smoke |
| devolução/reembolso | RET | RET-001 | invariante | smoke |
| dashboard | DSH | DSH-001 | componente | visual |
| backup/restore | BKP | BKP-001 | integração+negativo | security |
| mapa reflete a árvore (documental) | mapa de fontes | DOC-002 | scripts/test-project-map.ps1 + scripts/check-project-map.ps1; E3 local/remoto | CI candidato 37037225188 verde; fechamento exige CI no SHA atual, evidência em refs/notes/evidence |
