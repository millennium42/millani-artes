# Desenvolvimento assistido por IA

IA executa trabalho mecânico, não redefine silenciosamente escopo, regras financeiras, segurança, arquitetura, thresholds ou gates. Classifique alegações: E0 hipótese, E1 fonte identificada, E2 fonte verificada, E3 execução verificada, E4 aceite humano. Código, mock ou build não são E3 por si.

Para cada tarefa: fixe o comportamento observável; inspecione o estado; escreva o teste RED quando aplicável; implemente o mínimo; verifique testes, segurança e revisão contra a spec; registre aprendizados. P0/P1 bloqueiam conclusão. Em B6, `done/E3/passes:true` pode representar o candidato depois dos critérios, checks locais e segurança; o campo não libera dependentes. P-REV, CI remoto verde no SHA final de `main` e nota em `refs/notes/evidence` nesse SHA são gates de liberação. Não registre como liberado nem inicie o próximo item antes da nota completa. Outros fluxos continuam exigindo todos os gates antes de `passes:true`.

A hierarquia de autoridade está em [AGENTS.md](../AGENTS.md). Ordem de leitura e mapa de áreas orientam navegação, não precedência. Spec ou work item não pode dispensar um gate exigido por fonte superior; registre o conflito e siga a decisão humana ou fonte canônica aplicável.

Skills adaptadas da toolbox: `project-kickoff`, `granular-delivery` e `honest-verification`. Elas foram instaladas no perfil local; seu uso fica disponível em nova sessão. Estas regras preservam o comportamento enquanto isso.

O roteamento de modelos, esforços, limites de paralelismo e prova por agente é obrigatório em `MULTI_AGENT_POLICY.md`. Delegação acelera RECON/review, mas não flexibiliza a regra de um work item, os gates ou a responsabilidade do líder.

`TOKEN_ECONOMY_POLICY.md` é obrigatório para contexto, tools, cache e medição. Economia só é aceita quando preserva os gates desta política.
