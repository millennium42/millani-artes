# ADR-030 — cadastro de produto pela interface

Status: aceita sob autorização humana explícita para desenvolver UI-008–UI-016. Esta decisão limita a integração de UI-008; não comprova implementação nem liberação.

## Contexto

`create_product`, `ProductCreationStore` e `AdapterSqliteProductStore` já validam e persistem o template ativo com auditoria na mesma transação. O domínio admite preço sugerido zero e rejeita negativos. A interface local ainda não chama o caso de uso e a capability `main-local-deny` não concede esse comando.

## Decisões

- Expor somente `create_product`, registrando o comando em `build.rs` e `invoke_handler` e concedendo `allow-create-product` somente à capability local `main-local-deny`, selecionada para `webviews=["main"]`. Preservar os grants atuais e não adicionar permissões de plugin, shell, filesystem, rede, SQL ou origem remota.
- Receber `tauri::ipc::Request` e aceitar exatamente `name`, `description` e `suggestedPriceCents`. Rejeitar corpo não JSON, campos ausentes/extras, tipos inválidos, centavos não canônicos, negativos e overflow antes do caso de uso; retornar códigos estáticos sem refletir conteúdo.
- Converter o preço textual para `i64` sem ponto flutuante e chamar exclusivamente `application::create_product` com `AdapterSqliteProductStore`. Nome, descrição e regras autoritativas permanecem no servidor/domínio; texto válido é preservado literalmente.
- Usar o `Mutex` do adapter e o `AppDatabaseWriteLock` compartilhado. Recuperar o adapter após falha de armazenamento. Só confirmar sucesso depois de persistência atômica de produto e auditoria.
- Produto segue template: esta operação não cria estoque, item de venda, movimento, recebível ou alteração de saldo.
- Reutilizar React, parser decimal, estilos e testes já existentes. Sem migration, dependência, consulta/listagem, edição, desativação, rota ou design system.

## Alternativas consideradas

- Expor SQL, adapter ou dados do banco à UI: rejeitado por quebrar a fronteira UI → aplicação → domínio → repositório.
- Usar uma permissão ampla ou reaproveitar um comando/grant de outra jornada: rejeitado por ampliar autoridade ou mudar a semântica.
- Validar nome ou regra de domínio somente no React: rejeitado porque a UI não é autoridade e pode ser contornada.
- Converter preço com `Number`/float: rejeitado por perda de exatidão em centavos.

## Consequências e revisão

A interface pode solicitar a criação de um template de produto sem introduzir estoque ou movimento financeiro. Os testes de autoridade devem provar o grant somente para a origem local e o webview atual; essa configuração não prova isolamento de futuras janelas. A liberação depende dos checks, revisão, CI nos SHAs exatos e recibo remoto lido de volta. Reabrir se o contrato, a topologia, a capability ou o escopo de produto precisarem crescer.
