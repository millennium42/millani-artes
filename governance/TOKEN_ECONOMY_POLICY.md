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

- Selecione modelo e esforço conforme `MULTI_AGENT_POLICY.md`; comece baixo e escale só após falha, ambiguidade material ou risco alto.
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
