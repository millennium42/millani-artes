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

Fases: o Integrador contrata os departamentos de `AGENT_PERSONAS.md` pelo problema atual e escolhe o menor modelo/esforço suficiente pelo roteador daquele catálogo. RECON pode usar Cartógrafo, Produto ou Rastreabilidade; domínio, dados, UI, QA, segurança e operações entram somente se a spec os exigir. Após qualquer escrita, capture o baseline antes de review. `P-ESC` arbitra apenas P0/P1 concreto ou tentativa high bloqueada.

## Contrato de delegação e evidência

Delegação contém: persona de `AGENT_PERSONAS.md`, ID, SHA, pergunta única, fontes canônicas, modo read/write, arquivos permitidos, limite de saída e critério de aceite. Retorno contém no máximo três achados, arquivos/linhas inspecionados, conclusão, risco e comandos executados/não executados. Análise delegada vale E1/E2 no máximo; somente execução reproduzida pelo líder é E3. Handoff registra `persona/papel | modelo | esforço | escopo | saída usada/descartada | verificação do líder` e tokens/custo quando o cliente os expuser; caso contrário, `não registrado`.

Meça por tarefa: número de delegados, escalonamentos, retries evitados e tokens/uso quando o cliente expuser telemetria. Revise a política após 10 work items; não infira economia apenas por usar modelo menor.


## Trabalhadores locais no fluxo granular — decisão humana de 15/10/2026

Consultar [LOCAL_MODEL_ROUTING](LOCAL_MODEL_ROUTING.md). Os quatro modelos locais fazem o trabalho delegado com spec e paths focais; Codex/GPT recebe pacotes curtos nas revisões obrigatórias e casos extremos. O executor local é Aider com provedor Ollama dentro do sandbox do Codex CLI, sem inferência GPT no subprocesso, sem fallback pago automático; não é seleção de modelo local pela ferramenta de subagentes desta conversa. O checkout principal continua exclusivo do integrador. Máximo integrador+2 departamentos independentes, uma fila/modelo na GPU, um ID do roadmap por vez; WTs isolados para escrita e integração serial. O integrador reproduz checks para E3. Não marcar passes:true, publicar ou reaproveitar CI de outro SHA pela saída do executor. Sem tarefa recorrente agendada ou retomada automática da meta pausada.

Subagentes nativos do Codex fazem seu próprio trabalho de modelo/ferramentas e consomem mais allowance que uma execução comparável com um agente. Portanto, o padrão continua sendo um Integrador; abra delegados GPT só para trabalho independente e delimitado que possa avançar em paralelo. Uma cadeia serial, escrita concorrente ou espera pela única fila Ollama não justifica fan-out. Limites de tentativa local e a distinção entre inferência local e allowance do Codex estão em [LOCAL_MODEL_ROUTING](LOCAL_MODEL_ROUTING.md).


Decisão humana mais recente nesta entrega FIN-017: encerrar todos os modelos locais, concluir somente o item atual por roteamento econômico GPT e salvar o progresso; depois pausar a META, sem outro ID ou retomada automática. Essa decisão prevalece sobre a rota local-first anterior para esta entrega. Regras provisórias de tentativas locais acima não autorizam reinício de servidor/modelos. Custo não registrado.


FIN-017 | done/E3: source5a09ca2ec7117be7e39605bfe25848e7fc39acaf CI39090756071/job117566803340/attempt1success23/0skip/7marcadores exatos;121testesRust+8doctests. Entry+audit atômicos/6unit+3SQLite4falhas,critical app43/43linhas3/3fn+adapter51/51linhas5/5fn100%,controle0. Gateslocais/review/validators/stagedscan aprovados;docs antespassestrue. CI documental/nota95/liberação pendentes;WTarquivada,evidênciapreservada,locaisencerrados. DepoispausarMETA por decisão humana,nenhumpróximoID. ProdutoUIIPC/receitaFIN005/globalRust/setup.exe/E2E pendentes;custonão registrado.
