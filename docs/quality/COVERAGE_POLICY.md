# Política de cobertura

Quando Vitest existir: global lines/statements/functions ≥90%, branches ≥85%. Arquivos críticos de saldo, transferência, venda, recebimento, devolução, reembolso, recorrência, transação e backup/restore: 95/95/95/90, aplicado por glob/per-file. Não reduzir para passar; exigir exceção humana/ADR.

Se Rust contiver domínio, medir cobertura de linhas e habilitar branches se o toolchain permitir; documentar limitação se não permitir. Sempre rodar `cargo fmt --check`, clippy com warnings erro, test e audit.


## Aplicação no inventário — INF-007

O inventário TS desta revisão contém App.tsx/main.tsx de bootstrap, testes e configs; não contém cálculo de saldo, transferência, venda, recebimento, devolução, reembolso, recorrência, transação ou backup/restore. Logo há zero arquivos críticos atuais, sem aplicar o limite crítico ao bootstrap. main.rs também contém apenas inicialização Tauri, sem domínio para medir coverageRust.

vitest.config.ts aplica o global90/90/90/85 a todo src TS/TSX, inclusive não importado, excluindo somente testes. npm run coverage emite texto no console e grava json-summary, LCOV e HTML em coverage/; npm test conserva coverage habilitada. Relatórios locais continuam ignorados no Git e não são aprovação financeira; publicação condicional no CI segue o procedimento INF013 abaixo.

Cada work item que introduzir lógica crítica deve, antes de passes=true:
- identificar os paths reais pela função de negócio/implementação observada, incluindo infraestrutura de backup/restore quando aplicável;
- acrescentar esses paths ou globs comprovadamente abrangentes em coverage.thresholds, com perFile:true e lines95/statements95/functions95/branches90; não depender só de média do grupo;
- registrar classificação, testes explícitos de invariantes e evidência na matriz/spec/handoff, verificar que todos os arquivos críticos estão abrangidos e executar os gates;
- para domínio Rust, aplicar a regra Rust acima e documentar a capacidade/limitação real do toolchain.

ARCHITECTURE/MODULE_BOUNDARIES define responsabilidades, sem fixar diretórios TS; ARC001 e implementações posteriores definirão paths. Não pré-cadastrar caminhos fictícios. Quando surgir arquivo crítico, a ausência do gate porarquivo é pendência daquele item, não exceção implícita. Limiares acima permanecem canônicos.

O mecanismo nativo Vitest5.0.3 foi exercitado com arquivos sintéticos temporários: média global/glob≥95 não rejeita um arquivo mal coberto; perFile:true identifica/rejeita esse arquivo abaixo95/95/95/90, e cobertura completa passa. Esses probes e seu glob não persistem; não afirmam implantação financeira. [Reprodução](../operations/REPRODUCING.md) e [handoff](../../planning/handoffs/INF-007.md).

## Artefato CI — INF-013

O job Documentation publica frontend-coverage-<run_id>-<run_attempt> somente quando o seletor exige frontend e quality passou. O guard exige coverage-summary.json, lcov.info e lcov-report/index.html não vazios; ausência/vazio falha. Upload allowlist: summary, LCOV e árvore HTML com assets, sem coverage JSON completo/cache/perfis/dados reais. Vitest roda uma vez pelo quality; clean:true permanece, sem reports reaproveitados de cache. Falha de quality impede upload e não guarda relatório de falha. Thresholds globais e porarquivo crítico acima permanecem canônicos.

Action oficial fixada por SHA no registro, retenção1dia/compressão6/hiddenfilesfalse, mesmo job/permissões existentes. Diff documental pula Node/instalação/quality/guard/upload; mudança relevante ou execução manual exige frontend. Não há serviço externo, comentário em PR, badges ou site de coverage.

No run GitHub, abra Artifacts e baixe o ZIP com login; abrir lcov-report/index.html localmente ou usar lcov.info/coverage-summary.json. Retenção curta significa baixar em1dia; conteúdo/hash/métricas observados ficam no [handoff INF-013](../../planning/handoffs/INF-013.md) e nota Git. Esses reports mostram fontes já públicas e caminhos técnicos do runner, somente fixtures sintéticas; não publicar logs/backup/financeiro real/chaves. Retenção não é economia monetária medida.

Configuração e gates locais não comprovam publicação; somente artifact do run/SHA exato inspecionado permite E3 de upload. Estado/prova e limites no handoff; Rust/financeiro/instalador continuam pendentes.

## Medição Rust atual — ARC-002

Entrega93362971/[CI37865774054](https://github.com/millennium42/millani-artes/actions/runs/37865774054) confirmou cobertura frontend100%4lines/5statements/1function/2branches. Rust abaixo foi medido localmente, vinculado aos hashes da [spec](../../planning/specs/ARC-002.md); fechamento preserva fontes/thresholds.
DomainError/Result em [domain/mod.rs](../../src-tauri/src/domain/mod.rs) têm somente fmt executável: RustLLVMinstrumentation mediu3de3linhas/1de1função/5de5regiões,100%. Alias/enum/derives/implError vazio não têm statements de negócio medidos; regiões LLVM não equivalem a statements. Esse arquivo não calcula saldo/venda/transferência/transação/backup e não é crítico financeiro; gate local de linhas≥90 aplicado sem redução. Thresholdsglobais/perfile críticos90/90/90/85 e95/95/95/90 continuam para seus escopos canônicos. Não declarar percentual global Rust/Tauri/financeiro com esse report.
Stockrustc1.99.0 suporta -Cinstrument-coverage; llvm-tools-preview pin1.99 usaLLVM23.1.1. Controle sem testeDisplay:2testspass/coverage0% e gate rejeita; perfil novo completo3testspass/100% aceita. -Zcoverage-options=branch rejeitado pelo stable (nightlyrequired), exportbranches.count0; percentbranch não registrado/aplicável, nunca interpretar0counters como100%. SemRUSTC_BOOTSTRAP/nightly/threshold reduzido. [Reprodução](../operations/REPRODUCING.md)/[spec](../../planning/specs/ARC-002.md) registram a limitação, hashes e arquivos reais. Medição local e report efêmero não são CI remoto Rustcoverage; ampliar/gatesporarquivo ao surgir domínio financeiro crítico.


## Portas abstratas — ARC-003
[Traits](../../src-tauri/src/domain/ports.rs) têm zero linhas executáveis/defaults, sem percentual fictício. LLVM local existente mediu domain/mod.rs3/3linhas1/1função5/5regiões100%; controle5tests semDisplay0% rejeita90, perfil novo6tests100% aceita. [Três testes de injeção](../../src-tauri/src/domain/ports_tests.rs) provam tempo/sequência/falha, sem medir geração real. Branchcounter0/limite stableARC002 mantido, nenhum hack/threshold reduzido. [Spec](../../planning/specs/ARC-003.md): sem cobertura globalRust/financeiro/adapters registrada.


ARC-003 executor E3 validado na entrega6eafaa40/[CI37963747801](https://github.com/millennium42/millani-artes/actions/runs/37963747801), root13Rust/5negativos/reviewE2 sem achados. Fechamento documental preserva fontes/thresholds/inputs; reserva14 até CI final/nota59 no [handoff](../../planning/handoffs/ARC-003.md). Portas não comprovam UUID real/unicidade/relógio monotônico/invariantes financeiras.

## FIN-001 — Account

LLVM 23.1.1 mediu diretamente `src-tauri/src/domain/mod.rs` compilado com `rustc --test` (sem `lib.rs`, wrapper ou dependência): full 7 testes, 3/3 linhas executáveis, 1/1 função, 5/5 regiões; controle 6 testes, 0/3 linhas e rejeição pelo gate90. `Account` não contém statements executáveis, então não recebe cobertura percentual fictícia. Probe de branch no stable falhou com `nightly required`; nenhum percentual de branch foi inferido. CI de entrega `38444911363` não executa suíte/doctests/cobertura Rust. A medição local não equivale a cobertura Rust global/financeira e não altera thresholds. Relatórios/hashes na [spec](../../planning/specs/FIN-001.md); limites do CI no [handoff](../../planning/handoffs/FIN-001.md).
## FIN-002 — cobertura direta do domínio
LLVM 23.1.1 direto em domain/mod.rs: perfil full 8/8 linhas, 2/2 funções e 11/11 regiões; controle 3/8 linhas (37,5%) rejeitado pelo gate de 90%. Perfis full e controle separados; hashes dos relatórios b25bf05d1d88dc5547add9f19df1f0e65cf173d2eb3ac5343b874c0974f03f9f e bb8e169052280c3842cde642a4a17dd51dbd7756273c77f3087ba774f35e6891. Probe branch requer nightly, counters zero não equivalem a 100%. Mede somente o módulo, não Rust global nem regras financeiras.

CI de entrega 38483459997 executou testes Rust release padrão (57 ordinários+2 doctests) e passou, mas não coleta cobertura Rust. Artifact de coverage no CI cobre somente scaffold frontend: 2 arquivos/3 testes/100%; não inferir cobertura de domínio ou finanças a partir dele. Relatórios LLVM locais do domínio permanecem a evidência de cobertura direta.

## FIN-003 — enum AccountType
LLVM23.1.1 direto no módulo: full11 tests,8/8 linhas,2/2 funções,11/11 regiões; controle8 tests,3/8 linhas (37.5%) rejeitado no gate90. Reports 0e5f1e066772f403ec4d074acd543c8e2c94e8b1da0b7be70544f3996ae53e44 e 2afcc6d17b1a5306cf37e47d6e6870306fd67e792c34bfcfc143e893966ec53c. AccountType não contém statements executáveis e não recebe cobertura fictícia. Stable branch counters não medidos. Medição é local e limitada ao módulo, sem coverage global/financeira.

CI entrega38487235749 passou suíte padrão release58 ordinários+2 doctests/1 ignored, mas não executou Rust coverage. Artifact coverage remoto é frontend scaffold2 arquivos/3 testes/100%; não representa AccountType nem regras financeiras. Manter LLVM local como medida focal e não inferir cobertura global.

## FIN-004 — campo de saldo inicial
LLVM direto: full12 tests,8/8 linhas,2/2 funções,11/11 regiões; controle9 pass/3 filtered,3/8 linhas (37.5%),1/2 funções,5/11 regiões, rejeitado gate90. Reports full48b8b855c1d0ffd6411307f5e2251bc13f75fd3d3f0ec12447f5c8d9886fcffa e control2cf1509b5a2978da7e98a8fbcec37edd2689e83d6c976314d933465b1c85e1f6. Campo de dados não adiciona statement executável; sem percentual fictício. Branches stable não medidos; sem coverage global/financeira.


FIN-004 medição global E3: produção 301/312 linhas (96.474359%),30/33 funções (90.909091%),373/408 regions; gates existentes global90 e database95 linhas/funções PASS. Statements não medidos, branches indisponíveis no LLVM estável; lib/ports sem linhas executáveis N/A, binário normal não executado. Não alegar vetor completo nem cobertura financeira.

FIN-004: medição global Rust reportada é local E3. CI de entrega 9e2068b não executou coverage Rust; artifact remoto mede somente scaffold frontend. Contagens/limitações globais registradas no handoff; sem alegar vetor completo ou cobertura financeira.

CAT-001 declaração de dados não adiciona linhas executáveis. Métricas standalone: full8/8 linhas/2/2 funções; controle3/8 linhas e gate90 rejeitado. Evidência pertence à worktree; medir gates primários sem inferir statements de regions ou atribuir coverage artificial ao campo.

CAT-001 LLVM local primário full13:8/8 linhas,2/2 funções,11/11 regiões; controle10/3 filtrados:3/8 linhas,1/2 funções,gate90 rejeitado. Category sem linhas executáveis; regions não são statements, branches estáveis indisponíveis. Não confundir com coverage global FIN-004 nem inferir 100% para o campo.

CAT-001 métricas de módulo primário permanecem locais E3: full13,8/8 linhas,2/2 funções; control3/8 e gate90 rejeitado. CI final não rodou LLVM Rust. Category não tem linhas executáveis; sem inferência de statements a partir de regions.

FIN-010 standalone LLVM: full13, 8/8 linhas,2/2 funções; controle3/8 linhas (37,5%) rejeita90. Pertence à worktree; enum sem linhas executáveis é N/A. Coverage focal primária e global não medida nesta integração.

FIN-010 LLVM primário full14:8/8 linhas,2/2 funções,11/11 regiões; controle11/3 filtrados:3/8 linhas,1/2 funções,gate90 rejeitado. Enum sem linhas próprias (N/A); branches não disponíveis, regions não são statements. Não alegar coverage global.

FIN-010: métrica LLVM local do módulo permanece full8/8 linhas,2/2 funções e controle3/8 (gate90 rejeitado); CI não executa coverage Rust. Global coverage permanece não demonstrada, sem alteração de threshold.


## FIN-006 — código executável e cobertura
FIN-006 acrescenta SQL e registro por constantes/include_str!, sem novo código Rust executável. Os 43 testes SQLite exercitam migration, mas não substituem coverage Rust nem alteram thresholds. Métricas globais/criticas não foram reportadas no plano pré-execução; branches estáveis não medidos são N/A e LLVM regions não são statements.

FIN-006 medição focal E3: database.rs 276/280 linhas=98.57142857%,25/25 funções=100%; gates críticos ADR-019 >=95% linhas/funções PASS. Instantiations67/73 e regions343/368 são métricas distintas, não statements/funções; statements não medidos. Branches estáveis 0/N/A; outros módulos instrumentados excluídos. Não alegar vetor completo nem coverage global.


FIN-006 coverage focal local: database.rs 276/280 linhas (98.57142857%),25/25 funções (100%), gates críticos ADR-019 ≥95% PASS. Statements não medidos, branches N/A; métricas instantiations/regions distintas. CI não prova coverage global/vetor completo.

CAT-002 mantém somente evidência de cobertura focal Rust já registrada: full17 PASS, 13/13 linhas, 3/3 funções, 17/17 regions; controle11 PASS/6 filtrados, 3/13 linhas/1/3 funções/5/17 regions e gate90 rejeitado. Statements não medidos, branches 0/N/A; regions não são statements. CI exata 38616439285 fechou gates do executor, mas frontend coverage de 2 arquivos/3 testes/100% cobre apenas scaffold. Sem vetor Rust completo nem coverage global. Cleanup focal E3 verificado; drafts externos RustSec permanecem preservados.

FIN-011 focal primary E3: full LLVM20 PASS, 19/19 linhas, 4/4 funções, 25/25 regions (report `ea52cbdc9bbc07a469c69592398fa2c1bae39508d6c0ef968390ee53ea8f9d69`); controle excluindo os três testes Context:17 PASS/3 filtrados,13/19 linhas,3/4 funções,17/25 regions, gate90 rejeitado (report `e7ce1210b609fc50d66986fa31a636aaabcbfb242a30e9fe42684e2b94b55ead`). Branches0/N/A, statements não medidos; regions não são statements, sem claim global/full vector. Esse recibo local não representa artifact coverage frontend, que cobre só scaffold. Source CI executor concluído; publicação documental pendente.

FIN-007 primary LLVM E3: `database.rs` 276/280 linhas (98,5714%) e25/25 funções; gate crítico ADR-019 ≥95% PASS. Report `76e55e628ec8d3affdc9a2d595fc46443e30777def3f6cf21b5fe4144f45de56`; control sem SQL report `9d98945c7f3b11293d58655fab40ba11b24ff60a08375b77ed5718eabfef1908` rejeita95. Statements não medidos, branches0/N/A, regions/instantiations não equivalem a functions; sem global/full-vector claim. Source CI pendente.

Erratum FIN-007: source CI `38628675813`/job `116074330758` passou no SHA `a58ad20b7664db5f7b5fb85a3cb7b0b341a46978`; coverage acima é focal local, não cobertura executada pela CI. Sem claim global.

FIN-008 primary focal LLVM E3: application 50/50 linhas,4/4 funções,59/60 regiões; adapter 56/56 linhas,5/5 funções,55/55 regiões. Gates críticos de linhas/funções ≥95% passam. Full report `5c9210baab546ef0eb2bed61fff2089711523d9a4a901dd950e4a15556bfade1`; controle exclui7 app+3 SQL, e em 20 unit+45 SQL PASS deixa as duas produções sem linhas/funções cobertas e rejeita95 (report `0ac3a8a7d4828c39ada7732bb834b295af6836b4603e5de5c6573e1b81a8ba31`). Statements não medidos, branches0/N/A, regions não são statements; sem vetor completo/global. Limpeza de coverage e target própria verificada E3; CI não é alegada como coverage LLVM.
## FIN-008 — evidência do executor (E3)
FIN-008 passa os gates focais ≥95% de linhas e funções em application/adapter; statements não medidos e sem vetor/global coverage claim. Source CI 38765481430 no SHA eea53ff05f2df34c5ff8e9aa759b70dc0a25ab57.

## CAT-003 — integracao primaria 16/10/2026

LLVM23.1.1 focal database.rs276/280linhas(98,5714%),25/25funcoes(100%);gatescriticos>=95PASS;controle semSQL0linhas/0funcoes rejeita95. Statements nao medidos,branches stableN/A,regioes nao sao statements;sem global. Reports692744e2cd871c612cafc38bde8a907946a756a54072c2415bf5291f871bbf1f e bd9d0ead5e1ef8c1f7606b479c5501e09de42727ec1a5892de2dc612fabf1b82 preservados externamente. Fonte: planning/handoffs/CAT-003.md e planning/specs/CAT-003.md, SHA7573c65899416cf180bb25545f6cfae6335bb00e; E3 local, CI de entrega em execucao, item ainda aberto.


CAT-003 executor done/E3/passestrue apos CI38907719490/job116995418974/attempt1/SHA7573c65899416cf180bb25545f6cfae6335bb00e SUCCESS23/0. Evidencia e limites em planning/handoffs/CAT-003.md;CI documental final/nota87/liberacao pendentes. FIN-012/UI-001 continuam abertos.


## FIN-012 — integração primária em 16/10/2026

Report LLVM integrado: domain/mod.rs 19/19 linhas e 4/4 funções. Movement/MovementKind não adicionam linhas executáveis próprias. Statements não medidos, branches stable N/A; sem cobertura global ou percentual artificial para declarações. Fontes e evidência: planning/specs/FIN-012.md e planning/handoffs/FIN-012.md; source 7573c65, main 7cad038, produção idêntica. Item ainda aberto até fechamento documental.

FIN-012 executor done/E3/passes true após os checks e CI do código integrado, com fonte em planning/handoffs/FIN-012.md. CI documental final, nota88 e liberação pendentes. UI-001 aguarda liberação. Sem comprovação de consumidor, regras financeiras, coverage global ou instalador.


## UI-001 — verificação primária em 16/10/2026

Fonte: planning/handoffs/UI-001.md, source7573c65/main59a3789, E3 local.

```json
{
  "V8_shell": {
    "statements": "5/5",
    "branches": "2/2",
    "functions": "1/1",
    "lines": "4/4"
  },
  "configuração": "inalterada; limiares canônicos preservados",
  "limite": "sem coverage Rust global ou finanças"
}
```


UI-001 executor concluído: E3 de execução e inspeção nativa, checks do código idêntico e CI source38907719490 SUCCESS23/0; review integrado/visual E2 sem P0/P1/P2. Documentos e matrizes atualizados antes de passes true. Reserva retida até CI documental final no novo SHA, nota89/preservação88notas/Gitlimpo/WTarquivada e liberação do root. Código de saída nativo não registrado(null); sem claim exit0, WCAG numérico, mínimo de produto, jornada financeira, Rustglobal ou setup. Próximo após liberação: reconciliar elegibilidade FIN-005 no roadmap.


## FIN-009 — leitura de contas, E3 local em 17/10/2026

Adapter: 88/88 linhas, 7/7 funções. Controle: 56/88 linhas, 5/7 funções. Aplicação: 0/9 linhas, 0/3 funções. LLVM 23.1.1 compilou. Clippy passou. Cobertura não é global. Estes números são focais; full application9/9linhas3/3funções. Controle rejeita o gate95%; não mede percentual de chamadas. Statements não medidos e branches stableN/A; regiões não são statements. Cobertura global separada no registro abaixo.


FIN-009 LLVM23.1.1 global local E3:459/470linhas=97,659574%,46/49funções=93,877551%; global90PASS. Críticos conservadores create_account50/50linhas4/4funções, list_accounts9/9e3/3, accounts88/88e7/7, database276/280(98,5714%)e25/25:95PASS. Produção somente src-tauri/src;8arquivos reportados,lib.rs/ports.rs semlinhasexecutáveis N/A; testes/deps/gerados/tooling excluídos. Main normal incluído0/3linhas0/1função(não executado); infraestrutura startup6/10linhas1/3funções. Instrumentado98ordinários+8doctests include-ignored,0ignored. Statements não medidos,branches stableN/A;regiões não statements,sem vetorcompleto/finançasUI. Relatório e logs preservados externamente, FIN-009-global-rust-20261017.json e FIN-009-global-rust-closeout-20261017.json; perfilWebView2 próprio removido sem processo ativo após196entradas semreparse. Runner originalexit1 porcleanup pendente, métricas/testesexit0; closeout separado confirmoucleanup,sem reclassificar exit. Cache instrumentado ignorado preservado. Isto não é coverage executada na CI nem instalador.


FIN-009 executor done/E3/passestrue após CI39009285553/job117328105513/attempt1/SHA83e682303354c000d2319f512f8321882901aa35 SUCCESS23/0,97ordinários+8doctestsPASS/1ignored. Evidência local global459/470linhas46/49funções ecríticos95PASS não é coverage CI; statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4 atualizados; CI documental final/nota90/preservação89/liberação ainda pendentes. Sem UI/IPC/receita/setup; fonte planning/handoffs/FIN-009.md.


## FIN-013 — validação pura de entrada, E3 local em 17/10/2026

Classificação crítica financeira: src-tauri/src/domain/mod.rs, gate de linhas/funções >=95%. LLVM23.1.1 focal: full24/24linhas,5/5funções; controle sem domain::entry_tests conserva19/24linhas(79,1667%) e4/5funções(80%), rejeitando95. Statements não medidos, branches stableN/A, regiões não são statements. Não é coverage global; a medição global FIN-009 permanece histórica no source83e6823.


FIN-013 executor done/E3/passestrue após CI39021912847/job117365252558/attempt1/SHAa3dee6febce169d4dbac3779ce6a67c998e506ab SUCCESS23/0,102ordinários+8doctestsPASS/1ignored. Crítico domainmod24/24linhas5/5funções/controle rejeita95 é E3 local,não coverageCI/globalatual;statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4/validators atualizados;CIfinaldocumental/nota91/preservação90/liberação pendentes. SemUIIPC/persistência/receita/setup. Fonte planning/handoffs/FIN-013.md.


## FIN-014 — validação pura de saída, E3 local em 17/10/2026

Domínio crítico src-tauri/src/domain/mod.rs: LLVM23.1.1 full29/29linhas e6/6funções; controle sem exit_tests24/29linhas(82,7586%) e5/6funções(83,3333%), rejeitando95. Thresholds intactos. Statements não medidos, branches stableN/A, regiões não são statements. Esta métrica é focal; não comprova global atual/CI, e a medição global FIN-009 pertence ao source83e6823.


FIN-014 | CI source aprovado / fechamento funcional E3 | SHA c27b4eaf6c7c8462b0301b3489218ee927a83a89; run39026610317/job117378726603/attempt1 success,23 passos aprovados/0 ignorados,7 marcadores sourceSha exatos. Rust remoto107 ordinários(43lib+7bin+57SQLite)+8doctests PASS/1WebViewignored; artifact frontend único scaffold. PE AMD64/custom-protocol8.536.064 bytes SHA256 5c46f6e9944536be390c2937aebc1d4bb80aa1ece7b215124573fbef227eae51; não é instalador. Local fmt/clippy/debug/release/quality/build/scans/E4 aprovados; crítico domainmod29/29linhas6/6funções e controle24/29linhas5/6funções rejeita95. Revisão independente Sol6medium semP0/P1/P2. Quatro validadores415arquivos/28fontes/92itens PASS; stagedGitleaks14arquivos/703969bytes/0achados/framedSHA2562a94609b975ca6224b734c68833cd0a216a0ab410796d9f1ce31c6e6b2933602. E4 exata até23/10/2026, sem exceção nova. Evidência focal preservada em operacao/FIN-014-focal-evidence-20261017,9arquivos com hashes; WT arquivada recuperavelmente, scratch próprio removido. Sem código ou teste corrigido por GPT; Laguna local P-QA/P-DOM, integrador reproduziu gates. Custo não registrado. Esta entrega é validação pura de saída; não comprova jornada UI/IPC/persistência/receita, global Rust atual ou setup.exe. CI final/nota92 preservando91/97branches/checkpoint/Gitlimpo/liberação ainda pendentes; reserva ATIVA e próximo ID bloqueado até esse fechamento.


FIN-015 | VERIFY local | E3 | Migration005 main.movements STRICT e registro v5 adicionam sete campos que preservam Movement/MovementKind raw i64, context separado da conta, shape por kind e três FKs NO ACTION. Executor e migrations001–004 inalterados. Três testes físicos PASS: cadeia vazia1..5 e30roundtrips(3kinds×2contextos×5i64), schema/FKs/índicePK, reopen/integridade; upgradev4 com contas/categoria/audit e snapshots exatos; colisão com sentinel, rollback/ledger4 preservados, remoção apenas da fixture e retryv5. Probes shape/FK/delete preservam os dados. Nenhum banco da usuária utilizado.
Debug e release/custom-protocol:110 testes ordinários(43lib+7bin+60SQLite)+8doctests PASS;1WebViewignored. Fmt/clippy-Dwarnings, TSquality(format/lint/typecheck/coverage)/build PASS;frontend3testes2arquivos scaffold100%,sem finanças. npm120/0,Cargo422/0vuln/2E4,OSV542/3E4/0fora,plugin/SAST45PASS. E4 exatas nos hashes aprovados,expiram23/10/2026;nenhuma exceção nova ou threshold reduzido.
LLVM23.1.1 focal database.rs276/280linhas(98,5714%)/25/25funções(100%) passa95. Controle omite TODO harness SQL e mede0/280linhas0/25funções,rejeitando95; calibra o gate,não atribui cobertura aos três testes novos. Statements não medidos;branches stableN/A;regiões não são statements. Não comprova Rustglobal atual.
Review independente P-REV Sol6medium read-only4paths/spec semP0/P1/P2,não executou checks. LocaisQwen/Laguna/Muse propuseram/testaram;root rejeitou drafts e corrigiu resíduos focais após falhas. RodadaLaguna longa foi interrompida com evidência preservada: processo computava/contador avançava,mas contexto31,5K/32,8K em autotest-retry;erroi32/ToSql. Sem mudança no executor;servidor próprio45340 encerrado pelo controlador/porta11434fechada. Receiptsoriginais e cinco arquivos do job interrompido preservados em operacao/FIN-015-interrupted-correction-evidence-20261017.
Fontes/recibos FIN-015-main-integration/rust-gates/other-gates/verification-facts/GREEN-root/root-residue-20261017.json e artifacts/fin015-coverage/receipt-coverage.json. Custo não registrado. Validators/stagedscan/CI source/final/nota93preservando92/liberação pendentes,passesfalse. Não comprova adapters/consumidores/UIIPC/receita/transferência válida/setup.exe/E2E.


FIN-015 | CI source aprovado / executor concluído | E3 | SHA23459892a5682aef1d8c512f53badf5a3466f5b2,run39061456176/job117480113685/attempt1 success23/0skip;7marcadores sourceSha exatos. Rust remoto110ordinários(43lib+7bin+60SQLite)+8doctestsPASS/1WebViewignored, incluindo3testesfísicosFIN015. Únicoartifactfrontend scaffold. PE AMD64/customprotocol8.536.064B SHA256743847089a72cb894cd7042a262c2f69148ccbc49e65be457695c464874e371a,não é instalador. Quatrovalidadores420arquivos/28fontes/93itens e stagedGitleaks15arquivos/763917B/0achados/framedSHA256abb4bac21f06430e4cc4a54dddeaa45bb75f083c44e0502e2b6dd47af9287472PASS. Gateslocais/coveragecrítica/review descritos no VERIFY e recibosoperacionais;E4exatasaté23out. Evidênciafocal10arquivos preservados comhashes;originaisdojobinterrompido preservadosseparadamente;WTarquivada/scratchpróprioremovido. Executor done/E3/passestrue apósdocs/matrizes/log/handoff eCI source;CI documental final/nota93preservando92/97branches/checkpoint/mainlimpa/liberação ainda pendentes,reservaATIVA. SemjornadaUIIPC/receita/transferênciavalidada/Rustglobalatual/setup.exe/E2E. Custo não registrado.


FIN-016 | VERIFY local | E3 | Dois testes físicos novos da migration005 imutável:39linhas inválidas(38matrix+id15) e7probesPK/FK/NOACTION,com errosConstraintViolation eextendedPRIMARYKEY/FOREIGNKEY nas probes pertinentes. Cadeia real1..5 por open_database/executor; snapshots7camposmovements/4accounts,ledger/autocommit/integridade/FKcheck e reopen preservados. Seeds0/-5 continuam aceitos conforme rawschema. Nenhuma produção/migration/manifest/lock alterada;identidade byteexata do primary registrada.
RootRED compila e executa2testes;controleCHECK temporário só no testenovo daWT fazid15seraceito e primeiroteste falha/segundopassa. Runner restaura todosbytes econfirmahashesprodução,sem alteração SQLaplicada. GREENnormal2PASS/0warnings. Debug e releasecustomprotocol112ordinários(43lib+7bin+62SQLite)+8doctestsPASS/1WebViewignored. Fmt/clippy-Dwarnings/TSquality(format/lint/typecheck/coverage)/buildPASS;frontend3testes2arquivos scaffold100%,semfinanças.
npm120/0,Cargo422/0vuln/2E4,OSV542/3E4/0fora,plugin/SAST45PASS. E4 nos inputs aprovados até23out,semnovaexceção/thresholdreduzido. Cobertura críticaE3FIN015database276/280linhas25/25funções econtrole0/280/0/25 reusados porhashprodução exato;controle omiteTODOharnessSQL,não atribui métrica aos testesnovos. Statementsnão medidos/branchesstableN/A;semglobalatual/CIcoverage.
P-REVSol6medium read-only2paths/spec semP0/P1/P2,nãoexecutouchecks. LocaisLaguna/Muse produziram candidatos;integrador rejeitou bypass,corrigiu contexto faltante003 e resíduos de API/oráculos apósfalhas. Último jobboundedwrite semautotestretry declarouchecksnot_run;rootexecutouRED/GREENseparadamente. Recibosoriginais/candidatosrejeitados/jobserver interrompido preservados;executorinstalado intacto. Servidorpróprio58064 encerradopelocontrolador,11434fechada.
Fontes/recibos FIN-016-RED-root/GREEN-root/root-residue/main-integration/rust-gates/other-gates/verification-facts-20261017.json emoperacao. Custo não registrado. Validators/stagedscan/CI source/final/nota94preservando93/liberação pendentes;in_progress/passesfalse. SemjornadaUIIPC/receita/setup.exe/E2E.


FIN-016 | CI source aprovado / executor concluído | E3 | SHAf31c8286091af4285bd189a5b5348661fe4181a1,run39079462454/job117533270499/attempt1success23/0skip,7sourceShamarcadores exatos. Rust remoto112ordinários(43lib+7bin+62SQLite)+8doctestsPASS/1WebViewignored,incluindo2testesFIN016. Únicoartifactfrontend scaffold. PE AMD64/customprotocol8.536.064B SHA2567147d0a64af88ec112a8292a6f3d3dd44589678da4f779939a7c10eafe191f4b,nãoinstalador. Quatrovalidadores424arquivos/28fontes/94itens e stagedGitleaks13arquivos/762500B/0achados/framedSHA2560b1846d7fbc92b5d1f6c1a192ba1b4464fcea4891043d6aee9abf81bb1627086PASS. Local112+8/gates/securityE4/review aprovados;produçãoexataFIN015permaneceinalterada,coveragecrítica reutilizada porhash semclaimglobalatual. Evidênciafocal4arquivoshashverified e originaisrejeitados/interrompidospreservados;WTarquivada. Docs/matrizes/log/handoff atualizados antesdone/E3/passestrue. CIfinal/nota94preservando93/97branches/checkpoint/mainlimpa/liberaçãopendentes,reservaATIVA. SemjornadaUIIPC/receita/setup.exe/E2E. Custonão registrado.
## FIN-017 — cobertura crítica requerida (pendente)
A medição local futura deve incluir `application/create_entry.rs` e `infrastructure/movements.rs`, com gate inalterado de pelo menos 95% de linhas e funções e controle negativo que rejeite abaixo do limiar. Medição, controle, comandos, versão, saída e SHA: não registrados. Não inferir cobertura global Rust, cobertura CI ou aprovação por esta especificação E2.


FIN-017 | E3 local:6unit+3SQLite/4falhas,criaçãoEntry+audit atômicas/reopen/rollback/poison/inputinválido sem efeitos;Rust121+8docs/debug+release/fmt/clippy/TS/securityE4 PASS;coveragecrítica app43/43linhas3/3fn eadapter51/51linhas5/5fn,controle0,rejeita95. Review independente concluído;CI source/final/liberação pendentes,in_progress/passesfalse. Sem jornadaUIIPC/globalRust/setup/E2E;apósFIN017METApausará,nenhum próximoID. Ver spec/handoffFIN017;custonão registrado.


FIN-017 | done/E3: source5a09ca2ec7117be7e39605bfe25848e7fc39acaf CI39090756071/job117566803340/attempt1success23/0skip/7marcadores exatos;121testesRust+8doctests. Entry+audit atômicos/6unit+3SQLite4falhas,critical app43/43linhas3/3fn+adapter51/51linhas5/5fn100%,controle0. Gateslocais/review/validators/stagedscan aprovados;docs antespassestrue. CI documental/nota95/liberação pendentes;WTarquivada,evidênciapreservada,locaisencerrados. DepoispausarMETA por decisão humana,nenhumpróximoID. ProdutoUIIPC/receitaFIN005/globalRust/setup.exe/E2E pendentes;custonão registrado.


CAT-004 LLVM23.1.1 E3: create_category.rs47/47linhas4/4funções e categories.rs50/50linhas5/5funções100%; controle omite unit/SQLite novos e ambos0linhas/funções,rejeita95. Statements não medidos, branchesstableN/A; não éRustglobalatual/coverageCI. Hashes e relatório em artifacts/cat004-coverage/receipt-coverage.json e recibos externos;CI fonte/final pendentes.


CAT-004 executor done/E3/passestrue após CI39094721622 no SHAcbf40d8392e66a38146ebfd6abb61cd42766d47e success23/0 e130Rust+8docs; detalhes e limites no handoff CAT-004. CI documental final/nota96/liberação pendentes. MVP02 criação backend parcial; semUIIPC/E2E/setup. MetaE2Eativa.


CAT-005 LLVM23.1.1 E3: rename_category.rs49/49 linhas e4/4funções100%; categories.rs inteiro106/107 linhas99,0654% e8/8funções100%. Controle sem testes rename: aplicação0/49 e0/4; adapter50/107 linhas46,729% e5/8funções62,5%, preservando criação; rejeita gate95. Statements não medidos/branchesstableN/A; não é cobertura global Rust atual nem CI. Recibo artifacts/cat005-coverage/receipt-coverage.json preservado comhash em operacao; handoff CAT-005.


CAT-005 executor done/E3/passestrue após CI39098498169 no SHAaef71f2308dbfa1d11f4960568fc0b041d1e7221 success23/0 e141Rust+8doctests, incluindo8unit+3SQLite. Gates locais, cobertura crítica >=95 e controle, revisão independente e validators/stagedscan aprovados; detalhes no handoff CAT-005. CI documental final/nota97/liberação pendentes. MVP02 renomeação backend parcial; UIIPC/desativação/E2E/setup pendentes. Meta E2E ativa.


CAT-006 LLVM23.1.1 E3:deactivate_category.rs38/38linhas4/4funções100%;categories.rs153/155linhas98,7097% e11/11funções100%. Controle sem novos testes:app0;adapter106/155linhas68,3871% e8/11funções72,7273%,rejeita95. Statementsnão medidos/branchesstableN/A; nãoRustglobalatual/CIcoverage. Recibos/hash preservados; handoffCAT006.


CAT-006 done/E3/passestrue após CI39128570976 no SHA25311c0fb01d066e39d557e3779287d680553c7f,23success/0skip e151Rust+8docs;7unit+3SQLite e gateslocais/review/validators/stagedscan PASS. Fonte: handoffCAT006. CI documental final/nota98/liberação pendentes. MetaE2Eativa; UIIPC/histórico consumidor/setup/E2E pendentes.


CAT-007 LLVM23.1.1 atual:database276/280linhas98,5714% e25/25funções100%; appdeactivation38/38 e4/4; adaptercategories153/155 e11/11. Full>=95PASS; controle corrigido só após novo teste histórico cobrir desativação: omite ambos grupos,app0/adapter106/155e8/11,rejeita95. Repetido sócontrole,full/compilação preservados. Duas constantes novas não adicionam função executável; databasecontroleobservacional. Statementsnão medidos/branchesstableN/A;nãoRustglobal/CIcoverage. HandoffCAT007.


CAT-007 done/E3/passestrue após CI39131643684/SHA143190b374d186da75f2a44bdc85cb0bfad58219success23/0skip e153Rust+8docs,2SQLite novos. Migration6/FKhistórica física/reopen/rename/deactivate e gates/review/validators/stagedscan PASS. CI documental final/nota99/liberação pendentes;MetaE2Eativa. FixtureSQL não comprova atribuiçãoUIIPC/globalRust/setup/E2E. Fonte:handoffCAT007.


FIN-018 LLVM23.1.1 atual appcreate_exit43/43linhas3/3funções eadaptermovements97/97linhas9/9funções100%. Controle omite unit/SQLExit:app0/adapter51/97linhas52,5773% e5/9funções55,5556%,rejeita95. Statementsnão medidos/branchesstableN/A;nãoRustglobalatual/CIcoverage. Recibos/hashes preservados;handoffFIN018.


FIN-018 done/E3/passestrue após CI39134383058/SHA4f7e7703793ac8ef5632935ade79badd3f9d44fasuccess23/0skip e162Rust+8docs,6unit+3SQLite. Saída+audit atômicos/rollback4faults/poison e gates/review/validators/stagedscan PASS. CI documental final/nota100/liberação pendentes;MetaE2Eativa. Sem saldo/receitaFIN005/UIIPC/globalRust/setup/E2E. Fonte:handoffFIN018.
