# SEC-013 — avaliação de SAST útil

## Decisão para o scaffold
Reutilizar Biome2.5.15, já fixado no lockfile e executado em npm run lint/quality. As regras recomendadas noDangerouslySetInnerHtml e noGlobalEval produziram erro em fixtures reais com a configuração atual. Não adicionar scanner/dependência/job por enquanto. Esta decisão técnica vale para os padrões frontend testados; o requisito global SEC-007 continua parcial.

[Spec](../../planning/specs/SEC-013.md) e [harness](../../scripts/test-sast.py) definem configuração, casos e limites. O CI executa o harness antes de npm quality na etapa existente; erro interrompe antes do upload/build. Diferenças documentais continuam dispensando as sete etapas frontend/nativas, pois não mudam código/configuração. Erros e achados do lint real de produção conservam o gate original.

## Prova executada localmente
Root chamou a CLI instalada, sem --only, fixes, suppressões ou execução do código das fixtures. Configuração e package.json foram copiados byte a byte. Apenas VCS foi desabilitado na CLI isolada para o gitignore não ocultar o scratch; includes/regras/domínio React permaneceram iguais. O lint do produto usa VCS e configuração originais.

| Fixture sintética | Resultado observado |
|---|---|
| JSX dangerouslySetInnerHTML | exit1, noDangerouslySetInnerHtml/error |
| React.createElement com a mesma prop | exit1, mesma regra/error |
| eval direto | exit1, noGlobalEval/error |
| window.eval | exit1, mesma regra/error |
| alias inicializado com eval | exit1, referência eval detectada; não prova rastreamento de fluxo |
| texto escapado em JSX | exit0, zero diagnósticos |
| nomes dos padrões apenas em comentário | exit0, zero diagnósticos |
| window[key], key contendo eval | exit0: limite conhecido |
| new Function | exit0: limite conhecido |
| erro de sintaxe | exit1, parse/error |
| arquivo fora dos includes, sob src-tauri | exit não zero, zero arquivos analisados; exclusão, sem alegar cobertura |
| configuração inválida | exit não zero; scanner indisponível/config inválida não vira sucesso |
| guard PowerShell real, retorno filho0/1/2 | sucesso alcança marcador; falhas impedem marcador downstream |

15casos:12invocações lint reais e3PowerShell. Pin CLI/Node/npm conferidos; report exige um arquivo analisado nos casos aplicáveis, zero modificações e diagnósticos completos. Scratch próprio removido com containment/reparse conferidos. Não executar fixtures nem expor dados pessoais; nenhum network nas fixtures. Python otimizado é recusado para não remover asserts.

RED reproduzido: ausência do guard no workflow anterior retornou1/2.031s; diagnóstico focal SAST_CI_GUARD_MISSING. BUILD com guard:15casos/4.312s verdes. Uma hipótese de alias não detectado foi refutada no RECON e o caso corrigido antes do RED de integração. Probes stdin inconclusivos não contam como detecção/RED. Review/CI exactSHA ainda pendentes nesta fase.

## Alternativas examinadas — E2
Fontes oficiais atuais lidas em2026-10-08; nenhuma alternativa instalada, executada ou medida:

| Opção | Utilidade/condição | Decisão atual |
|---|---|---|
| Biome instalado | AST frontend; regras recomendadas e sinal observados com o pin local. [HTML perigoso](https://biomejs.dev/linter/rules/no-dangerously-set-inner-html/javascript/), [eval global](https://biomejs.dev/linter/rules/no-global-eval/javascript/) | reutilizar sem nova ferramenta |
| CodeQL | Suporte atual inclui TypeScript e Rust; análise/extraction/tooling específicos. [Linguagens/frameworks](https://codeql.github.com/docs/codeql-overview/supported-languages-and-frameworks/), [requisitos](https://codeql.github.com/docs/codeql-overview/system-requirements/) | não instalar neste scaffold; custo/tempo/sinal local não registrado |
| Semgrep | Suporte atual inclui Rust; recursos CE/Pro variam, não presumir equivalência. [Linguagens](https://docs.semgrep.dev/supported-languages) | não instalar; ruleset/sinal/custo/licenças operacionais não avaliados |

Inferência técnica: ferramentas adicionais podem ser úteis quando houver entradas e caminhos reais de negócio; a documentação de suporte não prova eficácia no nosso código. A escolha atual evita nova integração remota, sem alegar economia financeira medida.

## Limites e revisão
Não é análise de fluxo/taint completa, prova de ausência de XSS, CSP/runtime/IPC, Rust, SQL/SQLite, dinheiro, transações, backup ou restore. Computed eval e Function são limites observados, não padrões autorizados para produto. Clippy existente não substitui SAST de segurança. Cobertura de testes do scaffold100%/4linhas não mede cobertura de vulnerabilidades nem do harness Python.

Reavaliar ferramenta/ruleset e fixtures quando SEC-014/015 introduzirem sanitização e quando DB-001/BKP-001 ou novos comandos IPC criarem superfícies reais. Cada tarefa mantém testes de invariantes/segurança próprios; este item não libera requisitos globais, E4 ou release. Exceções OSV/Cargo preservadas até2026-10-23T00UTC(22/10 21hBrasília); fora desse escopo/prazo, bloqueio permanece.

## Entrega observada

E3 no SHAa8e02cf7c4938855007dab4a4f3e10f087e335f2/[CI37851893560](https://github.com/millennium42/millani-artes/actions/runs/37851893560):15casos/12CLI/3PS2.547s,configSHA267bcfe6fb64f38c9e3b050adfbb002c143a99b2ce7f9203396cceb77abee1ae/scratchremoved,quality/audits/buildPEAMD64 passaram. Root leu logs/coverage emmemória;100% limita-se4linhasdo scaffold. ReviewE2 semP0P1P2; [spec](../../planning/specs/SEC-013.md#executor--learn--entrega-validada) contém evidências/limites. Nenhuma alternativa instalada, custo/tokens não registrado. FechamentoCI/nota54/liberação ainda pendentes; não amplia escopo de segurança/E4/META.

## Extensão arquitetural — ARC-005

[Spec](../../planning/specs/ARC-005.md) amplia o mesmo [harness](../../scripts/test-sast.py), sem instalar scanner/dependência nem criar job. Biome2.5.15 continua o pin: noRestrictedImports bloqueia drivers/imports/subpaths, noCommonJs rejeita require, e [Grit local](../../scripts/no-ui-sql.grit) gera plugin/error com mensagem SQL_UI_DENIED em strings/templates SQL e comandos plugin:sql|.

RED real: configuração anterior deixou SELECT passar (CLI0); o oráculo recusou a fixture. BUILD local:45casos/42CLI/3PowerShell,8.141s, configSHA84986f2e37c479a77058373fb3b3ed1d104a5da8443213a10ae122bb0014653a e regraSHAe4f5d30e564f95a4540dacf8819bd7328bf7081dac3cfaf8919ec5fe9702bc6e. Preservados15casos anteriores; novos casos verificam SQL em UI/entrypoint/helper, strings/templates/interpolação, drivers e subpaths, require, IPC literal, arquivo limpo, plugin ausente/inválido e cleanup. CLI real, um arquivo analisado por caso elegível, zero modificações, erro/category/message estritos e sem executar fixtures.

Limites observados: concatenação SEL + ECT e escape JavaScript do S passam; Select an account como string JS é recusado conservadoramente, mas JSX textual é permitido. Prefixos, não parser SQL/taint: valores construídos, ofuscados ou prefixados por comentários não têm garantia. SQLite/SQLi/backend/restore permanecem sem prova deste guard. Coverage100% do scaffold não cobre o harness/regra. Review/CI exactSHA/nota61 pendentes; controles globais de segurança continuam parciais.


ARC-005 executor done/E3 após [entrega c5872e20](https://github.com/millennium42/millani-artes/commit/c5872e2026291512fd9eb879e3562200d72b7dd7)/[CI37974060505](https://github.com/millennium42/millani-artes/actions/runs/37974060505), tentativa1/23success.45casos SAST/42CLI/3PS, quality/audits/build Windows aprovados; [prova e limites](../../planning/specs/ARC-005.md#ci--learn--entrega-validada). Fechamento preserva fontes; reserva14 até CI final/nota61 pública. Guard de padrões não comprova SQLi/SQLite/finanças; META ativa/DB001 não iniciado.
