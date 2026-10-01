# Política de agentes, modelos e esforço

## Regra de economia

Use o menor modelo e esforço que produza uma saída acionável. Não use todos os modelos em cada tarefa: a economia vem de evitar contexto repetido, fan-out inútil e retries. Preço/consumo não foi verificado; esta é uma política de alocação, não uma alegação de economia medida. Disponibilidade e esforços efetivos são os que o cliente expuser na sessão.

## Roteamento disponível

| modelo | uso padrão | esforço inicial | escalar quando |
|---|---|---|---|
| `gpt-6-luna` | inventário, busca local, checklist, revisão de formato | low | resultado não acionável |
| `gpt-5.6-luna` | fallback de triagem em lote | low | indisponibilidade/continuidade |
| `gpt-5.6-terra` | tarefa rotineira limitada | low | integração exigir análise |
| `gpt-5.6-sol` | implementação/review de continuidade | medium | falha concreta ou conflito |
| `gpt-6-sol` | implementação/review comum | medium | risco P1 ou diagnóstico falho |
| `gpt-6.1-sol` | integração, implementação normal e decisão técnica | medium | risco P1, migration ou fronteira de segurança |
| `gpt-6-astra` | P0, perda de dados, conflito de ADR ou segurança sem solução | high | somente após tentativa documentada |
| `gpt-5.5` | fallback legado | low | não é padrão para trabalho novo |

Respeite os limites do cliente: `gpt-6.1-sol`, `gpt-6-astra` e `gpt-6-sol` aceitam low–ultra; `gpt-6-luna`, `gpt-5.6-terra` e `gpt-5.6-luna`, low–max; `gpt-5.5`, low–xhigh. Comece em minimal/low para leitura, medium para implementação. High é para migration, segurança, diagnóstico de teste falho ou conflito entre decisões. xhigh/max/ultra só após uma tentativa high bloqueada e registrada. Nunca escale por formatação ou redação.

## Paralelismo controlado

O padrão é um agente. Use no máximo três agentes ativos (líder e dois delegados) somente se as subtarefas forem independentes e evitarem um retry maior. No checkout compartilhado, o líder é o único escritor, executor de comandos de verificação, atualizador de evidência e autor do commit. Delegados são read-only; nunca editam os mesmos arquivos nem iniciam outro work item. Escrita paralela só é permitida em worktrees isolados, com paths sem sobreposição, dependências fechadas e integração serial pelo líder com todos os checks reexecutados.

Fases: até dois `luna` em RECON para fontes, árvore ou rastreabilidade; líder sozinho em RED/BUILD/TEST/SECURITY; um `sol` independente pode revisar o diff; `6.1-sol` high ou `astra` high arbitra apenas P0/P1 concreto. Após qualquer escrita, capture o baseline antes de nova revisão paralela.

## Contrato de delegação e evidência

Delegação contém: persona de `AGENT_PERSONAS.md`, ID, SHA, pergunta única, fontes canônicas, modo read/write, arquivos permitidos, limite de saída e critério de aceite. Retorno contém no máximo três achados, arquivos/linhas inspecionados, conclusão, risco e comandos executados/não executados. Análise delegada vale E1/E2 no máximo; somente execução reproduzida pelo líder é E3. Handoff registra `persona/papel | modelo | esforço | escopo | saída usada/descartada | verificação do líder` e tokens/custo quando o cliente os expuser; caso contrário, `não registrado`.

Meça por tarefa: número de delegados, escalonamentos, retries evitados e tokens/uso quando o cliente expuser telemetria. Revise a política após 10 work items; não infira economia apenas por usar modelo menor.
