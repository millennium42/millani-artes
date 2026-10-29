# SAL-008 — avaliação de GHSA-qwgh-2vcv-g2f7

Status atual (2026-10-29): a usuária aprovou explicitamente a exceção delimitada abaixo. O registro canônico está ativo para OSV. CI de origem verde no SHA `1aebfb4277d3f31992860be4b0df9ff930ac3ac9` (run `40883828192`, job `123482109146`). CI final documental verde no SHA `bfe94b4a79cd978f0e096ec85bfbcdbb7e88ac17` (run `40886435465`, job `123491288533`); nota de evidência 153 publicada nesse SHA e ainda presente nas refs remotas atuais. SAL-008 está fechado como `done`/`passes:true` no recorte backend aprovado; UI/IPC, E2E oficial, cobertura Rust global e release permanecem fora do escopo.

Histórico: o CI fonte40588569031/job122486496727 falhou no OSV no SHAe9da06b0c40190d7fe88aa66afcbf09f4133a3c1 antes da decisão humana. Naquele SHA nenhuma dependência, lockfile, scanner, threshold ou exceção vigente havia sido alterada.

## Evidência e classificação

O [aviso publicado pelo mantenedor](https://github.com/RustCrypto/utils/security/advisories/GHSA-qwgh-2vcv-g2f7) delimita o defeito a versões >=0.11.0,<0.12.1. A [base agregada usada pelo OSV](https://raw.githubusercontent.com/github/advisory-database/main/advisories/github-reviewed/2026/08/GHSA-qwgh-2vcv-g2f7/GHSA-qwgh-2vcv-g2f7.json) usa introduced0/fixed0.12.1. A [PR10698](https://github.com/github/advisory-database/pull/10698), ainda aberta na consulta de27/10, propõe corrigir o limite inferior para0.11.0-pre; não é uma correção já aceita.

Cargo.lock resolve somente block-buffer0.10.4, através de digest0.10.7/sha2 0.10.9. O checksum da crate é3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71. O integrador conferiu o arquivo.crate contra o lock e os fontes lib.rs/sealed.rs contra os membros desse arquivo, sem extração ou alteração do cache. O [código da tag0.10.4](https://raw.githubusercontent.com/RustCrypto/utils/block-buffer-v0.10.4/block-buffer/src/lib.rs) usa pos:u8 separado do buffer e não possui ReadBuffer.

Dois testes auxiliares executados contra o rlib em cache identificado pelo dep-info como0.10.4, três posições de cursor por teste, passaram após catch_unwind em callbacks de digest_blocks/set_data. O cursor permaneceu válido e o reset funcionou. Essa execução não foi Miri/ASan, não prova segurança de memória geral e não descarta outros defeitos. Revisão independente Sol6medium E2 confirmou a não aplicabilidade deste mecanismo específico; não executou testes nem aprovou uma exceção.

Na análise anterior à decisão humana, o scan OSV real foi reproduzido sem suprimir nada e continuou falhando, identificando exatamente crates.io/block-buffer/0.10.4/GHSA-qwgh-2vcv-g2f7. A classificação técnica é falso positivo de faixa para esta combinação, sustentada pelo mantenedor e pelo código travado; a usuária aprovou posteriormente somente a exceção delimitada nesta página.

## Proposta exata

Arquivo revisável: [SAL-008-OSV-PROPOSAL.json](SAL-008-OSV-PROPOSAL.json). approved:false e approvalRecord:null.

- Acrescentar somente crates.io / block-buffer /0.10.4 /GHSA-qwgh-2vcv-g2f7 à lista OSV.
- Alvo exclusivo:x86_64-pc-windows-msvc.
- Validade exclusiva antes de2026-11-06UTC, sem prorrogação.
- Cargo.lock SHA256:dcc811e8c3dfb8c4bab85bfb28b836ccc6c1816e16775348c31aed45b4e4d012.
- Cargo.toml SHA256:2d8b42708311f046d0e9c80451aaa8a582a76e15ce7d2a4a0b176f973b9a5b57.
- Preservar as duas entradas glib0.18.5 e proc-macro-error1.0.4 já aprovadas, seus IDs, prazo e hashes. Não alterar o gate cargo-audit.
- Ativação, somente após decisão humana explícita: atualizar OSV_WINDOWS_EXCEPTION.json e registrar a nova decisão/scope em OSV_WINDOWS_APPROVAL.json, preservando a aprovação anterior no Git. Continuar bloqueando qualquer outro pacote, versão, advisory, alvo, hash ou prazo.

SHA256 do arquivo de proposta:9a9e59c941ab1461bd436ba1afb38928d76aa77b3ac2e4077e07bc03a7d7881e.
Digest do escopo (approval_digest):9bca56f8cb316f24d6230414e87ac5982698afc184cc93f955ef28b15e2316f9.

Onze checks sintéticos offline passaram: rascunho desativado; aprovação humana anterior rejeita a entrada nova; combinação exata aceita somente na função check_report com conjunto sintético; cinco versões diferentes, outro advisory, pacote e ecossistema rejeitados. Os39guardchecks OSV do CI já passaram antes do scan real falhar. Nenhum registro humano sintético foi criado e nenhum scan real foi executado com aprovação fictícia. As duas exceções vigentes continuam byteidênticas.

Alternativa sem supressão: aguardar a correção upstream e a propagação ao OSV, conferir que0.10.4 deixa de ser retornada e só então reexecutar o CI no SHA atual. Não repetir workflows enquanto a causa permanece.

## Decisão registrada e limites

A política [DEPENDENCY_SECURITY.md](DEPENDENCY_SECURITY.md) diz: “Achados exigem correção ou exceção temporária aprovada.” Em 2026-10-29, a usuária aprovou explicitamente somente crates.io/block-buffer/0.10.4/GHSA-qwgh-2vcv-g2f7, alvo x86_64-pc-windows-msvc, hashes de entrada registrados acima e expiração antes de 2026-11-06 UTC. O escopo aprovado tem digest 9bca56f8cb316f24d6230414e87ac5982698afc184cc93f955ef28b15e2316f9; OSV_WINDOWS_APPROVAL.json registra a decisão, e OSV_WINDOWS_EXCEPTION.json vincula seu hash.

A proposta JSON permanece como snapshot do rascunho original (approved:false, approvalRecord:null); ela não é o registro canônico ativo. As exceções anteriores glib e proc-macro-error e o gate cargo-audit permanecem inalterados. Não há supressão para outra versão, advisory, ecossistema, alvo ou hash.

Teste de snapshot SAL008/4fixtures, dois mutantes,446Rust+28doctests e review permanecem aprovados localmente nos inputs de código/lock inalterados. Os validadores de docs/OSV/Gitleaks/cargo-audit e as auditorias npm/Cargo, frontend quality/coverage, build Tauri, testes Rust release e verificação PE passaram no CI de origem [40883828192](https://github.com/millennium42/millani-artes/actions/runs/40883828192), no SHA exato acima. O CI final documental [40886435465](https://github.com/millennium42/millani-artes/actions/runs/40886435465), job `123491288533`, passou no SHA `bfe94b4a79cd978f0e096ec85bfbcdbb7e88ac17`; os passos de produto foram pulados nesse run documental e estão cobertos pelo CI de origem. A nota153 está anexada a esse SHA e foi confirmada na ref de evidências remota. A revalidação de 2026-10-29 confirmou que o advisory do mantenedor delimita a faixa a `>=0.11.0,<0.12.1`, enquanto a entrada agregada do OSV ainda marca `introduced: 0` e a PR [#10698](https://github.com/github/advisory-database/pull/10698) segue aberta. UI/IPC, cobertura Rust global e release/E2E oficiais continuam fora deste recorte. Recibos e fontes auxiliares preservados na pasta operacional SAL-008-*20261027.
