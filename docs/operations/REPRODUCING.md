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

## Formatter/linter — INF-005

Biome2.5.15 é devDependency fixa; npm ci --ignore-scripts reproduz wrapper e binário opcional Windows do lock. Não usar npx com versão flutuante. A configuração usa schema local do pacote instalado, recommended lint, warnings como erro, formatter2espaços/LF e vcs/gitignore. Includes cobre src, vite.config.ts, tsconfig.json, package.json e biome.json; docs/locks/Rust/artefatos ficam fora dessa ferramenta. Rust conserva seus próprios checks.

```powershell
rtk proxy npm ci --ignore-scripts --no-fund --no-audit
rtk proxy npm run format:check
rtk proxy npm run lint
rtk proxy npm run build
```

Para aplicar formatação dentro do escopo: rtk proxy npm run format. Esse comando escreve; format:check/lint só verificam. Nenhum assist/refactor/plugin adicional configurado; linter recomendado não foi reduzido.

E3 em2026-10-04: CLI2.5.15/install-ignore-scripts/npmci27pacotes; formatter/linter7arquivos/nenhuma correção necessária. Probe temporária com formato divergente/debugger produziu exit1 nos dois checks, incluindo lint/suspicious/noDebugger; removida emfinally. Checks positivos/build strict depois passaram sem alterar fonte/configs/locks durante comandos read-only. 79entradas npm anteriores preservadas; 9novasBiome/88registry ao todo, pin2.5.15/fontes-integrity verificadas. Novo lock SHA256722d67e9765baf435e3440c47db94d141dd7980a789327c4d42e57e6c0440f4d.

Audits npm0vuln/Cargo0vuln classificadas+2avisos foraWindows247nodes mantidos semignore; fmt/clippy/test0 passaram. Tooling não prova coverage/Vitest/máquina limpa/finanças/release. [Registro](../../governance/TOOL_REGISTER.md), [configuração oficial](https://biomejs.dev/reference/configuration/) e [handoff](../../planning/handoffs/INF-005.md) guardam escopo/limites; CI atual continua somenteDocumentation.


## Testes do scaffold — INF-006

Vitest5.0.3 e provider V8 da mesma versão são devDependencies fixas. Node24.21.0/Vite8.3.2 existentes, sem instalação global/npx/DOM adicional. Com Node ativado:
```powershell
rtk proxy npm ci --ignore-scripts --no-fund --no-audit
rtk proxy npm test
rtk proxy npm run format:check
rtk proxy npm run lint
rtk proxy npm run build
```
npm test executa vitest run, sem watch, em Node e com coverage V8 habilitada. Para testes locais filtrados, o gate continua avaliando todo src TS/TSX: executar apenas App.test.tsx deve falhar enquanto main não estiver coberto. Não desabilitar/reduzir coverage para apresentar aprovação. Texto/json-summary em coverage/ e artefatos .vitest/ são ignorados; não publicar paths locais nem dados privados como relatório de produto.
vitest.config.ts inclui fonte não importada e exclui somente testes. Gate global90 lines/statements/functions e85 branches; autoUpdate não habilitado. Os arquivos financeiros/backup críticos ainda não existem, e quando existirem exigirão95/95/95/90 porglob/per-file conforme política. INF007 terá ciclo próprio; esta adoção não declara concluída sua configuração específica.
Os três testes verificam App real por SSR e bootstrap com/semroot, usando mocks só em document/ReactDOMclient e cleanup/reset porcaso. Não provam WebView/DOM real, eventos, estilos, integração ou acessibilidade da jornada. Strict cobre seis fontes próprias (App/main/doistestes/Vite/Vitest); Biome cobre10arquivos. App/main/CSS/Vite/Rust anteriores intactos.
E3 local2026-10-04: npmci59pacotes/lock preservado, Vitest5.0.3; suíte3testes/2files verde e100% App/main (lines4/4, statements5/5, functions1/1, branches2/2). Probe temporária com marca incorreta falhou AssertionError/exit1 e foi removida; App isolado passou teste e falhou gate lines25/statements20/branches0, mainincluído0. Depois suíte completa/build strict/format/lint/Rustfmt-clippy-test0/audits verdes. Cargo mantém dois avisos foraWindows247nodes semignore. Sem máquina limpa/coverageRust/finanças/release comprovados. [Registro](../../governance/TOOL_REGISTER.md), [config](../../vitest.config.ts), [handoff](../../planning/handoffs/INF-006.md) e [coverage oficial](https://vitest.dev/config/coverage.html).


## Coverage e relatórios — INF-007

Com Node ativado e dependências do lock jáinstaladas:
```powershell
rtk proxy npm run coverage
```
O comando executa Vitest5.0.3/provider V8 semwatch. npm test também conserva coverage habilitada; quality deve aproveitar uma única execução, sem duplicar os dois comandos. Nenhuma nova dependência/instalação/job foi necessária nesta tarefa.
O comando emite texto no console; coverage/ contém coverage-summary.json, lcov.info/LCOV e index.html/assets locais. LCOV serve para ferramentas futuras; HTML permite inspecionar fonte e linhas. Todo diretório é ignorado, inclusive código/paths técnicos do relatório. clean:true/reportsDirectory:coverage limpa só resultados desse diretório dentro do workspace. Upload remoto segue INF013 e não foi adicionado.

O global90/90/90/85 inclui todo srcTS/TSX não importado e exclui sótestes. [Política](../quality/COVERAGE_POLICY.md) registra classificação atual vazia e exige95/95/95/90 porarquivo nos paths reais introduzidos por futuros itens críticos. Não inferir classificação pelo nome de diretório ainda não definido. Para esses paths, adicionar entradas nativas coverage.thresholds com perFile:true; média do glob isolada não garante mínimo em cada arquivo. Semredução de thresholds/autoUpdate.

E3 em2026-10-04 local: comando coverage ausente/Missingscript antes; probesbad-good sintéticas globalpass98.91statements/99.03branches/98.46functions/99.41lines enquanto bad93.1/50/92.85/93.75. Glob crítico agregado95/95/95/90 também passou; perFile:true falhouexit1 nas4métricas, identificando arquivo ruim. Após cobrir todos caminhos/funções, passou100%. Probes/glob sintético removidos emfinally, configrestaurada; não sãofinanças fictícias persistentes.
Final real:3tests2files/100%App-main, lines4/4/statements5/5/functions1/1/branches2/2. LCOVduasfontesLF4-LH4/FNF1-FNH1/BRF2-BRH2; HTML5010bytes/22arquivos totais de reports locais. CódigoUI/testes/locks/pins intactos. format-lint/strict/build16módulos134ms/Rustfmt-clippy-test0/audits passaram; Cargo dois avisos fora247nodesWindows mantidos semignore. Não comprova domínioRust/finanças/SQLite/UIreal/cleanmachine/setup.exe. [Handoff](../../planning/handoffs/INF-007.md) e [referência oficial](https://vitest.dev/config/coverage.html) guardam limites.


## Comando typecheck — INF-008

Com Node ativado e as dependências do lock instaladas:
```powershell
rtk proxy npm run typecheck
```
O script chama tsc --noEmit, usando TypeScript7.0.2 e tsconfig.json já existentes. Inclui as seis fontes próprias atuais: App/main, seus dois testes e as configurações Vite/Vitest. Strict, noEmit e noUnused permanecem habilitados. O comando verifica tipos sem executar Vite ou gerar JS, mapas e declarações. Build conserva o pipeline tsc --noEmit seguido de Vite.
E3 local em2026-10-04: positivo exit0; probe temporária com number atribuído a string produziu TS2322/exit1 pelo npm script. Removida emfinally; positivo final0, três arquivos dist com hashes inalterados durante typecheck e nenhum emit emsrc/configs. Não é bug do produto nem teste financeiro.
Formatter/linter10arquivos, build16módulos, testes+coverage3testes e gatesRust/audits passaram. CoverageApp-main100%(4linhas/5statements/1função/2branches) não representa finanças/Rust/UIreal. Os dois avisosCargo foraWindows continuam semignore. Nenhuma nova dependência/instalação/workflow; qualityINF009 ainda não iniciado. [NoEmit oficial](https://www.typescriptlang.org/tsconfig/noEmit.html) e [handoff](../../planning/handoffs/INF-008.md).
