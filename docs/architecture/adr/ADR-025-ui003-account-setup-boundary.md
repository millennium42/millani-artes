# ADR-025 — integração local mínima para cadastro de contas

Status: aceita sob autorização humana explícita de 23/11/2026 para concluir UI-001–UI-016.

## Contexto

UI-003 precisa oferecer cadastro real de conta. O caso de uso FIN-008 e o adapter SQLite existem, mas o app ainda não inicializa banco, migrações ou comando Tauri. ADR-024 deixa app-data, startup e migrações a cargo de um item posterior. A configuração Tauri nega comandos custom e o build não filtra a lista de comandos. A interface precisa transportar `i64` sem arredondamento ou perda de precisão em JavaScript.

## Decisões

- Resolver no Rust `app_local_data_dir()` do app Tauri e usar o arquivo fixo `millani-artes.sqlite3`. Criar somente o diretório dessa pasta. O frontend não envia nem recebe caminho, conexão, consulta SQL ou localização de arquivo.
- Na inicialização, abrir pelo `database::open_database` e avançar sequencialmente o ledger versionado 0→17, usando os executores publicados e o backfill especializado da versão 15. Estado futuro/corrompido ou erro de migração impede a janela de abrir; mensagens externas permanecem estáticas.
- Compor `AdapterSqliteAccountStore`, `Clock` de sistema e `UuidGenerator` com `getrandom` e bits UUID v4 nos bytes aleatórios; disponibilizar um único comando de produto, `create_account`, pelo `invoke_handler`. O build declara `create_account` e o probe `security_probe_report` exclusivamente para que o harness `cfg(test)` existente continue compilando; o produto só registra `create_account`.
- Conceder a permissão gerada somente à capability local da janela/webview `main`; nenhuma permissão core/plugin, origem remota ou outra janela recebe acesso. A aplicação continua sem `core:default`, SQL, fs, shell, rede ou plugin.
- O contrato IPC recebe `initial_balance_cents` como string decimal e o converte para `i64` no Rust. Assim JavaScript não converte valores acima de `Number.MAX_SAFE_INTEGER`. O Rust mantém nome/tipo/saldo e validação em FIN-008; o frontend não duplica regras de domínio.
- Para entrada visual, aceitar reais com vírgula decimal, sem agrupamento, sinal `-` opcional e zero, uma ou duas casas decimais; converter por inteiro para centavos, sem arredondamento, e rejeitar texto fora de `i64`. Essa é apenas a sintaxe de apresentação; não restringe ou altera o contrato de domínio assinado.
- Adotar `@tauri-apps/api` 2.12.5 para `invoke`; adicionar ambiente jsdom e Testing Library às devDependencies para testar formulário, mensagens, foco e teclado usando a configuração Vitest existente. Essas ferramentas usam dados sintéticos, não inicializam Tauri/SQLite e não substituem inspeção real do app.
- Sem biblioteca visual ou plugin; estilos locais mantêm a paleta e o tamanho mínimo de UI-002.

## Consequências e limites

O app poderá persistir contas e audit_event no banco local. Erro na inicialização impede uso do app sem expor detalhes de path/SQL; falha no comando retorna erro seguro. O formulário fica disponível após salvar para cadastrar outra conta, sem listar, editar ou excluir contas. Teste React comprova a camada frontend; testes Rust cobrem comando/usecase/SQLite; a captura automatizada comprova apenas a apresentação da tela executada. E2E Tauri só será declarado se a jornada completa for executada.

Pins/licenças e riscos das ferramentas estão em [TOOL_REGISTER](../../../governance/TOOL_REGISTER.md) e nos lockfiles. Não há biblioteca financeira, plugin, capability ampla ou design system novo.
