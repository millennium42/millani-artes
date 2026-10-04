# Reprodução em Windows limpo

## Runtimes — INF-001
Pins: [.node-version](../../.node-version) define Node24.21.0 LTS/npm11.19.0; [rust-toolchain.toml](../../rust-toolchain.toml) define Rust1.99.0/profileminimal/rustfmt-clippy/targetx86_64-pc-windows-msvc. Instalação e comandos locais verificados (E3), conforme handoff. Consulte [registro](../../governance/TOOL_REGISTER.md) e [spec](../../planning/specs/INF-001.md).

Baixe o zip x64 Node e rustup-init1.29.1 dos URLs e hashes exatos no registro. Compare Get-FileHash -Algorithm SHA256 antes de extrair/executar; erro impede avanço. Extrair Node em %LOCALAPPDATA%\MillaniArtesDev\toolchains, validando destinos dos membros dentro dessa raiz. Rustup-init instala no usuário com:
```powershell
$millaniBootstrap = Join-Path $env:LOCALAPPDATA 'MillaniArtesDev\downloads\rustup-init-1.29.1.exe'
rtk proxy $millaniBootstrap -y --default-host x86_64-pc-windows-msvc --default-toolchain none --profile minimal --no-modify-path
$millaniRustup = Join-Path $env:USERPROFILE '.cargo\bin\rustup.exe'
rtk proxy $millaniRustup toolchain install 1.99.0 --profile minimal --component rustfmt --component clippy
```
Use o caminho completo do executável verificado ou inclua seu diretório na sessão. Para os runtimes instalados em cache/default do usuário, a partir da raiz do checkout:
```powershell
$millaniNode = Join-Path $env:LOCALAPPDATA 'MillaniArtesDev\toolchains\node-v24.21.0-win-x64'
$millaniCargo = Join-Path $env:USERPROFILE '.cargo\bin'
$env:PATH = "$millaniNode;$millaniCargo;$env:PATH"
rtk proxy node --version
rtk proxy npm --version
rtk proxy rustup --version
rtk proxy rustup show active-toolchain
rtk proxy rustc -vV
rtk proxy cargo --version
rtk proxy rustfmt --version
rtk proxy cargo clippy --version
```
Exigir Nodev24.21.0/npm11.19.0/rustc1.99.0/hostx86_64-pc-windows-msvc e seleção via rust-toolchain.toml. Rustfmt/Clippy devem existir como componentes do pin. O arquivo Node declara a versão; não instala nem muda sozinho o Node do PATH. Rustup resolve a toolchain a partir do arquivo do projeto. Ativação acima dura só a sessão, não altera PATH persistente.

## Produto — ainda planejado
Clonar → verificar runtimes fixados → verificar/instalar Microsoft C++ Build Tools com Desktop development with C++/WindowsSDK e WebView2 conforme [Tauri](https://v2.tauri.app/start/prerequisites/) → scaffold INF-002 → instalar dependências com lockfile → format/lint/typecheck/test/coverage → scans → builds → smoke produção. Ausência de cl/msbuild no PATH/defaultvswhere não prova ausência global. Não inferir que WebView2 está verificado apenas pela versão Windows. Build limpo/MSVC/WebView2 efetivos ficam para INF-002/INF-011/INF-015; instalador para REL-*.

Ainda não há package.json/Cargo.toml/lockfiles/código de produto para qualityTS/fmt-clippy-test-audit de crate/coverage/build/smoke. CI atual valida documentos/mapa/schema/log, não instala nem testa esses runtimes ou produto. [Handoff INF-001](../../planning/handoffs/INF-001.md) e recibo Git registrarão comandos/versões/hashes/SHA e limitações efetivamente verificados.
