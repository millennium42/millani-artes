# ADR-023 — Harness SQLite de testes

Status: decisão arquitetural aceita para o harness de testes. E4 humano aprovado apenas para os dois scopes de dependências documentados na spec/handoff DB-001; gates locais Cargo/OSV passaram após o aceite; CI/entrega ainda pendentes.

## Contexto

DB-001 precisa de conexões SQLite reais e isoladas para testes de integração. O aplicativo usa Rust/Tauri no Windows; tarefas futuras podem precisar provar comportamento de arquivo. Esta decisão limita-se ao harness DB-001.

## Decisão

Usar `rusqlite` como dev-dependency, pin exato 0.40.2, recursos padrão desativados e somente `bundled`. Abrir uma conexão SQLite em memória por teste com helper compartilhado. Não adicionar tempfile, FFI manual ou SQLite via Python.

O workflow de documentação executa apenas o target de integração SQLite, em release e após o build Tauri existente. Sem novo job/action/cache/upload e sem iniciar WebView.

## Consequências e limites

O harness usa engine SQLite real, com valores sintéticos, registra `sqlite_version()` e não tem path ou rotina de cleanup de arquivo. SQLite bundled evita depender de instalação do host; fonte vendorizada SQLite é domínio público, rusqlite/libsqlite3-sys são MIT. O build de teste compila fonte C e adiciona crates ao Cargo.lock. Um valor com aspas prova o caminho do parâmetro neste teste, não SQL injection em geral.

Teste em memória não prova arquivo/durabilidade, permissões, containment, locks entre processos, migrations, schema, transação, FK, backup/restore ou invariantes financeiras. Cada tarefa futura deve incluir a prova de arquivo necessária ao seu próprio requisito.

As aprovações Cargo/OSV vinculam hashes exatos de Cargo.toml/lock. O aceite humano recebido autoriza temporariamente apenas `glib 0.18.5` (`RUSTSEC-2024-0429`, `GHSA-wrw7-89jp-8q8g`) e `proc-macro-error 1.0.4` (`RUSTSEC-2024-0370`), no alvo `x86_64-pc-windows-msvc`, até `2026-10-23 00:00 UTC`, nos scopes Cargo `04e69804dd2e7772f766998773152330ef05eedce95a19192f7555613899e829` e OSV `ea0e2c2fbc7095b07d6d8ab063c8291563f7d72aee43203598ac891709224860`. A resposta humana literal e seus records vinculados estão registrados na [spec](../../../planning/specs/DB-001.md) e [handoff](../../../planning/handoffs/DB-001.md). A aprovação não cobre outros findings, dependências, alvos ou extensão do prazo. Os gates locais passaram nos inputs exatos aprovados; CI final e recibo de entrega continuam pendentes. Scans sem ignore não equivalem a segurança global. O código C de SQLite bundled não é coberto por scanners de coordenadas.

## Fontes

- [API `Connection` rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html)
- [Upstream rusqlite: recursos e licenças](https://github.com/rusqlite/rusqlite)
- API oficial crates.io para rusqlite 0.40.2 e libsqlite3-sys 0.38.2, consultada sem baixar crates.

O aceite E4 é restrito aos hashes recebidos e não conclui DB-001; permanece necessário concluir review final e CI no SHA exato e recibo de entrega antes de liberar a reserva.
