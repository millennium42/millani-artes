# Política de economia de tokens

## Objetivo e limite

Minimizar tokens, custo e tempo **por tarefa aprovada**, sem remover validação, revisão independente, segurança, acessibilidade, testes de invariantes ou evidência. Uma resposta curta que gera retry é desperdício. Quando não há telemetria do cliente, registre `não registrado`; não estime economia.

## Antes de chamar um modelo

1. Não chame LLM para ação determinística: use `rg`, schema, compilador, teste, formatter, scanner ou documentação já encontrada.
2. Defina uma pergunta, ID, saída esperada e orçamento; descarte contexto não necessário.
3. Passe paths e linhas relevantes, não o repositório, logs inteiros ou histórico completo. Resumos devem apontar para fontes canônicas e expirar após mudança do SHA.
4. Reutilize spec, ADR, resultado de RECON e handoff existentes; não peça a dois agentes a mesma leitura.
5. Peça o menor diff ou lista de achados possível. Use saída estruturada e limite explícito de itens/palavras quando a tarefa não requer explicação longa.

## Durante a chamada

- Selecione departamento, modelo e esforço conforme `AGENT_PERSONAS.md`; comece baixo e escale só após falha, ambiguidade material ou risco alto.
- Combine subtarefas que compartilham o mesmo contexto e podem retornar campos independentes; não combine decisões com diferentes permissões, riscos ou critérios.
- Para tarefas independentes, paralelize somente investigação/review com escopos distintos; um líder integra. Não use fan-out como votação ou para repetir a mesma análise.
- Estabilize instruções, tools e contexto compartilhado no prefixo; coloque ticket, diff, resultado de tool e dados voláteis no final. Quando uma API expuser cache, use breakpoints/TTL apenas após medir reutilização.
- Solicite ferramentas com saída filtrada: paths/linhas, erro mínimo e contagens; interrompa o loop após falha diagnóstica, em vez de coletar logs repetidos.
- Para edição, forneça somente o diff esperado/trechos modificados; não reenvie arquivos inalterados. Para tarefas repetidas, preserve artefato compacto versionado em vez de recontar a conversa.

## Depois da chamada

1. Execute primeiro o menor check que pode refutar a hipótese; só então rode a suíte proporcional.
2. Se falhar, envie erro sanitizado, comando, SHA e trecho relacionado — não toda a sessão.
3. Registre modelo, esforço, entradas/saídas/reasoning/cache tokens, custo, duração, retries e `pass/fail` quando expostos.
4. Compare configurações em tarefas representativas e mantenha a configuração mais leve que passa todos os gates. Revise a alocação a cada 10 work items.

## Cache e chamadas de API

Aplicável somente se o projeto vier a chamar uma API de modelos; não é uma promessa sobre o cliente Codex. Prefixos repetidos podem aproveitar cache em OpenAI, Anthropic e Gemini. Preserve a ordem e o conteúdo estável do prefixo; não pague por cache longo, pre-warming ou compactação sem medir hit rate, TTL, custo e qualidade. Separe contas/cache por usuário quando a plataforma oferecer esse controle; nunca use hit de cache como sinal de que dados privados existem.

## Anti-padrões proibidos

- reenviar contexto integral ou logs sem seleção;
- escalar modelo/esforço por estética, formatação ou ansiedade;
- abrir agentes sem pergunta independente, output limitado e critério de integração;
- resumir fonte canônica e depois tratar o resumo como fonte de verdade;
- reduzir testes, scans ou evidência para economizar;
- alegar economia por número de agentes, contexto menor ou cache sem métrica observada.

## Referências verificadas em 2026-10-01

- [OpenAI — seleção de modelo](https://developers.openai.com/api/docs/guides/model-selection): manter a configuração mais leve que satisfaz a qualidade.
- [OpenAI — latência](https://developers.openai.com/api/docs/guides/latency-optimization): reduzir saída, filtrar entrada, combinar chamadas compatíveis e paralelizar apenas passos independentes.
- [OpenAI — prompt caching](https://developers.openai.com/api/docs/guides/prompt-caching): prefixo compartilhado, breakpoints e telemetria de cache.
- [Anthropic — prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching): cache cobre prefixo de tools/sistema/mensagens e requer conteúdo estável.
- [Google — context caching](https://ai.google.dev/gemini-api/docs/generate-content/caching): cache serve para contexto inicial grande e repetido; usage metadata mede tokens cacheados.


## Decisão humana: trabalhadores locais primeiro — 2026-10-15

A solicitação posterior à pausa autoriza pesquisa ampla, instalação/configuração e CPU+GPU; não retoma a meta. Este roteamento específico orienta trabalho futuro, preservando hierarquia, revisão independente e gates:

| tarefa delimitada | trabalhador inicial | escalonamento |
|---|---|---|
| extração e rascunho curto | Qwen3.5:9b, preset local low, Ollama ROCm | saída inválida, contexto obrigatório acima do limite ou falha não resolvida |
| rascunho de código focal | Gemma4:12b-it-q4_K_M, preset local low | Qwen3.6:35b-a3b-q4_K_M medium seletivo após falha/ambiguidade concreta |
| falha local não resolvida, decisão difícil ou contexto incompatível | GPT Luna6 medium | conforme governança de risco vigente |
| revisão independente exigida | GPT Sol6 medium | conforme política de agentes |

Ferramentas determinísticas vêm primeiro. Presets locais low/medium não equivalem aos esforços Luna. Nenhum local passou os14 casos estritos:8 modelos/11 configurações/154 casos sintéticos completos, melhores11/14; baselines Luna low/medium14/14 com protocolo Codex diferente. Equivalência geral e economia líquida de tokens não comprovadas; custo não registrado.

O worker externo usa contexto curto/schema delimitado, verifica limites/identidade, reutiliza cache sem aprovação e devolve needs_review/needs_gpt/invalid_json_or_schema. Não autoriza commits, decisões de evidência, ferramentas ou patches automáticos. Revisar diff/callers antes de integrar e manter os checks do projeto. Uma GPU/modelo residente por vez; investigação independente pode ser paralela no limite da política, dependências/integração continuam seriais. Não repetir benchmark sem causa concreta.

Fonte externa: `C:\Users\milla\Documents\Pesquisa Modelos Locais\2026-10-15` (README.md, USO.md, routing.json, RESULTADOS.json); hashes no [registro](TOOL_REGISTER.md). Ollama é padrão pela execução híbrida35B; llama.cpp/Vulkan9B é alternativa executada. API somente loopback; chave do llama.cpp nunca deve entrar no Git/prompts/logs. Esta configuração não altera o provedor global do Codex: o integrador chama o trabalhador explicitamente. Meta pausada; nenhum work item fechado pela pesquisa.


## Preparação efetiva de quatro opções — decisão humana posterior em 15/10/2026

A seleção operacional passa a ser Gemma4-12B, Qwen3.5-9B, Laguna XS2.1 e Muse Glimmer30B, conforme [roteamento local](LOCAL_MODEL_ROUTING.md). Substitui a rota anterior de escalonamento local em Qwen3.6-35B; pesos dispensados podem ser removidos, preservando seus recibos históricos. Loop de leitura/edição/check/correção fica no executor local. GPT recebe spec focal, diff, contratos necessários, recibos e riscos nas revisões exigidas; exceções reais seguem governança. Não chamar GPT por comando rotineiro. Resultado needs_gpt é sinal ao integrador e não chamada de API. Contexto curto por tarefa, cache quando válido e nenhuma promessa de economia percentual. Custo não registrado.

## Registro operacional 17/10/2026 — allowance e custo de delegação

O uso compartilhado do Codex depende de modelo, local de execução, tarefa, contexto, esforço de raciocínio, velocidade e ferramentas; tarefa longa pode consumir substancialmente mais. Subagentes nativos acrescentam seu próprio trabalho de modelo/ferramentas e tendem a usar mais allowance que uma execução comparável com um único agente. Use-os somente para subtarefas independentes cujo paralelismo compense esse custo. O executor local Aider/Ollama executa inferência sem GPT; isso evita o consumo de inferência GPT, mas não torna gratuitas as chamadas do Integrador para preparar, interpretar ou revisar o resultado. Consulte [LOCAL_MODEL_ROUTING](LOCAL_MODEL_ROUTING.md) para a fila e o orçamento por tentativa.

Não faça polling GPT durante uma geração local longa. Aguarde o resultado/limite da chamada e use uma comunicação GPT quando surgir um achado, resultado ou bloqueio que exija decisão. Compare apenas métricas que o cliente realmente atribuir ao item: duração, saídas úteis/rejeitadas, correções, retries, checks e allowance/custo quando disponíveis. Não converta percentuais agregados da conta em custo do work item nem afirme economia sem atribuição observada. Esta atualização não executou benchmark; refinamentos futuros usam a evidência das tarefas normais, sem testes dedicados.

Fontes oficiais consultadas em 17/10/2026: [OpenAI — uso do Codex e allowance](https://help.openai.com/en/articles/11369540-using-codex-with-your-chatgpt-plan), [OpenAI — subagentes](https://learn.chatgpt.com/docs/agent-configuration/subagents), [OpenAI — seleção de modelos](https://developers.openai.com/api/docs/guides/model-selection), [Ollama — FAQ](https://docs.ollama.com/faq#how-does-ollama-handle-concurrent-requests).


Decisão humana mais recente nesta entrega FIN-017: encerrar todos os modelos locais, concluir somente o item atual por roteamento econômico GPT e salvar o progresso; depois pausar a META, sem outro ID ou retomada automática. Essa decisão prevalece sobre a rota local-first anterior para esta entrega. Regras provisórias de tentativas locais acima não autorizam reinício de servidor/modelos. Custo não registrado.
