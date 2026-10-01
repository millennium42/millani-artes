# Reprodução em Windows limpo

Planejado: clonar → instalar versões fixadas de Node/Rust/Tauri → instalar dependências com lockfile → format/lint/typecheck/test/coverage → scans → builds → smoke produção. Nesta revisão não há `package.json`, `Cargo.toml`, lockfile, tooling de produto, build ou smoke para executar. `INF-*` definirá e verificará cada comando.
