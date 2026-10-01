# Catálogo de invariantes

| ID | Invariante |
|---|---|
| INV-FIN-001 | saldo inicial não é receita |
| INV-FIN-002 | saldo é derivado de movimentos |
| INV-FIN-003 | transferência não altera resultado |
| INV-FIN-004 | recebimento aumenta exatamente uma conta |
| INV-FIN-005 | reembolso reduz exatamente uma conta |
| INV-FIN-006 | operação composta é atômica |
| INV-FIN-007 | falha intermediária não persiste metade da operação |
| INV-SAL-001 | venda não recebida não aumenta saldo |
| INV-SAL-002 | venda pendente exige cliente |
| INV-SAL-003 | venda pendente exige vencimento |
| INV-SAL-004 | recebimentos não excedem devido sem operação explícita |
| INV-SAL-005 | produto alterado não altera snapshot antigo |
| INV-SAL-006 | item livre não cria produto |
| INV-RET-001 | devolução não excede a quantidade disponível |
| INV-REC-001 | pagamento de recorrência não duplica saída |
| INV-BKP-001 | backup restaurado passa integridade |
| INV-BKP-002 | restauração inválida não substitui banco íntegro |
