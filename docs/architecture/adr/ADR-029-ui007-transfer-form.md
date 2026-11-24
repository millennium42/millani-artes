# ADR-029 — registro de transferência pela interface

Status: aceita sob autorização humana explícita para concluir UI-001–UI-016. Esta decisão limita a integração de UI-007; não comprova implementação nem liberação.

## Contexto

create_transfer, TransferCreationStore, AdapterSqliteMovementStore, trava transacional e list_accounts já existem. A interface local ainda não chama create_transfer. FIN-021 define validação de contas distintas/valor positivo, escrita atômica movimento/auditoria e erros estáticos.

## Decisões

- Expor somente create_transfer, mantendo list_accounts como consulta necessária. Declarar o comando no manifesto e invoke_handler e conceder allow-create-transfer somente a main-local-deny, local e limitada a webviews=["main"]. Não adicionar seletor de janela: seletores de janela/webview são OR e a configuração atual declara apenas main. Reavaliar se surgir outra janela/webview.
- Receber tauri::ipc::Request e validar o corpo JSON antes de conversão tipada; aceitar somente sourceAccountId, destinationAccountId, amountCents e context. IDs são arrays de 16 bytes; amountCents é string decimal canônica; context é valor existente. Campo extra, tipo/formato inválido retorna TRANSFER_INVALID estático sem refletir/registrar conteúdo.
- Converter valor com parser de centavos sem Number/float/arredondamento. Não validar na UI positividade, contas distintas, saldo ou neutralidade de relatório; chamar exclusivamente FIN-021.
- Reutilizar AppDatabaseWriteLock, inicializador de banco, AdapterSqliteMovementStore e recuperação já usada pela ponte de movimento. Sucesso só depois do retorno da transação movimento/auditoria.
- Erros de persistência/clock/IDs são códigos estáticos; não enviar SQL, valores ou caminhos à interface.
- Reutilizar React, estilos e testes. Sem migration, plugin, dependência, rota, design system, outro comando/grant, dados extras ou alteração de FIN-021.

## Alternativas consideradas

- Expor SQL, adapter ou estado financeiro à UI: rejeitado por quebrar a fronteira de domínio/aplicação.
- Usar create_entry/create_exit ou grants amplos para representar transferência: rejeitado por semântica/privilégio incorretos.
- Validar transferências ou resultados em React: rejeitado por duplicar FIN-021.
- Usar Number/float: rejeitado por perda de exatidão.

## Consequências e revisão

A UI poderá solicitar transferência; FIN-021 segue como autoridade financeira. Esta decisão não atesta E2E nativo, relatórios neutros após execução, segurança geral, WebView futuro ou WCAG geral. Reabrir se o contrato, topologia, grants, esquema, dependências ou UI precisarem crescer. Liberação depende de testes/reviews/CI exatos e recibo remoto lido de volta.
