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


FIN-011 integração primária documental: dependência formal somente FIN-010. E3 focal foi executado na WT: RED por TryFrom ausente, depois full17 PASS (14/14 linhas, 3/3 funções, 19/19 regions); controle omitindo somente os três testes novos rejeita gate90. Review WT E2 sem P0/P1/P2. Focal retest, review primary, fmt/clippy/Cargo/doctests, audit/OSV, Gitleaks, validators e CI no SHA integrado continuam pendentes. Sem consumer ou contrato externo. Status in_progress/E3/passes=false; ACTIVE24 retida até gates, nota final e liberação root.
