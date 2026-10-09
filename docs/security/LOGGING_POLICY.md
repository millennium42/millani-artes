# Política de logs sanitizados — SEC-003

## Limite e estado
O [requisito SEC-003](SECURITY_REQUIREMENTS.md) proíbe dados pessoais, financeiros e segredos em logs. Esta política rege logs e relatórios **diagnósticos** do frontend/core/plugins, console, erros/panics e pacotes de suporte; a regra também vale antes de persistir ou compartilhar saídas de ferramentas e evidências. Não existe exceção por nível debug, ambiente local ou destino temporário.

Ela não modifica o histórico financeiro ou o audit_event no SQLite. [Validação, escrita e evento técnico](../architecture/TRANSACTION_MODEL.md) continuam na mesma transação (DB-013/014); log diagnóstico não substitui auditoria nem gravação autoritativa.

SEC-003 define a política. [SEC-014](../../planning/specs/SEC-014.md) implementa a fronteira de inicialização e o hook Rust no [main real](../../src-tauri/src/main.rs): registros estáticos, sem payload. Cinco filhos capturados (quatro casos de privacidade + guard contra falha anterior à injeção), com Tauri build real/Err sintético na fronteira/panic/pipe fechado, provaram localmente ausência de telefone sintético. Revisão E2 sem achados após correção P2; entrega E3/CI37857821138 verde no SHA55b481af4d394aa624d384b6d38887bbdf898a04. CI não executa a suite Rust; fechamento/nota55 ainda pendentes. SEC-015 comprovou E3 de dados financeiros sintéticos na mesma fronteira, com revisãoE2/fontes e CI37860308161 de entrega verde no SHA96c06e66e53f8c9776d51576bf7f467102487ad2, conforme seção abaixo; liberação final pela nota56 pendente. Enforcement global permanece parcial: fluxos financeiros reais, [backup](BACKUP_SECURITY.md), plugins/emissores futuros e demais caminhos exigem provas próprias conforme [roadmap](../../planning/ROADMAP.md)/[matriz](SECURITY_TEST_MATRIX.md). Mensagem JS estática e configuração isolada não provam esses caminhos.

## Construir um registro permitido

ARC-001 move a fronteira/hook para [application](../../src-tauri/src/application/mod.rs) e o catálogo/emissor para [infrastructure](../../src-tauri/src/infrastructure/mod.rs), compostos pelo main. Os bytes estáticos, descarte do erro opaco, status e tratamento de sink fechado são preservados; [spec](../../planning/specs/ARC-001.md) registra a regressão real. Failure continua um enum técnico de inicialização, sem definir erro de domínio financeiro. Esta reorganização não amplia a prova para emissores futuros.

Criar um objeto novo a partir desta lista de campos. Não serializar o objeto original nem tentar retirar apenas chaves conhecidas depois. Dados de usuário, SQLite, plugins, OS e exceções de terceiros são entradas não confiáveis.

| Campo permitido | Valor permitido |
|---|---|
| level | enum fixo info, warn ou error |
| operation | enum técnico fixo definido na spec do emissor, como startup ou backup_create; nunca rótulo/ID inserido pela usuária |
| code | código estático do catálogo do emissor, como BACKUP_IO ou INTERNAL_ERROR; nunca texto bruto de erro |
| app_version (opcional) | versão fixa do build, sem metadados livres |
| time_utc (opcional) | instante UTC gerado/formatado pelo sistema, sem texto de entrada |

Campos desconhecidos são omitidos. Valores malformados não entram no registro: preservar operação/código seguros quando conhecidos e usar código estático de diagnóstico inválido/falha interna quando necessário, sem copiar o valor rejeitado. Validar tipos/formato/limites do catálogo antes emitir; rejeitar CR, LF, delimitadores ou caracteres de controle em valores e usar serialização estruturada adequada ao sink. Não criar campo message de texto livre; eventual texto legível vem de catálogo estático pelo code.

Exemplo sintético permitido, não implementação nem captura de runtime:

~~~json
{"level":"error","operation":"backup_create","code":"BACKUP_IO","app_version":"0.1.0"}
~~~

A classificação técnica mantém o diagnóstico útil sem expor nome/arquivo/conteúdo do backup. Falha em sanitizar/serializar/gravar diagnóstico nunca recorre a Debug/Display, console, stack ou erro original: preservar somente código estático seguro se o sink funcionar; descartar o conteúdo inseguro. Falha do sink não transforma uma falha de negócio em sucesso, não muda resultado/rollback e não grava parte da operação financeira por fora. Falha de audit_event SQLite segue o modelo transacional, não este fallback de diagnóstico.

## Conteúdo proibido em qualquer campo
- Nome, telefone, e-mail, endereço, identificador de pessoa, nome de usuário do computador e entrada livre da usuária.
- Valores, saldos, preços, quantidades, contas/categorias/contextos de negócio, IDs de clientes/vendas/movimentos e agregados derivados dos registros financeiros.
- Senha, token, chave de backup, credencial, connection string, cookies ou variáveis de ambiente com segredos.
- Payload/objeto/modelo completo, SQL/parâmetros/linhas, conteúdo/metadados sensíveis de banco/backup, caminho absoluto ou nome de arquivo inserido pela usuária.
- Mensagem/cause/stack/debug brutos de erro/plugin/OS/Rust/JavaScript, inclusive erro aninhado, interpolação, assertion e panic com esses valores.

Hash, mascaramento parcial ou últimos dígitos não autorizam incluir os dados acima. Uma regex de telefone ou scanner de segredos sozinho não atende a política: dados podem vir aninhados, reformatados, em erro, SQL ou campo desconhecido.

## Destinos e evidência
Não ativar persistência, upload, telemetria ou exportação de diagnóstico neste scaffold. Caso um executor futuro precise de armazenamento local, sua spec deve definir diretório controlado, acesso restrito, limites de tamanho/retenção/limpeza, formato e testes de falha antes habilitar; não registrar conteúdo extra para diagnóstico de falha. Compartilhamento solicitado exige relatório já sanitizado, sem logs brutos/dump/banco/backup reais.

Fixtures, captura de teste, review, CI, Git e pacotes de suporte usam somente dados sintéticos e o resumo necessário. Não publicar o marcador sensível real para demonstrar que foi removido. Dados financeiros da usuária não são fixture de segurança. O evento técnico financeiro canônico segue seu contrato próprio e não é exportado como relatório diagnóstico.

## Gate de implementação
Os executores devem capturar os sinks reais de sucesso e erro, incluindo propagação frontend-core/terceiros e falha de backup, com marcadores sintéticos de telefone, finanças, segredo, paths e objetos aninhados. Exigir ausência dos marcadores e de dados derivados em todos os outputs, preservando operação/code seguro; testar campo desconhecido, Unicode, CR/LF, erro bruto, serialização/sink indisponível e fallback. O banco ativo deve manter o comportamento previsto de sucesso/erro; logging não reescreve estado.

[SEC-014/015 e BKP](../../planning/ROADMAP.md) precisam de spec/código/testes/comandos/ambiente/SHA e CI para E3; mock ou teste isolado do sanitizador não prova todos os caminhos. Audits/Gitleaks não substituem essa execução. O [handoff SEC-003](../../planning/handoffs/SEC-003.md) registra os checks documentais e os limites, mantendo enforcement E0.

Referência externa lida em 2026-10-07: [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html#data-to-exclude) recomenda excluir dados sensíveis, validar eventos, prevenir injeção em logs e tratar falhas do logger sem vazamento. A allowlist e a proibição mais restrita acima são decisões deste projeto, subordinadas ao produto/ADRs.


## SEC-014 — catálogo da inicialização e hook Rust
Emissor privado recebe somente enum fechado; ambos os registros são bytes JSON fixos, sem campos derivados do erro. Startup retornado falho produz level:error/operation:startup/code:TAURI_STARTUP_FAILED e exit1. Hook Rust instalado antes do builder produz level:error/operation:runtime/code:INTERNAL_PANIC, omitindo payload/localização/backtrace; panic mantém falha. Sucesso não emite erro. Escrita stderr é fallible; erro do sink descarta somente registro seguro, sem Debug/Display/fallback original/sucesso falso.

[Tests reais](../../src-tauri/src/startup_tests.rs) capturam stdout/stderr/status de quatro processos filhos do binário de teste. TauriBuilder real inicializa sem janelas; callbackcfg(test) injeta tauri::Error::Io com dados sintéticos na mesma fronteira usada pelo main. Isso comprova Err/emissão/status nos sinks reais, não uma falha natural do Builder.run ou todos os caminhos da biblioteca. Nenhum plugin/dependência/grant. Variante aninhada/Unicode/CRLF/formatos telefone é opaca: nunca serializada para o sink. O controlador fecha stderr antes da barreira stdin para exercitar indisponibilidade real. Capturas limitadas16KiB/canal, timeout20s/kill-wait do próprio handle, sem publicar bytes brutos. Catch_unwind é somente controlador de teste, não tratamento de negócio.

Esta prova cobre erro retornado/hook Rust do scaffold atual, não toda emissão de terceiros, panic nativo/C++/FFI, futuro override de hook, frontend, finanças/SQLite/backup/restore. GUI Windows pode não ter stderr; código seguro é descartado se o sink falhar. Não habilitar arquivo/telemetria nem prometer UX de erro. Provas/review/CI/sourcehash ficam na [spec/handoff](../../planning/handoffs/SEC-014.md); requisito SEC-003 global continua parcial.

SEC014 VERIFYlocal:fixture final faultInjectionErrorIo pela fronteira do main,4filhos e6Rusttests0ignored/fullfmtclippy/audit/quality green. Nenhum plugin/helperdeprodução configurável, checkSEC0050. Limite: não simula falha natural do Builder.run ou todasemissões internas. Sourcehash/testcases eCI-review ainda pendientes na [spec](../../planning/specs/SEC-014.md#verify-final-local); requisitosfinanceirobackup/segurança global permanecemparciais.

SEC014 VERIFY final apósP2: cinco filhos (quatro privacidade + guard before-injection102), confirmação AtomicBool após build real impede falso positivo; seis Rusttests/zeroignored/fmtclippy green. [Prova e hashes finais](../../planning/specs/SEC-014.md#verify--correção-p2-executada). E3local/re-reviewCIpending/11ATIVA; fronteira Err com faultinjection, não falha natural da biblioteca; METAativa.

SEC014 executor done/passestrue/E3 após entrega55b481af4d394aa624d384b6d38887bbdf898a04/[CI37857821138](https://github.com/millennium42/millani-artes/actions/runs/37857821138)/attempt1/23success. Cincofilhos/seisRustlocal/P2corrigido/re-reviewsemachados; [prova/limites](../../planning/specs/SEC-014.md#ci--learn--entrega-validada). Fechamento9docs/reserva11ATIVAatéCIfinalnota55refsGitclean; SEC003global parcial/METAativa/SEC015nãoiniciado.

## SEC-015 — dados financeiros na fronteira atual

[Spec](../../planning/specs/SEC-015.md) →[tests reais compartilhados](../../src-tauri/src/startup_tests.rs) →[handoff](../../planning/handoffs/SEC-015.md). E3local: RED de sensibilidade expect antiga detectou vazamento financeiro, restauraçãofinally; cinco filhos financeiros/cincophone, seteRust0ignored/fmtclippyaudit green. Main produtivo inalterado. Centavos/saldos/preços/quantidade/conta/categoria/contextos/IDs/agregados e formato/UnicodeCRLF/nestedunknown sintéticos ausentes de stdoutstderr, saída e status estritos/injeção pósbuildreal/guard102. Não simula falha natural do Builder.run nem prova transação financeira/SQLite/backup/all-emitter/FFI; requisitoSEC003global continua parcial. Review/CI/nota56-liberação pendentes/passfalse/11ATIVA/METAativa/ARC001nãoiniciado.

SEC015 executor done/passestrue/E3 após entrega96c06e66e53f8c9776d51576bf7f467102487ad2/[CI37860308161](https://github.com/millennium42/millani-artes/actions/runs/37860308161)/attempt1/23success; [spec/prova/limites](../../planning/specs/SEC-015.md#ci--learn--entrega-validada). Cincofinance+cincophone/seteRustlocal/productionunchanged/reviewE2fontessemachados; CI não executaRustsuite. Reserva11ATIVAatéCIfinalnota56/refsGitclean/METAativa/ARC001nãoiniciado; globalSEC003parcial.

ARC-001 executor done/E3/passes:true: [entrega](https://github.com/millennium42/millani-artes/commit/e94eec776213725f72239c8d07aff2a293f6f632)/[CI37863336200](https://github.com/millennium42/millani-artes/actions/runs/37863336200) success23steps/buildWindows, std-only6controles e7Rust locais/reviewE2fontes semachados. Domínio sem regra; finanças/SQLite/backup/setup.exe pendentes/METAativa. Reserva22ATIVA atéCI final/nota57, fechamento11docs; detalhe na spec/handoffARC001. ARC002 não iniciado.

ARC002: [DomainError](../../src-tauri/src/domain/mod.rs) contém somente InvalidInput sem payload/causa, Display código fechado e Debugnome fixo. Compilernegatives rejeitam texto e From<String>; isso não transforma erros externos em seguros, nem autoriza imprimir Debug/Display do erro original capturado. Não há logging/IPC produtor novo; emit_failure bootstrap e seu catálogo permanecem byte a byte. Provas/limites na [spec](../../planning/specs/ARC-002.md); executor E3/entrega93362971/CI37865774054, fechamento documental reserva17. Enforcement global permanece parcial.
