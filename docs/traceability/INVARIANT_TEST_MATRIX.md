# Matriz de invariantes

| invariante | unit | integração | smoke | status |
|---|---|---|---|---|
| INV-FIN-001..005 | FIN-* | DB/FIN-* | FIN smoke | todo |
| INV-FIN-006..007 | FIN-* | DB-* | venda/restore | todo |
| INV-SAL-001..006 | SAL/RCV-* | DB/SAL-* | venda/recebimento | todo |
| INV-RET-001 | RET-* | DB/RET-* | devolução | todo |
| INV-REC-001 | REC-* | DB/REC-* | recorrência | todo |
| INV-BKP-001..002 | BKP-* | BKP-* | backup/restore | todo |

## Contas — detalhe planejado de INV-FIN-001/002

Vínculo [MVP-01/contas](REQUIREMENTS_MATRIX.md#mvp-01--contas-rastreabilidade-planejada), rastreado documentalmente por TRC-001. Detalha os dois IDs do range acima sem duplicar sua contagem ou alterar os outros invariantes. **Planejado / E0 de produto**: paths, comandos, outputs, coverage e CI dos testes financeiros não registrados; audit estrutural/CI documental não são prova dos comportamentos.

| invariante / fonte | unit planejado / oráculo | SQLite planejado / oráculo | smoke futuro | executores concretos / evidência |
|---|---|---|---|---|
| INV-FIN-001 — [inicial não receita](../product/INVARIANTS.md) / [negócio](../product/BUSINESS_RULES.md) / [centavos](../architecture/adr/ADR-013-money-in-cents.md) | inicial10000 centavos, sem movimento posterior: saldo10000, receita0; não contar inicial duas vezes | banco limpo/upgrade conforme migration; cadastro/reabertura preserva saldo inicial/histórico sem criar receita ordinária; forma física não decidida nesta matriz | SMK-003/SMK-009 (subfluxo contas/saldo): configurar conta no build de produção, fechar/reabrir, inicial/saldo10000 e receita0 | FIN-004, FIN-005, FIN-006, FIN-007, FIN-008, UI-003, SMK-003, SMK-009; testes/run/coverage não registrados |
| INV-FIN-002 — [saldo derivado](../product/INVARIANTS.md) / [negócio](../product/BUSINESS_RULES.md) / [SQLite](../architecture/adr/ADR-003-sqlite-source-of-truth.md) | inicial10000 + entrada2500 - saída900 =11600; sem movimentos mantém inicial; contrato não permite escrita direta do saldo derivado, prova por FIN-030 | histórico/query por conta coincidem após persistir/reabrir, outra conta inalterada; edição canônica reflete nova projeção e auditoria; rollback/atomicidade completos remetidos a INV-FIN-006/007/TRC-005 | SMK-003/SMK-009 (subfluxo contas/saldo): configurar, registrar entrada/saída e conferir11600 no extrato/projeção; reabrir e confirmar valor | FIN-024, FIN-025, FIN-017, FIN-018, FIN-021, FIN-028, FIN-029, FIN-030, FIN-026, DSH-002, SMK-003, SMK-009; testes/run/coverage não registrados |

Unit não prova SQLite; SQLite exige fixtures reais/temporárias e migration banco vazio/upgrade. Componente exibe saída do caso de uso; smoke usa build de produção. [Política de coverage](../quality/COVERAGE_POLICY.md), [estratégia](../quality/TEST_STRATEGY.md) e [smoke](../quality/SMOKE_TEST_POLICY.md) continuam gates dos executores. Cenários de entrada/saída/transferência e recebimento/reembolso têm rastreamento próprio futuro; suas provas não são fechadas aqui.

## Financeiro — testes planejados

Vínculo [MVP-02..06/financeiro](REQUIREMENTS_MATRIX.md#mvp-02-a-mvp-06--financeiro-rastreabilidade-planejada), detalhe local de TRC-002. As quatro linhas abaixo desdobram IDs dos ranges/síntese sem alterar a contagem canônica de 17 invariantes ou os [detalhes de contas](#contas--detalhe-planejado-de-inv-fin-001002). **Planejado / E0 produto**: testes, comandos, saídas, coverage e CI financeiro não registrados; adequação E2, checks documentais E3 não provam os oráculos.

| invariante / fonte | unit futuro / oráculo | SQLite futuro / oráculo | smoke futuro / executor | executores concretos / prova pendente |
|---|---|---|---|---|
| INV-FIN-003 — [transferência neutra](../product/INVARIANTS.md) / [ADR-009](../architecture/adr/ADR-009-transfer-neutral.md) | A10000/B3000/transf700 →9300/3700/soma13000; receita/despesa/resultado0; rejeição origem igual segundoFIN-020 | persistir caso de uso/reabrir conserva contexto Casa/Fábrica validado por FIN-010/011, saldos/resultado/histórico reconciliados sem classificar transferência como entrada/saída ordinária; falha intermediária reverte | SMK-004/SMK-009, subfluxo transferência/confirmar saldos no build de produção | FIN-010, FIN-011, FIN-019, FIN-020, FIN-021, FIN-022, FIN-023, FIN-024; DB-011, DB-012; testes/run/coverage não registrados |
| INV-FIN-006 — [operação composta atômica](../product/INVARIANTS.md) / [transação](../architecture/TRANSACTION_MODEL.md) | contrato de validações/erros do caso de uso; unit/mock não comprovam commit SQLite | sucesso mantém gravações/estado/auditoria na mesma unidade; não há operação parcial observável em transferência/pagamento; arquitetura física segue spec | SMK-004/SMK-007/SMK-009, subfluxos sucesso/saldos; smoke de sucesso não comprova rollback | DB-011, DB-014, FIN-021, REC-012; prova integração real pendente, cobertura global destas operações aguardaTRC-005 |
| INV-FIN-007 — [falha sem meia operação](../product/INVARIANTS.md) / [transação](../architecture/TRANSACTION_MODEL.md) | erro propagado segundo contrato; teste mockado não prova rollback | falha injetada após primeira escrita/antes auditoria em transferência ou pagamento mantém saldos/histórico/estado/eventos anteriores; não promover só metade | SMK-004/SMK-007/SMK-009 confirmam fluxo de produção; fault injection é integração explícita, não alegada como smoke executado | DB-012, DB-014, FIN-023, REC-014; fixturesSQLite/erros/run não registrados, demais operações futurasTRC-003/004/005 |
| INV-REC-001 — [pagamento sem saída duplicada](../product/INVARIANTS.md) / [negócio](../product/BUSINESS_RULES.md) | repetir pagamento da mesma obrigação1000 não cria segunda saída; comportamento de erro/retorno segue REC-015, não definir chave aqui | fixa e variável com valor informado: estado/saldo e uma saída após pagamento/repetição; falha preserva unidade anterior; testar por obrigação, não uma saída para toda série mensal | SMK-007/SMK-009, pagar e reconciliar saldo, subfluxo recorrência no build de produção | REC-012, REC-013, REC-014, REC-015; FIN-018, FIN-024; DB-011, DB-012; testes/run/coverage não registrados |

[Coverage](../quality/COVERAGE_POLICY.md), [camadas de teste](../quality/TEST_STRATEGY.md) e [smoke](../quality/SMOKE_TEST_POLICY.md) preservados. Categorias, Contexto/validações e dashboard têm cenários no vínculo financeiro; não inventar IDINV para regra sem ID. Recebimento/reembolso/venda/backup têm rastreabilidade específica futura e não são fechados por esses casos locais.
