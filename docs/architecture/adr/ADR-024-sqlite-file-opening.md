# ADR-024 — abertura de banco SQLite em arquivo

Status: aceita para o contrato de DB-002. Não altera o escopo de ADR-023, que continua tratando somente do harness de testes em memória.

## Contexto

DB-001 adicionou `rusqlite` 0.40.2 como dev-dependency e prova isolamento em memória. DB-002 precisa abrir um banco de arquivo no core antes de migrations, schema e startup do aplicativo.

## Decisão

Promover o mesmo `rusqlite` 0.40.2 para dependência normal, mantendo pin exato, `default-features = false` e `bundled`. Não adicionar nem remover pacotes ou versões; o lockfile permanece byte idêntico. Expor `millani_artes::database::open_database(&Path) -> Result<Connection, DatabaseOpenError>` pelo módulo público da biblioteca, apontando diretamente ao arquivo de infraestrutura e sem exportar o módulo de infraestrutura acoplado ao binário.

O core exige path absoluto e não vazio, diretório pai existente e controlado pelo caller, rejeita diretórios e usa `SQLITE_OPEN_READ_WRITE | SQLITE_OPEN_CREATE` sem URI. A consulta a `sqlite_schema` força a validação/leitura do arquivo. Erros não carregam path, SQL ou payload, não expõem `source` nem são registrados em log.

## Consequências e limites

O core cria ou abre arquivo no path fornecido e não escolhe o local. UI/IPC, app-data, startup, containment, permissões, migrations, FK, transações, backup/restore e schema permanecem fora deste ADR. Essa API isolada não prova segurança global do caminho nem invariantes financeiras.

A promoção muda `Cargo.toml`, mas não adiciona/remove pacotes ou versões e não altera `Cargo.lock`. Cargo Audit e OSV vinculam ambos manifest e lock; os scans crus encontraram somente as coordenadas históricas e o bloqueio por hash foi pré-aceite. E4 humana recebida no reply `call_94GXADvsRNfAUdlUAJn07ohk` para os scopes Cargo `d1ac88b7e1ae43a4778c083a3b4ee114ed0a99921892141036cfb9d30bf37098` e OSV `9a7aeb66435091eb0ca302b4b269478fb95bfac6a2a7fb6f69ec61c8d3314318`, Windows MSVC, findings existentes e prazo original até 22/10/2026 21h São Paulo. Os quatro records autorizados, revalidação dos gates e CI permanecem pendentes; os snapshots ignored `approved=false` continuam inalterados. Scanners de coordenadas Rust não cobrem o C bundled do SQLite.

## Referências

- [rusqlite 0.40.2 `open_with_flags`](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.open_with_flags)
- [SQLite `sqlite3_open_v2`](https://www.sqlite.org/c3ref/open.html)
- [ADR-023 — harness de testes](ADR-023-sqlite-test-harness.md)
- [Spec DB-002](../../../planning/specs/DB-002.md)
