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

INF-002 introduz manifests, lockfiles e scaffold sem regras financeiras. MSVC/Windows SDK estão presentes nesta máquina e os gates Rust/build de produção passaram; inspeção visual foi confirmada pela usuária e o encerramento terminou0; detalhes ficam no handoff. Coverage/testes de jornadas e tooling TS próprio estão nas tarefas futuras. CI atual valida documentos/mapa/schema/log, não instala nem testa esses runtimes ou produto. [Handoff INF-002](../../planning/handoffs/INF-002.md) e recibo Git registram os resultados de scaffold; INF-001 preserva a evidência dos runtimes. CI da publicação parcial37206061776 no SHA775478d foi registrado em nota Git, mantendo a reserva ativa e passes=false; fechamento exige novo CI no SHA final.

## Scaffold — INF-002

Node/Rust: ativar a sessão conforme INF-001 acima. WebView2 154.0.4258.53 foi encontrado no registro/pasta padrão; isso não comprova abertura do aplicativo. Em 2026-10-04, Build Tools17.14.41/build17.14.37710.0, MSVC14.44.35207 (cl19.44.35229.0) e Windows SDK10.0.26100.0 foram instalados e conferidos. SDK10.0.28000.0 também ficou disponível. Instalador terminou exit0, instance1013c2d6 completa/sem reboot pendente. Gates de compilação/janela ainda dependem dos resultados registrados no handoff.

Bootstrap Microsoft17.14.41 no cache dev conferido por SHA/assinatura Microsoft. Componentes mínimos e dependências na [configuração VS](../../scripts/windows-build-tools.vsconfig). Para uma máquina sem Build Tools, em PowerShell elevado, na raiz do checkout:
```powershell
$millaniBootstrap = Join-Path $env:LOCALAPPDATA 'MillaniArtesDev\downloads\vs-buildtools-17.14.41.exe'
$millaniConfig = (Resolve-Path -LiteralPath './scripts/windows-build-tools.vsconfig').Path
if ((Get-FileHash -LiteralPath $millaniBootstrap -Algorithm SHA256).Hash.ToLowerInvariant() -ne '985969f472caad75d993a5cb4c35a6a4271460cc12b343e2433b994d173aa990') { throw 'Hash incorreto; não executar.' }
$millaniSignature = Get-AuthenticodeSignature -LiteralPath $millaniBootstrap
if ($millaniSignature.Status -ne 'Valid' -or $millaniSignature.SignerCertificate.GetNameInfo([Security.Cryptography.X509Certificates.X509NameType]::SimpleName, $false) -ne 'Microsoft Corporation') { throw 'Assinatura Microsoft inválida; não executar.' }
rtk proxy $millaniBootstrap --wait --quiet --norestart --config $millaniConfig
```
Se a base já existe, adicionar componentes com modify no installPath existente. Em PowerShell elevado, na raiz do checkout:
```powershell
$millaniPf86 = [Environment]::GetEnvironmentVariable('ProgramFiles(x86)')
$millaniSetup = Join-Path $millaniPf86 'Microsoft Visual Studio\Installer\setup.exe'
$millaniInstallPath = Join-Path $millaniPf86 'Microsoft Visual Studio\2022\BuildTools'
$millaniConfig = (Resolve-Path -LiteralPath './scripts/windows-build-tools.vsconfig').Path
$millaniSignature = Get-AuthenticodeSignature -LiteralPath $millaniSetup
if ($millaniSignature.Status -ne 'Valid' -or $millaniSignature.SignerCertificate.GetNameInfo([Security.Cryptography.X509Certificates.X509NameType]::SimpleName, $false) -ne 'Microsoft Corporation') { throw 'Assinatura Microsoft inválida; não executar.' }
rtk proxy $millaniSetup modify --installPath $millaniInstallPath --config $millaniConfig --quiet --norestart
```
Iniciar setup.exe a partir do checkout, fora de sua própria pasta; não passar --wait ao setup.exe (é opção do bootstrap). Neste ambiente o líder executou esse modify por UAC e acompanhou o processo até exit0. Conferir todos os três IDs do vsconfig com vswhere -all -products * -requires ID -property installationPath, a instância completa, ausência de reboot pendente, arquivos e versões do compiler/SDK. Código de saída sozinho não prova componentes instalados. O mesmo modify pode ser aplicado pela interface, Mais → Importar configuração → Revisar detalhes → Modificar. Nenhum reinício automático.

Com pré-requisitos presentes, a partir da raiz:
```powershell
rtk proxy npm ci --ignore-scripts
rtk proxy npm run build
rtk proxy npm audit --audit-level=high
rtk proxy cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
rtk proxy cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
rtk proxy cargo test --locked --manifest-path src-tauri/Cargo.toml
rtk proxy npm run tauri -- build --no-bundle
```
Scanner dev fixo0.22.2 conferido pelo digest GitHub e licenças principais:
```powershell
$millaniAudit = Join-Path $env:LOCALAPPDATA 'MillaniArtesDev\toolchains\cargo-audit-0.22.2-win-x64\cargo-audit-x86_64-pc-windows-msvc-v0.22.2\cargo-audit.exe'
rtk proxy $millaniAudit audit --file src-tauri/Cargo.lock
```
npm scripts de terceiros ficam desabilitados; se um binário de tooling exigir script, registrar a necessidade antes de habilitar. Rustfmt/clippy/test/audit não implicam coverage; smoke inicial requer iniciar o binário e conferir janela/conteúdo. Build de produção sem bundle não gera setup.exe nem aprovação de release. [Spec](../../planning/specs/INF-002.md), [handoff](../../planning/handoffs/INF-002.md) e recibos guardam os resultados reais, sem antecipar passes.


Gates nativos nesta máquina (2026-10-04): fmt/clippy/test(0 testes)/cargo-audit e build de produção --no-bundle passaram. O executável release foi iniciado pelo líder sem servidor Vite; a usuária confirmou a aparência da janela e seu fechamento, com NativeExitCode0 observado pelo líder. O perfil WebView2 padrão fica em %LOCALAPPDATA%\io.github.millennium42.millaniartes; não contém funcionalidade financeira implementada. Nenhum artefato nativo/perfil é publicado no Git. Build de produção sem bundle ainda não é instalador/release aprovado.


Aceite humano E4 (2026-10-04): usuária confirmou "a janela apareceu certinha e eu fechei ela" em resposta à inspeção do aplicativo de produção. O líder executou o binário identificado por SHAef5c6ad800a09a5e4b51b332d7dc0490510345448dcab794ad3d1510c6eda78c e observou encerramento0. Aceite cobre aparência/conteúdo esperado e fechamento do bootstrap; método AltF4 e redimensionamento específico não relatados, sem alegar screenshot/visão nativa do líder. Sem controles financeiros nesta UI. Não é aceite de funcionalidades financeiras nem release/setup.exe.

## Lockfiles — INF-003

Os locks já foram commitados em INF-002, conforme DEPENDENCY_SECURITY. Não usar npm install ou cargo update para reproduzir a resolução aprovada. Ative Node/Rust conforme INF-001; a partir da raiz do checkout:

```powershell
$millaniLocks = @('package-lock.json', 'src-tauri/Cargo.lock')
$millaniBefore = @(Get-FileHash -LiteralPath $millaniLocks -Algorithm SHA256)
rtk proxy npm ci --ignore-scripts --no-fund --no-audit
if ($LASTEXITCODE -ne 0) { throw 'npm ci falhou.' }
rtk proxy npm run build
if ($LASTEXITCODE -ne 0) { throw 'Build frontend falhou.' }
rtk proxy cargo metadata --locked --format-version 1 --manifest-path src-tauri/Cargo.toml --filter-platform x86_64-pc-windows-msvc --features tauri/custom-protocol | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Resolução Cargo falhou.' }
$millaniAfter = @(Get-FileHash -LiteralPath $millaniLocks -Algorithm SHA256)
for ($millaniIndex = 0; $millaniIndex -lt $millaniBefore.Count; $millaniIndex++) {
    if ($millaniBefore[$millaniIndex].Path -ne $millaniAfter[$millaniIndex].Path -or $millaniBefore[$millaniIndex].Hash -ne $millaniAfter[$millaniIndex].Hash) { throw 'Lockfile alterado.' }
}
```

E3 local em 2026-10-04: manifests/roots/pins coerentes; 79 entradas npm HTTPS registry.npmjs.org com SHA512 e 416 entradas Cargo crates.io com checksums SHA256. Ambos rastreados e não ignorados. npm ci sem scripts instalou 25 pacotes nesta plataforma; strict tsc/Vite8.3.2 passou. Cargo metadata --locked/custom-protocol resolveu 247 nós Windows. Hashes antes/depois iguais aos blobs Git desta revisão, no [handoff](../../planning/handoffs/INF-003.md).

fmt/clippy/test(0 testes)/audits também passaram. npm audit: zero vulnerabilidades. Cargo audit: zero vulnerabilidades classificadas e dois avisos mantidos, RUSTSEC-2024-0370/proc-macro-error1.0.4 e RUSTSEC-2024-0429/glib0.18.5; ambos ausentes do grafo Windows executado. Nenhum ignore ou aprovação de outro sistema. Bases/detalhes no handoff; atualizar scanner/bases em nova reprodução. Caches locais usados: não é prova de máquina limpa INF-011, coverage, finanças ou release. Sem mudança de UI, smoke/aceite INF-002 preservado.

## TypeScript strict — INF-004

ADR002 exige strict. tsconfig.json mantém strict/noEmit e demais opções do scaffold; include cobre src e vite.config.ts. A configuração Vite também participa da validação estática antes do bundle. Com Node ativado e dependências do lock instaladas:

```powershell
rtk proxy node ./node_modules/typescript/bin/tsc --showConfig
rtk proxy node ./node_modules/typescript/bin/tsc --noEmit --pretty false
rtk proxy npm run build
```

E3 em 2026-10-04: showConfig inclui todas as três fontes próprias atuais (App.tsx, main.tsx, vite.config.ts). Probe temporária em src com parâmetro sem tipo e string=null produziu TS7006/TS2322; npm run build falhou exit1 antes de iniciar Vite. Probe temporária na configuração Vite produziu TS7006/exit1. Ambas foram removidas/restauradas em finally; configVite/locks permanecem iguais ao baseline. Typecheck e build reais depois passaram, Vite16módulos/151ms. Não confundir falhas negativas esperadas com build final quebrado.

Probes verificam o compilador/pipeline, não coverage de produto. Não há teste espelho/harness permanente nem novo pacote; lint/test/coverage e script typecheck próprios seguem INF005..009. Rustfmt/clippy/test(0)/audits também passaram; dois avisos Cargo fora do grafo Windows mantidos, semignore. Evidência, comandos e limites no [handoff](../../planning/handoffs/INF-004.md). UI inalterada preserva o smoke INF002, sem nova aprovação financeira/release.
