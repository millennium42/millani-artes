# Matriz de requisitos

Contagem do escopo e fontes: [inventário essencial](ESSENTIAL_REQUIREMENTS.md). Grupos, invariantes, jornadas e controles não são totais aditivos; as linhas de produto abaixo ainda planejam testes futuros.

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
| README de entrada reflete fontes/estado (documental) | README | DOC-003 | 9 links locais + comandos de scripts existentes; E3 local/remoto | CI entrega 37039057140 verde; fechamento exige CI no SHA final, nota refs/notes/evidence |
| hierarquia e gates seguem autoridade superior (documental) | governança/AGENTS | DOC-004 | inspeção/review E2 + workflow E3 | CI entrega 37040416220 verde; fechamento exige CI no SHA final, nota refs/notes/evidence |
| inventário conta grupos essenciais sem dupla contagem (documental) | escopo/fontes | DOC-005 | Python 13/17/7/9/8 + 42 links locais; workflow local/remoto; E3 documental | review E2 sem achados; CI entrega 37042390429 verde; fechamento exige CI no SHA final, nota refs/notes/evidence |
| regras vigentes revisadas nesta sessão (documental) | AGENTS/governança | GOV-001 | leitura/review E2; referências/diff mínimo locais + workflow local/remoto E3 documental | CI entrega 37043981692 verde; fechamento exige CI no SHA final, nota refs/notes/evidence |
| declarações de evidência consistentes (governança) | EVIDENCE_POLICY/work items | GOV-002 | scripts/test-evidence-states.ps1 + scripts/check-evidence-states.ps1; 25fixtures/11items locais e CI E3 do check, sem prova de produto | CI entrega 37046183514 verde; fechamento exige CI no SHA final, nota refs/notes/evidence |
| inventário revisado antes de instalar (governança) | TOOL_REGISTER | GOV-003 | versões/PATH + workflow E3; licenças locais/contexto E2; sem instalação/adoção | checks/review verdes; CI entrega 37049083478 success no SHA fba319e; fechamento exige CI no SHA final, nota refs/notes/evidence |
| exceção temporária somente quando aplicável (governança) | EXCEPTION_TEMPLATE/RELEASE_READINESS | GOV-004 | inventário/checks E3 documentais; aplicabilidade/gate/review E2; sem dispensa/E4 fabricado | gate expirado corrigido; CI entrega 37050459499 success no SHA6073273; fechamento exige CI no SHA final, nota refs/notes/evidence |
| aplicação de política multiagente (governança) | MULTI_AGENT_POLICY/AGENT_PERSONAS | GOV-005 | contrato/configuração solicitada e review E2; checks E3 do Integrador, único escritor | política preservada; checks/review verdes; CI pendente, passes=false |
