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
