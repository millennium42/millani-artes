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

FIN-011 focal LLVM WT E3: full17 PASS, 14/14 linhas, 3/3 funções, 19/19 regions (`61e4b4c939f466ce723ade62a37059fbc95cdb16f883bc32c3f5be68230be554`). Controle omitindo apenas três testes `context_*`: 14 PASS/3 filtrados, 8/14 linhas (57,142857%), 2/3 funções, 11/19 regions; gate90 rejeitado (`3b0e1e1e414ab2b3778757d89cbaf1bb864c4978cfdbd346e3b0af7f0f06bfb6`). Branches 0/N/A; regions não são statements. Resultados somente WT; primary retest e gates de cobertura aplicáveis pendentes, sem claim global/full vector.
