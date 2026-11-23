# ADR-028 — registro de saída pela interface

Status: aceita sob a autorização humana explícita para concluir UI-001–UI-016. Esta decisão limita a integração de UI-006; não comprova implementação nem liberação.

## Contexto

UI-006 apresenta o registro de saída já implementado por FIN-018. O caso de uso create_exit, o adapter SQLite e a consulta list_accounts existem, mas a interface Tauri ainda não expõe create_exit. A pessoa precisa escolher uma conta sem receber saldo ou outros dados financeiros que o formulário não usa.

## Decisões

- Expor somente o comando de produto create_exit, reutilizando list_accounts já liberado. Declarar create_exit no manifesto do build, atualizar a descrição da capability para mencionar também o fluxo de saída e conceder a permissão gerada allow-create-exit somente à capability main-local-deny, com origem local e webviews=["main"], sem seletor de janela. Os seletores Tauri de janela e webview são OR; o app atual declara um único par main. Não alegar que este seletor rejeita sozinho uma futura janela. Rever capability e testes se a topologia mudar.
- O handler Tauri recebe `tauri::ipc::Request` em vez de parâmetros tipados, porque falhas de desserialização anteriores ao handler podem refletir o valor recebido. Ele valida os três campos esperados do corpo JSON e mapeia tipo/formato inesperado a `EXIT_INVALID`; não devolver nem registrar diagnóstico de desserialização.
- create_exit recebe o identificador como bytes, centavos como string decimal e o contexto existente. Rust exige exatamente 16 bytes, faz parse estrito de i64, converte o contexto existente e chama exclusivamente o caso de uso FIN-018. A UI converte sintaxe monetária para centavos inteiros; não usa Number, float ou arredondamento e não valida positividade.
- A escrita reutiliza AppDatabaseWriteLock, o inicializador SQLite existente e AdapterSqliteMovementStore/FIN-018. O caso de uso e adapter permanecem responsáveis por validar saída positiva e persistir movimento e auditoria atomicamente. IDs, relógio, SQL e persistência permanecem no backend.
- Falhas externas usam códigos/mensagens estáticos; valores, caminho, SQL e erro interno não são enviados à UI nem registrados.
- list_accounts continua retornando somente pares de identificador de 16 bytes e nome. A UI não recebe saldo, categoria, movimentos ou auditoria.
- A tela reutiliza React, Tauri, estilos e ferramentas presentes. Não adiciona dependência, plugin, migration, rota/framework, design system, outra capability ou regra financeira. Campos de categoria, descrição e data ficam fora do contrato FIN-018.
- Testes cobrem argumentos IPC exatos, autoridade local/webview main, negação para origem remota/webview não selecionado e comandos/grants excedentes, erro opaco, lock/rollback/recovery em SQLite sintético, acessibilidade do formulário e inspeção automatizada do app construído. Capturas usam somente dados sintéticos e automação, sem CUA.

## Alternativas consideradas

- Expor AdapterSqliteMovementStore ou SQL ao frontend: rejeitado por quebrar a fronteira UI→caso de uso→domínio→repositório→SQLite.
- Reutilizar grant amplo, filesystem ou plugin: rejeitado por contrariar least privilege sem necessidade funcional.
- Passar Number/float ou validar positividade em React: rejeitado por perda de precisão e duplicação de regra de domínio.
- Adicionar descrição, categoria ou data: rejeitado porque FIN-018 não prevê esses campos para a operação e UI-006 não depende de outra decisão de produto.

## Consequências e revisão

A UI poderá solicitar uma saída e apresentar sucesso somente após o retorno do caso FIN-018 depois da transação. A nota liberada de FIN-018 comprova o backend, não UI→IPC→SQLite. Esta autorização não alega acessibilidade geral, segurança de WebView remoto, concorrência entre processos ou cobertura financeira global.

Reabrir este ADR se forem necessários outros dados/comandos, outra origem/janela/webview, migration, dependência, plugin ou novo contrato financeiro. A liberação de UI-006 continua sujeita a testes, segurança, review adversarial, CI remota no SHA final e nota release_evidence com mappings preservados.
