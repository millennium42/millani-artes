# ADR-027 — registro de entrada pela interface

Status: aceita sob a autorização humana explícita para concluir UI-001–UI-016, recebida em 20/11/2026: “autorizo explicitamente o que for preciso para avançar até UI 016”. Esta decisão limita a integração de UI-005; não comprova implementação nem liberação.

## Contexto

UI-005 apresenta o registro de entrada já implementado por FIN-017. O caso de uso `create_entry`, o adapter SQLite e o leitor `list_accounts` existem, mas a interface Tauri não os expõe. A capability atual permite somente `create_account` e `create_category`. A pessoa precisa escolher uma conta sem receber saldo ou outros campos financeiros que o formulário não usa.

## Decisões

- Expor somente os comandos de produto `list_accounts` e `create_entry`; declarar ambos no manifesto do build e conceder permissões geradas explícitas apenas à capability `main-local-deny`, com `local = true` e seletor `webviews = ["main"]` (sem seletor de janela). A política Tauri existente define seletores de janela e webview como alternativas OR, não uma interseção. O app atual declara um único par janela/webview `main`; não alegar que o seletor de capability, sozinho, rejeita qualquer janela futura. Rever capability e testes se a topologia de janelas/webviews mudar. Nenhuma permissão core/plugin, origem remota, caminho de arquivo, SQL ou comando genérico será adicionado.
- `list_accounts` reutiliza `AccountReadStore`/`list_accounts` e retorna somente pares de identificador de 16 bytes e nome. Não retorna tipo, saldo inicial, movimentos ou dados de auditoria. A UI normaliza o par para seu modelo de seleção.
- `create_entry` recebe o identificador em bytes, o valor em centavos como string decimal e o contexto como um dos valores existentes. Rust exige exatamente 16 bytes, faz parse estrito de `i64`, converte o contexto existente e chama exclusivamente o caso de uso FIN-017. IDs, relógio, SQL e persistência permanecem no backend.
- A escrita reutiliza o arquivo e inicializador SQLite existentes, a trava de escrita de processo compartilhada e o adapter transacional FIN-017, que confirma entrada e auditoria atomicamente. Falhas externas permanecem códigos estáticos; valores, caminho, SQL e erro interno não são registrados nem enviados à UI.
- O campo de valor usa sintaxe de apresentação exata em reais com vírgula e até duas casas, sem agrupamento, conversão por `Number` ou arredondamento. A UI converte por inteiro para centavos e não valida regras de domínio; validade financeira, incluindo entrada positiva, permanece no caso de uso/domínio.
- A tela reutiliza React, Tauri, estilos e ferramentas existentes. Não adiciona biblioteca, capability ampla, plugin, migration, rota/framework novo, design system ou regra de negócio. Campos de categoria, descrição e data não fazem parte do contrato FIN-017 e ficam fora desta fatia.
- Testes incluem a seleção mínima de dados, argumentos IPC exatos, acesso permitido/negado para a origem local e webview `main` atual, rejeição de origem remota e webview não selecionado, falhas opacas, atomicidade e auditoria em SQLite sintético, estados/teclado/foco na UI e inspeção automatizada da aplicação construída. Não inferir uma restrição independente por janela. Captura será automatizada sem CUA e não usará dados reais.

## Alternativas consideradas

- Retornar a estrutura `Account` completa: rejeitada por expor saldo e campos sem necessidade.
- Passar `Number`/float ou validar positividade em React: rejeitada por perda de precisão e duplicação das regras financeiras.
- Reutilizar grant amplo, filesystem ou plugin: rejeitada por violar negação por padrão e não ser necessária para a jornada.
- Implementar SQL ou persistência no frontend: rejeitada por duplicar o caminho transacional de FIN-017.

## Consequências e revisão

A UI poderá carregar opções mínimas de conta e solicitar o registro de uma entrada. O recibo de FIN-017 prova somente o caso de uso e adapter backend; a ligação UI→IPC→SQLite precisa de evidência própria. Reabrir este ADR se a UI precisar de outros dados/campos, se o escopo exigir outra origem, janela ou webview, se surgir uma migration/dependência, ou se o caminho de escrita deixar de reutilizar trava e transação existentes. A liberação de UI-005 continua sujeita a testes, segurança, revisão adversarial, CI remota no SHA final e nota `release_evidence` com mappings preservados.
