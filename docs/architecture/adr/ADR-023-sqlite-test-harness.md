# ADR-023 — Harness SQLite de testes

Status: proposta; aguarda aceite humano para aprovar o novo escopo dos scans ou publicar a entrega.

## Contexto

DB-001 precisa de conexões SQLite reais e isoladas para testes de integração. O aplicativo usa Rust/Tauri no Windows; tarefas futuras podem precisar provar comportamento de arquivo. Esta decisão limita-se ao harness DB-001.

## Decisão proposta

Usar `rusqlite` como dev-dependency, pin exato 0.40.2, recursos padrão desativados e somente `bundled`. Abrir uma conexão SQLite em memória por teste com helper compartilhado. Não adicionar tempfile, FFI manual ou SQLite via Python.

O workflow de documentação executa apenas o target de integração SQLite, em release e após o build Tauri existente. Sem novo job/action/cache/upload e sem iniciar WebView.

## Consequências e limites

O harness usa engine SQLite real, com valores sintéticos, registra `sqlite_version()` e não tem path ou rotina de cleanup de arquivo. SQLite bundled evita depender de instalação do host; fonte vendorizada SQLite é domínio público, rusqlite/libsqlite3-sys são MIT. O build de teste compila fonte C e adiciona crates ao Cargo.lock. Um valor com aspas prova o caminho do parâmetro neste teste, não SQL injection em geral.

Teste em memória não prova arquivo/durabilidade, permissões, containment, locks entre processos, migrations, schema, transação, FK, backup/restore ou invariantes financeiras. Cada tarefa futura deve incluir a prova de arquivo necessária ao seu próprio requisito.

As aprovações Cargo/OSV atuais vinculam hashes exatos de Cargo.toml/lock. Após alterá-los, checkers devem reportar `SCOPE_CHANGED`. Não copiar nem modificar aprovações anteriores. Resolução, instalação e scans crus podem ocorrer com os gates preservados; scans sem ignore não aprovam o escopo nem equivalem a PASS global. Aceite humano novo será necessário para classificar findings/exceções, declarar gates aprovados e publicar.

## Fontes

- [API `Connection` rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html)
- [Upstream rusqlite: recursos e licenças](https://github.com/rusqlite/rusqlite)
- API oficial crates.io para rusqlite 0.40.2 e libsqlite3-sys 0.38.2, consultada sem baixar crates.

Esta proposta não aceita o novo escopo de segurança; esse aceite deve registrar os hashes resolvidos de Cargo.toml/lock.
