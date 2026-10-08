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
