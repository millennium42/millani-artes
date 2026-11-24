# Política Tauri capabilities

Começar deny-by-default. Não habilitar `shell`, rede, updater ou filesystem arbitrário por conveniência. Dialog e filesystem futuro devem limitar comandos, janelas e diretórios necessários; cada capability referencia um `SEC-*`, ADR e teste. CSP bloqueia conteúdo remoto por padrão.

## SEC-001 — baseline do scaffold sem comandos IPC
A evidência SEC-001 registra o scaffold de 2026-10-05: naquele baseline, `main-local-deny` era local, selecionava `main` e tinha `permissions` vazias; o app ainda não expunha comandos de produto. Isso não descreve a configuração atual, que evoluiu com os grants mínimos de UI-003–UI-006 documentados abaixo. O scaffold não concedia `core:default`, shell, HTTP, filesystem, dialog ou updater. A seleção explícita evita habilitar automaticamente capabilities de arquivos futuros; mudanças exigem review e teste.

[Tests](../../src-tauri/src/security_tests.rs) exercitam o seletor da versão pinned com capability extra não selecionada e a autoridade compilada. O probe Windows/WebView2 separado usa assets de produção: carrega a tela local, solicita set_title e emit pelo IPC JavaScript real e verifica negação ACL, título intacto e ausência de evento no listener Rust. O endpoint de relatório existe somente em `cfg(test)`; no scaffold de 2026-10-05, `main` não registrava `invoke_handler`. Os comandos de produto atuais estão registrados e limitados conforme os registros UI-003–UI-006 abaixo. Janela oculta, perfil isolado ignorado e timeout de 30s; nenhum mock prova esse limite.

Após ativar Node/Rust conforme [reprodução](../operations/REPRODUCING.md), gerar dist e rodar:

~~~powershell
rtk proxy npm run build
rtk proxy cargo test --locked --manifest-path src-tauri/Cargo.toml
rtk proxy cargo test --locked --manifest-path src-tauri/Cargo.toml --features tauri/custom-protocol local_page_works_and_denied_ipc_has_no_effect -- --ignored --nocapture --test-threads=1
~~~

O terceiro comando requer Windows/WebView2 e ausência do perfil artifacts/sec001-webview-profile. Limpar somente esse absoluto validado, após encerramento do teste, com Remove-Item LiteralPath em PowerShell; rejeitar reparsepoints na raiz/ancestrais/árvore antes excluir. Não apagar perfil pessoal para repetir o teste. CI atual compila o produto, sem executar esse probe ignored; execuções reais e limites no [handoff SEC-001](../../planning/handoffs/SEC-001.md).

Capabilities governam comandos expostos core/plugins, não código Rust arbitrário nem todas as APIs do navegador. Comandos custom registrados por invoke_handler são permitidos por padrão sem AppManifest::commands; os futuros devem ser gated explicitamente/testados, não herdar uma alegação de negação desta configuração. CSP é a tarefa SEC-002; permissões de plugins/directórios/backup e seus positivos continuam SEC-004/005/BKP-004/UI-017. [Tauri](https://v2.tauri.app/security/capabilities/) documenta seleção, união de permissões e essa separação. Nenhuma alegação de segurança total ou scanner completo.

## SEC-002 — CSP de produção comprovada no scaffold
A CSP existente permanece intacta: fontes locais/IPC, imagens locais/data e frame/object/form/base restritos. devCsp mantém exceções locais HMR/style; o teste de fontes parsed permite só essas classes e rejeita wildcard/hosts remotos/unsafe-eval/inline de produção. Hashes/nonces locais são inseridos pelo [Tauri](https://v2.tauri.app/security/csp/).
O mesmo [harness](../../src-tauri/src/security_tests.rs) agora exige custom-protocol/is_dev=false, React/CSS local,5eventos securitypolicyviolation enforce com URI do coletor para script/style/img/frame/connect e nenhuma requisição HTTP recebida por std TCP127.0.0.1:0. Um preconnect TCP sem HTTP foi observado; CSP não é firewall. Eval nativo do probe é confiável e não comprova que eval de conteúdo arbitrário é permitido/negado.
Executar a suíte completa após gerar dist e verificar ausência do perfil/fixture guardados:
~~~powershell
rtk proxy cargo test --locked --manifest-path src-tauri/Cargo.toml --features tauri/custom-protocol -- --include-ignored --nocapture --test-threads=1
~~~
Root executou5Rustpassed0ignored/CSP5eventos/HTTP0/React-CSS-IPC positivos; controle csp=null recebeu5HTTP e reprovou mesmo oráculo, restaurado byte a byte. Cleanup perfil controlado conforme procedimento acima; não apagar perfil pessoal. Resultado/ambiente/checks no [handoff SEC-002](../../planning/handoffs/SEC-002.md). CI compila produto, não executa este probe ignored. Prova representa cinco vetores cross-origin loopback com assets produção em testbin debug; HMR/novos plugins/customcommands/backup/finanças/setup.exe permanecem próprios executores.

## SEC-004 — inventário de plugins e necessidade do MVP

**Lista de plugins necessários ao scaffold: vazia.** Nenhum plugin está declarado nos manifests/lockfiles consultados ou registrado no Builder de produto. Os comandos Tauri custom de UI-003–UI-006 são grants explícitos de produto, não plugins; nenhum grant `core:default` ou permissão de plugin foi adicionado por essa evolução. `tauri`, `tauri-build` e `@tauri-apps/cli` são core/build/CLI, não plugins. Fontes: [Cargo](../../src-tauri/Cargo.toml), [lock Rust](../../src-tauri/Cargo.lock), [npm](../../package.json), [lock npm](../../package-lock.json), [Builder](../../src-tauri/src/main.rs) e [config](../../src-tauri/tauri.conf.json). Presença transitiva de uma biblioteca também não é registro/autorização de plugin.

A lista atual não dispensa SQLite, backup criptografado, restore validado ou uma interface acessível. Não há conjunto obrigatório de plugins aprovado para o MVP completo; as escolhas abaixo continuam nos executores de cada função.

| Necessidade / plugin considerado | Decisão mínima nesta lista | Executor / fonte |
|---|---|---|
| Desktop, assets locais e instalador | Core/CLI existentes; nenhum plugin adicional necessário para o scaffold. Instalador ainda pendente. | ADR001/002; REL-* |
| SQLite / `sql` | Não selecionar plugin SQL por usar SQLite. Acesso permanece no repositório/core; biblioteca será decidida na tarefa DB. Não expor consultas genéricas ao frontend. A API [SQL](https://v2.tauri.app/plugin/sql/) oferece consultas JavaScript, o que exige avaliação distinta das regras de produto. | [Fronteiras](../architecture/MODULE_BOUNDARIES.md), ADR003/004/017; ARC-001, DB-001 |
| Arquivos e backup / `fs` | Não selecionar plugin filesystem para IO no core. Preferir stdlib; [Tauri](https://v2.tauri.app/plugin/file-system/) orienta usar bibliotecas Rust para essa IO. Validação de paths, reparse points, formato, staging e promoção continua obrigatória; código Rust não fica protegido por uma capability IPC por inferência. | BKP-004/008/009/010–014; [segurança de backup](BACKUP_SECURITY.md) |
| Seleção nativa de arquivo / `dialog` | Candidato condicionado ao fluxo real de backup/restore. Comparar solução nativa/plataforma e acessibilidade antes de escolher. [Dialog](https://v2.tauri.app/plugin/dialog/) fornece seleção abrir/salvar e retorna paths no Windows; selecionar arquivo não o torna confiável, nem autoriza IO irrestrita ou dispensa validação no core. Não instalado/aprovado aqui. | UI-017; BKP-004/008/009; ADR021 |
| Diagnóstico / `log` | Não selecionar plugin de log para um emissor ainda inexistente. Qualquer emissor deve cumprir allowlist/fallback da política, inclusive erros e sinks reais; plugin não garante sanitização. | [Logs](LOGGING_POLICY.md); SEC-014/015, BKP-003 |
| Crypto/chave / `stronghold` ou outro | Nenhum plugin/biblioteca escolhido. A POC compara mecanismo nativo e solução auditada, recuperação e limites; não resolver segurança de chave por nome de plugin. | ADR015/016; BKP-002/003; [segredos](SECRET_MANAGEMENT.md) |
| Persistência genérica / `store` | Não selecionar para dados financeiros; SQLite continua fonte única. Configuração futura precisa de contrato próprio, sem segundo banco autoritativo. | ADR003/004; ARC-003 |
| `shell`, `http`, `updater`, `opener`, `autostart`, notificações e outros | Não necessários ao scaffold e sem necessidade aprovada nesta lista. Não ativar recursos por conveniência; nova necessidade requer reavaliação. | [Escopo MVP](../product/SCOPE.md), ADR005/007/021 |

Antes de qualquer adoção, registrar necessidade/alternativa, ADR específico e [tool register](../../governance/TOOL_REGISTER.md), versões/locks, dados envolvidos, janela/comando/path mínimo, risco e teste real do fluxo permitido e do proibido. Reavaliar a seleção explícita, união de permissões e comandos custom; não herdar o aceite limitado da SEC-001. O gate estático SEC-005 está descrito abaixo; este inventário SEC-004 não prova execução do gate nem segurança runtime.

Inventário/revisão documental E2 e checks documentais E3 são distintos de segurança runtime. [Spec](../../planning/specs/SEC-004.md)/[handoff](../../planning/handoffs/SEC-004.md) registram execução e CI no SHA; requisitos globais SEC-001/008 permanecem parciais. O requisito SEC-004 trata arquivos externos; o executor SEC-004 trata esta lista de plugins.

## SEC-005 — gate estático de ADR por plugin

[Registro](TAURI_PLUGIN_REGISTER.json) vazio: nenhum plugin aprovado. [Check](../../scripts/check_tauri_plugins.py) e [Fixtures](../../scripts/test-tauri-plugins.py) usam Python≥3.12/stdlib TOML/JSON, sem instalar pacote. Cargo direto/target/build/dev/workspace-root e lock, npm deps/aliases/lock, config/capabilities declaradas e registros Rust identificáveis exigem associação por identidade de pacote. Pacotes com naming Tauri convencional são reconhecidos; scopes plugin-* tratados conservadoramente. Core/CLI não são plugins. Pacote transitivo reconhecido também necessita documentação, sem presumir registro/permissão.

Formato do registro: objeto JSON identidade→path relativo canônico docs/architecture/adr/ADR-NNN-nome.md. ADR deve conter linhas exatas Status: accepted e Plugin-package: identidade. ADR021 genérico não serve. Duplicatas, arquivo inválido/ausente, referência externa/traversal/escape de symlink, status pending e mismatch falham sem imprimir conteúdo. Uma adoção precisa ainda de tool register, locks, necessidade, escopo e testes reais; status é declaração sujeita a review, não E4.

Chamadas Rust .plugin/.plugins são inspecionadas conservadoramente: forma direta tauri_plugin_nome::... identificável exige ADR; variável/custom/batch não identificável falha. Não é parser Rust: comentários/strings podem gerar falsa rejeição, macros/include/codegen/nomes não convencionais exigem extensão e prova antes da adoção. Não inferir segurança de Rust arbitrário, ACL ou comportamento runtime a partir deste gate. [Spec](../../planning/specs/SEC-005.md)/[handoff](../../planning/handoffs/SEC-005.md) registram execução.

Rodar python -B scripts/test-tauri-plugins.py e python -B scripts/check_tauri_plugins.py; o mesmo step CI de schema/evidência executa ambos antes seleção/build e verifica o exitcode explicitamente. Nenhum job/action/setup/cache novo. CI/nota final ainda pendentes.

## UI-003 — comando local para cadastrar conta (candidato local; release pendente)

A configuração seleciona main-local-deny com local=true, webviews=[main] e apenas allow-create-account; tauri.conf.json declara uma janela chamada main. A permissão selecionada autoriza somente create_account. A allowlist de build inclui `security_probe_report` para compilar o harness e gera sua definição de permissão; porém, a capability do produto não seleciona `allow-security-probe-report` e o handler de produto registra somente `create_account`. O builder do probe adiciona a capability do reporter apenas no teste. Assim, a definição gerada não é um grant ativo nem expõe o reporter no produto. Não há plugin nem permissão core:default, filesystem, shell ou rede.

Na versão Tauri fixada, correspondências de windows e webviews dentro de uma capability são alternativas OR, não uma interseção. A capability atual só enumera o webview main, e o app configura uma única janela/webview main; windows=[] não é uma segunda restrição por interseção. O teste verifica a origem local no par main/main e rejeita origem remota, labels não correspondentes e comando não autorizado. Se a configuração ganhar outras janelas ou webviews, a política e os testes precisam ser revistos: um webview com label main satisfaz a seleção mesmo que um futuro window label seja diferente.

O teste unitário de ACL passou dentro de 5 testes de segurança. O probe WebView2 que exercita a negação IPC real permanece marcado `#[ignore]`, portanto não é executado na suíte padrão/CI por exigir Windows/WebView2 e perfil isolado; foi selecionado explicitamente pelo root em 23/11/2026 e passou 1/1 depois da correção da sentinela. Registrou a marca carregada, `set_title` e `event|emit` negados sem efeito e os checks CSP descritos na matriz. A validação é focal, não prova segurança geral do WebView, IPC futuro ou configuração de runtime arbitrária. UI-003 só muda para released após CI final-main no SHA exato e nota lida de volta com mappings preservados.

### UI-003 — reconciliação do estado em 23/11/2026

O parágrafo de UI-003 acima é um snapshot anterior à liberação. A nota remota `released` em `refs/notes/evidence` no SHA final-main `05190e0f404a6865329fa4ceb3698025b5c727cf` confirma a liberação do status-sync, com CI final-main `45168644799` e mappings remotos preservados. O produto ainda concede somente `allow-create-account`; não inferir capabilities de versões candidatas.

### UI-004 — capability de categoria (candidato, sem liberação)

Nesta branch, a capability local `main-local-deny` seleciona exatamente `allow-create-account` e `allow-create-category` para o webview/janela local `main`. `create_category` recebe somente `name` e usa o caso de uso existente; nenhuma permissão core/plugin, filesystem, shell, rede, outra origem ou janela foi concedida. `build.rs` e a permissão gerada `permissions/autogenerated/create_category.toml` definem o grant estreito. `security_tests.rs` prova o par local main/main permitido e nega origem remota, labels divergentes e comandos de leitura/edição/desativação/exclusão não autorizados. A decisão está em [ADR-026](../architecture/adr/ADR-026-ui004-category-setup.md). Estado do candidato: in_progress/E3/passes:false; review exata, CI final-main e nota remota de release ainda pendem.

### UI-004 — recibo atual (23/11/2026)

O parágrafo anterior é um snapshot candidato anterior à liberação. A nota remota `released` no SHA final-main `0d5846231b49b80ed81b4a599873556c9ae27615`, CI final `45198674102`, confirma a liberação de UI-004. A capability atual ainda concede somente `create_account` e `create_category`; nenhum comando de UI-005 está ativo nesta base.

### UI-005 — contrato aceito; implementação pendente

[ADR-027](../architecture/adr/ADR-027-ui005-entry-form.md) aceita, sob autorização humana para concluir UI-001–UI-016, a adição de apenas `list_accounts` e `create_entry` à capability local `main-local-deny`, com permissão gerada explícita por comando. O leitor deve retornar somente ID de 16 bytes e nome; a escrita recebe ID, string de centavos e contexto, chama FIN-017 e reutiliza a trava/processo/SQLite existentes. A capability deve selecionar a origem local e `webviews=["main"]`, sem um seletor de janela adicional: esta política documenta que os seletores de window/webview da Tauri são OR, não interseção. O app atual declara um único par janela/webview `main`; não se afirma que a capability, isoladamente, negue uma janela futura independentemente. A decisão não descreve grants ativos na base atual: `tauri.conf.json`, `build.rs`, capability, permissões geradas e testes de autoridade ainda precisam ser alterados e verificados na fatia UI-005. Rever a política se surgir outra janela/webview. Não são autorizados permissões core/plugin, origem remota, filesystem, SQL, migration ou novos dados.

## UI-005 — implementação candidata E3 (23/11/2026)

Esta branch implementa o contrato aceito em ADR-027: mantém os grants existentes `create_account`/`create_category` e acrescenta somente `list_accounts`/`create_entry` à capability local `main-local-deny`, com permissões geradas por comando, `local=true` e `webviews=["main"]`. O manifesto Tauri registra explicitamente os comandos. `list_accounts` devolve apenas ID de 16 bytes e nome; `create_entry` recebe ID, string decimal de centavos e contexto e reutiliza FIN-017 e a trava compartilhada. Não há capability core/plugin, grant remoto, SQL/filesystem no frontend ou dado financeiro adicional.

Prova local E3 e checks estão em [handoff UI-005](../../planning/handoffs/UI-005.md). A autorização da capability está nesta branch candidata, mas ainda não no main vigente; estado `in_progress/E3/passes:false` até review/CI/nota released. O seletor continua limitado à topologia atual de um par `main`; seletores Tauri de janela/webview são OR. Rever se essa topologia mudar.


## UI-005 — recibo remoto released (23/11/2026)

Os parágrafos anteriores preservam snapshots candidatos de UI-005. A nota remota refs/notes/evidence no SHA final-main 671d9ab0cc0bb8f206bfdffcabd481f83642e4db marca UI-005 released após source CI 45233997098 e final-main CI 45236415336 success 23/23. A capability ativa em main permite os grants explícitos de UI-003/004/005 registrados no recibo; não repetir UI-005.

## UI-006 — capability mínima liberada pelo recibo

ADR-028 foi implementada para UI-006: somente create_exit/allow-create-exit foi adicionado ao manifest e build.rs; a capability main-local-deny continua local e limitada ao webview main, com a descrição incluindo saída. A UI reutiliza list_accounts; o comando recebe `tauri::ipc::Request`, valida o objeto e mapeia tipos inesperados a erro estático antes de chamar FIN-018 com ID, centavos textuais e contexto sob a trava compartilhada. Testes locais verificam grant local autorizado, origem remota/webview não selecionado e comandos excedentes negados, corpo JSON válido atravessando parser com persistência/audit, além de tipo IPC sensível com erro estático e lock poisoned sem escrita. Scans pinned locais passaram no SHA de código `6af29e42d72b1046525e30ebfea7df20701f7912`; CI source `45275229408` e final-main `45278077539` passaram 23/23 sem skips nos SHAs exatos. O recibo remoto `release_evidence/released` foi lido de volta no SHA final-main `b49c29fe02faf7278a86d88421e83ef65d34e23b`, nota blob `95d594717800c3c9a672ba8f8010e455b259202e`, mappings 230→231; o status-sync PR #93 passou source/final-main CIs `45286352431`/`45286756420` (16/23, sete passos produto/build ignorados). A topologia atual tem um par main; seletores de janela/webview Tauri são OR, sem alegar restrição independente de janela futura. Não concede permissão core/plugin, remota, filesystem ou SQL.
