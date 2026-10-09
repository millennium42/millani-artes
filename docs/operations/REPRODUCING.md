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

## Comando agregado de qualidade — INF-009

Depois de ativar os runtimes acima, execute na raiz:

```powershell
rtk proxy npm run quality
```

A sequência é format:check → lint → typecheck → coverage, usando os scripts existentes e && para parar na primeira falha. Ela valida o formato sem reescrever arquivos; coverage executa os testes Vitest uma vez e aplica os thresholds existentes. Não precisa executar npm test e npm run coverage novamente para duplicar a mesma verificação. Build, checks Rust e audits continuam separados; este comando valida o frontend atual.

Probes temporárias confirmaram falha no primeiro gate (formato), no typecheck (TS2322) e no último gate (coverage abaixo de 90% em linhas/statements/funções apesar dos três testes passarem). Depois de removidas em finally, quality passou com três testes/dois arquivos/App-main 100% (quatro linhas). Nas falhas de formato/tipo, o relatório anterior permaneceu intacto; format:check não reescreveu a probe. Dist não mudou durante quality e não houve emit/Vite. [Spec](../../planning/specs/INF-009.md) e [handoff](../../planning/handoffs/INF-009.md) registram comandos, resultados e limites; esse resultado não é aceite financeiro ou do instalador.

## CI Windows com qualidade condicional — INF-010

O único job do workflow Documentation continua validando documentos e registros. Em push main e PR, compara commits; diff vazio ou só documentos canônicos/registros de planejamento pula Node, npm ci e quality. Todo path não documental ou desconhecido executa quality; source/CSS/locks/configs/gitignore/workflow/gitattributes disparam. Falha de Git/base/target bloqueia, sem assumir que pode pular. Execução manual força quality; initialzero considera toda árvore.

```powershell
rtk proxy pwsh -NoProfile -File scripts/test-frontend-check-state.ps1
```

São20fixtures Git sintéticas em artifacts/inf010-selector-fixtures, com cleanup do diretório absoluto verificado. O teste inclui rename de source para Markdown, remoção, entradas desconhecidas e base/target inválidos. Não usa dados reais.
Setup-node e checkout ficam fixos por SHA, Node vem de .node-version e cache npm usa package-lock.json. npm ci --ignore-scripts --no-audit --no-fund precede npm run quality; coverage executa Vitest uma vez. Sem matrix/buildTauri/upload/scans remotos nesta tarefa; trabalhos posteriores mantêm seus gates. .gitattributes preserva LF dos arquivos formatados no checkout Windows; probe com autocrlf=true exportou10files iguais aos blobs.

[Spec](../../planning/specs/INF-010.md)/[handoff](../../planning/handoffs/INF-010.md) registram runs/SHA/steps quando executados; configuração ou seleção isolada não prova CI verde, finanças ou release. Nenhum segredo ou dado financeiro no cache/fixture.

## Build limpo com caches isolados — INF-011

E3 no SHA87f996255ddf7d9d9ebc3f7743d9fdbb44439ee4: clone público/caches novos/target vazio antes do build, dependências pelos locks e produção Tauri --locked --no-bundle. Executável PEWindowsx64/8.574.976bytes/SHA2566ab2a29b01b183f0dcbad54d085e31a2eb5932a10a0d6f052c8adcb9871520f1; gates/tempos/hashes/avisos em [handoff](../../planning/handoffs/INF-011.md). Compiladores jáinstalados são pré-requisitos; as menções anteriores a máquina limpa eram objetivos não comprovados. Esta prova não declara OS recém-instalado/VM/determinismo bit a bit/instalação limpa do setup.exe (REL002).

Use sessão dedicada com pins acima e ambiente Developer x64. O líder ativou VsDevCmd.bat -no_logo -arch=x64 -host_arch=x64 -winsdk=10.0.26100.0, confirmou cl19.44.35229/SDK26100/vswhere-utf8 e passou variáveis somente ao filho. Pythonstdlib orquestrou comandos; os blocos PowerShell são procedimento equivalente, parseados e revisados E2, não executados como script inteiro.

Na raiz do projeto, escolher diretório novo, sem sobrescrever. Clone core.autocrlf=false preserva Cargo.lock LF; .gitattributes conserva LFfrontend. Verificar HEAD/tree/index limpos e ausências antes de instalar; um clone futuro pode requerer fetch do SHA testado:
```powershell
$millaniWorkspace = (Get-Location).Path
$millaniCleanRoot = [IO.Path]::GetFullPath((Join-Path $millaniWorkspace 'artifacts/inf011-clean-build'))
if (-not $millaniCleanRoot.StartsWith($millaniWorkspace + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or (Test-Path -LiteralPath $millaniCleanRoot)) { throw 'Raiz existente ou fora do workspace.' }
$null = New-Item -ItemType Directory -Path $millaniCleanRoot
$millaniSource = Join-Path $millaniCleanRoot 'source'
$millaniRef = '87f996255ddf7d9d9ebc3f7743d9fdbb44439ee4'
rtk proxy git clone --config core.autocrlf=false --depth 1 --single-branch --branch main https://github.com/millennium42/millani-artes.git $millaniSource
if ($LASTEXITCODE -ne 0) { throw 'Clone falhou.' }
rtk proxy git -C $millaniSource fetch --depth 1 origin $millaniRef
if ($LASTEXITCODE -ne 0) { throw 'Fetch falhou.' }
rtk proxy git -C $millaniSource checkout --detach $millaniRef
if ($LASTEXITCODE -ne 0) { throw 'Checkout falhou.' }
$millaniActualRef = (rtk proxy git -C $millaniSource rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $millaniActualRef -ne $millaniRef) { throw 'Snapshot diferente.' }
$millaniNpmCache = Join-Path $millaniCleanRoot 'npm-cache'
$millaniCargoHome = Join-Path $millaniCleanRoot 'cargo-home'
$millaniCargoTarget = Join-Path $millaniCleanRoot 'cargo-target'
$null = New-Item -ItemType Directory -Path $millaniNpmCache,$millaniCargoHome,$millaniCargoTarget
foreach ($millaniOutput in @('node_modules','dist','coverage','.vitest','src-tauri/target','src-tauri/gen')) {
    if (Test-Path -LiteralPath (Join-Path $millaniSource $millaniOutput)) { throw 'Output anterior presente.' }
}
$env:NPM_CONFIG_CACHE = $millaniNpmCache
$env:NPM_CONFIG_USERCONFIG = Join-Path $millaniNpmCache 'user.npmrc'
$env:NPM_CONFIG_GLOBALCONFIG = Join-Path $millaniNpmCache 'global.npmrc'
$env:CARGO_HOME = $millaniCargoHome
$env:CARGO_TARGET_DIR = $millaniCargoTarget
foreach ($millaniOption in @('RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTUP_TOOLCHAIN','CARGO_BUILD_TARGET','CARGO_BUILD_TARGET_DIR')) {
    [Environment]::SetEnvironmentVariable($millaniOption, $null, 'Process')
}
Set-Location -LiteralPath $millaniSource
$millaniStatus = @(rtk proxy git status --porcelain)
if ($LASTEXITCODE -ne 0 -or $millaniStatus.Count) { throw 'Snapshot sujo antes dos checks.' }
$millaniLocks = @('package-lock.json','src-tauri/Cargo.lock')
$millaniBefore = @(Get-FileHash -LiteralPath $millaniLocks -Algorithm SHA256)
```
Executar sequencialmente, interrompendo na primeira falha; qualidade já executa testes/coverage uma vez e Tauri já faz buildfrontend. Não executar clippy/test antes do primeiro nativebuild, para provar targetvazio:
```powershell
rtk proxy npm ci --ignore-scripts --no-audit --no-fund
if ($LASTEXITCODE -ne 0) { throw 'npm ci falhou.' }
rtk proxy npm run quality
if ($LASTEXITCODE -ne 0) { throw 'Quality falhou.' }
rtk proxy cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
if ($LASTEXITCODE -ne 0) { throw 'Rustfmt falhou.' }
if (@(Get-ChildItem -LiteralPath $millaniCargoTarget -Force).Count) { throw 'Target não vazio antes do build.' }
rtk proxy npm run tauri -- build --no-bundle -- --locked
if ($LASTEXITCODE -ne 0) { throw 'Build falhou.' }
rtk proxy cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { throw 'Clippy falhou.' }
rtk proxy cargo test --locked --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { throw 'Rust test falhou.' }
rtk proxy npm audit --audit-level=high
if ($LASTEXITCODE -ne 0) { throw 'npm audit falhou.' }
$millaniAudit = Join-Path $env:LOCALAPPDATA 'MillaniArtesDev/toolchains/cargo-audit-0.22.2-win-x64/cargo-audit-x86_64-pc-windows-msvc-v0.22.2/cargo-audit.exe'
rtk proxy $millaniAudit audit --file src-tauri/Cargo.lock
if ($LASTEXITCODE -ne 0) { throw 'Cargo audit falhou.' }
$millaniAfter = @(Get-FileHash -LiteralPath $millaniLocks -Algorithm SHA256)
for ($millaniIndex = 0; $millaniIndex -lt $millaniBefore.Count; $millaniIndex++) {
    if ($millaniBefore[$millaniIndex].Hash -ne $millaniAfter[$millaniIndex].Hash) { throw 'Lock alterado.' }
}
$millaniExe = Join-Path $millaniCargoTarget 'release/millani-artes.exe'
Get-Item -LiteralPath $millaniExe | Select-Object Length
Get-FileHash -LiteralPath $millaniExe -Algorithm SHA256
$millaniStatus = @(rtk proxy git status --porcelain)
if ($LASTEXITCODE -ne 0 -or $millaniStatus.Count) { throw 'Fonte ou lock alterado após os checks.' }
```
Registrar versões/exits/tempos/locks, SHAexe/PEWindows/custom-protocol/dist e status vazio; não aceitar fonte/lock modificado. Para os dois avisos Cargo, executar metadata --locked/--filter-platform x86_64-pc-windows-msvc/--features tauri/custom-protocol e comparar packages aos resolve.nodes; o líder verificou ausência glib/proc-macro-error em247nodes, semignore. Audit é snapshot, não garantia futura.

Depois de reter resumo sanitizado, retornar ao workspace e remover só a raiz temporária absoluta exata, após validar contenção/ausência de reparsepoints externos, usando Remove-Item -LiteralPath. Logs/caches/reports/exe não vão ao Git; fechar sessão dedicada descarta env local. FonteUI igual preserva aceite históricoINF002 sem ampliar E4. Sem finanças/instalador/release; INF012/INF015/REL002 têm seus próprios gates.

## Catálogo de versões — INF-012

Consulte [versões executadas](EXECUTED_VERSIONS.md) para o snapshot de 2026-10-05: seleção explícita dos executáveis, respostas/exits/tempos, pacotes diretos instalados/resolvidos e comparação aos pins/locks. Gates locais quality/build frontend/fmt/clippy/test/audits passaram; consultas de disponibilidade MSVC/SDK/WebView2 têm escopo próprio. Build nativo isolado INF-011 e aceite visual INF-002 são históricos, com fontes iguais, sem novo smoke ou build de produção. Catálogo não comprova finanças, instalador ou determinismo bit a bit. Estado de review/CI/publicação no [handoff INF-012](../../planning/handoffs/INF-012.md).

## Reports publicados no CI — INF-013

[Política/procedimento](../quality/COVERAGE_POLICY.md#artefato-ci--inf-013) define formatos, condição e download. A mudança acrescenta guard/upload ao único job; não repete npm run quality ou testes. Comandos locais de quality e os runtimes/pins acima permanecem iguais. Guard extraído do workflow real foi executado com reports frescos e negativos de ausência/vazio, restaurando bytes;20fixtures do seletor passaram com cleanup.

Allowlist evita enviar outros arquivos de coverage/ ou caches/perfis; reports locais contêm paths do checkout e não são enviados pelo líder. O artifact remoto contém paths técnicos do runner/fontes públicas, acessível autenticado em1dia. SHA/run/artifact/digest/zip/métricas só são declarados depois da inspeção no [handoff](../../planning/handoffs/INF-013.md). Sem novo nativebuild/smoke/setup.exe/alteração financeira.

## Build Tauri no CI — INF-014

O mesmo seletor INF-010 passa a controlar três etapas nativas no único job Windows: preparar o pin Rust, compilar produção e verificar o executável. Diff só documental pula as oito etapas de frontend/nativo; source, teste, configuração e paths desconhecidos disparam. O timeout do job é 15 minutos. Não há novo job, cache Rust/target, instalação Visual Studio/SDK ou upload do executável; coverage INF-013 continua o único artifact, por um dia.

Rustup lê rust-toolchain.toml e instala a versão exata com profile minimal, rustfmt/clippy e target x64 MSVC; seleção ativa deve coincidir com o arquivo. MSVC/SDK vêm da imagem Windows hospedada e descoberta da toolchain. A imagem flutua, e as versões Microsoft específicas usadas não são introspectadas por este build. Os logs registram Rust/Cargo/Node e ImageOS/ImageVersion quando disponíveis. Com runtimes ativados na sessão e dependências do lock instaladas:

```powershell
rtk proxy npm run tauri -- build --no-bundle -- --locked
```

Tauri executa beforeBuildCommand/build frontend uma vez. O workflow compara hashes dos dois locks antes/depois e exige inputs de produto sem diff. Depois confere MZ/PE32+ AMD64, bytes/SHA256 e fingerprint compilado de tauri com custom-protocol, emitindo NATIVE_EVIDENCE JSON sanitizado. Fingerprint comprova feature compilada, sem prova de protocolo em runtime nem vínculo criptográfico ao EXE. Checkout CI sem cache de target evita fingerprint herdado; o build local deste item reutilizou caches e não substitui o build isolado INF-011.

E3 local em 2026-10-05: blocos reais Prepare/Build/Verify executados, Rust1.99.0 x64 por arquivo; build39.629s com frontend16módulos, exe8.557.056bytes/SHA25630326dff2185b815251554b3d453f210ba8b31acf8f59ac3723cfc5ab5aa54f5. Positivo e quatro negativos (exe ausente, MZ inválido, machine incorreta, custom-protocol ausente) passaram; bytes/fingerprints restaurados em finally. Quality3testes/100%App-main, Rustfmt/clippy/test0 e audits passaram. CI entrega37315900046 no SHAfae44697fbd7e693154852d29e7d5827dbf01c06 passou com JSON real conferido; [handoff](../../planning/handoffs/INF-014.md) registra hash/bytes/versões e limites. CI final/recibo de liberação ainda pendentes. Finanças, runtime, instalador e release permanecem pendentes.

No runner ImageOS win25-vs2026/ImageVersion20260925.250.1, Rust/Cargo1.99.0 e Node24.21.0 foram executados. Build nativo265s e beforeBuildCommand uma vez; exe8.560.128bytes, diferente do local (não se promete reprodução bit a bit). package-lock.json é LF pelo atributo; Cargo.lock foi CRLF no checkout Windows. Seu hash de bytes remoto74c1fd05f356bf1d46013b9ce6c5bc09284bea0868f72d56fb52ebde9c3e2093 corresponde exatamente ao blob Git LF convertido para CRLF; hash canônico LF d7bbb4532172d16c991a533620b7a2e4ad181612d53fc753af1f35f41fd57bca. Cada lock conservou seus bytes antes/depois do build. A primeira comparação local-remoto assumiu LF e foi corrigida no verificador de logs, sem editar locks/workflow nem repetir CI. Logs/credenciais foram usados só em memória; nenhum log completo/token/exe foi publicado pelo líder.

## Checkout limpo após checks — INF-015

O guard abaixo exige SHA exato e status Git vazio, incluindo arquivos novos não ignorados. Executar antes/depois dos checks num clone público temporário, com MILLANI_SOURCE/MILLANI_SHA definidos apenas na sessão. Outputs ignorados continuam existindo: comparar mapa de hashes de arquivos rastreados antes/depois e registrar diretórios gerados separadamente. Não executar cleanup sobre o checkout principal.

```powershell
function Assert-MillaniCleanCheckout {
    param([string]$Source, [string]$ExpectedSha)
    $actual = @(rtk proxy git -C $Source rev-parse HEAD)
    if ($LASTEXITCODE -ne 0 -or $actual.Count -ne 1 -or $actual[0].Trim() -cne $ExpectedSha) { throw 'Unexpected checkout SHA.' }
    $state = @(rtk proxy git -C $Source status --porcelain=v1 --untracked-files=all)
    if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect checkout.' }
    if ($state.Count) { throw 'Checkout has tracked, staged or nonignored untracked changes.' }
}
Assert-MillaniCleanCheckout -Source $env:MILLANI_SOURCE -ExpectedSha $env:MILLANI_SHA
```

Procedimento: raiz artifacts/inf015-clean-checkout nova/absoluta/validada sem reparsepoints; git clone --config core.autocrlf=false --depth 1 --single-branch --branch main da URL pública, fetch do SHA exato se necessário e checkoutdetached. Exigir ausência node_modules/dist/coverage/.vitest/src-tauri/target/src-tauri/gen e Git limpo antes npmci. Usar os runtimes INF012 existentes, PATH somente filho, npmci --ignore-scripts --no-audit --no-fund; quality/buildfrontend/fmt/clippy/test/audits e checksdocumentais, guard no fim. CARGO_TARGET_DIR original src-tauri/target reutiliza compilados, sem copiar fonte/perfil; não é prova de caches/target vazios (INF011). Build nativo/installer/smoke não são repetidos. Preservar bytes rastreados e locks, exercitar negativos temporários tracked/staged/untracked com restauraçãofinally. Reter resumo sanitizado, recusar reparsepoints em toda árvore e remover só a raiz exata por Remove-Item -LiteralPath em PowerShell, validar ausência. [Spec](../../planning/specs/INF-015.md)/[handoff](../../planning/handoffs/INF-015.md) registram execução e limites; procedimento executado pelo líder neste item, conforme registro abaixo.

E3 em2026-10-05 no SHAab4e0c09cfcafe6c19be370b6cb42eec464b2bf1: clone público204files/HEADcorreto/status inicial e final vazios. Seis outputs ausentes inicialmente; apósgates presentes node_modules/dist/coverage/src-tauri/gen, todos ignorados; targetRust reutilizado fora do clone no checkout principal, .vitest/target internos ausentes. Os204 hashes individuais permaneceram iguais (digest do mapa ordenado a9419e9cd6ee24b4154527005eb365ad47079783d3316ee29ff8937e3a03822b); locks/pins iguais. Guard real positivo e quatro negativos exit1 (tracked/staged/untrackednãoignorado/SHAerrado), restauraçãofinally e positivo final. Gates8incluindo npmci passaram; novechecksdocumentais e seletor20/cleanup passaram. Cópia temporária removida após prova; cachetarget existente foi mantido. Nenhuma recompilação de produção/smoke/instalador.

Para cleanup, voltar à raiz do checkout principal e usar somente a raiz temporária exata criada neste procedimento. O bloco real abaixo foi executado, verificando2129entries/zeroreparsepoint antes remoção; recusa ancestrais/entradas reparse e destino fora do workspace. Não trocar o literal pela raiz de um checkout de trabalho.

```powershell
$ErrorActionPreference='Stop'
$millaniWorkspace=[IO.Path]::GetFullPath((Get-Location).Path)
$millaniExpected=[IO.Path]::GetFullPath((Join-Path $millaniWorkspace 'artifacts/inf015-clean-checkout'))
$millaniTarget=(Resolve-Path -LiteralPath $millaniExpected).ProviderPath
if ($millaniTarget -cne $millaniExpected -or -not $millaniTarget.StartsWith($millaniWorkspace+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) { throw 'Cleanup target refused.' }
foreach($millaniAncestor in @($millaniWorkspace,(Join-Path $millaniWorkspace 'artifacts'))) {
    if ((Get-Item -LiteralPath $millaniAncestor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Reparse ancestor refused.' }
}
$millaniEntries=@(Get-Item -LiteralPath $millaniTarget -Force)+@(Get-ChildItem -LiteralPath $millaniTarget -Recurse -Force)
if (@($millaniEntries | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }).Count) { throw 'Reparse entry refused.' }
Remove-Item -LiteralPath $millaniTarget -Recurse -Force
if (Test-Path -LiteralPath $millaniExpected) { throw 'Cleanup incomplete.' }
@{cleanup='removed';entries_checked=$millaniEntries.Count;reparse_points=0;absolute_guard=$true}|ConvertTo-Json -Compress```

Comandos npm/Cargo e runtimes são os existentes nas seções acima; o líder selecionou executáveis explícitos e PATH/CARGO_TARGET_DIR somente no processo filho. Os blocos guard/cleanup reais foram executados; a orquestração completa foi Pythonstdlib, sem helper persistido. Sem inferir que todos os exemplos PowerShell anteriores tenham sido executados como um script único. Coverage é scaffold, não finanças/Rust/UIreal; [handoff INF-015](../../planning/handoffs/INF-015.md) guarda pins/hashes/tempos/avisos/review/CI. CI documental de entrega37320497876 no SHAb39098532468b111ce2ea7ea312497fb478302c7 passou, com oito etapas frontend/nativas puladas e zeroartifact; fechamento no SHA final/recibo de liberação ainda pendentes.

## Gitleaks local — SEC-006
Instalação portátil e probes locais em [GITLEAKS](GITLEAKS.md); origem fixa/integridade/licença no registro. Gate remoto de scanner pertence a SEC007; nenhum build de produto novo requerido por esta documentação.

## OSV-Scanner — ferramenta local

[OSV-Scanner2.6.0 portátil](OSV_SCANNER.md) instalado/cache ignorado/Apache2/hash-PE-CLI conferidos. Não requer administrador/PATH/Go/winget/Docker. Instalação SEC008 não executou scan de dependências nem gateOSV;CI seráSEC009. Procedimento registra versão/origem/license/probes/NotSigned/SLSA não verificado/remoção dedicada e limites. Produto/pins/locks/thresholds intactos;entregaCI/review/liberação ainda pendentes.

## SEC-008 — CI de entrega / fechamento documental
Entrega pública c6b4a27bae6ca31e17e80eacd984c5f07163d1d2 / [CI37774382975](https://github.com/millennium42/millani-artes/actions/runs/37774382975), job113301471481/attempt1/main/SHAexato/success19etapas(11success8skip)/zeroartifacts. Root conferiu API dos runs/jobs/steps/artifacts;etapa Scan Git history for secrets obrigatória success antesbuild. Seletor real false baseefb25a13→entrega;nenhum OSV remoto/download/scan integrado,novo job/action/cache/upload,rerun ou CI negativo. Descoberta ghapi indisponível;credentialmanagerGit existente reutilizado somente em memória para API GitHub,sem log/credencial publicado e sem redirects autenticados.
Executor SEC008 done/passestrue/E3 somente instalação local:ausênciaRED/3arquivos oficiais fixos/version2.6.0/4probesCLI/AuthenticodeNotSigned/6casos documentais apósP2/3hashes originais preservados/cleanup concluído/re-reviewsemP0P1P2/checks docs/diffsegredos/CI entrega verde. GatesOSV/dependências sãoSEC009 e posteriores,semclaimzero vulnerabilidades/E4novo. Root4validadores finais/10paths/50JSONanteriores intactos245tuplas/51items48done240files271links;log208642base preservado,entrega213920. Scansegredos diff finalentrega exit0/0achados/0.447s/SHA2564adb72f5e9421b59f32a88a4eac22a2a2e638a7709e2f9eff4f2f3384b95fb05. Reserva10ATIVA até CI do fechamento noSHAexato/nota49preserva48/publicrefs-Gitlimpo/liberação. Recibo final em notas Git evita commit recursivo. Semproduto/manifests/pins/locks/grants/thresholds/workflow/seletor alterado. METAativa:finanças/SQLite/backup/setup.exe/release pendentes. PróximoSEC009←SEC008,INF010;não iniciado neste ciclo.

## SEC-014 — teste de telefone sem log bruto

Ativar pins/Developerx64 conforme procedimento existente. Executar cargo fmt --check, clippy --all-targets --features tauri/custom-protocol -- -D warnings; npm run build antes de cargo test --locked --manifest-path src-tauri/Cargo.toml --features tauri/custom-protocol -- --include-ignored --nocapture. include-ignored executa também o probe realWebView2/CSP/IPC que o teste defaultpula; não substituir essa prova por default4passed1ignored. Audits existentes/políticaE4 e thresholds não mudam.

Novo testephone usa4filhos próprios/mesmoexe de teste, CLI/envcfg(test),Tauriinitreal/contextosemjanela,faultinjectionErrorIo na fronteira produtiva/hookRust/stdoutstderr capturados/sinkstderrfechado. Buffers sóemmemória/bounded16KiB, timeout20s porhandle/kill-wait, sem rawdump/perfil/banco/log persistido. Produção não lê o env/args de fixture. Marcador PHONE_REDACTION_EVIDENCE contém contagens/flags, nenhum telefone. [Spec](../../planning/specs/SEC-014.md)/[handoff](../../planning/handoffs/SEC-014.md) registram SHA/checks/limites. SuiteRust é executada localmente; workflow existente compila produção/audita, sem alegar executar essa suite no runner. GUIsemstderr/falha de sink descarta registro seguro; mantém failure e não habilita fallback bruto. Não cobre finanças/backups/FFI ou todos os emissores.

Regressão WebView existente retém artifacts/sec001-webview-profile; antes de repetir, exigir origem da execução própria/handleterminal, targetabsolutoexato dentroartifacts e todaárvore/ancestrais semreparse, então remover somente essa raiz. Não limpar perfil preexistente desconhecido. Root corrigiu esseestado apósbaseline e passou6tests0ignored, depois removeu perfil controlado. Novo phoneprobe usa somenteprocessos/capturas emmemória, semperfil. Detalhes de falha/checks/sourcehash/faultinjection estão na spec.

SEC014 VERIFY final apósP2: cinco filhos (quatro privacidade + guard before-injection102), confirmação AtomicBool após build real impede falso positivo; seis Rusttests/zeroignored/fmtclippy green. [Prova e hashes finais](../../planning/specs/SEC-014.md#verify--correção-p2-executada). E3local/re-reviewCIpending/11ATIVA; fronteira Err com faultinjection, não falha natural da biblioteca; METAativa.

SEC014 executor done/passestrue/E3 após entrega55b481af4d394aa624d384b6d38887bbdf898a04/[CI37857821138](https://github.com/millennium42/millani-artes/actions/runs/37857821138)/attempt1/23success. Cincofilhos/seisRustlocal/P2corrigido/re-reviewsemachados; [prova/limites](../../planning/specs/SEC-014.md#ci--learn--entrega-validada). Fechamento9docs/reserva11ATIVAatéCIfinalnota55refsGitclean; SEC003global parcial/METAativa/SEC015nãoiniciado.

## SEC-015 — dados financeiros na fronteira atual

[Spec](../../planning/specs/SEC-015.md) →[tests reais compartilhados](../../src-tauri/src/startup_tests.rs) →[handoff](../../planning/handoffs/SEC-015.md). E3local: RED de sensibilidade expect antiga detectou vazamento financeiro, restauraçãofinally; cinco filhos financeiros/cincophone, seteRust0ignored/fmtclippyaudit green. Main produtivo inalterado. Centavos/saldos/preços/quantidade/conta/categoria/contextos/IDs/agregados e formato/UnicodeCRLF/nestedunknown sintéticos ausentes de stdoutstderr, saída e status estritos/injeção pósbuildreal/guard102. Não simula falha natural do Builder.run nem prova transação financeira/SQLite/backup/all-emitter/FFI; requisitoSEC003global continua parcial. Review/CI/nota56-liberação pendentes/passfalse/11ATIVA/METAativa/ARC001nãoiniciado.

SEC015 executor done/passestrue/E3 após entrega96c06e66e53f8c9776d51576bf7f467102487ad2/[CI37860308161](https://github.com/millennium42/millani-artes/actions/runs/37860308161)/attempt1/23success; [spec/prova/limites](../../planning/specs/SEC-015.md#ci--learn--entrega-validada). Cincofinance+cincophone/seteRustlocal/productionunchanged/reviewE2fontessemachados; CI não executaRustsuite. Reserva11ATIVAatéCIfinalnota56/refsGitclean/METAativa/ARC001nãoiniciado; globalSEC003parcial.


## Camadas atuais — ARC-001
Paths reais e responsabilidades em [MODULE_BOUNDARIES](../architecture/MODULE_BOUNDARIES.md#estrutura-atual--arc-001). Raízes Rust: src-tauri/src/domain/mod.rs e application/mod.rs; Tauri/stderr: infrastructure/mod.rs; composição: main.rs; UI: src/ui/App.tsx, chamada por src/main.tsx.
Root executou npm run quality/build e cargo fmt/clippy all-targets custom-protocol/test --include-ignored/audit, sob RTK e ambiente MSVC/SDK já registrado. Refatoração não altera locks/thresholds/CSP/grants. Sete testes Rust incluem WebView2 real; CI atual compila produção e testa frontend, não executa essa suite Rust.
Probe de arquitetura local: criar diretório próprio ignorado artifacts/arc001-layer-check somente se inexistente, copiar domain/mod.rs e application/mod.rs byte a byte para domain/mod.rs e application/mod.rs nesse diretório. Wrapper probe.rs declara mod domain; mod application; e main chama application::run_startup(||Ok::<(),()>(()), |_|{}), assertando ExitCode::SUCCESS. Executar rtk proxy rustc --edition=2021 probe.rs -o probe.exe com cwd desse diretório e depois rtk proxy ./probe.exe. Root também verificou erro opaco sem Debug/Display e panic; em cópias, acrescentar use tauri::Runtime; ou use crate::infrastructure; ou use rusqlite::Connection; em cada raiz deve falhar com E0432/E0433. Restaurar a cópia entre6controles e remover somente o diretório próprio/contained/semreparse após processo terminal. Árvores sem raízes devem falhar E0583, distinto de erro do runner.
Prova de isolamento destes módulos, sem gate permanente de todos os imports nem regra financeira. [Spec](../../planning/specs/ARC-001.md) e [handoff](../../planning/handoffs/ARC-001.md) registram resultados, hashes e CI exato.


ARC-001 executor done/E3/passes:true: [entrega](https://github.com/millennium42/millani-artes/commit/e94eec776213725f72239c8d07aff2a293f6f632)/[CI37863336200](https://github.com/millennium42/millani-artes/actions/runs/37863336200) success23steps/buildWindows, std-only6controles e7Rust locais/reviewE2fontes semachados. Domínio sem regra; finanças/SQLite/backup/setup.exe pendentes/METAativa. Reserva22ATIVA atéCI final/nota57, fechamento11docs; detalhe na spec/handoffARC001. ARC002 não iniciado.

## Result/DomainError e medição local — ARC-002
LibRustsrc-tauri/src/lib.rs exporta domain; Cargo detecta lib automaticamente, main mantém sócomposição app/infra. [Tipos/testes](../../src-tauri/src/domain/mod.rs) têm catálogo unitInvalidInput sempayload, stdResult e Displaystatic. Fmt/clippyalltargets/testincludeignored produzem3libtests+7bintests; doctests0. UI unchanged/3testsquality, locksE4/pins intactos.
Ferramenta oficial [rustcLLVMcoverage](https://doc.rust-lang.org/rustc/instrument-coverage.html): rtk proxy rustup component add llvm-tools-preview --toolchain1.99.0-x86_64-pc-windows-msvc somente local, LLVMtools23.1.1 conforme registro. Não foi instalada em CI, semcargo-llvm-cov/nightly. Descobrirtoolroot por rustc --printsysroot + lib/rustlib/x86_64-pc-windows-msvc/bin; conferir llvm-cov --version == LLVMversion do rustc -vV.
Criar artifacts/arc002-coverage-check próprio/novo/ignored/contained/semreparse. Da raizrepo: rtk proxy rustc --edition=2021 --test src-tauri/src/lib.rs -Cinstrument-coverage -o artifacts/arc002-coverage-check/domain-tests.exe. Processo com cwd próprioscratch e LLVM_PROFILE_FILE=<scratch>/full/%m-%p.profraw executa rtk proxy ./domain-tests.exe (3tests). llvm-profdata merge -sparse <perfis próprios> -o full.profdata; llvm-cov export domain-tests.exe -instr-profile=full.profdata -summary-only. Parse somente registro filename do source real domain/mod.rs; exigir linhas.count>0/coverage≥90, registrar count/covered/functions/regions/branches. Root observou3/3linhas/1/1função/5/5regiões; não contar testes/outrosarquivos nem chamarregiõesstatements.
Controle em diretório/perfil novo separado executa mesmoexe --skip domain::tests::error_has_static_display_and_no_source:2pass/0linhascovered, deve rejeitar gate90. Nunca merge perfis negativos com completos ou reusar report. Stockstable1.99 rejeita -Zcoverage-options=branch exit1/nightlyrequired; branches.count0 documentado/sem100% fictício. Remover somente scratchpróprio após processos terminais e checkscontained/semreparse. [Spec](../../planning/specs/ARC-002.md)/[handoff](../../planning/handoffs/ARC-002.md) ligamreporthash ao código; percentual não cobre Rustglobal, finanças ou CI.
