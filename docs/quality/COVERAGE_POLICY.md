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
