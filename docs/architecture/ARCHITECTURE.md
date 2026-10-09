# Arquitetura

Aplicativo Windows Tauri 2 com React, TypeScript strict e Vite. SQLite é o único banco local compartilhado. A UI chama casos de uso; regras puras vivem no domínio; persistência fica em repositórios; o core Tauri expõe somente comandos/capabilities necessárias.

O artefato de distribuição do MVP será um instalador Windows `setup.exe` (NSIS), gerado no Windows. A aplicação instalada funciona sem rede; a estratégia para instalar WebView2 será testada no work item de release. O desenho é candidato aprovado, não implementação existente.

ARC-001 organiza o scaffold em [quatro camadas](MODULE_BOUNDARIES.md#estrutura-atual--arc-001): composição em main, inicialização em application, adaptadores Tauri/log em infrastructure e tela em src/ui. Domain compila sem dependências externas e ainda não tem regras financeiras. SQLite, repositórios, transações, backups e instalador permanecem pendentes; [handoff](../../planning/handoffs/ARC-001.md) registra execução e limites.

ARC-001 executor done/E3/passes:true: [entrega](https://github.com/millennium42/millani-artes/commit/e94eec776213725f72239c8d07aff2a293f6f632)/[CI37863336200](https://github.com/millennium42/millani-artes/actions/runs/37863336200) success23steps/buildWindows, std-only6controles e7Rust locais/reviewE2fontes semachados. Domínio sem regra; finanças/SQLite/backup/setup.exe pendentes/METAativa. Reserva22ATIVA atéCI final/nota57, fechamento11docs; detalhe na spec/handoffARC001. ARC002 não iniciado.

ARC002 usa [biblioteca Rust pura](../../src-tauri/src/lib.rs) para exportar o [contrato Result/DomainError](../../src-tauri/src/domain/mod.rs), preservando main/application/infra como bootstrap. Aliasstd e um erro técnico de domínio sempayload/Display estático; não define regra financeira/transação. [Handoff](../../planning/handoffs/ARC-002.md) delimita execução e pendências.
