# Roadmap end-to-end, granular e com dependências

Fonte canônica de IDs. Formato: `ID — intenção observável ← dependências`; `—` indica que pode começar sem predecessor do bloco. Toda dependência exige `passes=true`, handoff e CI no SHA. RECON read-only pode ler antes, mas não editar. Um líder reserva paths; escrita em paralelo só em worktrees isolados, paths disjuntos e integração serial (`MULTI_AGENT_POLICY.md`).

`DOC-001` tem evidência local histórica. `DOC-002` fechado em `263e8db`; `DOC-003` em `00ba4f2`; `DOC-004` em `0bca4a5`; `DOC-005` em `d60e8f7`; `GOV-001` em `db11682`; `GOV-002` em `b0cc139`; `GOV-003` em `cc43154`; `GOV-004` em `8c7e7bc`; `GOV-005` em `999b420` com CI 37092061099 verde, evidência final em `refs/notes/evidence`. `GOV-006` fechado em `13333ad` com CI37093457937 verde, décimo recibo/revisão de alocação em `refs/notes/evidence`. `GOV-007` fechado em `75b7fcb` com CI37094308358 verde e nota `refs/notes/evidence`. `GOV-008` fechado em `e49f349` com CI37094921664 verde e nota `refs/notes/evidence`. `GOV-009` fechado em `69c2236` com CI37095660870 verde e nota `refs/notes/evidence`. `PLN-001` fechado em `e0bd668` com CI37096490952 verde e nota `refs/notes/evidence`; template preservado. `PLN-002` fechado em `b65c361` com CI37097624217 verde e nota `refs/notes/evidence`. `PLN-003` fechado em `4bc2d66` com CI37098337691 verde e nota `refs/notes/evidence`. `PLN-004` fechado em `0560d72` com CI37099570659 verde e nota `refs/notes/evidence`. `PLN-005` fechado em `0c8fc12` com CI37100652290 verde e nota `refs/notes/evidence` publicada/liberação. `TRC-001` fechado em `692bd36` com CI37133989599 verde/nota publicada/liberação. `TRC-002` fechado em `1bcb7a8` com CI37135745415 verde/nota publicada-liberação. `TRC-003` fechado em `e81897c` com CI37137575872 verde/nota publicada-liberação. `TRC-004` fechado em `d815183` com CI37156295362 verde/nota publicada-liberação. `TRC-005` fechado em `575af01` com CI37157531585 verde/nota publicada-liberação. `TRC-006` fechado em `c07e71a` com CI37159051022 verde/nota publicada-liberação. `PLN-006` fechado em `032628d` com CI37175540259 verde/nota publicada-liberação. INF-001 final6bed951/CI37177579406/nota publicada-reserva liberada; runtimes locais E3. INF-002 done/passestrue após gates locais/visualE4 e CI entrega37241434563 no SHAfb8faee; reserva até CI final-notapub/liberação. INF-002 CI final37241566670 no SHA04d5b17 success/nota publicada/reserva liberada. INF-003 done/passestrue após reprodução local/review e CI entrega37242789186 no SHA61c61bf; reserva até CI final-nota publicada/liberação. INF-003 CI final37243031187 no SHA6e6e911 success/nota publicada/reserva liberada. INF-004 done/passestrue após compiler/build/probes/review e CI entrega37243910100 no SHAc23e583; reserva até CI final-nota publicada/liberação. INF-004 CI final37244081876 no SHA4416b35 success/nota publicada/reserva liberada. INF-005 done/passestrue após install-ci-pin/format-lint-probes/review e CI entrega37245206424 no SHAc63fcad; reserva até CI final-notapub/liberação. INF-005 CI final37245343129 no SHAd6df89d success/nota publicada/reserva liberada. INF-006 done/passestrue após suíte/probes/coverage/gates/review e CI entrega37247106290 noSHA6889b80; reserva atéCI final-nota publicada/liberação. INF-006 CI final37247330680 noSHAbc4a7e0 success/nota publicada/reserva liberada. INF-007 done/passestrue apóscoverage-reports/probes/gates/review-correção eCIentrega37248725654 noSHA4e5efec; reserva atéCIfinal-nota/ref-liberação. INF-007 CI final37248915292 noSHA5ddbfc5 success/nota publicada/reserva liberada. INF-008 CI final 37250250061 no SHA b9db95e success/nota pública/ref verificada/reserva liberada. INF-009 CI final37251558957 noSHA639db7e success/nota pública/ref verificada/reserva liberada. INF-010 CI final37301128915 noSHA87f9962 success/frontend skipped/nota pública-ref verificada/reserva liberada. INF-011 CI final37305834951 noSHAab3f751 success/nota pública-ref verificada/reserva liberada. INF-012 done/passestrue após catálogo/queries/gates locais/review e CI entrega37309073885-SHAe0f7145; CI final/note-ref/liberação pendentes. INF-013 done/passestrue após gates/review/CI entrega37311827221-SHA7e61fa7/artifact11346536176 ZIP-digest/coverage verificados; CI final/note-ref/liberação pendentes. INF-014 done/passestrue após gates/review e CI entrega37315900046-SHAfae4469/JSON nativo real conferido; CI final/nota-ref/liberação pendentes. INF-015 done/passestrue após clone limpo/guard/checks/cleanup/review e CI entrega37320497876-SHAb390985 success8skips0artifact; CI final/nota-ref/liberação pendentes. Demais todo. Próximo SEC-001 após liberação INF015.

SEC-001 in_progress/passesfalse: seleção explícita/RED-resolver/probeWebView2 real local/3Rustpassed/controle permitido-restauração/gates E3; review/docs/CI pendentes. Cinco linhas ARC/UI restauradas ao grafo canônico ab4e0c09 após regressão textual INF015;245IDs únicos/deps conferidos. Próximo após fechamento SEC-002.

## B0 — Governança e planejamento

- DOC-002 — validar mapa contra árvore real ← —
- DOC-003 — revisar README de entrada ← DOC-002
- DOC-004 — conferir hierarquia documental ← DOC-002
- DOC-005 — contar requisitos essenciais ← DOC-004
- GOV-001 — revisar AGENTS por sessão ← DOC-003
- GOV-002 — validar estados de evidência ← GOV-001
- GOV-003 — revisar tool register antes de instalar ← GOV-001
- GOV-004 — aprovar exceção temporária ← GOV-002
- GOV-005 — aplicar política multiagente ← GOV-001
- GOV-006 — aplicar política de tokens ← GOV-005
- GOV-007 — registrar baseline de uso disponível ← GOV-006
- GOV-008 — institucionalizar personas operacionais ← GOV-005, GOV-006
- GOV-009 — instituir departamentos e roteador dinâmico ← GOV-008
- PLN-001 — validar template de spec ← GOV-001
- PLN-002 — validar schema de work item ← PLN-001
- PLN-003 — validar template de handoff ← PLN-001
- PLN-004 — validar progress log append-only ← PLN-003
- PLN-005 — reservar paths por work item ← GOV-005
- TRC-001 — rastrear requisito contas ← DOC-005
- TRC-002 — rastrear requisito financeiro ← DOC-005
- TRC-003 — rastrear requisito vendas ← DOC-005
- TRC-004 — rastrear requisito backup ← DOC-005
- TRC-005 — rastrear invariantes ← DOC-005
- TRC-006 — rastrear segurança ← DOC-005
- PLN-006 — canonicalizar dependências do roadmap ← PLN-002, TRC-005

## B1 — Ambiente, qualidade e segurança

- INF-001 — fixar Node/Rust ← DOC-003
- INF-002 — criar Tauri mínimo ← INF-001
- INF-003 — commitar lockfiles ← INF-002
- INF-004 — ativar TypeScript strict ← INF-002
- INF-005 — configurar Biome ← INF-004
- INF-006 — configurar Vitest ← INF-004
- INF-007 — configurar coverage ← INF-006
- INF-008 — criar typecheck ← INF-004
- INF-009 — criar quality script ← INF-005, INF-006, INF-008
- INF-010 — configurar CI Windows ← INF-009
- INF-011 — provar build limpo ← INF-002, INF-003
- INF-012 — registrar versões executadas ← INF-011
- INF-013 — publicar coverage no CI ← INF-007, INF-010
- INF-014 — build Tauri no CI ← INF-010, INF-011
- INF-015 — provar checkout limpo ← INF-012
- SEC-001 — capabilities deny-by-default ← INF-002
- SEC-002 — CSP sem remoto ← INF-002
- SEC-003 — política de logs sanitizados ← GOV-002
- SEC-004 — listar plugins necessários ← SEC-001
- SEC-005 — bloquear plugin sem ADR ← SEC-004
- SEC-006 — instalar Gitleaks ← INF-003
- SEC-007 — Gitleaks no CI ← SEC-006, INF-010
- SEC-008 — instalar OSV-Scanner ← INF-003
- SEC-009 — OSV no CI ← SEC-008, INF-010
- SEC-010 — npm audit high/critical ← INF-003
- SEC-011 — cargo audit ← INF-002
- SEC-012 — audits no CI ← SEC-010, SEC-011, INF-010
- SEC-013 — avaliar SAST útil ← INF-002
- SEC-014 — testar redaction de telefone ← SEC-003
- SEC-015 — testar redaction de dados financeiros ← SEC-003

## B2 — Arquitetura e migrations

- ARC-001 — criar camadas domínio/aplicação/infra/UI ← INF-002
- ARC-002 — definir Result/erro de domínio ← ARC-001
- ARC-003 — injetar relógio/UUID ← ARC-001
- ARC-004 — definir fronteira de transação ← ARC-002
- ARC-005 — bloquear SQL em componente ← ARC-001
- DB-001 — criar harness SQLite temporário ← INF-006
- DB-002 — abrir banco vazio ← DB-001
- DB-003 — definir executor de migration ← DB-002
- DB-004 — registrar schema version ← DB-003
- DB-005 — testar migration vazia ← DB-004
- DB-006 — criar snapshot de upgrade ← DB-004
- DB-007 — testar upgrade ← DB-006
- DB-008 — testar rollback de migration ← DB-003
- DB-009 — ativar foreign keys ← DB-002
- DB-010 — provar foreign keys ← DB-009
- DB-011 — definir transação SQLite ← ARC-004, DB-002
- DB-012 — provar rollback genérico ← DB-011
- DB-013 — criar audit event técnico ← DB-011
- DB-014 — auditar na mesma transação ← DB-013

## B3 — Contas, categorias e movimentos

FIN-001 executor done/E3 após CI de entrega `38444911363` no SHA `31af12adcf8bebe617539d9e5dde286a64674deb`; review E2 sem achados. Fechamento documental/CI final e liberação ACTIVE21 pendentes. O contrato cobre somente ID opaco de 16 bytes e nome próprio; regras e invariantes financeiras permanecem E0. Próximo FIN-002 somente após fechamento; ainda não iniciado.

- FIN-001 — definir Account ← ARC-002
- FIN-002 — validar nome de conta ← FIN-001
- FIN-003 — definir tipo de conta ← FIN-001
- FIN-004 — definir saldo inicial em centavos ← FIN-001
- FIN-005 — provar saldo inicial fora de receita ← FIN-004
- FIN-006 — migration de contas ← DB-005, FIN-001
- FIN-007 — testar constraints de conta ← FIN-006
- FIN-008 — criar caso de uso de conta ← FIN-002, FIN-003, FIN-004, FIN-006
- FIN-009 — listar contas ← FIN-008
- CAT-001 — definir categoria ativa ← ARC-002
- CAT-002 — validar nome de categoria ← CAT-001
- CAT-003 — migration de categoria ← DB-005, CAT-001
- CAT-004 — criar categoria ← CAT-002, CAT-003
- CAT-005 — renomear categoria ← CAT-004
- CAT-006 — desativar categoria ← CAT-004
- CAT-007 — preservar categoria histórica ← CAT-006
- FIN-010 — definir contexto Casa/Fábrica ← ARC-002
- FIN-011 — rejeitar contexto inválido ← FIN-010
- FIN-012 — definir movimento ← FIN-001, CAT-001, FIN-010
- FIN-013 — validar entrada positiva ← FIN-012
- FIN-014 — validar saída positiva ← FIN-012
- FIN-015 — migration de movimentos ← DB-005, FIN-012
- FIN-016 — testar constraints de movimento ← FIN-015
- FIN-017 — persistir entrada ← FIN-013, FIN-015, DB-011
- FIN-018 — persistir saída ← FIN-014, FIN-015, DB-011
- FIN-019 — definir transferência ← FIN-012
- FIN-020 — rejeitar origem igual ao destino ← FIN-019
- FIN-021 — persistir transferência atômica ← FIN-019, FIN-015, DB-011
- FIN-022 — provar transferência neutra ← FIN-021
- FIN-023 — provar rollback de transferência ← FIN-021, DB-012
- FIN-024 — calcular saldo por conta ← FIN-004, FIN-017, FIN-018, FIN-021
- FIN-025 — provar saldo derivado ← FIN-024
- FIN-026 — listar extrato ← FIN-024
- FIN-027 — calcular resultado por contexto ← FIN-017, FIN-018, FIN-022
- FIN-028 — editar lançamento pelo caso canônico ← FIN-017, FIN-018, DB-013
- FIN-029 — auditar edição ← FIN-028, DB-014
- FIN-030 — provar sem saldo direto ← FIN-028, FIN-025

## B4 — Recorrências

- REC-001 — definir recorrência fixa ← ARC-002
- REC-002 — definir recorrência variável ← ARC-002
- REC-003 — estado aguardando valor ← REC-002
- REC-004 — estado a pagar ← REC-001, REC-002
- REC-005 — estado pago ← REC-004
- REC-006 — estado vencido ← REC-004, ARC-003
- REC-007 — migration de recorrência ← DB-005, REC-001
- REC-008 — gerar obrigação fixa ← REC-007, REC-004
- REC-009 — gerar pendência variável ← REC-007, REC-003
- REC-010 — informar valor pendente ← REC-009
- REC-011 — vencer obrigação ← REC-008, REC-006
- REC-012 — pagar recorrência ← REC-010, FIN-018, DB-011
- REC-013 — provar saída única ← REC-012
- REC-014 — provar rollback ← REC-012, DB-012
- REC-015 — impedir pagamento repetido ← REC-012

## B5 — Produtos, clientes e vendas

- PRD-001 — definir produto template ← ARC-002
- PRD-002 — validar nome de produto ← PRD-001
- PRD-003 — validar preço sugerido ← PRD-001
- PRD-004 — migration de produto ← DB-005, PRD-001
- PRD-005 — criar produto ← PRD-002, PRD-003, PRD-004
- PRD-006 — alterar produto ← PRD-005
- PRD-007 — desativar produto ← PRD-005
- CUS-001 — definir cliente mínimo ← ARC-002
- CUS-002 — validar nome ← CUS-001
- CUS-003 — validar telefone opcional ← CUS-001
- CUS-004 — migration de cliente ← DB-005, CUS-001
- CUS-005 — criar cliente ← CUS-002, CUS-003, CUS-004
- SAL-001 — definir item congelado ← ARC-002
- SAL-002 — validar quantidade positiva ← SAL-001
- SAL-003 — validar desconto ← SAL-001
- SAL-004 — calcular total do item ← SAL-001
- SAL-005 — aceitar item livre ← SAL-001
- SAL-006 — provar item livre não cria produto ← SAL-005
- SAL-007 — criar item de produto ← PRD-005, SAL-001
- SAL-008 — provar snapshot imutável ← PRD-006, SAL-007
- SAL-009 — definir venda ← SAL-001
- SAL-010 — migration de venda ← DB-005, SAL-009
- SAL-011 — migration de item ← SAL-010, SAL-001
- SAL-012 — adicionar item de produto ← SAL-007, SAL-010, SAL-011
- SAL-013 — adicionar item livre ← SAL-005, SAL-010, SAL-011
- SAL-014 — remover item aberto ← SAL-012
- SAL-015 — calcular total da venda ← SAL-004, SAL-009
- SAL-016 — definir pagamento parcial ← SAL-009
- SAL-017 — definir pendência ← SAL-016
- SAL-018 — exigir cliente pendente ← SAL-017, CUS-005
- SAL-019 — exigir vencimento pendente ← SAL-017
- SAL-020 — finalizar venda transacional ← SAL-012, SAL-013, SAL-015, SAL-017, DB-011
- SAL-021 — provar pendência sem saldo ← SAL-020, FIN-024
- SAL-022 — provar rollback de venda ← SAL-020, DB-012

## B6 — Recebimentos, devoluções e dashboard

- RCV-001 — definir conta a receber ← SAL-020
- RCV-002 — migration de recebível ← DB-005, RCV-001
- RCV-003 — criar recebível pendente ← RCV-002, SAL-017
- RCV-004 — calcular saldo devido ← RCV-003
- RCV-005 — definir recebimento parcial ← RCV-001
- RCV-006 — registrar recebimento ← RCV-005, FIN-017, DB-011
- RCV-007 — criar entrada pelo recebimento ← RCV-006
- RCV-008 — provar uma conta aumentada ← RCV-007, FIN-024
- RCV-009 — rejeitar excesso recebido ← RCV-004, RCV-006
- RCV-010 — fechar recebível integral ← RCV-004, RCV-006
- RCV-011 — provar rollback ← RCV-006, DB-012
- RET-001 — definir devolução por item ← SAL-020
- RET-002 — calcular quantidade disponível ← RET-001, SAL-012
- RET-003 — rejeitar excesso devolvido ← RET-002
- RET-004 — migration de devolução ← DB-005, RET-001
- RET-005 — registrar devolução parcial ← RET-004, RET-002
- RET-006 — reduzir dívida sem reembolso ← RET-005, RCV-004
- RET-007 — definir reembolso ← RET-005
- RET-008 — selecionar conta de saída ← RET-007, FIN-018
- RET-009 — permitir conta diferente ← RET-008
- RET-010 — criar saída de reembolso ← RET-008, DB-011
- RET-011 — provar uma conta reduzida ← RET-010, FIN-024
- RET-012 — cancelar venda paga ← RET-010, RCV-010
- RET-013 — provar rollback ← RET-010, DB-012
- DSH-001 — saldo total ← FIN-024
- DSH-002 — saldo por conta ← FIN-024
- DSH-003 — entrou hoje ← FIN-017
- DSH-004 — saiu hoje ← FIN-018
- DSH-005 — vendas do mês ← SAL-020
- DSH-006 — a receber ← RCV-004
- DSH-007 — a pagar ← REC-008
- DSH-008 — resultado fábrica ← FIN-027
- DSH-009 — gastos casa ← FIN-027
- DSH-010 — provar projeção sem escrita ← DSH-001, DSH-009

## B7 — UI, backup, smoke e instalador

- UI-001 — shell acessível ← INF-002, ARC-001
- UI-002 — foco e teclado do shell ← UI-001
- UI-003 — configuração inicial de contas ← UI-001, FIN-008
- UI-004 — cadastro de categoria ← UI-001, CAT-004
- UI-005 — formulário de entrada ← UI-001, FIN-017
- UI-006 — formulário de saída ← UI-001, FIN-018
- UI-007 — formulário de transferência ← UI-001, FIN-021
- UI-008 — cadastro de produto ← UI-001, PRD-005
- UI-009 — cadastro de cliente ← UI-001, CUS-005
- UI-010 — carrinho de venda ← UI-001, SAL-020
- UI-011 — recebimento ← UI-001, RCV-010
- UI-012 — recorrência ← UI-001, REC-015
- UI-013 — devolução/reembolso ← UI-001, RET-013
- UI-014 — dashboard ← UI-001, DSH-010
- UI-015 — estados vazio/erro/loading ← UI-005, UI-006, UI-007
- UI-016 — labels/contraste/overflow ← UI-002, UI-014
- BKP-001 — metadados/versionamento ← SEC-003
- BKP-002 — escolher crypto por POC ← BKP-001
- BKP-003 — gestão de chave ← BKP-002, SEC-003
- BKP-004 — diretório permitido ← SEC-001
- BKP-005 — backup manual consistente ← DB-011, BKP-001, BKP-002, BKP-004
- BKP-006 — validar backup novo ← BKP-005
- BKP-007 — backup automático ← BKP-005, ARC-003
- BKP-008 — rejeitar arquivo malformado ← BKP-001
- BKP-009 — rejeitar path traversal ← BKP-004, BKP-008
- BKP-010 — restore em staging ← BKP-008, DB-002
- BKP-011 — preservar banco ativo ← BKP-010
- BKP-012 — integrity check ← BKP-010
- BKP-013 — promover só validado ← BKP-011, BKP-012
- BKP-014 — provar restore seguro ← BKP-013
- UI-017 — tela backup/restore ← UI-001, BKP-014
- SMK-001 — iniciar build produção ← INF-011
- SMK-002 — abrir banco novo ← SMK-001, DB-005
- SMK-003 — conta/categoria ← SMK-002, UI-003, UI-004
- SMK-004 — entrada/saída/transferência ← SMK-003, UI-005, UI-006, UI-007
- SMK-005 — produto/cliente/venda ← SMK-004, UI-008, UI-009, UI-010
- SMK-006 — pendência/recebimento ← SMK-005, UI-011
- SMK-007 — recorrência ← SMK-006, UI-012
- SMK-008 — devolução/reembolso ← SMK-007, UI-013
- SMK-009 — confirmar saldos ← SMK-008, UI-014
- SMK-010 — backup/restore controlado ← SMK-009, UI-017
- REL-001 — gerar NSIS Windows ← INF-014, SMK-001
- REL-002 — testar instalação limpa ← REL-001
- REL-003 — testar menu Iniciar/desinstalação ← REL-002
- REL-004 — decidir WebView2 ← REL-001
- REL-005 — decidir assinatura pública ← REL-001
- REL-006 — inspeção visual produção ← SMK-010, UI-016
- REL-007 — revisar matriz funcional ← SMK-010, TRC-001, TRC-004
- REL-008 — revisar P0/P1/P2 ← REL-006, REL-007
- REL-009 — CI remoto no SHA ← REL-008
- REL-010 — handoff/release ← REL-003, REL-004, REL-005, REL-009

SEC-001 done/passestrue/E4 configuração aceita2026-10-05 após4Rusttests/WebView2-guardjunction/review-correção/gates/docs e CI entrega37347356818-SHA2e6edff success19steps/native201s/1coverageartifact. CI final/nota42-ref-liberação pendentes; grafo245IDs-deps canônico restaurado. PróximoSEC-002 apósliberação; nenhuma tarefa posterior iniciada/metaativa.

SEC-002 in_progress/passesfalse: CSP existente será provada no probe WebView2 real com5vetores cross-origin/controle CSPnull/CSS local;10paths ATIVA, sem novo job/dependência. Próximo após fechamento calculado pelo grafo.

SEC-002 entrega pronta/E3/passesfalse:5Rustpassed0ignored/CSP5enforce-HTTP0-TCPpreconnect1/ReactCSS local/controle null5HTTP/configintacta/gates10docs/reviewE2 semP0P1P2;commit/CI entrega-final/nota43-liberação pendentes/metaativa.

SEC-002 done/passestrue/E3 após5Rust/CSP-runtime/fontesparsed/gates10docs/review e CI entrega37711001108-SHAb307c2c success19steps/native170s/1coverageartifact. Fechamento CI/nota43-ref/liberação pendentes;45items42done/245tuplas intactas. PróximoSEC-003 após liberação; não iniciado/metaativa.

SEC-003 in_progress/passesfalse: política de logs/allowlist/fallback/sinks/audit_event distinta e enforcementsSEC014/015/BKP futuros;11paths ATIVA/checks-review-CI pendentes. Fonteproduto intacta, startupRust expect semgarantia de sanitização;metaativa/próxima nãoiniciada.

SEC-003 entrega pronta/E3documental: políticaE2/review5.6Solmedium semP0P1P2/5validators/218files46items42done/45anteriores245tuplas preservados. Sourceproduto intacto/enforcementE0,commit-CIentrega-final-nota44-liberação pendentes/passfalse/metaativa.

SEC-003 tarefa de política done/passestrue/E3documental após review/checks e CI entrega37712913326-SHA539e674 success10success8skips0artifact;CIfinal/nota44-ref/liberação pendentes.46items43done/245tuplas/sourceproduto intactos;enforcementSEC003global E0/SEC014015BKP futuros. PróximoSEC004 apósliberação/nãoiniciado/metaativa.

SEC-004: inventário documental em andamento na política Tauri; lista atual vazia e escolhas do MVP mapeadas aos executores, sem adoção/grant. Spec/item/handoff reservam8paths; checks documentais/review sem achados materiais aprovados; CI/nota45-liberação pendentes/passfalse. Nenhuma tarefa posterior iniciada; metaativa.

SEC004 done/passestrue/E3 documental, CIentrega37714626133-SHA7f0ff510b488e5537b48ca21a418c26b0468d9cd verde10success8skipped0artifact;47items44done221files245tuplas. Reserva8ATIVA atéCIfinal/nota45-ref-Gitclean/liberação;próximoSEC005←SEC004 não iniciado/metaativa.

Executor SEC-005 done/passes true/E3 estático: 37 fixtures e 2 casos CLI, check real, re-review sem achados e entrega corrigida 8f35b78e5ba6dc0b9545791bb6758fcc491356c1 / [CI 37717088022](https://github.com/millennium42/millani-artes/actions/runs/37717088022) success, 19 etapas/uma job. Alias e identidade real exigidos; nenhum plugin/grant instalado. Requisitos globais SEC-001/008 continuam parciais, runtime e E4 não ampliados. Reserva13 ATIVA até CI final/nota46 preservando45/refs/Git limpo. META ATIVA, próximo SEC-006 não iniciado. 48 itens/45 done/227 arquivos/245 tuplas.

Executor SEC006 done/passestrue/E3 instalação portátil Gitleaks8.30.1, hash/licença/PE/version/probes locais e scan do diff, review/checksroot/entregad54eee5a5d09bf67eb5af3ef8420b0fc3266d750/[CI37719495570](https://github.com/millennium42/millani-artes/actions/runs/37719495570) success10success8skip0artifact. Gate de scanner remotoSEC007 e segurança global continuam pendentes, sem E4 novo. Reserva10ATIVA atéCI final/nota47preserva46/publicrefs-Gitlimpo/liberação. PróximoSEC007 não iniciado/META ATIVA.49items46done231files245tuplas.

SEC007 emandamento/E3local/15checks149commits zeroachados; review/CIentrega-final/nota48-liberação pendentes/reserva12ATIVA.50items46done236files245tuplas previstas;49prévios preservados/METAativa/SEC008nãoiniciado.

SEC007 CORRECT: P2 de cache extra reproduzido (RED8.977s) e corrigido antes execução. Cache aceita exatamente os três arquivos obrigatórios e, opcionalmente, somente o ZIP oficial validado de SEC006.16casos passaram/9.135s;scan149commits zeroachados/1.494s. Re-review/checks/diffscan/CI exato pendentes;passesfalse/reserva12ATIVA/METAativa/SEC008nãoiniciado.

SEC007 re-review E2 semP0P1P2 após corrigirP2cache;root16checks/149commits0achados/4docsvalidators verdes;12paths49prévios245tuplas preservados. Entrega/CIexato ainda pendentes;passesfalse/reserva12ATIVA/METAativa/SEC008nãoiniciado.

SEC007 executor done/passestrue/E3:16checks/root,reviewP2corrigido/re-reviewsemachados,entrega2597cf4047b870ee30e9bca8f029f9884269c6cf/[CI37768499862](https://github.com/millennium42/millani-artes/actions/runs/37768499862) success20etapas/1coverageartifact. Gate7 obrigatório/103commits públicos disponíveis/0achados;local pré-entrega149inclui notas. Requisito globalSEC007 continua parcial, sem E4 novo. CI final/nota48preserva47/refsGitclean/liberação pendentes/reserva12ATIVA/METAativa/SEC008nãoiniciado.
50items47done236files245tuplas;49prévios inalterados.

SEC008 emandamento/E3instalaçãoCLI4probes/OSV2.6.0/hash-license-PE/sem scan;review/CI/nota49-liberação pendentes,reserva10ATIVA/METAativa/SEC009nãoiniciado.51items47done240files245tuplas previstas/50prévios intactos.

## SEC-008 — CI de entrega / fechamento documental
Entrega pública c6b4a27bae6ca31e17e80eacd984c5f07163d1d2 / [CI37774382975](https://github.com/millennium42/millani-artes/actions/runs/37774382975), job113301471481/attempt1/main/SHAexato/success19etapas(11success8skip)/zeroartifacts. Root conferiu API dos runs/jobs/steps/artifacts;etapa Scan Git history for secrets obrigatória success antesbuild. Seletor real false baseefb25a13→entrega;nenhum OSV remoto/download/scan integrado,novo job/action/cache/upload,rerun ou CI negativo. Descoberta ghapi indisponível;credentialmanagerGit existente reutilizado somente em memória para API GitHub,sem log/credencial publicado e sem redirects autenticados.
Executor SEC008 done/passestrue/E3 somente instalação local:ausênciaRED/3arquivos oficiais fixos/version2.6.0/4probesCLI/AuthenticodeNotSigned/6casos documentais apósP2/3hashes originais preservados/cleanup concluído/re-reviewsemP0P1P2/checks docs/diffsegredos/CI entrega verde. GatesOSV/dependências sãoSEC009 e posteriores,semclaimzero vulnerabilidades/E4novo. Root4validadores finais/10paths/50JSONanteriores intactos245tuplas/51items48done240files271links;log208642base preservado,entrega213920. Scansegredos diff finalentrega exit0/0achados/0.447s/SHA2564adb72f5e9421b59f32a88a4eac22a2a2e638a7709e2f9eff4f2f3384b95fb05. Reserva10ATIVA até CI do fechamento noSHAexato/nota49preserva48/publicrefs-Gitlimpo/liberação. Recibo final em notas Git evita commit recursivo. Semproduto/manifests/pins/locks/grants/thresholds/workflow/seletor alterado. METAativa:finanças/SQLite/backup/setup.exe/release pendentes. PróximoSEC009←SEC008,INF010;não iniciado neste ciclo.

## Checkpoint SEC-009
Gate OSV em preparação/29checks locais/inventário537/3registros2avisosCargo;proposta deexceçãoWindows desativada atéaceite humano/review/CI. PrimeiratarefaSEC009←SEC008,INF010;13pathsATIVA,52items48done246files esperados/245ID-deps preservados;SEC010nãoiniciado/METAativa.

SEC-009: 34 casos locais passaram; P1 do vínculo entre aceite e conteúdo corrigido, re-review pendente. Ensaio online real somente com aceite sintético: 537 pacotes, 3 registros da proposta, 0 outros. Proposta Windows inativa, sem E4/CI novo; passes:false, reserva 13 ATIVA. Digest cb8a47f5d84b722cdd1f6274c7807112132bc6e008acdd073972ddeb1b8b6921. META ativa; SEC-010 não iniciado.


SEC-009 (estado vigente): 38 casos locais passaram; referência ao registro separado impede reutilizar aceite anterior. Proposta e registro desativados, sem E4/CI novo; re-review pendente. Reserva 14 ATIVA, 52 itens/48 done/247 arquivos. Digest cb8a47f5d84b722cdd1f6274c7807112132bc6e008acdd073972ddeb1b8b6921; META ativa, SEC-010 não iniciado.


SEC-009: exceção aprovada pela usuária em 2026-10-08 e ativada no escopo cb8a47f5d84b722cdd1f6274c7807112132bc6e008acdd073972ddeb1b8b6921; registro d9e4cb4821fcfbbd7630d361b1eef8bb5092c11520000ba9d796380660188152. 38 testes root verdes; scan real537pacotes/3excepted/0outros. Expira23/10/2026 00hUTC; mudanças/expiração bloqueiam. E4 somente a dispensa delimitada, passes:false até CI exactSHA e fechamento. Reserva14 ATIVA; outros gates/funcionalidades/instalador pendentes.


SEC-009: checkout dos dois inputs Cargo com LF explícito, mesmos hashes/aceite/prazo. 39 casos verdes, incluindo Git checkout Windows autocrlf=true. CI2c263c7/run37790695323 falhou no binding antes do build; correção e novo CI exactSHA pendentes, sem rerun manual. Reserva15ATIVA/passfalse; META final pendente.


SEC-009 executor done/passes:true/E4 limitado ao aceite e E3 ao gate OSV: 39 casos/inventário537/3registros aprovados/0outros; CI entrega corrigida37791933046/SHA1f16a6a3bfde511522fef8fcf5824cc8fd1b7668/21etapas verdes/nativePEAMD64. ExceçãoWindows expira23/10/2026 00hUTC, mesmosinputs/hash/IDs; globalSEC007 parcial e demaisgates pendentes. Coverage frontend atual100%, wrapperPython percentualnão registrado. FechamentoCI final/nota50/liberação15 pendentes; META final não concluída.

## SEC-010 — execução local npm audit

[Spec](specs/SEC-010.md)/[handoff](handoffs/SEC-010.md): reserva11ATIVA; E3 root38casos/4CLIreaisloopback/7.439s e auditonline120dependências/zero reportadas/1.967s; highcritical/erro/relatório parcial bloqueiam. Sem instalar/atualizar dependências ou modificar locks/manifests/workflow; integraçãoCIaudits permaneceSEC012. Review/checks/CIexato/nota51-liberação pendentes, passes:false; METAfinal pendente. Próximo candidatoSEC011 só após liberação; não iniciado.

SEC010 VERIFY: root40casos/4npmCLIreaisloopback/7.799s, auditonline120dependências/0reportadas/1.967s, hashesinputs preservados. DoisP2review reproduzidos/corrigidos; re-reviewP-REV5.6-solmedium E2semachados. Root docs/map/schema/evidence/log/ignore261links/diffsecurity verdes;52items anteriores(em24JSONfiles)/245tuplas/prefixo234620bytes preservados. Produto/manifests/locks/pins/thresholds/workflow/seletor intactos;253tracked/53items49done previstos naentrega. P-QA6-lunalow oráculo+P-REV5.6-solmedium revisão,2delegadosreadonly/perguntasdistintas/0escalonamentos; tokens/custo/cache/latênciamodellos/retriesevitados não registrado. Integração auditCI permaneceSEC012. EntregaCIexactSHA/fechamentoCIexactSHA/nota51-liberação ainda pendentes;passesfalse/reserva11ATIVA/METAfinalpendente.

## SEC-010 — entrega comprovada / fechamento

SEC010 executor done/passes:true/E3 após entrega públicaa93b3de4bb2dc13a864c263f73a4c934647c3de9/[CI37815340184](https://github.com/millennium42/millani-artes/actions/runs/37815340184)/job113442423726/attempt1/main/success21etapas/uma job/1artifactcoverage existente. Root conferiu API/logs reais em memória: OSV39casos/3.66s/537pacotes/3registros aprovados3excepted0outros/expiry2026-10-23, Gitleaks8.30.1/111commits/0achados/0.77s, sourceSHAexato. NativePE32+AMD64/custom-protocol/8563200bytes/SHA256edcb99371fd46f252b0c53b642c9d9da39bb1c049f1f0dee8992d5f595780aee, RustCargo1.99/Node24.21/image20260925.250.1; produto/locks/pins/grants/thresholds preservados. Qualityfrontend passouformat/lint/typecheck/coverage; artifact11566902508/retention1dia solicitado(86399s) lido em memória:100%4linhas5statements1function2branches, thresholds90/90/90/85 inalterados. Isso é apenas frontendscaffold atual, nãofinanças/Rust/Python/setup.exe.
Gate npm40casos locais/4CLIreaisloopback/7.799s, auditreal120dependências/0reportadas/1.818s noSHAentrega; inputs{"package-lock.json":"bb9dd4c788dc028f426b4a7d7c3ef3be7ce02b5374ac555455382e9dfb52b284","package.json":"475d1d3c95ab6a79ae5944eba4c34d100e15fc890a5f21aafac687970d59484a"} preservados. Checker/testes npm NÃO executados remotamente; integraçãoCIaudit éSEC012. Nenhuma nova etapa/job/action/upload/cache ou rerun nesta tarefa. Review2P2corrigidos/re-reviewE2semachados; fonte/testes sem mudanças depois das verificações. Doisdelegadoseconômicos (P-QA6-lunalow/P-REV5.6-solmedium),0escalonamentos; custo/cache/tokens/latênciamodellos/retriesevitados não registrado. Contexto sem rerodar scans/testes após mudanças puramente documentais.
passes:true refere-se ao executor/CIentrega observado; LIBERADA ainda exige CI no SHAfinal/nota51 preservando50/publicrefs/Gitlimpo/cleanup e liberação11paths. Reserva11ATIVA até esse recibo externo àárvore.53items50done253tracked/52itens(em24JSONprévios)245tuplas/prefixo234620preservados. META final appfinanceiro/SQLite/backup/setup.exe/release não concluída; próximoSEC011 só após liberação/não iniciado.

## SEC-011 — gate Cargo / aceite próprio

SEC-011 in_progress/E3+E4/passfalse/reserva13ATIVA. GateCargo47cases/6CLIreais, audit417deps/0vulnsclassificadas/2excepted; scope/prazo aprovados semampliação e reviewfocalE2 semachados. [Spec](specs/SEC-011.md)/[handoff](handoffs/SEC-011.md) registram checks e literalE4. CIentrega/fechamentoexactSHA/nota52-liberação pendentes; SEC012nãoiniciado, METAsetup.exe pendente.

SEC-011 executor/E3+E4: 47casos/6CLIreais/gateCargo417deps0vulnsclassificadas2excepted, entrega3af54a6/CI37824390707 verde21steps. E4scope/prazo/hashes intactos; OSV537/3excepted/Gitleaks114commits0; cobertura100% apenasfrontendscaffold4linhas. Reviewsemachados/checks green/53itemsprévios245tuplas/prefixo245790preservados. passestrue=executor; liberação13 dependeCIfinal/nota52 preserva51. CargoCISEC012pendente; METAsetupfinanceiro pendente. Detalhes na spec/handoffSEC011.

## SEC-012 — audits no CI

SPEC/RECON E2, passes:false, reserva14 ATIVA conforme [spec](specs/SEC-012.md)/[handoff](handoffs/SEC-012.md). Dependências liberadas e CI baseline revalidados; integração sem job/upload/rerun novo. SEC-013 não iniciado; META ativa.

## SEC-012 — entrega corrigida validada

Executor E3/passes:true: [entrega680e8d2](https://github.com/millennium42/millani-artes/commit/680e8d2922f8149959df53c62e9b80390651e5c0)/[CI37832138829](https://github.com/millennium42/millani-artes/actions/runs/37832138829) success23steps/1coverageartifact. Remoto:40npm+47Cargo+42integração/14PowerShell; scan120npmzero/417Cargo0vulns2excepted, OSV537/3excepted, Gitleaks117/0. E4/prazo/inputs/thresholds preservados; coverage100% sóscaffoldfrontend4linhas. FalhainicialRTK ausente reproduzida/corrigida semalterarcasos/instalarferramenta; re-reviewE2semachados. Reserva16ATIVA atéCIfinal/nota53;55items52done267files/245tuplas. Detalhes na spec/handoffSEC012, METAativa/SEC013nãoiniciado.

## SEC-013 — avaliar SAST útil

SPEC/RED/BUILD,E3local/CIpendente,passesfalse,reserva11ATIVA: [spec](specs/SEC-013.md)/[avaliação](../docs/security/SAST_EVALUATION.md)/[handoff](handoffs/SEC-013.md). ReutilizarBiome2.5.15/15fixtures reais, sem nova ferramenta/job/action/upload. Segurança de padrões frontend limitada; METAfinanceira/setup.exe ativa. SEC-014 não iniciado.

SEC-013 VERIFY E3local/reviewE2 semachados,15fixtures/quality/42integração/validators green; CI entrega/final/nota54-liberação ainda pending/passfalse/11ATIVA. Nenhuma etapa/job/tool novo, SAST limitado/METAativa/SEC014nãoiniciado.

SEC-013 executor done/passestrue/E3,entrega a8e02cf7/[CI37851893560](https://github.com/millennium42/millani-artes/actions/runs/37851893560)/23success/15SAST/quality/audits/build; [spec](specs/SEC-013.md#executor--learn--entrega-validada). SASTlimitado/SEC007parcial;fechamento9docs/11ATIVA atéCIfinal/nota54-liberação/METAativa/SEC014nãoiniciado.

## SEC-014 — testar redaction de telefone

[Spec](specs/SEC-014.md)/[handoff](handoffs/SEC-014.md),E3local/4filhos reais/REDvazamento comprovado/catalogohook; fullRustchecks-review-CIpending/passfalse/reserva11ATIVA. Escopo startup/hookRust parcial, semnovo logger/job/plugin produtivo; METAativa/SEC015nãoiniciado.

SEC014 VERIFYlocal:fixture final semplugin/erroIo injetado na fronteira real/buildTauri real/4filhos/6Rustpassed0ignored,fmtclippyauditquality green; falha de biblioteca natural não simulada. Review/CIpending/passfalse/11ATIVA/METAativa/SEC015nãoiniciado.

SEC014 P2corrigido: cinco filhos/quatroprivacidade+guard102, seisRust0ignored/fmtclippy/E3local; re-review/CIpending/passfalse/11ATIVA/METAativa/SEC015nãoiniciado.

SEC014 executor done/passestrue/E3,entrega55b481af4d394aa624d384b6d38887bbdf898a04/CI37857821138/23success/cincofilhosseisRustlocal/re-reviewE2semachados; [spec](specs/SEC-014.md#ci--learn--entrega-validada). Fechamento9docs/11ATIVAatéCIfinalnota55refsGitclean/METAativa/SEC015nãoiniciado.

## SEC-015 — testar redaction de dados financeiros

[Spec](specs/SEC-015.md)/[handoff](handoffs/SEC-015.md),E3local/REDsensibilidade10.625s/5finance+5phone/7Rust0ignored/fmtclippyaudit; mainprodutivo unchanged/catalogo reutilizado. ReviewCIpending/passfalse/reserva11ATIVA; requisitoSEC003parcial/METAativa/ARC001nãoiniciado.

SEC015 executor done/passestrue/E3,entrega96c06e66e53f8c9776d51576bf7f467102487ad2/CI37860308161/23success/5finance5phone7Rustlocal/reviewE2fontessemachados; [spec](specs/SEC-015.md#ci--learn--entrega-validada). Fechamento9docs/reserva11ATIVAatéCIfinalnota56refsGitclean/METAativa/ARC001nãoiniciado.


## ARC-001 — camadas do bootstrap
[Spec](specs/ARC-001.md)/[handoff](handoffs/ARC-001.md): composição main → application/infra e tela src/ui; domínio somente raiz compilada. E3local std-only/6controles/quality/build/audit; review/CI pendentes/passes:false/reserva22ATIVA. ARC002/repositórios/SQLite/finanças/setup.exe não iniciados; META ativa.


ARC-001 executor done/E3/passes:true: [entrega](https://github.com/millennium42/millani-artes/commit/e94eec776213725f72239c8d07aff2a293f6f632)/[CI37863336200](https://github.com/millennium42/millani-artes/actions/runs/37863336200) success23steps/buildWindows, std-only6controles e7Rust locais/reviewE2fontes semachados. Domínio sem regra; finanças/SQLite/backup/setup.exe pendentes/METAativa. Reserva22ATIVA atéCI final/nota57, fechamento11docs; detalhe na spec/handoffARC001. ARC002 não iniciado.

## ARC-002 — Result/erro estático
[Spec](specs/ARC-002.md)/[handoff](handoffs/ARC-002.md): bibliotecaRustdomainResult e InvalidInput fechado sempayload,3domain+7regressões/coverage3linhas100%controle0%/5compilernegatives. Executor done/E3/passestrue, security/review e entrega93362971/CI37865774054 verificados. Branchcounterstableindisponível documentado. Reserva17ATIVA até CI final/nota58; FIN/DB/transação/META pendentes, ARC003 não iniciado.


## ARC-003 — portas injetáveis
[Spec](specs/ARC-003.md)/[handoff](handoffs/ARC-003.md): Clock/SystemTime e UuidGenerator/16bytes/falha opaca, sem adapter/default. E3local3testsnovos+10regressões=13Rust0ignored/std-only5negativos/coverage3linhas100%controle0%/traits0executáveis. Review/securitydiff/CI/nota59 pendentes/passfalse/reserva14ATIVA; finanças/UUID real/vencimento/backup/META pendentes, ARC004 não iniciado.


ARC-003 executor done/E3/passestrue: entrega6eafaa40/[CI37963747801](https://github.com/millennium42/millani-artes/actions/runs/37963747801)/23success, root13Rust0ignored/5negativos/reviewE2semachados. Fechamento11docs/27fontes preservadas/reserva14ATIVA até CI final/nota59/publicrefsGitclean. Próximo candidato ARC004 somente após liberação, não iniciado; META ativa.


## ARC-004 — fronteira de transação

[Spec](specs/ARC-004.md)/[handoff](handoffs/ARC-004.md): caso de uso possui unidade de validação do estado, gravações e auditoria; sucesso somente após commit. Definição documental E2, sem executor/API SQLite ou prova financeira. Base8d951d2/CI anterior e nota59 LIBERADA revalidados; checks/security/review/CI pendentes/passfalse/reserva10ATIVA. Próximo candidato ARC-005 somente após fechamento; META ativa.

ARC-004 executor documental done/E3: entrega9414698f/[CI37968888746](https://github.com/millennium42/millani-artes/actions/runs/37968888746),16success/7skip/0artifacts; reviewE2 semP0P1P2/validators e Gitleaks0. Contrato E2/INV-FIN006007 produtoE0. Fechamento6docs preserva27fontes/reserva10ATIVA atéCIfinalnota60/refsGitclean. PróximoARC005 não iniciado; META ativa.

## ARC-005 — bloquear SQL no frontend

[Spec](specs/ARC-005.md)/[handoff](handoffs/ARC-005.md): lint Biome existente + Grit local + restrição de imports/CommonJS;45casos reais42CLI3PS e quality3tests100% do scaffold. RED SQL deixou passar CLI0 antes da regra; GREEN8.141s. Sem dependência/workflow novo, sem SQLite ou finanças. Executor in_progress/passfalse/reserva14ATIVA; review/CI/nota61 pendentes. Próximo candidato DB001 após fechamento; não iniciado/META ativa.


ARC-005 executor done/E3 após entregac5872e20/[CI37974060505](https://github.com/millennium42/millani-artes/actions/runs/37974060505),23success/45SAST/3frontend100%scaffold/buildWindows/audits/reviewE2semachados. Fechamento11docs preserva28fontes/reserva14ATIVA atéCIfinalnota61/refsGitclean; limites de padrões não são prova SQLite/SQLi/finanças. PróximoDB001←INF006 só após liberação, não iniciado/META ativa.

## DB-001 — criar harness SQLite temporário

[Spec](specs/DB-001.md)/[handoff](handoffs/DB-001.md): rusqlite 0.40.2 dev-only, `bundled`, SQLite em memória real; RED no stub e GREEN E3 local. Root confirmou fmt/clippy, 14 testes Rust e teste release dedicado (SQLite 3.53.2), scans crus (2 warnings Cargo, 0 vuln Cargo, 3 findings OSV nas coordenadas antigas; 5 novas sem findings; removidos 0), e review P-REV 5.6-sol-medium E2 sem P0/P1/P2. Aceite E4 humano recebido para os scopes Cargo `04e69804dd2e7772f766998773152330ef05eedce95a19192f7555613899e829` e OSV `ea0e2c2fbc7095b07d6d8ab063c8291563f7d72aee43203598ac891709224860`, alvo Windows MSVC, expiração inalterada em 2026-10-23 00:00 UTC. Novos records vinculam a resposta literal; gates Cargo/OSV locais passaram após o aceite: Cargo 422/0 vulnerabilidades/2 avisos aprovados; OSV 542/3 exceções/0 não excepcionados. Gitleaks do snapshot pré-E4 da proposta 0 findings/50.749 bytes/SHA `ed2fe7fff59cc93f6748ca7bfbe027b69efeedeadd0faea7057ca652d9bcc1a1`; 4 validators passaram. Executor `done`/E3/`passes=true`/`humanGateRequired=true`; CI Documentation 38095838774/job114341423695, push/attempt1, sucesso 23/23 etapas no SHA de entrega 315fd0feabdb3bc55223441154a9bbdc615c2fd2; artifact frontend-coverage-38095838774-1 retido. Commit documental/CI no SHA final de fechamento ainda não registrados; depois publicar nota final e liberar explicitamente a reserva. Scanners não cobrem código C SQLite; harness não prova finanças, SQLi global, arquivo ou backup. DB-002 não iniciado; reserva 20 ativa até nota pública e liberação explícita.

## DB-002 — abrir banco SQLite vazio em arquivo

[Spec](specs/DB-002.md)/[handoff](handoffs/DB-002.md)/[ADR-024](../docs/architecture/adr/ADR-024-sqlite-file-opening.md): abertura SQLite em arquivo no core; GREEN E3 local, review E2 sem achados e gates Cargo/OSV pós-aceite aprovados. Entrega `b38892374b80aac4b0324f6958d7eb92b4ff7d9b` validada por CI 38103942047, 23/23; main/branch confirmadas. Executor `done`/E3/`passes=true`; DB-002 fechado: CI38104862341/job114368093002 16/7/0 e nota63 pública. `humanGateRequired=true`; startup, containment e domínio financeiro seguem fora do contrato.

## DB-003 — executar migrations SQLite transacionalmente

[Spec](specs/DB-003.md)/[handoff](handoffs/DB-003.md): entrega `04725381191395b7807105833169a1adae9638b5`; E3 local, review E2 sem P0/P1/P2 e CI 38197556421 passou 23/23. CI final documental 38198511557 também passou. Executor `done`/E3/`passes=true`/`humanGateRequired=false`; nota 64 publicada e reserva liberada antes de DB-004.

## DB-004 — registrar e selecionar versões de schema SQLite

[Spec](specs/DB-004.md)/[handoff](handoffs/DB-004.md): executor `done`/E3/`passes=true`/`humanGateRequired=false`; 29 testes Rust, release harness 16/16, review E2 sem achados. CI final 38299901151/job114980000210 no SHA 94e37d38f5df375ffb9492e3f80d272f2f44b331 passou 16 success/7 skipped/0 artifact. Nota 65 publicada; 64 notas e refs preservados e reserva liberada.

## DB-005 — testar migration versionada vazia

[Spec](specs/DB-005.md)/[handoff](handoffs/DB-005.md): E3 local root, RED controlado 0 vs 1 sem falha de produção; GREEN com dois testes. Rust 31/31 (6 lib/7 bin/18 harness), release harness 18/18 (SQLite 3.53.2), fmt/clippy; review final E2 sem P0/P1/P2. Dois arquivos isolados, aplicação versionada com SQL vazio, ledger/constraints/reabertura/integrity_check. Controle é só do harness, não migrations de produto/startup. Executor `done`/E3/`passes=true`/`humanGateRequired=false`; CI entrega 38303443795/job114991660147 passou 23/23 sem skips, release18/18/SQLite3.53.2; scan staged e validators da entrega passaram. CI documental final 38304515870/job114995197572 no SHA 1d92fdd185cf24aececd1fa5b5708d9d0efeab03 passou 16 etapas/7 skipped/0 artifacts; nota66 publicada nas refs 0c75692168c5c220c364733567b5dc28a01a5739, preservadas 65 notas/refs e reserva11 liberada.


## DB-006 — snapshot lógico para teste de upgrade

[Spec](specs/DB-006.md)/[handoff](handoffs/DB-006.md): fixture SQL sintética e estática aplicada em arquivos SQLite de teste por `execute_migration`, sem fluxo versionado ou alteração de produção. RED comment-only compilou (8.05s) e falhou no oráculo 0 passados/1 falho/18 filtrados; 0 objetos encontrados vs 2 esperados; GREEN preencheu fixture. Root E3: fmt2.992s, clippy4.673s, Rust32/32 (6 lib/7 bin/19 harness), release19/19 SQLite3.53.2; 18 testes prévios preservados. Review E2 sem P0/P1/P2. Executor `done`/E3/`passes=true`/`humanGateRequired=false`; CI entrega 38307470576/job115004892529 no SHA 0b8f2a7d0dc8d236d91498cc20e28b358c62dcce passou 23/23, release19/19 SQLite3.53.2. DB-006 fechado: CI final 38308867688/job115009442508 no SHA dd60e198c7ae93bebc41ac873442e5be8193a748, 16 success/7 skipped/0 artifacts; nota67 pública nas refs 69f8ea41a260c442944713b9cff6552f39cbd4a2, 66 notas anteriores preservadas e reserva12 liberada pelo root. Snapshot não comprova migration financeira, startup, backup/restore nem setup.


## DB-007 — upgrade versionado do snapshot

[Spec](specs/DB-007.md)/[handoff](handoffs/DB-007.md): RED controlado ledger `[1]` vs `[1,2]`; GREEN caller versionado. Root E3 Rust33/33, release20/20 SQLite3.53.2, 19 antigos preservados; review sem P0/P1/P2. CI entrega e documental passaram; executor `done`/E3/`passes=true`. DB-007 fechado: CI final 38320496225/job115047171038 no SHA 323e336b53c74147c4b70bdec44391dfad29fe72, 16 success/7 skipped/0 artifacts; nota68 pública refs f9786502025de68b33275fff766f0428cc862bd1, 67 notas preservadas e reserva11 liberada.


## DB-008 — evidência de rollback

[Spec](specs/DB-008.md)/[handoff](handoffs/DB-008.md): tarefa documental sem código; cinco fixtures existentes PASS em stdout debug/release. Root fmt/clippy/Rust33/33/release20/20; review E2 sem achados. CI documental entrega 38322109825/job115052349852 no SHA 08a214888bf581d40e74a143bfdfdc48dea0878a passou 16 success/7 skipped/0 artifact (docs apenas). Executor `done`/E3/`passes=true`; DB-008 fechado: CI final38322839469/job115054651101 no SHA dfbe60893150c222b17987518f65a161d3e7ef77, 16/7/0; nota69 refs054bfc0f6bce94a795158a89ded50d20d71bd989, 68 notas preservadas/reserva10 liberada. Sem prova de falha física de rollback.


## DB-009 — ativar foreign keys ao abrir

[Spec](specs/DB-009.md)/[handoff](handoffs/DB-009.md): baseline ON, controle OFF fail-closed; E3 local 34/34 e release21/21; CI entrega 38324569231/job115060146427 passou 23/23. Executor `done`/E3/`passes=true`. DB-009 fechado: CI final 38325659041/job115063600279 no SHA 17f5397efc996091bfb4271a5169d4eede4249fe (16/7/0); nota70 refs 097a4650bac277404275272876296cbd5e682dbb, 69 notas preservadas e reserva12 liberada pelo root.


## DB-010 — provar enforcement de foreign keys (fechado)

[Spec](specs/DB-010.md)/[handoff](handoffs/DB-010.md): executor fechado no baseline SHA `083ac68e073667db3cba474722962b89ca7cdaa5`; CI `38387475775` SUCCESS 16/7/0; nota71 pública `dc2a872282bfaf9fb035cef0a0c336e6bdc8ff19`, 70 anteriores preservadas e reserva11 liberada. Prova limitada a schema sintético.

## DB-011 — definir transação SQLite (E3 local; fechamento pendente)

[Spec](specs/DB-011.md)/[handoff](handoffs/DB-011.md): API `execute_write_transaction` executa BEGIN IMMEDIATE e migrations mantêm DEFERRED no helper compartilhado. Variante final produção SHA-256 `fbf2d6d274091ea65f444e88e639c411e332930f83a3f530abd607444d13fdbe`, harness `aec307745a413a46ba209d3a3228dd0ba14505999b309468d3ad4c5c16cc7913`. Root reexecutou fmt/clippy, Rust39/39 e release26/26; quatro testes novos passam debug/release. Review E2 e CI de entrega passaram; executor fechado E3. Fechamento documental DB-011 confirmado no SHA `0c0660cf5abe9b1e51a5067c49c11e7364069764`, CI final `38397098609`, nota72 publicada, 71 anteriores preservadas e reserva15 liberada.

## DB-012 — verificar rollback genérico (E3 local; CI pendente)

[Spec](specs/DB-012.md)/[handoff](handoffs/DB-012.md): executor `done`/E3/`passes=true` após entrega CI `38400843616` (SHA `03042f6f980ae90f4c74ad989d3dc6c6c7a19a01`, 23/23, harness31/31, cinco `db012_*` PASS). fmt/clippy, Rust44/44, release31/31, Cargo/OSV e validators4/344 files/28 sources/74 items passaram. Produção restaurada ao SHA `fbf2d6d274091ea65f444e88e639c411e332930f83a3f530abd607444d13fdbe`, harness `7bf3b01606b6a1d99ced6dc38280f75f1b7c8de6aec859847392a0d4254b32f4`. Scans remotos registrados; Gitleaks staged entrega11 paths/45.749B/0 findings. Review E2 sem P0/P1/P2, escopo limitado ao diff. INV-FIN-006/007 produto continua E0. CI documental final, nota73 e liberação13 pendentes; DB-013 não iniciar.

## DB-013 — evento técnico de auditoria (E3 local; entrega pendente)

Executor `done`/E3/`passes=true` após CI entrega `38429912850` no SHA `63ef7f6004f3e697f365f34f681c499726f2f867` (23/0 skipped, release36/36, cinco db013_* PASS). Review E2 sem achados impeditivos. Migration v2 imutável; detalhes/limites na [spec](specs/DB-013.md). CI documental, nota74, Git limpo/liberação ACTIVE16 pendentes; DB-014 não iniciado.


## DB-014 — auditoria na mesma transação (E3 local; CI entrega pendente)

Cinco testes SQLite físicos passaram; RED detectou persistência indevida após erro de audit, produção restaurada byte a byte ao baseline. Hashes, comandos, review e limites no [handoff](handoffs/DB-014.md). Executor done/E3/passes:true após CI `38438041786` no SHA `af1c5aabbefa785b8d26b7351cddf538ad68769b` (23/0, harness41/41). Nota75/preservação, Git limpo e liberação ACTIVE13 ainda pendentes. INV-FIN-006/007 seguem E0.
FIN-002 VERIFY local E3/root: RED Cargo101/wrapper1/E0599; fmt2.148s, clippy18.29s; debug58 ordinários+2 doctests/0 falhas ou ignorados; audit Cargo422/0/2 aprovados, OSV542/3 exceções/0 fora; LLVM direto domain/mod.rs full8/8 linhas, controle3/8 rejeitado em90, branches não medidos stable; review E2 sem achados. Cleanup E3: processos WebView próprios0, artifacts/sec001-webview-profile e artifacts/fin002-coverage ausentes após containment/reparse guard. Estado permanece in_progress/E3/passesfalse; Gitleaks staged, validators e CI pendentes. FIN-003 aguarda fechamento/liberação.
FIN-002 executor concluído E3: CI de entrega 38483459997/job115591336901/attempt1/SHA395a5352405c4465dfd5644a8caab2f70ffdff1c SUCCESS23/0; suíte release57 ordinários+2 doctests/0 falhas/1 ignored, 3 account_name e junction PASS. Scans remotos Gitleaks169/0, npm120/0, Cargo422/0 vuln/2 aprovados, OSV542/3 exc/0 fora, SAST45 limitado; Rust coverage não executada no CI. Falha anterior38482025966 preservada. CI documental final/nota77/liberação ACTIVE25 pendentes; FIN-003 somente após liberação.
FIN-003 VERIFY local E3/root sobre base b5fabc: RED2.618s E0432/E0560/E0609; fmt/clippy; debug58 ordinários+2 doctests/0 fail/1 ignored; audit Cargo422/0/2 E4, OSV542/3exc/0fora; LLVM full8/8 linhas, controle3/8 rejeitado90; review E2 sem P0/P1/P2. Enum AccountType não tem statements, branch stable indisponível. Cleanup cobertura/fixture confirmado. Estado in_progress/E3/passesfalse; validators, Gitleaks staged e CI pendentes; FIN-004 aguarda liberação.
FIN-003 executor E3 concluído: CI38487235749/job115603712741/attempt1/SHA02c86f9f3fe623f809944b0f1d8636da319c94e9 SUCCESS23/0; release58 ordinários+2 doctests/0 fail/1 ignored, tipo/name+junction PASS. Scans Gitleaks171/0, npm120/0, Cargo422/0 vuln/2 aprovados, OSV542/3 exc/0 fora, SAST45 limitado; Rust coverage não executada no CI. MVP-01 parcial e regras financeiras E0. CI documental final/nota78/liberação ACTIVE23 pendentes; FIN-004 aguarda liberação.
FIN-004 VERIFY local E3/root sobre base3ef70f1: RED7 erros por campo ausente; fmt/clippy PASS; default suite59 ordinários+2 doctests/0 falha/1 ignored; audit422/0/2, OSV542/3exc/0fora; LLVM full8/8 linhas, control3/8 rejeitado90; review E2 sem achados; cleanup seguro. Fonte preserva seis valores i64 sem cálculo, não prova regra financeira. Validators/Gitleaks staged/CI pendentes; FIN-005 aguarda liberação.


FIN-004 VERIFY atualizado: startup test corrigido, instrumentação global mede 96.474359% linhas/90.909091% funções; gates existentes global90 e crítico database95 PASS. Statements/branches não medidos; sem semântica financeira. FIN-004 permanece in_progress até CI corretiva, nota79 e liberação; FIN-005 aguarda.

FIN-004 executor E3 concluído na entrega 9e2068b: CI38504766831 SUCCESS23/0, suíte release60+2 doctests/0 falhas/1 WebView2 ignored; nota79 e CI documental final ainda necessárias para liberação. Próxima sequência após liberação: integração serial CAT-001; FIN-006 e FIN-010 elegíveis em paralelo; FIN-005 segue sem prova de receita real, sem nova dependência formal.

CAT-001: integração primária iniciada após FIN-004 liberado; Category permanece metadado puro (ID opaco, nome, bool), sem workflow de ativação inferido. Worktree standalone/review existentes são evidência focal, não substituem os gates primários. CAT-001 in_progress até checks/CI/nota80; FIN-010 pode desenvolver em worktree paralelo, integração serial.

CAT-001 VERIFY local primário: fmt/clippy, 61 testes+4 compile_fail doctests/0 falhas/1 ignored, audit/OSV e LLVM full passaram; controle 3/8 linhas rejeita90. Category é declaração sem execução própria. Gitleaks staged, validators finais e CI pendentes; permanece in_progress.

CAT-001 executor E3 concluído: CI38507827612 no SHA c4def86 SUCCESS23/0, release61+4 doctests/0 falhas/1 ignored; 79 executores concluídos/166 restantes. Liberação documental ACTIVE24 ainda pendente. Próxima integração operacional FIN-010; FIN-006 requer SPEC/review antes de schema; FIN-005 sem consumidor real.

FIN-010 documentação integrada ao primário sobre base CAT-001 dfa8572; dependência formal ARC-002 apenas. Implementação standalone E3/review E2 não substitui verificações primárias. FIN-010 aguardando gates primários; FIN-006 permanece independente para desenvolvimento paralelo.

FIN-010 VERIFY local primário E3: fmt/clippy, Rust62+4 doctests, audit/OSV PASS; LLVM full8/8 linhas e controle3/8 rejeita90. Review primária, Gitleaks staged, validators e CI pendentes; sem commit SHA ainda. Estado histórico anterior à entrega verde abaixo.

FIN-010 executor concluído (E3, passes:true): entrega CI ea1f689c22bf0321734efc86e5b848cdd61b7166, run 38512646302/job 115684416671, sucesso23/0; 62 testes ordinários+4 doctests, 1 WebView2 ignored. CI documental, nota81 e liberação ACTIVE24 ainda pendentes. FIN-005 segue sem consumidor real de receita; MVP não concluído.


## FIN-006 — migration de contas (E3 local; integração pendente)
[Spec](specs/FIN-006.md)/[handoff](handoffs/FIN-006.md): migration imutável 003 e executor versionado exercitados no WT. Harness SQLite físico 43/43, SQLite 3.53.2: banco vazio v1/v2/v3 e ledger `[1,2,3]`, upgrade com audit preservado, reopen/integrity e round-trip das quatro colunas, NUL inicial e extremos i64. Review E2 sem P0/P1/P2. Work item permanece `in_progress`/E3/`passes=false`; Cargo, security, validators e CI integrados pendentes. Dependências formais somente DB-005 e FIN-001; sequência operacional após FIN-010 não cria dependência.

FIN-006 primário: FIN-010 agora fechado/liberado pela nota81; fmt/clippy, Cargo64+4, audit/OSV e review primária E2 passaram. FIN-006 permanece in_progress/E3/passes:false enquanto coverage focal, Gitleaks staged, validators e CI integrada aguardam execução. Evidência de SQLite cobre persistência física, não startup, consumer nem receita FIN-005.

FIN-006 focal coverage primária E3 concluída: database.rs 276/280 linhas=98.57142857%,25/25 funções=100%, gates ADR-019 >=95% passam. Statements não medidos; branches N/A; sem vetor completo/global. Próximos gates: Gitleaks staged, validators e CI integrada.


FIN-006 executor concluído E3: CI 38570028370/job115872486265 no SHA914b230edc705644303a27e0b8d18296e80ef002, 23/0; suite64+4 PASS/1 WebView ignored. Liberação ACTIVE18 pendente de CI documental, nota82, preservação de 81 tuplas/3 refs, Git limpo e autorização root. Próximo operacional CAT-002, depois FIN-011; FIN-007 após fechamento formal.

CAT-002 executor concluído E3/done/passes:true/humano false: CI 38616439285/job116032611143 SUCCESS 23/0 no SHA `3a94855181576ed354a73831dca3e1964641e79d`; release67 testes+4 compile-fail PASS/1 WebView ignored, junction guard/category tests PASS. Gitleaks staged12/30.621B/0; OSV542/3/0; audit422/0/2 aprovadas; NPM120 deps/0 vulns/0.700s; SAST45/4.453s; validators mapa377/28/83 e log337.080B preservado. Review final E2 sem P0/P1/P2. Publicação final pendente. Sem consumer, persistência, cobertura Rust global, FIN-005 ou setup/instalador. Próximo operacional FIN-011, depois FIN-007 após liberação.


FIN-011 executor concluído `done`/E3/`passes=true`/humangate false. Source CI38621842911/job116051059765/attempt1/main/push SUCCESS23/0 no SHA `809626e89dabbcce158fb466f63401c0a7b02c99`; 70 testes ordinários+4 compile-fail doctests PASS/1 WebView ignored; três Context/junction PASS. Gitleaks184/0, OSV542/3 approved/0 outside, Cargo422/0 vulns/2 approved, NPM120/0, SAST45, staged Gitleaks0 e validators4 PASS. Review E2 sem achados impeditivos. Primary LLVM full20 PASS/19-19 linhas/4-4 funções/25-25 regions; controle17 PASS/3 filtrados/13-19 linhas rejeita90; sem coverage global. Publicação final pendente de CI documental/nota84 e liberação ACTIVE24 após preservação de 83 tuplas/ancestral/refs/Git/roots/scanner-cache. Contagem oficial: 245 executores,83 done,162 restantes (BOOTSTRAP-INDEX achatado). Sem consumer/contrato externo/META completo; próximo operacional FIN-007, depois FIN-008.

FIN-007 primary local E3: fmt/clippy/Cargo72+4 doctests PASS, audit/OSV e review E2 sem P0/P1/P2; critical `database.rs` coverage 276/280 linhas e25/25 funções (gate95 PASS). Cleanup E3 concluído. Permanecem staged Gitleaks, 4 validators e source CI; executor `in_progress`/`passes=false`, ACTIVE15 retida.

FIN-007 executor concluído `done`/E3/`passes=true`/humanGate false. Source CI `38628675813`/job `116074330758` SUCCESS23/0 no SHA `a58ad20b7664db5f7b5fb85a3cb7b0b341a46978`; release72+4 doctests PASS/1 WebView ignored, Gitleaks187/0, OSV542/3 aprovadas/0 fora, npm120/0, Cargo audit422/0 vulnerabilidades/2 aprovadas, SAST45 (não Rust), validators4 e junction smoke PASS. Publicação final/nota85/liberação ACTIVE15 ainda pendentes; sem alegação de publicação.

FIN-008 executor concluído done/E3/passes=true/humanGateRequired=false: source CI 38765481430/job116527438722 SUCCESS23/0 no SHA eea53ff05f2df34c5ff8e9aa759b70dc0a25ab57; release82+4 compile-fail PASS/1 WebView ignored; Gitleaks staged19/76.690B/0; OSV542/3/0; npm120/0; Cargo audit422/0 vulnerabilidades/2 aprovadas E4; SAST45 (não Rust); validators4/388 arquivos/28 fontes/86 itens. CI documental, nota86, 85 tuplas/3 refs e liberação ACTIVE24 pendentes. Próximo operacional CAT-003; FIN-009 é dependente formal e não iniciado. Sem UI/IPC/startup/receita/INV-FIN-001/coverage global/setup.


## CAT-003 — integracao primaria 16/10/2026

Integracao inicial humana autorizada por fast-forward fa62d588->7573c65;97branches ancestrais/86notas preservadas e refs main/retomada publicadas. CAT-003 ainda in_progress/E3/passesfalse,com gateslocaisPASS;CI38907719490 em execucao. FIN-012 e UI-001 permanecem abertos,160IDs pendentes;sem alterar grafo. Fonte: planning/handoffs/CAT-003.md e planning/specs/CAT-003.md, SHA7573c65899416cf180bb25545f6cfae6335bb00e; E3 local, CI de entrega em execucao, item ainda aberto.


CAT-003 executor done/E3/passestrue apos CI38907719490/job116995418974/attempt1/SHA7573c65899416cf180bb25545f6cfae6335bb00e SUCCESS23/0. Evidencia e limites em planning/handoffs/CAT-003.md;CI documental final/nota87/liberacao pendentes. FIN-012/UI-001 continuam abertos.


## FIN-012 — integração primária em 16/10/2026

CAT-003 liberado após CI final 38909068737 e nota 87. FIN-012 está em fechamento serial; UI-001 aguarda sua liberação. Grafo formal preservado. Fontes e evidência: planning/specs/FIN-012.md e planning/handoffs/FIN-012.md; source 7573c65, main 7cad038, produção idêntica. Item ainda aberto até fechamento documental.

FIN-012 executor done/E3/passes true após os checks e CI do código integrado, com fonte em planning/handoffs/FIN-012.md. CI documental final, nota88 e liberação pendentes. UI-001 aguarda liberação. Sem comprovação de consumidor, regras financeiras, coverage global ou instalador.


## UI-001 — verificação primária em 16/10/2026

Fonte: planning/handoffs/UI-001.md, source7573c65/main59a3789, E3 local.

```json
{
  "FIN-012": "liberado com CI38910468674 e nota88,87notas preservadas",
  "UI-001": "inspeção nativa concluída; fechamento/CI final/nota89/liberação pendentes",
  "próximo": "reconciliar elegibilidade FIN-005 após liberação; grafo preservado"
}
```


UI-001 executor concluído: E3 de execução e inspeção nativa, checks do código idêntico e CI source38907719490 SUCCESS23/0; review integrado/visual E2 sem P0/P1/P2. Documentos e matrizes atualizados antes de passes true. Reserva retida até CI documental final no novo SHA, nota89/preservação88notas/Gitlimpo/WTarquivada e liberação do root. Código de saída nativo não registrado(null); sem claim exit0, WCAG numérico, mínimo de produto, jornada financeira, Rustglobal ou setup. Próximo após liberação: reconciliar elegibilidade FIN-005 no roadmap.


FIN-009 in_progress/E3/passesfalse: leitura integrada localmente,97+8 debug/release e98+8 instrumentadoPASS;global459/470linhas46/49funções/críticos95PASS;reviewsemP0P1P2. CI source/final/nota90/liberação pendentes;supersede nota histórica de não iniciado,semjornadaUIIPC/setup.


FIN-009 executor done/E3/passestrue após CI39009285553/job117328105513/attempt1/SHA83e682303354c000d2319f512f8321882901aa35 SUCCESS23/0,97ordinários+8doctestsPASS/1ignored. Evidência local global459/470linhas46/49funções ecríticos95PASS não é coverage CI; statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4 atualizados; CI documental final/nota90/preservação89/liberação ainda pendentes. Sem UI/IPC/receita/setup; fonte planning/handoffs/FIN-009.md.


FIN-013 in_progress/E3/passesfalse: validação pura Entrypositivo integrada localmente;5focais e102+8 debug/releasePASS/1ignored;domainmodcrítico24/24linhas5/5funções/controlerejeita95;security/E4/reviewPASS. CI source/final,nota91/liberação pendentes. SemUIIPC/persistência/receita/setup. FIN-005 ainda sem consumidor de receita.


FIN-013 executor done/E3/passestrue após CI39021912847/job117365252558/attempt1/SHAa3dee6febce169d4dbac3779ce6a67c998e506ab SUCCESS23/0,102ordinários+8doctestsPASS/1ignored. Crítico domainmod24/24linhas5/5funções/controle rejeita95 é E3 local,não coverageCI/globalatual;statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4/validators atualizados;CIfinaldocumental/nota91/preservação90/liberação pendentes. SemUIIPC/persistência/receita/setup. Fonte planning/handoffs/FIN-013.md.


FIN-014 in_progress/E3/passesfalse: saída positiva pura integrada;5focais e107+8 debug/releasePASS/1ignored,crítico29/29linhas6/6funções/controlerejeita95,security/E4/reviewPASS. CI source/final/not92/liberação pendentes;semUIIPC/persistência/receita/setup/globalatual.


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

FIN-017 permanece in_progress/E2/passesfalse sob roteamento humano de 17/10: modelos locais encerrados pelo controlador (PID próprio 49056; porta 11434 fechada), P-QA GPT Sol 6 medium preparou testes, P-INT implementa e revisão P-REV independente pendente. Sem GREEN/gates/CI ou fechamento; custo não registrado. Esta anotação não altera ordem/dependências nem seleciona próximo ID.


FIN-017 | E3 local:6unit+3SQLite/4falhas,criaçãoEntry+audit atômicas/reopen/rollback/poison/inputinválido sem efeitos;Rust121+8docs/debug+release/fmt/clippy/TS/securityE4 PASS;coveragecrítica app43/43linhas3/3fn eadapter51/51linhas5/5fn,controle0,rejeita95. Review independente concluído;CI source/final/liberação pendentes,in_progress/passesfalse. Sem jornadaUIIPC/globalRust/setup/E2E;apósFIN017METApausará,nenhum próximoID. Ver spec/handoffFIN017;custonão registrado.


FIN-017 | done/E3: source5a09ca2ec7117be7e39605bfe25848e7fc39acaf CI39090756071/job117566803340/attempt1success23/0skip/7marcadores exatos;121testesRust+8doctests. Entry+audit atômicos/6unit+3SQLite4falhas,critical app43/43linhas3/3fn+adapter51/51linhas5/5fn100%,controle0. Gateslocais/review/validators/stagedscan aprovados;docs antespassestrue. CI documental/nota95/liberação pendentes;WTarquivada,evidênciapreservada,locaisencerrados. DepoispausarMETA por decisão humana,nenhumpróximoID. ProdutoUIIPC/receitaFIN005/globalRust/setup.exe/E2E pendentes;custonão registrado.


CAT-004 em VERIFY local E3,6unit+3SQLite e130Rust+8docs/debug+release,critical100%/controle0,securityE4 e reviewsemP0/P1/P2;in_progress/passesfalse atéCI source. FIN005 permanece sem consumidor real de receita. MetaE2E ativa; sem mudar grafo. Fonte planning/handoffs/CAT-004.md.


CAT-004 executor done/E3/passestrue após CI39094721622 no SHAcbf40d8392e66a38146ebfd6abb61cd42766d47e success23/0 e130Rust+8docs; detalhes e limites no handoff CAT-004. CI documental final/nota96/liberação pendentes. MVP02 criação backend parcial; semUIIPC/E2E/setup. MetaE2Eativa.


CAT-005 | VERIFY local E3 | 8 unitários + 3 SQLite e 141 Rust + 8 doctests/debug/release PASS, qualidade/segurança E4 e revisão independente sem P0/P1/P2. In_progress/passesfalse até CI fonte. FIN-005 ainda sem consumidor real. Meta E2E ativa; detalhes no handoff CAT-005.


CAT-005 executor done/E3/passestrue após CI39098498169 no SHAaef71f2308dbfa1d11f4960568fc0b041d1e7221 success23/0 e141Rust+8doctests, incluindo8unit+3SQLite. Gates locais, cobertura crítica >=95 e controle, revisão independente e validators/stagedscan aprovados; detalhes no handoff CAT-005. CI documental final/nota97/liberação pendentes. MVP02 renomeação backend parcial; UIIPC/desativação/E2E/setup pendentes. Meta E2E ativa.


CAT-006 VERIFY E3:7unit+3SQLite e151Rust+8docs/debug/release PASS,quality/securityE4 e revisão independente semP0/P1/P2;in_progress/passesfalse atéCI. MetaE2Eativa; FIN005 ainda sem consumidor. Fonte: handoffCAT006.


CAT-006 done/E3/passestrue após CI39128570976 no SHA25311c0fb01d066e39d557e3779287d680553c7f,23success/0skip e151Rust+8docs;7unit+3SQLite e gateslocais/review/validators/stagedscan PASS. Fonte: handoffCAT006. CI documental final/nota98/liberação pendentes. MetaE2Eativa; UIIPC/histórico consumidor/setup/E2E pendentes.


CAT-007 VERIFY E3: migration6 aditiva e2SQLite;Rust153+8docs/debug/release,quality/securityE4/critical>=95controle/review PASS;in_progress/passesfalse atéCI. FIN005 ainda sem consumidor;MetaE2Eativa. Fonte: handoffCAT007.


CAT-007 done/E3/passestrue após CI39131643684/SHA143190b374d186da75f2a44bdc85cb0bfad58219success23/0skip e153Rust+8docs,2SQLite novos. Migration6/FKhistórica física/reopen/rename/deactivate e gates/review/validators/stagedscan PASS. CI documental final/nota99/liberação pendentes;MetaE2Eativa. FixtureSQL não comprova atribuiçãoUIIPC/globalRust/setup/E2E. Fonte:handoffCAT007.


FIN-018 VERIFY E3:6unit+3SQLite/4faults e162Rust+8docs/debug+release,quality/securityE4/critical100%controle/review PASS;in_progress/passesfalse atéCI. FIN005 sem consumidor;MetaE2Eativa. Fonte:handoffFIN018.


FIN-018 done/E3/passestrue após CI39134383058/SHA4f7e7703793ac8ef5632935ade79badd3f9d44fasuccess23/0skip e162Rust+8docs,6unit+3SQLite. Saída+audit atômicos/rollback4faults/poison e gates/review/validators/stagedscan PASS. CI documental final/nota100/liberação pendentes;MetaE2Eativa. Sem saldo/receitaFIN005/UIIPC/globalRust/setup/E2E. Fonte:handoffFIN018.


FIN-019 VERIFY E3:validate_transfer puro/5unit e167Rust+8docs/debug+release,quality/securityE4/critical100%controle/reviewPASS;in_progress/passesfalse atéCI. FIN005 sem consumidor;MetaE2Eativa. Fonte:handoffFIN019.


FIN-019 done/E3/passestrue após CI39136146173/SHA7ac497730b507c5576b05bff8f285169215b21cbsuccess23/0skip e167Rust+8docs,5unit. Validação pura Transferpositivo e gates/review/validators/stagedscan PASS. CI documental final/nota101/liberação pendentes;MetaE2Eativa. FIN020/021/022/UIIPC/globalRust/setup/E2E pendentes. Fonte:handoffFIN019.


FIN-020 VERIFY E3:contasiguais rejeitadas no validador puro;REDregressão/GREEN7/Rust169+8docs/debug+release/quality/securityE4/critical/reviewPASS;in_progress/passesfalse atéCI. FIN005semconsumidor;MetaE2Eativa.


FIN-020 done/E3/passestrue após CI39137975946/SHA78beb2f62b024899d923b136251d3907b937b82asuccess23/0skip/7marcadores,Rust169+8docs/7transferunit(2novos). Contasiguaisrejeitadas/contasdistintas/metadados preservados,gates/review/validators/stagedscanPASS. CI final/nota102/liberaçãopendentes;MetaE2Eativa;FIN021/022/023/UIIPC/globalRust/setup/E2Ependentes. Fonte:handoffFIN020.


FIN-021 VERIFY E3:create_transfer+audit atômicos7unit+3SQLite/5falhas,reuso/poison/errosopacos;179Rust+8docs/debug+release/quality/securityE4/criticalcontrole/P-REV PASS. In_progress/passesfalse atéCI;FIN022neutralidade/FIN023matrizrollback/FIN024saldos/receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. MetaE2Eativa;fonte:handoffFIN021.


FIN-021 done/E3/passestrue após CI39142344305/SHA1ba38ca0cca75a313431cc6ab5ae89ef50db88f3success23/0skip/7marcadores,Rust179+8docs/7unit+3SQLite. Gates/review/validators/stagedscanPASS;CI final/nota103/liberaçãopendentes;MetaE2Eativa;FIN022neutralidade/FIN023matrizrollback/FIN024saldos/receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;fonte:handoffFIN021.


FIN-022 VERIFY E3/INV-FIN003:contribuiçãorealporMovement Entry+/Exit-/Transfer0validada,5unit/RED-GREEN/metadados/limites/contextos/neutralidadecomposição;184Rust+8docs/debug+release/quality/securityE4/LLVMdomaincriticalcontrole/P-REV PASS. In_progress/passesfalse atéCI;relatórioagregado/filtrocontextoFIN027,saldoFIN024,matrizrollbackFIN023,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN022.


FIN-022 done/E3/passestrue após CI39144045457/SHAe42ec5ca331be38c7e0c5d9dfb6cca4b3bd12b3bsuccess23/0skip/7marcadores,Rust184+8docs/5unit. ContribuiçãoEntry+/Exit-/Transfer0validada/gates/review/validators/stagedscanPASS;CI final/nota104/liberaçãopendentes;relatórioagregado/filtrocontextoFIN027,saldoFIN024,matrizrollbackFIN023,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN022.


FIN-023 VERIFY E3:3falhas adicionais físicas auditPK/auditIGNORE/COMMITFK, snapshots/reopen/poison/novo adapter;RED2mutantes/GREEN1,produção inalterada. Rust185+8docs/debug+release/quality/securityE4/coveragefamília/P-REV PASS;in_progress/passesfalseatéCI;saldoFIN024/relatórioFIN027/receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN023.


FIN-023 done/E3/passestrue após CI39147317365/SHAac7659754b5dc5733c82ce369c5b6c4053610d3bsuccess23/0skip/7marcadores,Rust185+8docs/novoSQLite3casos. Produçãoinalterada/RED2mutantes/gates/review/validators/stagedscanPASS;CI final/nota105/liberaçãopendentes;saldoFIN024/relatórioFIN027/receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN023.


FIN-024 VERIFY E3:saldo real porAccount comi128checked/erroopaco/5unit+1SQLite persistência/list_accounts/saldos-1179/7669. Rust191+8docs/debug+release/quality/securityE4/LLVMdomaincriticalcontrole/P-REV PASS. In_progress/passesfalseatéCI;prova de edição/derivaçãoFIN025,leitor público/extratoFIN026,resultadoFIN027,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN024.


FIN-024 done/E3/passestrue após CI39175166911/SHAd6c3ba969dd9daeb9f3b02f881518a8d3dfabadfsuccess23/0skip/7marcadores,Rust191+8docs/5unit+1SQLite. SaldoAccount real i128checked/gates/review/validators/stagedscanPASS;CI final/nota106/liberaçãopendentes;prova de edição/derivaçãoFIN025,leitor público/extratoFIN026,resultadoFIN027,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN024.


FIN-025 VERIFY E3:1unit replayreal2contas/prefixos/reverse/vazio/repetição;2mutantesRED/GREENreal/produçãozero. Rust192+8docs/debug+release/quality/securityE4/LLVMdomaincriticalcontrole/P-REV PASS. In_progress/passesfalseatéCI;edição canônicaFIN028/prova semsaldo diretoFIN030,leitor público/extratoFIN026,resultadoFIN027,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN025.


FIN-025 done/E3/passestrue após CI39176957384/SHAc408c42a3a06135b6792262d7d330479ac9cf1f1success23/0skip/7marcadores,Rust192+8docs/1unitnovo replay/2mutantesRED eproduçãozero. Gates/review/validators/stagedscanPASS;CI final/nota107/liberaçãopendentes;edição canônicaFIN028/prova semsaldo diretoFIN030,leitor público/extratoFIN026,resultadoFIN027,receitaFIN005/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN025.


FIN-026 VERIFY E3:leitorcanônico real port+adapterstrict/erroopaco/1call/read-only e saldo pelo leitorpúblico. 3unit+3SQLnovos+1SQLsaldoPASS/Rust198+8docs/debug+release/quality/securityE4/LLVMapp+adaptercriticalcontrole/P-REV PASS. Writers/domínio/migrations/deps preservados. In_progress/passesfalseatéCI;UIIPC/cronologia/resultadoFIN027/receitaFIN005/ediçãoFIN028/030/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN026.


FIN-026 done/E3/passestrue após CI39180075013/SHA6b8b77b7840579b8a4d6c50a1c4c8de0d15f71f1success23/0skip/7marcadores,Rust198+8docs/3unit+3SQLnovos+SQLsaldo viareaderreal. Gates/review/validators/stagedscanPASS,writerspreservados;CI final/nota108/liberaçãopendentes;UIIPC/cronologia/resultadoFIN027/receitaFIN005/ediçãoFIN028/030/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN026.


FIN-027 VERIFY E3:resultado líquido realporcontexto viaport/helperdomínio,Transfer0/validaantesfiltro/i128checked/erroopaco. 5unit+2SQLnovos+1SQLsaldoPASS/Rust205+8docs/debug+release/quality/securityE4/LLVMnewappcriticalcontrole/P-REV PASS. In_progress/passesfalseatéCI;provaFIN005/ediçãoFIN028/030/UIIPC/cronologia/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN027.


FIN-027 done/E3/passestrue após CI39181782882/SHAd438ab051db80153f5f2c505a56f710f1f0d0d41success23/0skip/7marcadores,Rust205+8docs/5unit+2SQLnovos+SQLsaldo chamandoresultadoreal. Gates/review/validators/stagedscanPASS;CI final/nota109/liberaçãopendentes;provaFIN005/ediçãoFIN028/030/UIIPC/cronologia/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN027.


FIN-005 VERIFY E3:1SQLreal/openingsMIN/-1/0/1/MAX semEntry/report0,2entradasreais/report1e2;REDmutanteAccountStore/GREENreal. Rust206+8docs/debug+release/quality/securityE4/LLVMcalcsourceidêntica-controle/P-REV PASS. Produçãointacta/depformalFIN004preservada/consumerFIN026/027real. In_progress/passesfalseatéCI;ediçãoFIN028/030/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes. Fonte:handoffFIN005.


FIN-005 done/E3/passestrue após CI39184150887/SHAfd0f63a5decdefe4ea1715aee67e3a975cd4a192success23/0skip/7marcadores,Rust206+8docs/1SQLinitial+mutanteRED/GREENreal. OpeningsnãoEntry/report0,2entradasreais/report1e2,consumidoresFIN026/027reais;gates/review/validators/stagedscanPASS/produçãointacta. CI final/nota110/liberaçãopendentes;ediçãoFIN028/030/UIIPC/currentglobalRust/setup.exe/instalação/atualização/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoffFIN005.


FIN-028 VERIFY E3: edição canônica real/identidade estável/UPDATE+audit atômicos/rollback e preservação de dados. 6unit+4SQL/Rust216+8docs/quality/security/LLVM app+adapter>=95 com controle/review PASS. In_progress/passesfalse até CI. FIN-029/030, UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte: handoff FIN-028.


FIN-028 done/E3/passestrue após CI39187261583/SHA15020e5ea781b2ac58edb501c1abebb459a4493e success23/0skip/7marcadores;Rust216+8docs/6unit+4SQL. Edição canônica real/ID estável/UPDATE+audit atômicos/rollback/dados preservados;gates/review/validators/stagedscan PASS. CI final/nota111/liberação pendentes;FIN-029/030, UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;Meta E2E ativa;fonte:handoff FIN-028.


FIN-029 VERIFY E3:1SQL real/sete edições/audits exatos e imutáveis/rollback físico,2mutantes RED/GREEN;produção intacta. Rust217+8docs/quality/security/critical existente com controle/review PASS. In_progress/passesfalse até CI;FIN-030, UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte: handoff FIN-029.


FIN-029 done/E3/passestrue após CI39189813304/SHA7ba07364c37a1624fa349f04897e59c981ae6abb success23/0skip/7marcadores,Rust217+8docs/1SQLauditnovo/2mutantesRED/GREENreal/produçãointacta. Gates/review/validators/stagedscan PASS;CI final/nota112/liberação pendentes;FIN-030, UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff FIN-029.


FIN-030 VERIFY E3:reuso de testes reais puro/SQLite confirma saldo derivado após edição,aberturas/contas/ledger intactos;2mutantesRED/GREEN. Rust217+8docs/quality/security/critical existente com controle/review PASS;produção/testes intactos. CI documental pendente,sem Rust remoto atual;UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff FIN-030.


FIN-030 done/E3/passestrue após CI documental39191779815/SHAb05b88280ef9b39b23ed4cc61df5eb210b4a9764success16/7skip/5marcadores. E3 localRust217+8docs/reuso2testes/2mutantesRED/GREEN/gates/review;produção/testes intactos;sem Rust remoto no novoSHA. CI final/nota113/liberação pendentes;UI/IPC, cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff FIN-030.


REC-001 VERIFY E3:FixedRecurrence pura/nome obrigatório/valorpositivo/dia1..31/erroexistente/semDebugDisplay;RED/GREEN4unit+2doctestsprivacidade. Rust221+10docs/quality/security/LLVMdomain79/79 e10/10 com controle/review PASS;domínioanteriorpreservado. CIs pendentes;REC-002..015 (variável,estados,calendário de meses curtos,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-001.


REC-001 done/E3/passestrue após CI39193231687/SHAd52b3d3a6a8342867cc85dba78221d4becdea51asuccess23/0skip/7marcadores,Rust221+10docs/4unit+2doctestsnovos. FixedRecurrencepura/gates/review/validators/stagedscanPASS;CI final/nota114/liberação pendentes;REC-002..015 (variável,estados,calendário de meses curtos,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-001.


REC-002 VERIFY E3:VariableRecurrence pura/semvalor/nomeobrigatório/dia1..31/erroexistente/semDebugDisplay;RED/GREEN3unit+3doctests. Rust224+13docs/quality/security/LLVMdomain84/84 e11/11 com controle/review PASS;FixedRecurrence e domínio anterior preservados. CIs pendentes;REC-003..015 (estados,valorporocorrência,calendário de meses curtos,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-002.


REC-002 done/E3/passestrue após CI39195265289/SHAbb69d0f8c584a5e766b07ffa5faefe1ccf5916aasuccess23/0skip/7marcadores,Rust224+13docs/3unit+3doctestsnovos. VariableRecurrencepura/gates/review/validators/stagedscanPASS;CI final/nota115/liberação pendentes;REC-003..015 (estados,valorporocorrência,calendário de meses curtos,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-002.


REC-003 VERIFY E3:AwaitingValue inicial de VariableRecurrence validada por método real,semvalor/efeitos;RED/GREEN3unit+1doc. Rust227+14docs/quality/security/LLVMdomain88/88 e12/12 com controle porfunções/review PASS;domínioanteriorpreservado. CIs pendentes;REC-004..015 (demais estados,valorporocorrência,calendário,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-003.


REC-003 done/E3/passestrue após CI39197268222/SHA3aa07d06f19515688d5a09c1f21de12b1d5cac93success23/0skip/7marcadores,Rust227+14docs/3unit+1doctestnovo. AwaitingValuevalidado/gates/review/validators/stagedscanPASS;CI final/nota116/liberação pendentes;REC-004..015 (demais estados,valorporocorrência,calendário,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-003.


REC-004 VERIFY E3:Payablepositivo/Statevalidate/Fixedinitialreal/Awaitingpreservado/semefeitos;RED/GREEN4unit+2docs. P2doctestomitidorestaurado/verificado;Rust231+16docs/quality/security/LLVMdomain102/102 e14/14comcontrole/review PASS. CIs pendentes;REC-005..015 (demais estados,valorporocorrência,calendário,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-004.


REC-004 done/E3/passestrue após CI39199353402/SHAfd2787d91af0082b7fcb0628ecabb00cdece0ac9success23/0skip/7marcadores,Rust231+16docs/4unit+2doctestsnovos. Payablevalidado/gates/review/validators/stagedscanPASS;CI final/nota117/liberação pendentes;REC-005..015 (demais estados,valorporocorrência,calendário,geração,persistência,pagamento),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-004.


REC-005 VERIFY E3:Paid e mark_paid puros,apenasPayable+Exitválida devalorigual/IDnãonulo;domínio/docs anteriores preservados. RED/GREEN4unit,Rust235+16docs/quality/security/LLVMdomain123/123 e15/15 comcontrole/review PASS. CIs pendentes;REC-006..015 (valor por ocorrência,calendário,geração,persistência,pagamento transacional,conta/contexto,saída única e idempotência persistida),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-005.


REC-005 done/E3/passestrue após CI39201697719/SHA1656a218b2bc0f74735cafc7e56bb9d8bacab5c8success23/0skip/7marcadores,Rust235+16docs/4unitnovos. Paid/mark_paid puros/gates/review/validators/stagedscanPASS;CI final/nota118/liberação pendentes;REC-006..015 (valor por ocorrência,calendário,geração,persistência,pagamento transacional,conta/contexto,saída única e idempotência persistida),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-005.


REC-006 VERIFY E3:Overduepositivo/mark_overdue puro comClockinjetado e now>due_at;mark_paidreal aceita vencido. RED/GREEN4unit/Rust239+16docs/quality/security/LLVMdomain142/142 e16/16 comcontrole/review PASS. Fixture1ns corrigida100ns após probeWindows,originais/falha preservados,produçãointacta. CIs pendentes;REC-007..015 (calendário,fuso,geração,valor por ocorrência,persistência,pagamento transacional,conta/contexto,saída única e idempotência persistida),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-006.


REC-006 done/E3/passestrue após CI39204073999/SHA5f145b619b43eb7213357235d12e8e680824f4c5success23/0skip/7marcadores,Rust239+16docs/4unitnovos. Overdue/Clockdeadline/mark_paid/gates/review/validators/stagedscanPASS;fixture100ns/probe/falhaoriginais registrados;CI final/nota119/liberação pendentes;REC-007..015 (calendário,fuso,geração,valor por ocorrência,persistência,pagamento transacional,conta/contexto,saída única e idempotência persistida),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-006.


REC-007 VERIFY E3:migration7recurrences/obligationsSTRICT,templates/estados/FK/UNIQUE/contextosnapshot;5SQLitevazio+upgrade+constraints+reopen+rollbacksegundaDDL/retry/2mutantesPASS. Rust244+16docs/quality/security/LLVMdatabase276/280 e25/25 comcontroleTODOSSQL/review PASS. CIs pendentes;REC-008..015 (calendário/fuso,geração,valor por ocorrência,pagamento transacional,Exit/valor/contexto correspondentes,saída única e idempotência do caso real),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-007.


REC-007 done/E3/passestrue após CI39206760143/SHA0717d00ae478ffc2dedee6d9ad24f923084d01c8success23/0skip/7marcadores,Rust244+16docs/5SQLnovos. Migration7/constraints/upgrade/rollback/retry/2mutantes/gates/review/validators/stagedscanPASS;CI final/nota120/liberação pendentes;REC-008..015 (calendário/fuso,geração,valor por ocorrência,pagamento transacional,Exit/valor/contexto correspondentes,saída única e idempotência do caso real),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-007.


REC-008 VERIFY E3:geraçãofixaPayable/calendárioBRTUTC-3fixo(defaultintegrador)+configobligationauditSQLatômicos/sem movimentos. RED3stages/GREEN4domain6app5SQL+2docs;Rust259+18docs/quality/security/LLVM3críticos100% comcontrole/review PASS. CorreçõesDATA locais efixmecânicoClippy registrados/originais preservados. CIs pendentes;REC-009..015 (variável,valor por ocorrência,vencer,pagamento transacional,saída única e idempotência do caso real),política de fuso fixa sujeita a resposta humana,UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes. Fonte:handoff REC-008.


REC-008 done/E3/passestrue após CI39212810372/SHA128cd0ab765b87786bb38945d8f7c6a081b81733success23/0skip/7marcadores,Rust259+18docs/10unit5SQL2docsnovos. Geraçãofixa/calendárioBRTfixo/configobligationauditTx/rollback+poison/gates/review/validators/stagedscanPASS;CI final/nota121/liberação pendentes;REC-009..015 (variável,valor por ocorrência,vencer,pagamento transacional,saída única e idempotência do caso real),política de fuso fixa sujeita a resposta humana,UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;MetaE2Eativa;fonte:handoff REC-008.


REC-009 VERIFY E3:geração variável AwaitingValue/NULL e auditoria em Tx compartilhada com Fixed,sem movimentos;9unit+5SQL novos,Fixed regressões273+18docs/gates/security/LLVM3críticos100% com controle negativo/review PASS. CI fonte/final pendentes;REC-010..015 (informar valor por ocorrência,pagamento transacional,saída única e idempotência de sucesso),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;calendário reaproveita default do integrador último dia/fim do dia UTC-3 fixo,sem IANA/DST histórico/aceite humano. Fonte:handoff REC-009.


REC-009 done/E3/passestrue após CI39220204263/SHA5504bae8b2938761f0b245fe330be7e303af504bsuccess23/0skip/7marcadores,Rust273+18docs/9unit+5SQLnovos. Geração variável AwaitingValue/NULL/Tx compartilhada/Fixed regressões/rollback+poison/gates/review/validators/stagedscan PASS;CI final/nota122/liberação pendentes;REC-010..015 (informar valor por ocorrência,pagamento transacional,saída única e idempotência de sucesso),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;calendário reaproveita default do integrador último dia/fim do dia UTC-3 fixo,sem IANA/DST histórico/aceite humano;MetaE2Eativa;fonte:handoff REC-009.


REC-010 VERIFY E3:informar valor em uma ocorrência AwaitingValue→Payable positivo/metadata/templateNULL preservados/únicoUPDATEguardado+auditTx/rollback+poison/sem movimentos;9unit+5SQLnovos287+18docs/gates/security/LLVM3críticos100% comcontrole negativo/reviewP2corrigido PASS. Correçõesprodução integrador registradas;CIs pendentes;REC-011..015 (vencimento persistido,pagamento transacional,saída única e idempotência de sucesso),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;não há edição de valor já informado neste item. Fonte:handoff REC-010.


REC-010 | CORRECT/VERIFY staged | E3 | Publicação abortou antescommit emdiffcheck:espaçosfimdelinha dentrostringSQL. P-INT removeu somente whitespace,sequênciatokens de todoarquivo idêntica/únicoUPDATEguardado/params intactos. Originais10relatórios/hashes preservados emoperacao/REC-010-before-staged-correction-20261019 além33focais. Rustfmt/clippy/debug/release eLLVM refeitos apóstrimPASS,287+18docs/9-36-67linhas100% comcontrole0;7outroschecks anteriores válidos semrerun. Perfis anteriores preservados eLLVM nova pastaafter-format;nenhumthresholdalterado. E2P-REV anterior/P2resolvido,semnovachamada porwhitespace. CIfonte/final/nota123/liberação pendentes;MetaE2Eativa.


REC-010 done/E3/passestrue após CI39231570957/SHA2e7d8fb14a8ca4fde22b54d817a3e71439f68095success23/0skip/7marcadores,Rust287+18docs/9unit+5SQLnovos. Informar valor de uma ocorrência/Payable/únicoUPDATEguardado+audit/rollback+poison/gates/LLVM/reviewP2corrigido/validators/stagedscanPASS;CI final/nota123/liberação pendentes;REC-011..015 (vencimento persistido,pagamento transacional,saída única e idempotência de sucesso),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;não há edição de valor já informado neste item;MetaE2Eativa;fonte:handoff REC-010.


REC-011 VERIFY E3:Payable→Overdue estritamente apósClock/100ns/epochi128ano1,appClock1ID1Store/portaTimeexata/UPDATEguardado+auditTx/rollbackpoison/semalterarvalores;10unit+5SQLnovos302+18docs/gates/security/LLVM3críticos100% comcontrole negativo/review PASS. Rootfixtures/importsfmtcorreçõesregistradas;decisãohumanacalendárioemBUSINESS_RULES;CIs pendentes;REC-012..015 (pagamento transacional,saída única e idempotência),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;implementação de calendário atual usaUTC-3fixo,semprovaIANA/DSThistórico,oaceitehumanofoiparapolíticaúltimodia/fimdiaBrasília. Fonte:handoff REC-011.


REC-011 done/E3/passestrue apósCI39241758403/SHAc9db2729c3812feddacd2dabd8e82032432c3716success23/0skip/7marcadores,Rust302+18docs/10unit+5SQLnovos. Vencerobrigação/Clock100ns/epochi128ano1/guardasSQL/auditTx/rollbackpoison/gates/LLVM/review/validators/stagedscanPASS;decisãohumanacalendárioemBUSINESS_RULES;CI final/nota124/liberação pendentes;REC-012..015 (pagamento transacional,saída única e idempotência),UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;implementação de calendário atual usaUTC-3fixo,semprovaIANA/DSThistórico,oaceitehumanofoiparapolíticaúltimodia/fimdiaBrasília;MetaE2Eativa;fonte:handoff REC-011.


REC-012 VERIFY E3:pagarPayable/Overdue→Exit+Paidlink+auditnumaTx,centavos/contexto preservados/2IDs1Clock1Store/guardas/rollbackpoison/semsegundaExit;6unit+5SQLnovos313+18docs/gates/security/LLVM2críticos100% comcontrole negativo/review PASS. CorreçõesexecutáveisROOT emAPParm/preIOvalidationsregistradas/SQLlocalintacto;CIs pendentes;REC-013..015 exigem provas próprias desaídaúnica/rollback/concorrência/repetição,idempotência de sucesso nãoprometida;UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;calendárioaprovadoúltimodia/fimdiaBrasília registrado,implementaçãoUTC-3fixosemprovaIANA/DSThistórico. Fonte:handoff REC-012.


REC-012 done/E3/passestrue apósCI39248473269/SHAb4ee66f594ef2c723df3f595c7d14aa35d82cee3success23/0skip/7marcadores,Rust313+18docs/6unit+5SQLnovos. PagarrecorrênciaExit+Paidlink+auditTx/guardas/rollbackpoison/replaybásico/gates/LLVM/review/validators/stagedscanPASS;correçõesexecutáveisROOTregistradas;CI final/nota125/liberação pendentes;REC-013..015 exigem provas próprias desaídaúnica/rollback/concorrência/repetição,idempotência de sucesso nãoprometida;UI/IPC,cobertura Rust global atual e release Windows com instalação/atualização/backup-restauração/hash/assinatura/E2E pendentes;calendárioaprovadoúltimodia/fimdiaBrasília registrado,implementaçãoUTC-3fixosemprovaIANA/DSThistórico;MetaE2Eativa;fonte:handoff REC-012.


REC-013 VERIFY E3:1SQLnovo/8combinações16pagamentos/umaExit por obrigação em2meses/saldo eresultado exatos/reopen/histórico incremental preservado. Review1P2 corrigido e3mutantes finais rejeitados/GREEN6SQL/Rust314+18docs/gates/security PASS; produção/migrations imutáveis, coverageREC012 histórica referenciada sem nova medição. CIs/nota126/liberação pendentes;MetaE2Eativa;REC014/015/UIIPC/globalRust/instalador/E2Ependentes;fonte:handoff REC-013.


REC-013 done/E3/passestrue apósCI39252069599/SHA1434f282bd06445994baca20333207888ac030fbsuccess23/0skip/7marcadores,Rust314+18docs/1SQLnovo. UmaExit por obrigação/2meses/8combinações16pagamentos/saldo e resultado reais/reopen/históricoincremental/3mutantes/reviewP2corrigido/gatesPASS;produção e7migrations intactas,coveragehistórica referenciada;CI final/nota126/liberação pendentes;REC-014/015 ainda exigem provas próprias de rollback completo/concorrência/repetição; sucesso idempotente não prometido. UI/IPC, cobertura Rust global atual, instalador Windows/instalação/atualização/backup-restauração/hash/assinatura e E2E oficial pendentes. Política mensal último dia/fimdiaBrasília aceita e registrada; implementação UTC-3 fixa sem prova IANA/DST histórica;MetaE2Eativa;fonte:handoff REC-013.


REC-014 VERIFY E3:1SQLnovo/64cenários físicos8combinações×8falhasinclusiveCOMMITdeferred/snapshot+reopen/projeções reais preservadas/poison+freshválido;2mutantes parciais rejeitados/GREEN7SQL/Rust315+18docs/gates/security/reviewPASS. Produção/7migrations intactas;coverageREC012histórica referenciada. CIs/nota127/liberação pendentes;REC015/UIIPC/globalRust/instalador/E2Ependentes;Metaativa;fonte:handoffREC-014.


REC-014 done/E3/passestrue apósCI39255062132/SHA921465015075a33f16368761f39bcf53c1a56c1fsuccess23/0skip/7marcadores,Rust315+18docs/1SQLnovo. Rollbackpagamento64cenários/snapshot+reopen/leitores+projeções/poison+freshválido/FKdeferredCOMMIT/2mutantes/review/gatesPASS;produção e7migrations intactas,coveragehistórica referenciada;CI final/nota127/liberação pendentes;REC-015 ainda exige provas próprias de concorrência/repetição;sem retry automático/sucesso idempotente prometido. Não prova powerloss/WAL/panic-abort/falha física deROLLBACK/recuperação global. UI/IPC,cobertura Rust global atual e instalador Windows/instalação/atualização/backup-restauração/hash/assinatura/E2E oficial pendentes. Política mensal último dia/fimdiaBrasília aceita;implementaçãoUTC-3fixa sem provaIANA/DSThistórica;Metaativa;fonte:handoffREC-014.


REC-015 VERIFY E3:2SQLnovos/8seriais+8corridas/Paid0UUID/staleUUIDnovoereusadosemsegundaExit/1vencedor+loserpoison/outramêspagável/PaidFK-audit-payload8col/reopen/histórico/projeções exatos;3REDs atuais verificados/GREEN9SQL/P2corrigidopeloroot/Rust317+18docs/gates/security/reviewPASS. Parsercorrigidosemrerunválido,rawpreservado;skillsadaptadas aplicadas. Produção/7migrations intactas,coveragehistórica referenciada;CIs/nota128/liberação pendentes;UIIPC/globalRust/instalador/E2Ependentes;Metaativa;fonte:handoffREC-015.


REC-015 done/E3/passestrue apósCI39261345263/SHAdfc4c405366cce24ee28be8db8bc147d1f3d4652success23/0skip/7marcadores,Rust317+18docs/2SQLnovos. Repetição8seriais+8corridas/Paid0UUID/staleguard/1vencedor/loserpoison/outramêspagável/payload8col-audit-PaidFK/reopen/históricoprojeções/3REDs/gatesPASS;P2darevisãocorrigidopeloroot/0pendentes;parserreclassificadosemrerunválido eoriginais preservados,skillsadaptadasaplicadas;produção e7migrations intactas,coveragehistórica referenciada;CI final/nota128/liberação pendentes;Repetição retorna erro,sem sucesso idempotente anterior/chave deintenção/retryautomático. Perdedor concorrente pode falhar noBEGINIMMEDIATE ou guarda apóscommit;guarda éprovada independentemente na sequência serial comUUIDsnovos. Não prova vencedor fixo/processos separados/distribuição/powerloss/WAL/panic-abort/falha físicaROLLBACK/recuperação global. UI/IPC,coberturaRustglobal e instaladorWindows/instalação/atualização/backuprestauração/hashassinatura/E2E oficial pendentes. Política mensal último dia/fimdiaBrasília aceita;implementaçãoUTC-3fixa semprovaIANA/DSThistórica;Metaativa;fonte:handoffREC-015.


PRD-001 VERIFY E3:MVP07 representação Product id/nome/descrição/preçosugerido/ativo;1unit+2compile_fail/RED/GREEN/Rust318+20docs/gates/security/reviewPASS,domínioanterior e7migrations preservados. LLVMdomain>=95 com controle;tipo sem linhas executáveis/sem percentual próprio. Validações/CRUD/SQLite/UI e INV-SAL005 pendentes. CIs/nota129/liberação pendentes;MetaE2Eativa;fonte:handoff PRD-001.


PRD-001 done/E3/passestrue apósCI39271100208/SHAc092545f872849245405f65b4529e68e3d5ec60dsuccess23/0skip/7marcadores,Rust318+20docs/1unit+2doctestsnovos. Product template puro/RED/GREEN/LLVMdomaincomcontrole/gates/security/reviewPASS;domínio anterior e7migrations preservados;CIfinal/nota129/liberação pendentes;PRD-002..007 (validações,migration,criar,alterar,desativar),vendas/snapshots e INV-SAL-005 ainda pendentes. Representação não é aceite de entrada inválida por operação. UI/IPC,cobertura Rust global e release Windows/instalação/atualização/backup-restauração/hashassinatura/E2E oficial pendentes. Product tem zero linhas executáveis;percentual próprio não aplicável;stable branches não medidos;sem coverage global ou remotoRust.;Metaativa;fonte:handoff PRD-001.


PRD-002 VERIFY E3:MVP07 Product.validate_name puro/nome obrigatório conforme trim/erro existente/sem normalizarcampos nem acoplarpreçoativo;2unitnovos+PRD001/doctests preservados/RED/GREEN3/Rust320+20docs/gates/security/reviewPASS. LLVMdomain>=95 com controle sem2novos rejeitado;domínio anterior e7migrations intactos. CIs/nota130/liberação pendentes;MetaE2Eativa;fonte:handoff PRD-002.


PRD-002 done/E3/passestrue apósCI39277622649/SHAeda023c04e073daa00f981c6210bba554398247bsuccess23/0skip/7marcadores,Rust320+20docs/2unitnovos. Product.validate_name puro/Unicodeblank/semnormalização-preçoativo/RED/GREEN3/LLVMdomaincomcontrole/gates/security/reviewPASS;domínio anterior e7migrations preservados;CIfinal/nota130/liberação pendentes;PRD-003..007 (preço,migration,criar,alterar,desativar),vendas/snapshots e INV-SAL-005 pendentes. validate_name não valida produto inteiro nem autoriza operação financeira;whitespace segue str::trim,não é sanitização geral. UI/IPC,cobertura Rust global e release Windows/instalação/atualização/backup-restauração/hashassinatura/E2E oficial pendentes;stable branches/statements/global/CI Rustcoverage não medidos.;Metaativa;fonte:handoff PRD-002.


PRD-003 VERIFY E3:MVP07 preço sugerido não negativo/zero válido conforme humano19out;validate_suggested_price puro/2unitnovos/RED/GREEN5/Rust322+20docs/gates/security/reviewPASS. LLVMdomain>=95controle omitindo5Productrejeitado;domínio anterior e7migrations intactos. CIs/nota131/liberação pendentes;Metaativa;fonte:handoffPRD-003.


PRD-003 done/E3/passestrue apósCI39288285719/SHAc8c11fb388b7ea7d4ab86768fb947842f53fc91asuccess23/0skip/7marcadores,Rust322+20docs/2unitnovos. Preço zero válido/negativosrejeitados/validaçãopura/RED/GREEN5/LLVMdomaincontrole5Product/gates/security/reviewPASS;domínio anterior e7migrations intactos;CIfinal/nota131/liberação pendentes;PRD-004..007(migration/criar/alterar/desativar),vendas/snapshots/INV-SAL005,UI/IPC,cobertura Rustglobal,releaseWindows/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. Stablebranches/statements/global/CI Rustcoverage não medidos;validação focal não valida produto inteiro.;Metaativa;fonte:handoffPRD-003.


PRD-004 VERIFY E3:MVP07 migration8ProductSTRICT5campos/zero válido/negativosrejeitados/2constantesRustzeroexec/5SQLitefísicosREDGREEN+2mutantes/gates/security/reviewPASS. LLVMdatabase>=95controle semTODOSSQLite rejeitado;6SQLanteriores+v1vazia/executores/domíniointactos. CIs/nota132/liberaçãopendentes;Metaativa;fonte:handoffPRD-004.


PRD-004 done/E3/passestrue apósCI39292809030/SHAc93e947e0a1b4d2fdea9793aa02a25276022fef5success23/0skip/7marcadores,Rust327+20docs/5SQLnovos. Migration8productsSTRICT/zero válido/2constantesRustzeroexec/REDGREEN+2mutantes/LLVMdatabasecomcontrole/gates/security/reviewPASS;executores/domínio/6SQLanteriores+v1vaziaintactos;CIfinal/nota132/liberaçãopendentes;PRD005..007(criar/alterar/desativar),vendas/snapshots/INVSAL005,UIIPC,coberturaRustglobal,releaseWindows/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. SQLname<>vazio não replicaUnicodeblank do domínio. STRICT permite coerções sem perda;sem percentualSQL/globalRust/CI/statements/branchesstable.;Metaativa;fonte:handoffPRD-004.


PRD-005 VERIFY E3:MVP07 criaçãoProductativo/validaçãonomepreçoantesportas/SQL+auditIMMEDIATEparametrizado/rows1/poisoner-freshrecovery/7unit4SQLnovosREDGREEN/gates/security/reviewPASS. LLVM2arquivos>=95controleomitenovosrejeitado;7SQLmigrationsanteriores+v1vazia/domíniointactos. CIs/nota133/liberaçãopendentes;Metaativa;fonte:handoffPRD-005.


PRD-005 done/E3/passestrue apósCI39298191906/SHA0ecf5a59905791a2eae452716c0b8c2220207246success23/0skip/7marcadores,Rust338+20docs/7unit4SQLnovos. CriaçãoProductativo/validaçõesantesportas/SQL+auditIMMEDIATE/rows1/poisoner-freshrecovery/REDGREEN/LLVM2arquivoscomcontrole/gates/security/reviewPASS;7migrationsSQL+v1vazia/domainintactos;CIfinal/nota133/liberaçãopendentes;PRD006/007(editar/desativar),vendas/snapshots/INVSAL005,UIIPC,coberturaRustglobal,releaseWindows/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. UUIDversão/uniqueness pertencem ao adapterUuidGenerator;zeroaudit/entity/timeinválidos são rejeitados porappend_audit_event e revertidosSQL. Stablebranches/statements/global/CI Rustcoverage não medidos.;Metaativa;fonte:handoffPRD-005.


PRD-006 in_progress/E3 local/passesfalse em23/10/2026: edição Product,9unit+5SQL reais,352Rust+20docs e coverage dos2arquivos100%/controle rejeitado. Vite8.3.3/Tokio1.53.3 corrigem achados novos. Gates Cargo/OSV aguardam aceite exato da proposta c86bf6753ee0bb8d579e74e699e4bab108222835; CIs/nota134/liberação pendentes. Meta ativa; nenhum próximo item iniciado. Fonte: handoffs/PRD-006.md.


PRD-006 VERIFY E3 local aprovado23out: ediçãoProduct/9unit5SQL/352Rust+20docs/LLVM2arquivos100%controle rejeitado/gates/security/reviewPASS. Vite8.3.3/Tokio1.53.3;E4somente2exceçõesWindows/hashescorrentes/antes06novUTC conforme aceite da proposta c86bf6753ee0bb8d579e74e699e4bab108222835. ProduçãoGPTfallback0saídaslocaisusadas;2P2docs corrigidos. CIs/nota134/liberação pendentes/passesfalse;INVSAL005SAL008/E2Ependentes;fonte:handoffPRD-006.


PRD-006 done/E3/passestrue apósCI39998764928/SHA46ddaf405b7dc4a4ab7b7b0683d87a6fd47d9c0esuccess23/0skip/7marcadores,Rust352+20docs/9unit5SQLnovos. EdiçãoProduct/validaçõesantesportas/IDativopreservados/UPDATE+auditIMMEDIATE/rows1/NotFound/poisoner-freshrecovery/REDGREEN/LLVMapp+adaptercomcontrole/gates/security/reviewPASS;PRD005/migrations/domainintactos;CIfinal/nota134/liberaçãopendentes;PRD007(desativar),vendas/snapshotsreais/INVSAL005provaSAL008,UIIPC,coberturaRustglobal,releaseWindows/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. Metadataeditpreservaativo inclusiveinativo,padrãoCategoryrename;nenhumasnapshotfictíciausada. Stablebranches/statements/global/CI Rustcoverage não medidos.;Metaativa;fonte:handoffPRD-006.


PRD-007 VERIFY E3: desativação Product/7 unit+5 SQL/364 Rust+20 doctests/LLVM dois arquivos 100% e controle rejeitado/gates de qualidade e segurança PASS; P-REV E2 sem achados. Uma inferência local P-DAT/121,752s usada sem correção de comportamento; app por reuso determinístico de Category. CIs/nota135/liberação pendentes; passesfalse; E2E oficial pendente; fonte: handoff PRD-007.


PRD-007 done/E3/passestrue após CI 40004684967/SHA b861d473dc1b4fb26771ea7d600923a5d1b397fc, success23/0 skip/7 marcadores; Rust364+20 docs/7 unit+5 SQL novos. Desativação preserva metadados, aceita já inativo com nova auditoria; UPDATE+audit IMMEDIATE/rows1/NotFound/rollback/poisoner/fresh recovery; criação/edição anteriores preservadas. RED/GREEN/LLVM app+adapter com controle/gates/security/review PASS. CI final/nota135/liberação pendentes; Meta ativa. Snapshots reais de venda/INV-SAL-005 dependem de SAL-008; jornadas UI/IPC, cobertura Rust global e instalador Windows com instalação/atualização/backup/restauração/hash/assinatura/E2E oficial pendentes. LLVM local mede somente linhas/funções dos dois arquivos inteiros; branches/statements/global/CI Rust coverage não medidos. Fonte: handoff PRD-007.


CUS-001 VERIFY E3: Customer mínimo puro id/nome/telefone opcional, dados sintéticos exatos/sem Debug ou Display; 1 unit+2 compile_fail novos, Rust365+22docs/gates/security/P-REV PASS. LLVM domain95 com controle, tipo sem percentual próprio. CUS002..005/UI/IPC/E2E pendentes; CI/nota136/liberação pendentes; passesfalse/Meta ativa; fonte: handoff CUS-001.


CUS-001 done/E3/passestrue após CI 40013227015/SHA d498cc325cbc5c7af2957b232419b2d70ea4bf15, success23/0skip/7 marcadores, Rust365+22docs/1unit+2doctests novos. Customer mínimo puro id/nome/telefone opcional/sem Debug ou Display; RED/GREEN/LLVM domain com controle/gates/security/review PASS. Domínio anterior/migrations/deps/exceções preservados; CI final/nota136/liberação pendentes; Meta ativa. Representação não decide aceite de entrada: CUS-002/003 validam nome/telefone e CUS-004/005 implementam persistência/criação. Formato/tamanho/normalização/unicidade do telefone não decididos aqui. Vendas/INV-SAL-002, UI/IPC, cobertura Rust global e instalador/instalação/atualização/backup/restauração/hash/assinatura/E2E oficial pendentes. Customer tem zero linhas executáveis; percentual próprio não aplicável. LLVM local cobre somente domain/mod.rs inteiro; branches/statements/global/CI Rust coverage não medidos. Fonte: handoff CUS-001.


CUS-002 VERIFY E3: Customer.validate_name por reuso literal Product, vazio/whitespace inválidos, conteúdo bruto preservado/telefone independente. RED/GREEN3Customer/2unit novos/Rust367+22docs/LLVMdomain95 e controle/gates/security/P-REV PASS.0inferências/runtime não iniciado. CUS003..005/UI/IPC/E2E pendentes; CI/nota137/liberação pendentes; passesfalse/Meta ativa; fonte: handoff CUS-002.


CUS-002 done/E3/passestrue após CI 40015925340/SHA 6c17d78f192d91623107470e8403151b20b03529,success23/0skip/7 marcadores,Rust367+22docs/2unit novos. Customer.validate_name puro por reuso Product, blank rejeitado/texto bruto e campos preservados/telefone independente. RED/GREEN/LLVM domain com controle/gates/security/review PASS.0inferências; domínio anterior/migrations/deps/exceções preservados. CI final/nota137/liberação pendentes; Meta ativa. Não valida Customer inteiro nem formato/tamanho/normalização/unicidade de telefone. CUS-003/004/005, vendas/INV-SAL-002, UI/IPC, Rust global, instalador/instalação/atualização/backup/restauração/hash/assinatura/E2E oficial pendentes. LLVM local cobre domain/mod.rs inteiro; controle sem domínio prova instrumentação, não causalidade isolada do método. Percentual isolado/branches/statements/global/CI Rust coverage não medidos. Fonte: handoff CUS-002.


CUS-003 VERIFY E3: None válido/Some máscara ASCII(DD)9XXXX-XXXX/DDD67/inicial9, sem mutação/nome/IO. Regra humana no produto/fontes Anatel preservadas. RED/GREEN7Customer/4unitnovos/Rust371+22docs/LLVMdomain95controle/gates/security/P-REV PASS. Laguna timeout603,471s/0output usado; P-INT fallback; runtime encerrado. CUS004005/UI/IPC/E2E pendentes; CI/nota138/liberação pendentes; passesfalse/Metaativa; fonte: handoff CUS-003.


CUS-003 done/E3/passestrue apósCI40048716627/SHAb94b80800dc6ad82e9be36da5271b4d34b8000c8,success23/0skip/7marcadores,Rust371+22docs/4unitnovos. Customer.validate_phone opcional, formato exato/DDD67/primeiro9/ASCII/sem mutação/panic/nome; regra humana registrada/fontes Anatel. RED/GREEN/LLVMdomaincontrole/gates/security/review PASS; métodos anteriores/migrations/deps/exceções preservados. CIfinal/nota138/liberação pendentes; Meta ativa. Validação sintática e de DDD; não confirma existência, atividade, titularidade ou WhatsApp. Some vazio ou fora da máscara é inválido; telefone ausente permanece opcional. CUS004005/persistência/UI/IPC/vendas/INV-SAL-002/Rustglobal/instalador/instalação/atualização/backup/restauração/hashassinatura/E2Eoficial pendentes. LLVM local somente domain/mod.rs inteiro; controle sem domínio prova instrumentação, não causalidade isolada; sem percentual isolado/branches/statements/global/CI Rustcoverage. Fonte: handoff CUS-003.


CUS-004 VERIFY E3: migration009customersSTRICT/ID16/nome/phoneNULLoumaskASCII15DDD67,duasconstantes9/zerofunções. RED/GREEN5SQLite/vazio-upgrade8histórico-rollback-colisão/2mutantesrejeitados/Rust376+22docs/LLVMdatabase95controleTODOSSQL/gates/security/P-REV PASS;7SQLanterioresintactos.0inferências/runtime não iniciado. CUS005/UIIPC/globalRust/instalador/E2Ependentes;CI/nota139/liberação pendentes;passesfalse/Metaativa;fonte:handoff CUS-004.


CUS-004 done/E3/passestrue apósCI40051436747/SHAc37983da2d3c588807bd0e1f6b312471455ae680,success23/0skip/7marcadores,Rust376+22docs/5SQLnovos. Migration009customersSTRICT/phoneopcionalmaskDDDaprovados/2constv9/zerofunções;vazio/upgrade8histórico/rollback9/colisão/constraints/mutantes/LLVMdatabasecontrole/gates/security/review PASS;7SQLanterioresintactos. CIfinal/nota139/liberação pendentes;Meta ativa. SQLname<>empty segue Product/NULsafe, não implementa trimUnicode do domínioCUS002. Phoneassegura sintaxe/DDD, não linha ativa ou titularidade. Constantes/SQLsempercentualexecutável próprio; LLVMdatabase inteiro mede executores existentes, controle sem TODOSSQLite prova instrumentação. SemRustglobal/CI/SQLpercent/statements/branchesstable. CUS005/criação/clienteemvendapendente/UIIPC/vendas/instalador/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. Nove versões significam oitoSQLfísicos maisv1vazia. Fonte:handoff CUS-004.


CUS-005 VERIFY E3: criarCustomer name/phoneNULLSomebrutos/validarantesportas/2IDsClockchecked/INSERT+customer_createauditIMMEDIATE/rows1/invaliddiretoreusável/rollback-poisoned-freshrecovery. RED/GREEN7unit4SQL/Rust387+22docs/LLVM2arquivos95controlegates/security/P-REV PASS.8SQLanterioresintactos;0inferências/runtime. CIs/nota140/liberação pendentes;MVP09UI/clienteemvendapendente/SMK005/globalRust/instalador/E2Ependentes;passesfalse/Metaativa;fonte:handoff CUS-005.


CUS-005 done/E3/passestrue apósCI40054116789/SHAa3020fb565cc0125d7d55f771e7df2d885648d74,success23/0skip/7marcadores,Rust387+22docs/7unit4SQLnovos. CreateCustomer validaçõesantesportas/camposNoneSome/2IDsClockchecked/INSERT+auditIMMEDIATE/rows1/rollback/poisoned-freshrecovery/opacos;REDGREEN/LLVM2arquivoscontrole/gates/security/review PASS.8SQL/domínio/produtoanterioresintactos;0inferências. CIfinal/nota140/liberação pendentes;Meta ativa. Não criaUI/IPC/listagem/edição/desativaçãodecliente/CRM/estoque/venda. ClienteemvendapendenteINV-SAL-002/jornadaMVP09/UI009/SMK005, coberturaRustglobal,instalador/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. UUIDversão/unicidade pertence ao fornecedor;MAXms/overflow testados no helper,semalegarSystemTimeMAXdoplatforma. LLVMlocal sódoisarquivosinteiros comcontroleomitenovos7unit4SQL;sembranches/statements/global/CI Rustcoverage. Custo não registrado. Fonte:handoff CUS-005.


SAL-001 VERIFY E3: SaleItem puro7campos/quantidadeinteira/centavos/valorespróprios; decisão humana R$ ou percentual registrada. RED/GREEN1unit2compile_fail/gates/security/P-REV PASS;Rust388+24docs/LLVMdomain95controle. Validação/cálculo/SQL/imutabilidade operacional/UI/IPC/E2E pendentes;CI/nota141/liberação pendentes;passesfalse/Metaativa;fonte:handoff SAL-001.


SAL-001 done/E3/passestrue apósCI40147508621/SHA77468ca1aabd84daa88768fd1c03925be8683c6a,success23/0skip/7marcadores,Rust388+24docs/1unit2docsnovos. SaleItem7campos próprios/quantidadeinteira/centavos/semDebugDisplay;RED/GREEN/LLVMdomain95controle/gates/security/reviewPASS;domínio/migrations/deps/exceções preservados;CIfinal/nota141/liberação pendentes;Metaativa. Representação não prova validação, conversão percentual, cálculo, persistência ou imutabilidade operacional SAL-008/INV-SAL-005. SAL-002/003/004 e demais consumidores seguem seus IDs. Precisão percentual/arredondamento não definidos neste tipo puro. UI/IPC, cobertura Rust global e release Windows com instalação/atualização/artefato/hash/assinatura/backup/restauração/E2E oficial pendentes. SaleItem tem zero linhas executáveis e não recebe percentual próprio. LLVM local mede somente domain/mod.rs inteiro; branches/statements/global/CI Rust coverage não medidos. Fonte:handoff SAL-001.


SAL-002 VERIFY E3: validate_quantity inteiroi64>0, limitesMIN/-1/0 e1/2/MAX,7campos preservados;RED/GREEN3módulo2unitnovos/Rust390+24docs/LLVMdomain95controle/gates/security/P-REV PASS.0inferências. Monetários/cálculo/SQL/snapshot operacional/UI/IPC/E2E pendentes;CI/nota142/liberação pendentes;passesfalse/Metaativa;fonte:handoff SAL-002.


SAL-002 done/E3/passestrue apósCI40148872782/SHAa06c4c81037d92b184c7c5f2579824cea10dec43,success23/0skip/7marcadores,Rust390+24docs/2unitnovos. validate_quantity inteiroi64>0/limites/preservação7campos;RED/GREEN/LLVMdomain95controle/gates/security/reviewPASS;0inferências;domínio/migrations/deps/exceções preservados;CIfinal/nota142/liberação pendentes;Metaativa. Não valida item inteiro, campos monetários ou descrição nem calcula/converte desconto percentual. SAL-003/004 e demais consumidores, snapshot operacional SAL-008/INV-SAL-005, UI/IPC, Rust global, instalador Windows/instalação/atualização/artefato/hash/assinatura/backup/restauração/E2E oficial pendentes. LLVM local cobre domain/mod.rs inteiro; controle sem domínio prova instrumentação, não causalidade isolada do método. Percentual isolado/branches/statements/global/CI Rust coverage não medidos. Fonte:handoff SAL-002.


SAL-003 VERIFY E3: descontoR$ ou0..100%/2casas/meio centavo para cima;5unit17fixturesDecimal/10001pb×7brutos/2docsprivacidade. RED/GREEN5/Rust395+26docs/LLVMdomain95controle/gates/security/P-REV PASS.1Laguna130,6s/correção determinística/CLIrefresh/runtimeoff. Parser/cálculo líquido/SQL/snapshot operacional/UI/IPC/E2E pendentes;CI/nota143/liberação pendentes;passesfalse;objetivoE2E íntegro;Metaformalblocked;fonte:handoffSAL-003.


SAL-003 done/E3/passestrue apósCI40152206878/SHA6ab5cc31645be236d4988f486db0794007178945,success23/0skip/7marcadores,Rust395+26docs/5unit2docsnovos. validate_discount R$/%/limites/centavos/meio para cima;RED/GREEN/LLVMdomain95controle/gates/security/reviewPASS;1Laguna130,6s/correção/root/runtimeoff;domínio/migrations/deps/exceções preservados;CIfinal/nota143/liberação pendentes;objetivoE2E íntegro. Entrada textual commais2casas deve ser rejeitada no futuro consumidor; não há parser/UI implementado. Cálculo bruto/líquido e validação de snapshot SAL-004,criação/persistência/snapshot operacionalSAL-008/INV-SAL-005,UI/IPC,Rustglobal,releaseWindows instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. LLVM local mede domain/mod.rs inteiro;controle sem domínio prova instrumentação,não causalidade isolada;sem percentuais isolados/branches/statements/global/CI Rustcoverage. Meta formal observada blocked após resposta humana; decisão de produto resolvida e execução autorizada,sem declarar retomada formal inexistente. Fonte:handoffSAL-003.


SAL-004 VERIFY E3: calculate_total i128/líquido0..i64MAX/descontoaplicado1vez/7campospreservados;5unitnovos/GREEN8/Rust400+26docs/LLVMdomain95controle/gates/security/P-REV PASS. Testes24out reutilizados porigualdade hashes25out. LagunaPDOM117,032s/QwenPTRCconferido/runtimesoff. CI/nota144/liberação pendentes;passesfalse;uma tarefa manualsemretomarMeta;SAL005nãoiniciar;fonte:handoffSAL-004.


SAL-004 done/E3/passestrue apósCI40168424623/SHAc63029e20d31a263835b9f2e273d933df2895dcf,success23/0skip/7marcadores,Rust400+26docs/5unitnovos. Total i128/resultadolíquidoi64/desconto1vez/preservação7campos;RED/GREEN/LLVMdomain95controle/gates/security/reviewPASS;Laguna117,032s+Qwenconferido/runtimesoff;domínio/migrations/deps/exceções preservados;CIfinal/nota144/liberação pendentes;única tarefa manual/Metaformalblocked. Não cria nem persiste item/venda, não prova snapshot operacionalSAL008/INV-SAL-005,UI/IPC,coberturaRustglobal ou releaseWindows instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial. LLVMmede domain/mod.rs inteiro;controle semdomínio prova instrumentação,não causalidade isolada;sempercentuaisisolados/branches/statements/global/CI Rustcoverage. Fechamento manual de uma única tarefa autorizado25out,sem retomar Meta formalblocked;SAL-005 não iniciar. Fonte:handoffSAL-004.


SAL-005 VERIFY E3: factorylivre validado/productNone/descriçãopreservada,desconto/total corretos/helperwide/APIi64 compatível;8unitnovos/9Decimal100digits/REDGREEN/Rust408+26docs/LLVMdomain95controle/gates/security/P-REV PASS. Laguna603,293s timeout0diff/rootfallback/errodainferênciagenéricacorrigido/oldcallersintactos/runtimeoff. CI/nota145/liberação pendentes;passesfalse;Metaativa;INV-SAL006 completo eUIE2E pendentes;fonte:handoffSAL-005.


SAL-005 done/E3/passestrue apósCI40194173733/SHA1f9a943c230cbbe7409f56728449ef140321cd72,success23/0skip/7marcadores,Rust408+26docs/8unitnovos. Factorylivre/productNone/descriçãopreservada/desconto-totalvalidos/helperwideprivado/APIi64mantida;RED/GREEN/LLVMdomain95controle/gates/security/reviewPASS;Laguna603,293stimeout/rootfallback/correçãodecompatibilidade/oldtestsintactos/runtimeoff;domínio/migrations/deps/exceções preservados;CIfinal/nota145/liberação pendentes;Metaativa. Factory puro e product_idNone não provam criação/persistência de venda ou INV-SAL006 completo: SAL006/013 próprios;imutabilidade operacionalSAL008,UI/IPC,coberturaRustglobal,releaseWindows instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. Exigência de descrição-rótulo é escolha técnica explícita da spec por reuso do padrão de nomes,não decisão humana nova atribuída. LLVM local cobre domain/mod.rs inteiro;controle semdomínio prova instrumentação,não causalidade isolada;sem percentuaisisolados/branches/statements/global/CI Rustcoverage. Metaformal retomadaactive confirmada;manter atéE2E,sempre encerrar locais antespausa/interrupção. Fonte:handoffSAL-005.


SAL-006 VERIFY E3 parcial INV-SAL006/MVP08: catálogo/estado lógico preservados no factory público em2SQLite;controles reais COUNT/ROWS rejeitados,produtoNone;produção intacta. 410Rust+26docs/2SQLnovos/gates/security/LLVMdomain95controle/P-REV PASS;CI/nota146/liberação pendentes;passesfalse. Prova parcial INV-SAL006/MVP08 na fronteira factory/catálogo: factory puro não recebe store/conexão/caminho. Persistência de item na venda/ausência de efeitos no consumidor futuro continua SAL013/SMK005, não satisfeita aqui. UI/IPC, Rustglobal, instalador/instalação/atualização/artefato/hash/assinatura/backuprestauração/E2E oficial pendentes. LLVMdomain inteiro inalterado; controle0 confirma instrumentação, não causalidade isolada nem percentual dos novos testes/global/CI/branches/statements. Meta formal observada blocked, retomada depende controle humano do app; heartbeat autoriza este ID, sem reativação simulada. Custo não registrado. Fonte:handoff SAL-006.
