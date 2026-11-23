# ADR-026 — cadastro local de categorias

Status: aceita sob a autorizacao humana explicita para concluir UI-001–UI-016 em 23/11/2026.

## Contexto

CAT-004 fornece o caso de uso create_category e AdapterSqliteCategoryStore, incluindo validacao de dominio e escrita atomica de categoria/audit_event. A UI ainda nao chama esse fluxo. UI-003 ja inicializa o banco local na versao 17 e concede somente create_account a capability local main. Um novo comando requer registrar a permissao exata e serializar escritas concorrentes sobre o mesmo arquivo SQLite.

## Decisoes

- Adicionar apenas o comando de produto create_category, cuja entrada IPC e somente o nome literal; o comando reutiliza o caso de uso e adapter de CAT-004. Retorno e unitario e erros externos sao codigos estaticos.
- Selecionar somente allow-create-category junto da permissao existente allow-create-account na capability main-local-deny, local, para a janela/webview main. Preservar deny-by-default: sem grants core, plugin, filesystem, shell, rede, dialog ou updater.
- Inicializar CategorySetupState no mesmo arquivo app_local_data fixo e usar a inicializacao/migracoes v1–v17 ja existentes, extraindo um helper compartilhado de conexao em vez de duplicar a sequencia. Nenhuma migration nova.
- Reutilizar Clock/UuidGenerator de sistema existentes, fazendo apenas as mudancas de visibilidade interna necessarias. Nao aceitar caminhos, IDs, horario, saldo ou SQL do frontend.
- Compartilhar uma trava de processo entre comandos que gravam nesse arquivo para serializar as transacoes BEGIN IMMEDIATE de conta e categoria. Novos comandos de escrita local devem reutilizar essa trava.
- Em falha de persistencia, o adapter consumido e substituido por um novo aberto pelo inicializador compartilhado; somente o erro estatico e devolvido.
- O formulario pode criar e repetir categorias; nao lista, renomeia, desativa, apaga ou define politica de selecao. A UI nao duplica validacao alem de required HTML e nao altera o nome enviado.
- Reutilizar React, Tauri, estilos e ferramentas de teste instalados. Nenhuma nova dependencia, plugin, framework de rotas, design system ou capability ampla.

## Alternativas consideradas

- Grant de comandos ou permissao filesystem/core mais ampla: rejeitado por exceder a necessidade do formulario.
- Reimplementar validação, IDs, relogio ou SQL na UI: rejeitado porque duplicaria regra/domain e exporia detalhe de armazenamento.
- Copiar a rotina de migracoes para outro modulo: rejeitado por criar dois caminhos de inicializacao sujeitos a drift.
- Deixar duas conexoes gravarem sem coordenacao: rejeitado porque nao ha busy_timeout configurado e ambas escrevem no mesmo arquivo.

## Consequencias e riscos

O cadastro passa pela UI real ate o caso de uso e SQLite; categoria e auditoria continuam atomicas. Categoria e conta usam conexoes distintas mas a trava compartilhada serializa chamadas dos comandos de produto. A UI mostra mensagem generica e permite nova tentativa depois da recuperacao do adapter. O schema fica inalterado. Testes de componente nao provam IPC; testes Tauri de autoridade e jornada nativa isolada sao necessarios. A inspeção visual exige executavel de producao e captura automatizada sem CUA. Nenhuma alegacao de WCAG geral ou de instalador/produção e adicionada.

## Gatilhos de revisao

Revisar este ADR quando houver selecao/listagem de categorias, outras janelas/origens, comandos de escrita paralelos, migracao ou mudança do arquivo local. A revisao e liberação de UI-004 exigem review adversarial, checks e CI remota no SHA exato, e nota release_evidence/released com mappings preservados.
