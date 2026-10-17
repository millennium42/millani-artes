# Roteamento local para desenvolvimento granular

## Decisão e estado

Preparação autorizada em 15/10/2026 para retomar o Millani em uma nova conversa. A pausa do desenvolvimento permanece: CAT-003 → FIN-012 → UI-001 são os itens abertos preservados. Instalar/configurar ferramentas não fecha um work item nem comprova o produto.

Pasta operacional: `C:\Users\milla\Documents\Pesquisa Modelos Locais\2026-10-15\operacao`. Configuração executável em runtime.json; executor em local-agent.py; controle do servidor em runtime-control.py. Não alterar o provedor global do aplicativo Codex. O integrador dispara Aider isolado com Ollama local dentro do sandbox de escrita do Codex CLI, sem inferência GPT nesse subprocesso, e recebe um recibo compacto ao terminar. O agente nativo Codex/Ollama não editou a fixture e OpenCode encontrou EPERM ao abrir Git no Bun; essas tentativas foram rejeitadas e não são provas de funcionamento.

## Quatro modelos selecionados

| chave | modelo/tag local | função inicial | contexto configurado |
|---|---|---|---:|
| gemma | millani-gemma:12b — Gemma 4 12B Q4_K_M | implementação focal, testes e correção | 32768 |
| qwen | millani-qwen:9b — Qwen 3.5 9B Q4_K_M | spec como rascunho, inventário, documentos e handoff | 16384 |
| laguna | millani-laguna:xs-2.1 — Laguna XS 2.1 Q4_K_M | contratos e alterações mais complexas, segunda tentativa local fundamentada | 32768 |
| muse | millani-muse:30b — Muse Glimmer 30B Q4_K_M | raciocínio difícil e investigação local após falha concreta | 32768 |

Rótulos de esforço local não representam equivalência com Luna. Gemma/Qwen já eram instalados; Laguna/Muse foram obtidos dos repositórios oficiais em revisões fixadas e conferidos pelos SHA256 LFS. Consultar model-sources.json e *-installation.json. Os quatro perfis são uma seleção operacional, não ranking universal nem qualidade comprovada para todo o roadmap.

## Personas e fases

Reutilizar as personas existentes, sem agentes abertos por rotina. Cada missão contém persona, ID, base SHA, pergunta única, fontes, paths, modo, saída limitada e critério de aceite.

| fase | persona | rota |
|---|---|---|
| SPEC | P-PRO e P-TRC quando necessário | Qwen rascunha critérios e aponta fontes/ambiguidades; decisão humana/produto/ADR permanece autoritativa |
| RECON | P-MAP | ferramentas determinísticas primeiro, Qwen sintetiza somente arquivos relevantes |
| RED / TEST | P-QA | Gemma prepara teste focal; ferramentas executam; integrador reproduz |
| BUILD / CORRECT | P-INT ou P-UI | Gemma; Laguna após falha/ambiguidade concreta; mesma tarefa e menor diff |
| domínio e dados | P-DOM / P-DAT | Laguna com contrato fechado; revisão exigida para dinheiro/migration/atomicidade permanece |
| SECURITY | P-SEC | scanners determinísticos e triagem local; Muse quando a análise exigir raciocínio adicional |
| REVIEW | P-REV independente | GPT Sol 6 medium nas revisões exigidas pelo projeto |
| impasse | P-ESC | Muse para hipótese local; GPT para falha não resolvida, arquitetura/segurança/dados ou contradição material; esforço segue política vigente |
| VERIFY / CI / integração | P-INT integrador | reproduzir gates, conferir SHA/run remoto, integrar e publicar somente segundo o fluxo autorizado |
| LEARN / handoff | P-TRC | Qwen rascunha registros; integrador confere recibos e preserva log append-only |

Máximo: integrador e dois departamentos independentes. Uma GPU/modelo residente e uma fila de inferência. Leitura/review independente pode ocorrer em paralelo; escrita somente em worktree isolado registrado, paths disjuntos e dependências fechadas. Um ID do roadmap por vez. O checkout principal é escrito apenas pelo integrador.

## Como economizar GPT

O executor local recebe uma spec focal, arquivos alteráveis, fontes de leitura e comando de teste autorizado; Aider conduz a edição e o ciclo de correção. GPT recebe apenas o pacote de revisão obrigatório: ID/base, critérios, diff, contratos dos callers afetados, recibos dos checks e riscos. Preservar todas as revisões obrigatórias; não chamar GPT por cada comando ou erro rotineiro. Retornar ao executor local os achados focais para correção. Não reenviar o repositório, histórico completo, cadeia de raciocínio ou logs crus.

O recibo inclui diff real e a última resposta do assistant, limitada a8KiB com hash e indicação de corte, sem prompt/logs/raciocínio. Toda saída local continua needs_review; falha técnica vira needs_gpt, que é indicação para o integrador, não chamada paga automática. Um erro rotineiro diagnosticado pode ser corrigido localmente. Não repetir tentativas idênticas sem progresso. Afirmações do modelo sobre testes são E2; E3 exige reprodução pelo integrador e CI verde exige run remoto do SHA exato. Nenhum mecanismo concede passes:true, commit, push ou publicação ao executor.

## Recursos do computador

Ryzen 5 5500: 6 núcleos/12 threads, 31,89 GiB RAM; RX7600: 8176 MiB VRAM. Todos os Modelfiles aplicam num_thread=12. Flash Attention e KV q8_0 configurados; geração até8192 para Gemma,4096 para Qwen e16384 para Laguna/Muse. Ollama carrega um modelo por vez, paralelismo1, cache residente30min, timeout de carga30min. Gemma usa limite40 camadas GPU e Qwen33, somente texto; Laguna/Muse usam distribuição automática. O agendador distribui pesos/buffers entre CPU/RAM e GPU/VRAM, respeitando os perfis; margem inicial512MiB de VRAM. A RAM do sistema e memória compartilhada da GPU são a mesma capacidade física, nunca somar como memória extra.

Jogo/outro aplicativo aberto pode reduzir a VRAM disponível. Não encerrar aplicativos da usuária. Após liberar recursos, descarregar/recarregar o modelo para refazer a distribuição. Usar WDDM em resources.py para amostras de alocação dedicada/compartilhada da árvore própria e memória RAM disponível; não confundir amostra com pico/residência. hipMemGetInfo no Windows só considera o processo observador e não comprova VRAM global livre. Modelo gigante pode usar RAM e ficar lento; isso é permitido, sem overclock, mudança de voltagem, limites térmicos ou swap forçado.

Windows na tomada: timeout de suspensão e hibernaçãoAC configurados em0; tela e bateria não alteradas. Valores anteriores em power-before.json para reversão. Não há tarefa agendada do roadmap nem inicialização automática de inferência: o computador pode permanecer ligado, a execução começa quando o integrador a dispara.

## Executar ao retomar

1. Confirmar decisão humana de retomada, branch/HEAD/status, fontes canônicas, handoff e dependências de CAT-003. Ler apenas os trechos aplicáveis. Não promover o conjunto consolidado inteiro à main.
2. Conferir runtime.json, versões/hashes, memória e quatro modelos; iniciar servidor próprio:

```powershell
Set-Location 'C:\Users\milla\Documents\Pesquisa Modelos Locais\2026-10-15\operacao'
rtk proxy python -B runtime-control.py start
rtk proxy python -B resources.py
```

3. O integrador cria/anexa um worktree isolado segundo a política do projeto, baseado no estado correto, fecha a spec e reserva paths. Não usar o checkout principal no executor.
4. Preencher um JSON como o template abaixo, com SHA real e somente paths alteráveis. O agente pode ler fontes públicas necessárias no worktree; allowed_paths é contrato de diff persistente, não sandbox de leitura por arquivo. read_paths enumera fontes fornecidas ao agente; test_command é comando confiável definido pelo integrador. Esta versão exige arquivos existentes em allowed_paths: criação de arquivos/pacotes requer preparação do integrador e novo contrato.

```json
{"id":"CAT-003","instruction":"Ler a spec e handoff aplicáveis, cumprir a missão focal e executar os checks definidos. Seguir a hierarquia e não fechar o ID.","base_sha":"PREENCHER_SHA_REAL","allowed_paths":["PREENCHER_PATH_REAL"],"mode":"write","read_paths":["PREENCHER_SPEC_E_CALLER"],"test_command":"PREENCHER_CHECK_FOCAL_AUTORIZADO"}
```

5. Disparar a persona/modelo selecionados, sem reabrir toda a conversa:

```powershell
rtk proxy python -B local-agent.py --model gemma --persona P-INT --task tarefa.json --worktree 'CAMINHO_ABSOLUTO_DA_WT' --timeout 86400
```

6. Ler o recibo *.job.json; revisar diff, reproduzir checks e cumprir REVIEW→CORRECT→VERIFY→CI→LEARN. O executor recusa checkout primário, base divergente, contexto/perfil inválido, concorrência ou paths de diff fora do contrato. O sandbox de escrita é aplicado pelo Codex CLI; não há bypass. HOME/XDG/CODEX_HOME/TEMP/TMP/APPDATA são diretórios efêmeros dentro da WT, removidos antes de auditar o diff; nenhuma credencial cloud é herdada. O backend Windows atual não oferece isolamento comprovado de leitura externa e mantém rede habilitada para Ollama: não fornecer dados privados/segredos nem alegar confinamento de rede ao localhost. Aider recebe só fontes públicas selecionadas e não possui executor livre de comandos neste fluxo; checks são definidos pelo integrador. O modo read fornece arquivos --read ao Aider e exige diff zero ao terminar; o sandbox :workspace permite a escrita do estado efêmero e não torna o checkout somente leitura no SO. Contexto/qualidade em tarefas reais requerem observação.
7. Ao terminar: preservar evidência sanitizada, encerrar processo próprio e limpar outputs dispensados:

```powershell
rtk proxy python -B runtime-control.py stop
```

Não apagar gpu.lock automaticamente: pode indicar um processo ainda vivo após timeout. Conferir PID/executável/horário e encerrar somente a árvore própria antes de recuperar o lock. Recibos são necessários mesmo em falhas; conservar hashes e resultados originais.

## Execuções e limites

Os smokes Qwen/Laguna/Muse responderam READY com contextos configurados e12 threads; não são benchmark. Laguna:26,03s, amostra WDDM6,33GB dedicada e RAM disponível16,22GB; Muse:39,20s,6,86GB dedicada e RAM disponível16,70GB. São bytes decimais arredondados e amostras, não picos. Qwen:9,76s, concluído antes da amostra de20s. Gemma/Aider editou somente calc.py na fixture base b4b9e1169eb85bbf8e751069d6de9188a3d01fb6; três testes reproduzidos pelo integrador passaram. Qwen/Aider respondeu à leitura com diff zero. São provas sintéticas, não jornadas do produto; execução com ferramentas por Laguna/Muse em tarefa real ainda não verificada. Recibos em agent-proof.json/read-proof.json;12 selfchecks passaram; revisão independente Sol6 medium não encontrou bloqueio novo. Iniciar/parar o servidor próprio passou, portas11434/11435 fechadas ao encerrar. Consulte setup-final.json. Não há prova de estabilidade contínua por24h.

## Limpeza e continuidade

Removidos7 tags/modelos dispensados, encoders de imagem/áudio sem uso, Granite, runtimes/ZIPs substituídos, logs crus, temporário de diagnóstico,5 diretórios gerados ignorados e12 cópias que correspondem a notas publicadas. Quatro modelos/pesos foram protegidos e os SHA256 recalculados após a remoção. Arquivos removidos somam74,18GiB; aumento observado de espaço livre71,47GiB e88,98GiB livres no encerramento (medidas distintas). Preservados86 notas, código, fontes humanas, recibos, fixtures reproduzíveis e ambiente Aider utilizado. Docker não foi usado nesta preparação; inventário retornou erro, imagens não verificadas nem removidas. Não havia unidade E: montada na consulta.

Dependências do frontend e outputs Rust foram removidos: reinstalar/recompilar pelos comandos canônicos ao retomar; nenhum build/teste do produto executado nesta preparação. O runtime de uso é Aider0.86.2+Ollama0.40.4; OpenCode e llama.cpp portáteis dispensados foram removidos, preservando origem/hash/recibos históricos. A pesquisa antiga permanece evidência histórica, suas rotas/modelos removidos foram substituídos por este guia.

## Retomada em nova conversa

Texto sugerido: “Retome Millani Artes a partir de planning/handoffs/RETOMADA-2026-10-15.md e governance/LOCAL_MODEL_ROUTING.md. Confirme o estado real e continue CAT-003, um ID por vez. Use os quatro executores locais e suas personas para o trabalho; GPT apenas nas revisões exigidas e falhas/casos extremos. Preserve os gates e evidência por SHA. Não repita a pesquisa ou benchmark sem causa.”

Modelos/esforços nesta preparação: Luna6 medium adaptou o executor; Luna6 low pesquisou fontes; Gemma/Qwen executaram localmente; Sol6 medium revisou; integrador GPT (modelo/esforço não registrado) corrigiu, documentou e verificou. Consumo de GPT, custo e economia líquida não registrados. A configuração não comprova consumidores financeiros, coverage Rust global nem setup.exe. Recibos de instalação, execução, revisão e limpeza ficam nesta pasta; consultar o fechamento final para os resultados realmente executados.

## Decisão posterior de16/10 — integração inicial da próxima META

Somente a próxima conversa deve iniciar o desenvolvimento. Nela, reconciliar todas as branches e unificar primeiro sobre main, preservando as3 frentes e os gates, depois continuar CAT-003 → FIN-012 → UI-001 e demais IDs elegíveis até E2E oficial. Esta decisão humana supera os trechos anteriores que impediam promover o checkpoint completo à main. Todas as97 branches locais eram ancestrais de d374791 (main0 exclusivos/3 atrás); reconfirmar e preferir fast-forward, sem merges redundantes. Refs remotas/push/CI ficaram sem verificação por falha DNS/transporte em16/10. Ler repo-checkpoint-receipt.json e o prompt externo PROMPT-META-RETOMADA-2026-10-16.md para o estado final. Não há autorização para começar nesta conversa.

## Decisão humana 16/10/2026 — economia total durante execução local
Pedido humano: parar consultas repetidas de progresso e reduzir consumo Codex. P-INT aguarda conclusão do executor local sem ciclos de GPT para narrar espera. Só comunica conclusão de fase, achado concreto ou bloqueio. Contexto e recibos curtos; não reler histórico nem duplicar revisão obrigatória. Se tarefa grande não entregar, recortar por arquivo/critério com causa registrada. Manter personas e Sol6 medium somente nas revisões exigidas/casos locais não resolvidos. O editor Aider whole corta cercas Markdown internas: rascunhos Markdown devem usar código inline, sem cercas aninhadas; executor permanece intacto. Uso por conversa e custo não medidos; percentuais da conta não atribuem consumo ao item.

## Correção comprovada do timeout — FIN-009
O limite86400 do supervisor não era repassado ao Aider; default600s causou cinco APIConnectionError/respostas vazias, sem diff. Emenda necessária adicionou somente --timeout str(a.timeout) ao argv do Aider instalado, respeitando validação1..86400. Backup/hashes e baseline setup-manifest preservados; sandbox/perfis/roteamento/auditoria intocados. 12 selfchecks offline e parser Aider real60/3600/86400 passaram E3. Review Sol6 medium E2 sem P0/P1/P2; deadlines compartilhados mantêm limite externo. Recibo externo runtime-client-timeout-amendment-20261016.json. Não confundir esses checks com teste funcional FIN-009. Esperar recibo de conclusão sem polling do GPT.

FIN-009 correção de integração local: output Laguna rejeitado por recriar produção dentro do teste, clone de Account nãoClone e import inválido. Cópia/diff rejeitados preservados externamente antes de substituir por imports reais, sem funcionalidade nova. Aider --no-git usa diretório comum dos arquivos; test_command deve referir helper por caminho absoluto, nunca artifacts relativo ao cwd presumido. RED exige imports reais e >=3 testes e erro E0432 dos três símbolos faltantes; erro de outro import não é prova. Contrato funcional/aceite permanecem. Repetição local fundamentada pela correção de caminho e feedback concreto, sem implementação GPT.

Aider --no-git usa raiz comum dos arquivos editáveis: o filename da resposta deve ser o NOME EXIBIDO PELO AIDER (por exemplo accounts.rs), e não repetir prefixo do repo se a raiz já for source/infrastructure. allowed_paths continua repo-relativo exato. Não ampliar escopo para acomodar arquivos duplicados: preservar/quarentenar, revisar candidato e reproduzir. Teste auxiliar fora do escopo causou rejeição correta no BUILD FIN-009; sem mudança nas permissões/auditoria.

## Correção necessária do helper de teste local
Local dependia de rustc shim .cargo/bin sob USERPROFILE efêmero: tentava auto-instalar toolchain e falhava linker link.exe, impedindo feedback real. Root preservou saída/erro e corrigiu só helper autorizado artifacts/fin009-check.py para compiler1.99.0 e linkerMSVC14.44/SDK10.0.28000 já instalados por paths absolutos e libs explícitas; RUSTUP_AUTO_INSTALL=0. Não altera executor/permissões nem instala ferramentas. Sondagem no sandbox com4unit reais pendente. Testes de falha recebidos foram rejeitados antes de execução root por usar caminhos relativos em vez TempDirectory, namespace incorreto e lógica de fixture incompleta; cópia externa preservada. Quatro unitários/dois físicos E3 do root anteriores continuam válidos, mas afirmação local sobre execução nunca foi aceita.


## FIN-009 — leitura de contas, E3 local em 17/10/2026

Saída local é rascunho até inspeção e execução real. Qwen e Laguna produziram afirmações documentais incorretas; originais preservados e resíduos corrigidos pelo integrador com recibos canônicos. Teste truncado não foi aceito como execução; helper usa compiler/linker instalados por paths absolutos. Modelos/esforços operacionais registrados; custo não registrado.


## FIN-013 — validação pura de entrada, E3 local em 17/10/2026

FIN-013: Gemma terminou vazio; Laguna produziu RED parcial e método; Muse completou limites, com resíduos de tipos corrigidos pelo integrador. Não aceitar exit0/needs_review ou alegação local como execução. Helpers devem distinguir nomes de testes de métodos falsos e normalizar quebras de linha antes de substituições. Rejeitar derives extras usados para mascarar erro de fixture. Código de negócio ficou local; root tratou somente integração/resíduos comprovados. Esforços locais são rótulos operacionais; custo não registrado.
