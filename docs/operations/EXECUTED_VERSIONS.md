# Versões executadas — INF-012

Snapshot consultado em 2026-10-05T12:10:31.076977+00:00; gates locais encerrados em 2026-10-05T12:11:37.251594+00:00. Checkout base `ab3f751d6fe4be57037c83071f890ac6fc28428c`, branch `codex/inf-012-executed-versions`. Datas em UTC; todos os tempos abaixo são wall-clock medidos pelo líder, em segundos.

Este catálogo distingue pins declarados, pacotes instalados/resolvidos, consultas de versão, execução de checks e observação visual. Cada consulta foi executada pelo P-INT e tem evidência E3 para seu escopo. A consulta de um CLI comprova sua resposta; metadata de arquivo/instância comprova disponibilidade. Nenhuma dessas consultas substitui um build ou a abertura do produto.

[Reprodução](REPRODUCING.md), [registro histórico de ferramentas](../../governance/TOOL_REGISTER.md), [spec](../../planning/specs/INF-012.md) e [handoff](../../planning/handoffs/INF-012.md) preservam origem, procedimentos e limites. O registro histórico não foi sobrescrito com este snapshot.

## Seleção dos executáveis

Node veio de `%LOCALAPPDATA%/MillaniArtesDev/toolchains/node-v24.21.0-win-x64/node.exe`; npm foi chamado por esse Node com `node_modules/npm/bin/npm-cli.js` da distribuição. Rust veio dos shims `%USERPROFILE%/.cargo/bin/`; rustup confirmou seleção pelo `rust-toolchain.toml` do checkout. O PATH dos filhos recebeu apenas esses diretórios para esta execução. `.node-version` declara o pin e não seleciona Node automaticamente.

Git/RTK/Python/PowerShell foram resolvidos no ambiente da sessão: Git em `%USERPROFILE%/Tools/MinGit/cmd/git.exe`, RTK no link WinGet de `%LOCALAPPDATA%/Microsoft/WinGet/Links/`, Python em `%LOCALAPPDATA%/Programs/Python/Python312/python.exe` e PowerShell no runtime do Codex em `%USERPROFILE%/.cache/codex-runtimes/codex-primary-runtime/dependencies/native/powershell/pwsh.exe`. São ferramentas do ambiente, sem novo pin de produto.

Os CLIs frontend foram chamados pelo Node selecionado, usando o campo `bin` de cada package instalado, sem npx ou instalação global. Cargo-audit veio do path fixo descrito na reprodução. MSVC/SDK/VS foram consultados por paths explícitos em `%ProgramFiles(x86)%`, e não por inferência de ausência/presença no PATH.

## Respostas dos CLIs

Comandos abaixo representam os argv reais dos filhos. O orquestrador foi Python stdlib chamado por `rtk proxy`; paths completos foram resolvidos conforme a seção anterior. Os blocos PowerShell adiante são um procedimento equivalente, parseado; não foram executados como script inteiro.

| Ferramenta | Resposta observada | Argumentos | exit | segundos |
|---|---|---|---|---|
| Git | `git version 2.55.0.windows.3` | `git --version` | 0 | 0.035 |
| RTK | `rtk 0.48.0` | `rtk --version` | 0 | 0.024 |
| Python | `Python 3.12.10` | `python --version` | 0 | 0.013 |
| PowerShell | `7.6.5` | `pwsh -NoProfile -Command $PSVersionTable.PSVersion.ToString()` | 0 | 0.547 |
| Node | `v24.21.0` | `node --version` | 0 | 0.028 |
| npm | `11.19.0` | `node <npm-cli.js> --version` | 0 | 0.164 |
| rustup | `rustup 1.29.1 (d95a37b6a 2026-08-13)` | `rustup --version` | 0 | 0.06 |
| Toolchain ativa | `1.99.0-x86_64-pc-windows-msvc; overridden by rust-toolchain.toml` | `rustup show active-toolchain` | 0 | 0.037 |
| rustc | `1.99.0 / b940084d7 / 2026-09-28; host x86_64-pc-windows-msvc; LLVM 23.1.1` | `rustc -vV` | 0 | 0.061 |
| Cargo | `cargo 1.99.0 (5f94df478 2026-08-27)` | `cargo --version` | 0 | 0.058 |
| rustfmt | `rustfmt 1.10.0-stable (b940084d7e 2026-09-28)` | `rustfmt --version` | 0 | 0.061 |
| Clippy | `clippy 0.1.99 (b940084d7e 2026-09-28)` | `cargo clippy --version` | 0 | 0.111 |
| cargo-audit | `cargo-audit-audit 0.22.2` | `cargo-audit audit --version` | 0 | 0.017 |

O hash completo reportado por rustc foi `b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`. LLVM é um componente reportado por rustc, sem consulta de um CLI LLVM separado. Rustup é o gerenciador, não o compilador. Rustfmt/Clippy acompanham a toolchain 1.99.0 e têm numeração própria.

| CLI frontend | Versão | bin instalado chamado com Node / argumento | exit | segundos |
|---|---|---|---|---|
| typescript | `Version 7.0.2` | `node_modules/typescript/bin/tsc --version` | 0 | 0.084 |
| @biomejs/biome | `Version: 2.5.15` | `node_modules/@biomejs/biome/bin/biome --version` | 0 | 0.078 |
| vite | `vite/8.3.2 win32-x64 node-v24.21.0` | `node_modules/vite/bin/vite.js --version` | 0 | 0.166 |
| vitest | `vitest/5.0.3 win32-x64 node-v24.21.0` | `node_modules/vitest/vitest.mjs --version` | 0 | 0.074 |
| @tauri-apps/cli | `tauri-cli 2.12.1` | `node_modules/@tauri-apps/cli/tauri.js --version` | 0 | 0.058 |

## Pins e pacotes realmente consultados

Node 24.21.0 coincide com [.node-version](../../.node-version); npm 11.19.0 acompanha a distribuição consultada, sem pin npm separado. Rust 1.99.0 coincide com [rust-toolchain.toml](../../rust-toolchain.toml), profile minimal, rustfmt/clippy e target Windows x64. Os manifestos/locks permaneceram intactos.

`node <npm-cli.js> ls --depth=0 --json`: exit 0, 0.845s. Todos os dez diretos instalados coincidem com [package.json](../../package.json) e as entradas respectivas do [lock](../../package-lock.json). Isso é metadata instalada, não execução de cada pacote.

| Pacote direto frontend | Pin exato = instalado = lock |
|---|---|
| `react` | `19.3.0` |
| `react-dom` | `19.3.0` |
| `@biomejs/biome` | `2.5.15` |
| `@tauri-apps/cli` | `2.12.1` |
| `@types/react` | `19.3.0` |
| `@types/react-dom` | `19.3.0` |
| `@vitest/coverage-v8` | `5.0.3` |
| `typescript` | `7.0.2` |
| `vite` | `8.3.2` |
| `vitest` | `5.0.3` |

Consulta de módulos: `node --input-type=module -e "import React from 'react';import ReactDOM from 'react-dom';console.log(JSON.stringify({react:React.version,'react-dom':ReactDOM.version}));"` retornou ambas 19.3.0 (exit 0, 0.061s). Importar módulos em Node não prova versão do engine WebView2.

Backend [Cargo.toml](../../src-tauri/Cargo.toml): millani-artes 0.1.0, edition 2021; tauri =2.12.1 e build-dependency tauri-build =2.7.1. Consulta `cargo metadata --locked --format-version 1 --manifest-path src-tauri/Cargo.toml --filter-platform x86_64-pc-windows-msvc --features tauri/custom-protocol`: exit 0, 0.303s; versões resolvidas 2.12.1/2.7.1, 247 nós, custom-protocol presente. Resolução metadata não compila os crates. O audit do lock considera 417 dependências, inclusive outras plataformas; não confundir esse conjunto com o grafo filtrado.

SHA256 dos locks antes/depois dos gates, iguais ao snapshot anterior:
- package-lock.json: `bb9dd4c788dc028f426b4a7d7c3ef3be7ce02b5374ac555455382e9dfb52b284`.
- src-tauri/Cargo.lock: `d7bbb4532172d16c991a533620b7a2e4ad181612d53fc753af1f35f41fd57bca`.

## MSVC, SDK e WebView2

Raiz VS consultada: `%ProgramFiles(x86)%/Microsoft Visual Studio/2022/BuildTools`; compiler `VC/Tools/MSVC/14.44.35207/bin/Hostx64/x64/cl.exe`. Raiz SDK: `%ProgramFiles(x86)%/Windows Kits/10/bin/10.0.26100.0/x64/rc.exe`. vswhere foi chamado da pasta Installer com `-all -products * -requires` e os três IDs de [windows-build-tools.vsconfig](../../scripts/windows-build-tools.vsconfig), mais `-format json -utf8`. Consulta atual exit0/0.031s: instância 1013c2d6, display 17.14.41 (September 2026), installationVersion 17.14.37710.0, path esperado, completa/launchable, sem reboot pendente.

| Consulta atual | Valor observado | exit / segundos | Escopo |
|---|---|---|---|
| cl.exe sem argumentos | banner 19.44.35229 x64 | 0 / 0.014 | resposta do compiler; sem compilação |
| rc.exe /? | banner 10.0.10011.16384 | 0 / 0.014 | ajuda do resource compiler |
| FileVersionInfo cl.exe | FileVersion 19.44.35229.0; ProductVersion 14.44.35229.0 | 0 / 0.62 | metadata PE, consulta compartilhada com rc |
| FileVersionInfo rc.exe | FileVersion 10.0.26100.7705 (WinBuild.160101.0800); ProductVersion 10.0.26100.7705 | mesma consulta | metadata do arquivo no SDK 10.0.26100.0 |
| FileVersionInfo msedgewebview2.exe | pasta/FileVersion/ProductVersion 154.0.4258.53 | 0 / 0.594 | disponibilidade em %ProgramFiles(x86)%/Microsoft/EdgeWebView/Application |

A versão do diretório MSVC, o banner cl, as versões PE e o número do SDK são campos diferentes. O banner rc não é o número do SDK selecionado. Na INF-011 o líder ativou VsDevCmd com `-arch=x64 -host_arch=x64 -winsdk=10.0.26100.0` e executou produção Tauri com esse ambiente. A metadata atual não substitui essa execução histórica.

O smoke INF-002 inclui início do exe identificado, exit0 e confirmação humana E4 de aparência/fechamento. A observação visual não consultou a versão exata do engine pelo app. A disponibilidade WebView2 acima não é um novo smoke, nem aceite financeiro/instalador.

## Gates executados neste snapshot

Não houve instalação, upgrade, alteração persistente de PATH, novo clone, novo build Tauri de produção ou nova janela. As ferramentas foram exercitadas pelos gates existentes com caches locais; essa execução não é outra prova de build limpo.

| Gate local | Comando real (paths dos executáveis conforme seleção) | exit | segundos |
|---|---|---|---|
| quality | `node <npm-cli.js> run quality` | 0 | 4.903 |
| frontend-build | `node <npm-cli.js> run build` | 0 | 1.281 |
| rustfmt | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | 0 | 0.18 |
| clippy | `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | 0 | 8.58 |
| rust-tests | `cargo test --locked --manifest-path src-tauri/Cargo.toml` | 0 | 5.944 |
| npm-audit | `node <npm-cli.js> audit --audit-level=high --json` | 0 | 1.209 |
| cargo-audit | `cargo-audit audit --file src-tauri/Cargo.lock --json` | 0 | 2.776 |

Quality executou formatter/linter (10 arquivos, sem correções), strict typecheck e coverage/testes uma vez. Três testes em dois arquivos, App/main com lines 4/4, statements 5/5, functions 1/1, branches 2/2 (100%). Build frontend validou tipos e Vite/16 módulos. Cargo test executou zero testes: não há prova de invariantes financeiros ou coverage Rust.

npm audit: zero vulnerabilidades em todas as classes, 120 entradas de metadata. Cargo audit 0.22.2: zero vulnerabilidades classificadas, mantendo RUSTSEC-2024-0370/proc-macro-error 1.0.4 (unmaintained) e RUSTSEC-2024-0429/glib 0.18.5 (unsound); nenhum ignore. Ambos ausentes dos 247 nós Windows/custom-protocol consultados. Base RustSec: 1.290 avisos, commit `ef6173cbc5c50ec8166f9a5b28f07834144373ee`, atualização 2026-10-03T10:14:03+02:00. Audit é snapshot, sem garantia futura.

Build nativo isolado E3 reutilizado: INF-011 no SHA `87f996255ddf7d9d9ebc3f7743d9fdbb44439ee4`, `npm run tauri -- build --no-bundle -- --locked`, exit0/129,08s, PE AMD64/PE32+ de 8.574.976 bytes/SHA256 `6ab2a29b01b183f0dcbad54d085e31a2eb5932a10a0d6f052c8adcb9871520f1`. Comparação Git de 22 inputs nativos/frontend/configs/pins/locks selecionados confirmou igualdade com a base atual; o gate de escopo também exige que somente os oito registros reservados mudem. [Handoff INF-011](../../planning/handoffs/INF-011.md) contém isolamento, limpeza, comandos e limites. O artefato temporário foi removido depois da retenção do resumo; não é um instalador ou prova de determinismo bit a bit.

## Procedimento equivalente de consulta

Na raiz do checkout, com os runtimes verificados da reprodução, resolva os executáveis explicitamente. Não atualizar dependências para obter versões. Os comandos a seguir consultam CLIs/metadata e preservam pins/locks:

```powershell
$millaniNodeRoot = Join-Path $env:LOCALAPPDATA 'MillaniArtesDev/toolchains/node-v24.21.0-win-x64'
$millaniNodeExe = Join-Path $millaniNodeRoot 'node.exe'
$millaniNpmCli = Join-Path $millaniNodeRoot 'node_modules/npm/bin/npm-cli.js'
$millaniCargoRoot = Join-Path $env:USERPROFILE '.cargo/bin'
rtk proxy $millaniNodeExe --version
rtk proxy $millaniNodeExe $millaniNpmCli --version
rtk proxy (Join-Path $millaniCargoRoot 'rustup.exe') show active-toolchain
rtk proxy (Join-Path $millaniCargoRoot 'rustc.exe') -vV
rtk proxy (Join-Path $millaniCargoRoot 'cargo.exe') --version
rtk proxy (Join-Path $millaniCargoRoot 'rustfmt.exe') --version
rtk proxy (Join-Path $millaniCargoRoot 'cargo.exe') clippy --version
rtk proxy $millaniNodeExe ./node_modules/typescript/bin/tsc --version
rtk proxy $millaniNodeExe ./node_modules/@biomejs/biome/bin/biome --version
rtk proxy $millaniNodeExe ./node_modules/vite/bin/vite.js --version
rtk proxy $millaniNodeExe ./node_modules/vitest/vitest.mjs --version
rtk proxy $millaniNodeExe ./node_modules/@tauri-apps/cli/tauri.js --version
rtk proxy $millaniNodeExe $millaniNpmCli ls --depth=0 --json
rtk proxy (Join-Path $millaniCargoRoot 'cargo.exe') metadata --locked --format-version 1 --manifest-path src-tauri/Cargo.toml --filter-platform x86_64-pc-windows-msvc --features tauri/custom-protocol
```

Registrar cada exit e resultado; interromper em erro. Para cl sem argumentos, o coletor inicialmente esperava exit2 e recebeu exit0 nesta instalação, falhando na própria classificação. Foi corrigido para aceitar 0/2 apenas nessa consulta informativa; segunda coleta completa passou, sem repetir os gates de build/qualidade/audits. Leituras iniciais usando RTK diretamente com Get-Content falharam por ser cmdlet, depois foram feitas via Python. São correções do coletor/leitura, sem falha de produto ou redução de gate.

CI, revisão e fechamento são registrados no handoff e na nota Git `refs/notes/evidence`; este catálogo não transforma consulta local em execução remota. O produto financeiro, suas migrations/invariantes, coverage de jornadas e setup.exe ainda dependem dos próximos work items.
