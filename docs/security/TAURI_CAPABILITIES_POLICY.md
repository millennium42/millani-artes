# Política Tauri capabilities

Começar deny-by-default. Não habilitar `shell`, rede, updater ou filesystem arbitrário por conveniência. Dialog e filesystem futuro devem limitar comandos, janelas e diretórios necessários; cada capability referencia um `SEC-*`, ADR e teste. CSP bloqueia conteúdo remoto por padrão.

## SEC-001 — scaffold sem comandos IPC
A seleção em [tauri.conf.json](../../src-tauri/tauri.conf.json) contém apenas a capability inline main-local-deny, local, janela main, permissions vazias. Nenhuma permissão core:default, shell, HTTP, filesystem, dialog ou updater é concedida. A tela atual usa HTML/React local e não precisa de IPC. Seleção explícita evita habilitar automaticamente capabilities de arquivos futuros; arquivo/config adicional via build ou alteração da seleção exige review e teste.

[Tests](../../src-tauri/src/security_tests.rs) exercitam o seletor da versão pinned com capability extra não selecionada e a autoridade compilada. O probe Windows/WebView2 separado usa assets de produção: carrega a tela local, solicita set_title e emit pelo IPC JavaScript real e verifica negação ACL, título intacto e ausência de evento no listener Rust. Endpoint de relatório existe somente no módulo cfg(test); main de produto não registra invoke_handler. Janela oculta, perfil isolado ignorado e timeout de30s; nenhum mock é prova desse limite.

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

**Lista de plugins necessários ao scaffold: vazia.** Nenhum plugin está declarado nos manifests/lockfiles consultados ou registrado no Builder de produto; a capability selecionada continua com permissions vazias. `tauri`, `tauri-build` e `@tauri-apps/cli` são core/build/CLI, não plugins. Fontes: [Cargo](../../src-tauri/Cargo.toml), [lock Rust](../../src-tauri/Cargo.lock), [npm](../../package.json), [lock npm](../../package-lock.json), [Builder](../../src-tauri/src/main.rs) e [config](../../src-tauri/tauri.conf.json). Presença transitiva de uma biblioteca também não é registro/autorização de plugin.

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

Antes de qualquer adoção, registrar necessidade/alternativa, ADR específico e [tool register](../../governance/TOOL_REGISTER.md), versões/locks, dados envolvidos, janela/comando/path mínimo, risco e teste real do fluxo permitido e do proibido. Reavaliar a seleção explícita, união de permissões e comandos custom; não herdar o aceite limitado da SEC-001. A tarefa SEC-005 implementará o bloqueio de plugin sem ADR; esta lista não prova esse gate.

Inventário/revisão documental E2 e checks documentais E3 são distintos de segurança runtime. [Spec](../../planning/specs/SEC-004.md)/[handoff](../../planning/handoffs/SEC-004.md) registram execução e CI no SHA; requisitos globais SEC-001/008 permanecem parciais. O requisito SEC-004 trata arquivos externos; o executor SEC-004 trata esta lista de plugins.
