# Roteamento da Meta UI-001–UI-016

> Arquivo histórico, mantido para rastreabilidade. Deixou de orientar trabalho ativo em 24/11/2026.

## Decisão e escopo

Decisão humana de 24/11/2026 para esta Meta: usar GPT-6.2 Sol no esforço medium. Esta regra específica prevalece sobre as rotas antigas de modelo e de trabalhadores locais somente enquanto esta Meta estiver ativa. Não altera a política geral do repositório nem autoriza iniciar ou reiniciar inferência local.

A Meta cobre exclusivamente UI-001–UI-016. UI-001–UI-007 já têm recibos `released`; não repetir seu desenvolvimento ou E3. Continuar a partir do primeiro item ainda não liberado, um work item por vez. Não iniciar UI-017, BKP, SMK, REL ou outra faixa.

## Personas, modelo e autoridade

| Persona | Responsável e modelo/esforço | Trabalho autorizado nesta Meta | Limite de saída/autoridade |
|---|---|---|---|
| P-INT | Root, GPT-6.2 Sol medium | Único escritor e integrador; fecha spec, implementa, executa checks, mantém evidências, prepara commit/CI e release | Um ID por vez; único executor de checks e autor de E3. |
| P-UI | Root no papel de implementação, GPT-6.2 Sol medium | Componente, estados, teclado, foco, rótulos e layout da fatia atual | Nenhuma regra de domínio ou SQL na UI; sem segunda pessoa escrevendo código. |
| P-QA | Root no papel de teste, GPT-6.2 Sol medium | Definir a menor prova por critério e escrever testes RED antes do comportamento | Runner determinístico executado pelo root; mock de IPC não vale como prova SQLite ou Tauri. |
| P-REV | Um delegado independente, GPT-6.2 Sol medium, read-only | Revisar diff capturado, spec, resultados e limites do item atual | Até três achados; não edita, não executa checks e não revisa hipótese sem diff. Retorno E1/E2. |
| P-SEC | Root começa por scanners/testes determinísticos; se houver pergunta separada, um delegado read-only GPT-6.2 Sol medium | Autoridade, capability, entrada IPC, erros, dependências e ameaças introduzidas pelo diff | Não duplicar a pergunta de P-REV. Sem ampliar capabilities para shell, filesystem ou rede. |
| P-PRO / P-TRC / P-MAP | Root, leitura de fontes e validadores determinísticos; GPT-6.2 Sol medium read-only apenas se surgir ambiguidade real | Critério de produto, rastreabilidade e inventário de paths | Sem implementação delegada nem repetição de leitura já resolvida. |
| P-OPS | Root; comandos determinísticos e conectores de CI | Build, testes de release, inspeção nativa, hashes, workflow e recibo no SHA exato | Não declarar release a partir de CI de outro SHA nem de inspeção mockada. |
| P-ESC | GPT-6.2 Sol high somente após tentativa medium falha ou risco P0/P1 concreto | Resolver impasse técnico, de segurança ou integridade com evidências conflitantes | Registrar a tentativa medium e o motivo. High não é padrão; xhigh/max não serão usados nesta Meta. |

P-DOM e P-DAT entram apenas se o contrato de produto exigir decisão de domínio ou schema/migration; UI-008 reutiliza o caso de uso e o adapter de produto já liberados, sem migration. As personas são papéis operacionais: P-UI e P-QA não geram chamadas de modelo extras quando o root já executa esses papéis.

## Regras de execução e economia

1. Começar por fontes, `rg`, compilador, formatter, testes e scanners existentes. Não usar modelo para tarefa determinística.
2. Root é o único escritor, executor de verificação, atualizador de evidências e responsável pela liberação. Delegados são read-only.
3. Um P-REV independente por item com diff de produto. No máximo dois delegados além do root, somente para perguntas independentes e com critério de aceite separado. Sem fan-out ou revisão repetida.
4. P-REV e P-SEC recebem somente spec, paths/linhas do diff, checks e fontes necessárias; nunca dados reais de clientes, backups ou segredos. E3 depende da reprodução do root.
5. Manter GPT-6.2 Sol medium como padrão desta Meta. Não alternar para Luna Max ou worker local para economizar sem medição comparável. Escalar só pela regra P-ESC.
6. Registrar no work item/handoff a persona, modelo, esforço, motivo, retorno usado/descartado e retries. Tokens/custo: `não registrado` quando o produto não atribuir telemetria ao item.

## Tendência econômica observável

A descrição oficial apresenta GPT-6.2 Sol como modelo para trabalho cotidiano, incluindo programação, e GPT-6 Luna como eficiente para tarefas focadas e de alto volume. **Inferência para esta Meta:** Sol medium é a escolha padrão para integrar mudanças granulares na base existente; Luna pode ser econômica em subtarefas focadas e repetitivas, desde que não substitua revisão, testes ou gates. No preço padrão da API, GPT-6.2 Sol custa US$ 2 por milhão de tokens de entrada e US$ 10 por milhão de tokens de saída; GPT-6 Luna custa US$ 0,10 e US$ 0,50. Para o mesmo número de tokens, o custo bruto da API Sol é 20 vezes maior. Isso não equivale ao consumo do plano ChatGPT/Codex: o allowance depende de modelo, tarefa, tamanho de contexto, esforço, ferramentas e trabalho de subagentes. O custo desta Meta não está registrado; não declarar economia ou gasto total sem telemetria atribuída.

Fontes OpenAI consultadas em 24/11/2026: [seleção de modelos](https://developers.openai.com/api/docs/guides/model-selection), [GPT-6.2 Sol](https://developers.openai.com/api/docs/models/gpt-6.2-sol), [GPT-6 Luna](https://developers.openai.com/api/docs/models/gpt-6-luna) e [uso do Codex no plano ChatGPT](https://help.openai.com/en/articles/11369540-using-codex-with-your-chatgpt-plan).
