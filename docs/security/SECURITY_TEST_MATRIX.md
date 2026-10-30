# Matriz de segurança

TRC-006 mapeia oito IDs de [SECURITY_REQUIREMENTS](SECURITY_REQUIREMENTS.md), em ordem, sem aumentar a contagem. A coluna SEC-ID é **requisito**; a coluna executores referencia tarefas do [roadmap](../../planning/ROADMAP.md). IDs homônimos são namespaces distintos: requisitoSEC002 é SQL, tarefaSEC002 é CSP; requisitoSEC007 é gate de scans, tarefaSEC007 é Gitleaks no CI. Não presumir equivalência numérica.

**Planejado/E0 nos controles globais**: as linhas abaixo descrevem oráculos completos ainda pendentes; a prova parcial E3 do scaffold SEC-001 está registrada ao final. Todo nesta tabela não é estado do work itemTRC006. AuditE3/reviewE2 de documentos não provam controles executados. [Threat model](THREAT_MODEL.md), [capabilities](TAURI_CAPABILITIES_POLICY.md), [segredos](SECRET_MANAGEMENT.md), [dependências](DEPENDENCY_SECURITY.md), [ADR020](../architecture/adr/ADR-020-security-policy.md) e [ADR021](../architecture/adr/ADR-021-tauri-least-privilege.md) são fontes, sem criar permissões/algoritmos novos.

| SEC-ID (requisito) | ameaça/controle e fonte | executores (roadmap) | oráculos/testes/scans futuros | evidência/gate pendente | status |
|---|---|---|---|---|---|
| SEC-001 | capability excessiva; negar por padrão e declarar mínimo por janela/comando segundo [política](TAURI_CAPABILITIES_POLICY.md)/ADR021 | SEC-001, SEC-004, SEC-005; BKP-004; UI-017 | Inspeção config e teste core/WebView: comando/janela não autorizados negados sem efeito; operação necessária explicitamente permitida funciona com escopo mínimo. Diretório permitido tem caso positivo; externo/shell/rede sem autorização falham. Não basta desabilitar toda função ou inspecionar texto da config. | scaffold config/36negações/WebView2-guard E3 e aceite E4, detalhe abaixo; plugins/paths/positivos futuros não registrados | parcial no scaffold; cenários futuros pendentes |
| SEC-002 | injeção/corrupção; queries parametrizadas e constraints conforme [requisito](SECURITY_REQUIREMENTS.md)/[ameaças](THREAT_MODEL.md) | DB-001, DB-002, DB-003, DB-004, DB-005, DB-007, DB-009, DB-010; FIN-007, FIN-016; ARC-001 | Inspeção de bindings no repositório e integração SQLite real: apóstrofo/pontuação SQL sintéticos são dados, não instruções; conteúdo persistido/lido como literal e esquema/histórico íntegros. Constraints/foreign keys inválidas rejeitam sem estado parcial; migrations banco vazio/upgrade preservam contrato. DB-003 comprova atomicidade do script estático interno e estado após reopen; DB-004 valida sequência/estado coerente do ledger, não validação de SQL externo nem autenticidade. Validações/limites específicos aguardam specs. | query/bindings/schema/constraints/fixtures negativas-reabertura/comando-saída-SHA; DB-003 E3 entregue; DB-004 E3 entregue; CI final 38299901151 no SHA 94e37d38f5df375ffb9492e3f80d272f2f44b331 passou 16 success/7 skipped/0 artifact; nota 65 publicada, 64 notas e refs preservados. DB-005 executor done/E3: ledger vazio versionado em arquivo e rejeições PK/CHECK; CI entrega 38303443795 passou 23/23, release18/18, Gitleaks146/0, Cargo422/0classvuln/2aprovados, OSV542/3exceções/0restantes, npm120/0 e SAST45 (limites frontend; sem Rust/backend SQL/restore). Scan staged/validators da entrega passaram; CI documental final 38304515870/job114995197572 no SHA 1d92fdd185cf24aececd1fa5b5708d9d0efeab03 passou 16 etapas/7 skipped/0 artifacts; nota66 refs 0c75692168c5c220c364733567b5dc28a01a5739 publicada, preservadas 65 notas/refs e reserva11 liberada. Cargo 422/0 classvuln/2 avisos, OSV 542/3 exceções/0 restantes, Gitleaks 210 commits/0; os limites da evidência de cada tarefa estão nas [specs DB-004](../../planning/specs/DB-004.md) e [DB-005](../../planning/specs/DB-005.md). Auditoria C SQLite, coverage Rust global e fault injection não executados. | parcial E3 / controle global pendente |
| SEC-003 | vazamento por logs/chaves; não registrar dados pessoais-financeiros ou segredos conforme [requisito](SECURITY_REQUIREMENTS.md)/[segredos](SECRET_MANAGEMENT.md) | SEC-003, SEC-014, SEC-015; BKP-003 | Unit/integração do caminho real de erro/diagnóstico: marcadores sintéticos de telefone/dados financeiros/chave ausentes de logs/relatórios; mensagem sanitizada mantém diagnóstico permitido. Testar sucesso/erro e registro de falha de backup. Chave não hardcoded/commitada/exibida; armazenamento/recuperação aindaPOC. Gitleaks isolado não prova redaction. | [política](LOGGING_POLICY.md)/[SEC014](../../planning/specs/SEC-014.md)/[tests reais](../../src-tauri/src/startup_tests.rs):5filhos (4privacidade+guard102)/catálogo startup e hook Rust sem telefone sintético, seisRustlocal/E3; P2corrigido/re-reviewE2 semachados/CI37857821138 entrega verde. CI não executa suiteRust; fechamento/nota55 pendentes. SEC015: cincofilhos financeiros sintéticos na mesma fronteira/seteRustlocal/E3; reviewE2fontes semachados/CI37860308161 entrega verde. Fechamento/nota56 pendentes. Fluxos financeiros reais/backup/chave/demais sinks não comprovados | parcial/E3 local startup/hook; demais caminhos E0 |
| SEC-004 | arquivo externo/path malicioso; validar antes de uso, diretórios controlados conforme [backup](BACKUP_SECURITY.md)/[ameaças](THREAT_MODEL.md) | BKP-001, BKP-002, BKP-003, BKP-004, BKP-008, BKP-009; SEC-001; UI-017 | Unit/SQLite/filesystem controlados: malformado/truncado/adulterado/versão incompatível/credencial inválida quando aplicável rejeitados; traversal/fora allowlist mantém ativo e sentinelas externos. Formato/autenticidade no core antes de uso/promoção, não só filtro na UI. Arquivo válido segue fluxo; limites/normalização/reparse points/chaves dependem contrato/POC. | formato/configuração crypto/validação/path/negativos e caso permitido; [planoMVP13](../traceability/REQUIREMENTS_MATRIX.md#mvp-13--backuprestore-rastreabilidade-planejada); não registrados | todo/E0 |
| SEC-005 | restore destrutivo; preservar atual/staging/integridade-leitura antes de promoção conforme [backup](BACKUP_SECURITY.md)/[operação](../operations/BACKUP_RESTORE.md) | BKP-010, BKP-011, BKP-012, BKP-013, BKP-014; UI-017; SMK-010 | SQLite/filesystem reais: backupA válido/ativoB íntegro→validarA/staging/integridade-leitura/preservarB/promover/reabrirA. Arquivo autenticado mas SQLite corrompido/leitura inválida e fault injectionIO/staging/promoção mantêm B íntegro/operável. SQLrollback não desfaz arquivo por presunção; protocolo/conexões Windows futuros. | INV-BKP001002/positivos-negativos/preservação/reabertura/falhas; [detalhe](../traceability/INVARIANT_TEST_MATRIX.md#backuprestore--testes-planejados); não registrados | todo/E0 |
| SEC-006 | escrita parcial/duplicação; composto financeiro atômico/validação atual-gravações-audit juntos conforme [transação](../architecture/TRANSACTION_MODEL.md)/[negócio](../product/BUSINESS_RULES.md) | DB-011, DB-012, DB-013, DB-014; FIN-021, FIN-023, FIN-028, FIN-029; SAL-020, SAL-022; RCV-006, RCV-011; RET-010, RET-013; REC-012, REC-013, REC-014, REC-015 | Unit contrato/SQLite real por caso: sucesso persiste unidade; falha após primeira escrita/antes audit preserva histórico/saldos/devido/quantidade/estado/eventos aplicáveis. Recorrência repetida não duplica saída; idempotência dos demais casos segue spec, não botão/relógio. Não confundir retorno de erro/mock/smoke sucesso com rollback. | INV-FIN006007/INV-REC001; [global planejado](../traceability/INVARIANT_TEST_MATRIX.md#operações-compostas--testes-globais-planejados)/fixtures/comandos-saídas-SHA; não registrados | todo/E0 |
| SEC-007 | segredo/dependência vulnerável; scans são gates quando tooling existir conforme [dependências](DEPENDENCY_SECURITY.md)/ADR020 | SEC-006, SEC-007, SEC-008, SEC-009, SEC-010, SEC-011, SEC-012, SEC-013; INF-010, INF-011, INF-012 | Gitleaks/OSV/npm audit(high-critical)/cargo audit e SAST útil aprovado conforme fontes; lockfiles versionados/tool register antes de adoção. Provar configuração detecta fixture sintética aplicável e falha do scanner/achado bloqueia gate; job vazio ou scanner indisponível não prova verde. Achado corrige ou tem exceção temporária humana registrada; sem reduzir thresholds. | [GitleaksSEC007](../../planning/specs/SEC-007.md)/[OSVSEC009](../../planning/specs/SEC-009.md)/[npmSEC010](../../planning/specs/SEC-010.md)/[CargoSEC011](../../planning/specs/SEC-011.md)/[CISec012](../../planning/specs/SEC-012.md)/[SASTSEC013](../../planning/specs/SEC-013.md): execuções E3 no [CI37851893560](https://github.com/millennium42/millani-artes/actions/runs/37851893560), SHAa8e02cf7c4938855007dab4a4f3e10f087e335f2; E4 temporários OSV/Cargo preservados, SAST15casos de padrõesfrontend limitado | parcial/E3: scans e padrõesfrontend testados; segurança global/RustSQLrestore pendentes |
| SEC-008 | remoto/shell/filesystem amplo; ADR e capability específica exigidos; CSP bloqueia remoto por padrão conforme [política](TAURI_CAPABILITIES_POLICY.md)/[requisito](SECURITY_REQUIREMENTS.md) | SEC-001, SEC-002, SEC-004, SEC-005; BKP-004; UI-017 | Inspeção ADR/capability/CSP e teste WebView/core: recurso remoto e comando shell/filesystem amplo não autorizados negados; recursos locais/fluxo necessário restrito funcionam. Plugin sem ADR não passa gate. Permissão geral por conveniência proibida; futuros usos exigem decisão e teste específico, sem conceder nesta tabela. | ADR/justificativa por plugin/comando-janela-path/CSP/testes runtime-negativos e positivo mínimo; não registrados | todo/E0 |

[Camadas de teste](../quality/TEST_STRATEGY.md)/[coverage](../quality/COVERAGE_POLICY.md)/[smoke](../quality/SMOKE_TEST_POLICY.md) preservados: global90/90/90/85 e críticos95/95/95/90 quando tooling existir, Rust conforme política. Integração SQLite/filesystem/core/WebView exige execução real adequada à superfície, mocks não provam limites; smoke de produção futuro é complemento, não scanner/fault injection. Sem instalar ferramenta/job/dependência neste diff; segurança operacional continuaE0. Cada executor precisa de spec/código/config/teste/fixture sintética/comando/versão/saída sanitizada/SHA e run quando CI, antes de declarar o controle realizado.

## SEC-001 — prova parcial do scaffold
Em 2026-10-05, [config explícita](../../src-tauri/tauri.conf.json), [tests](../../src-tauri/src/security_tests.rs), [política/reprodução](TAURI_CAPABILITIES_POLICY.md) e [handoff](../../planning/handoffs/SEC-001.md) registram E3 local: seletor exclui capability extra, autoridade compilada nega36combinações, WebView2 real/React local funciona com set_title e emit negados por ACL e sem efeito (título/evento). Controle temporariamente permitido muda título/emite evento e reprova o mesmo oráculo; restauração byte a byte. Rust3passed/0ignored explicitamente; productionPE/custom-protocol e quality/audits com limites no handoff.
É prova do bootstrap da tarefa SEC-001, não conclusão dos requisitos globais SEC-001/SEC-008: positivos de plugins/directórios/backup, novas janelas/comandos custom e CSP continuam pendentes nos executores respectivos. Nenhum plugin de produto/customcmd ou grant de conveniência. O endpoint de relatório é cfg(test), ausente do produto. CI de entrega37347356818-SHA2e6edff success; CI final/nota-ref/liberação pendentes; o probe ignored é local, não executado no CI atual. Não alegar bloqueio de OS/APIs WebView/rede por capability, scanner pleno, crypto/SQLite/instalador ou segurança total.

Correção P2 aplicada: preflight absoluto/reparse agora no próprio teste, junction real negada com sentinela intacta/cleanup não recursivo. Execução final4Rustpassed/0ignored e WebView2 real5flags true; contagem de autoridade36combinações (6×3×2). Re-review/CI/gatehumano pendentes.

SEC001 bootstrap concluído: done/passestrue após revisãoP2corrigido/4Rusttests/probeWebView2-guardjunction/gates/docs/CI entrega37347356818-SHA2e6edff success19steps/JSONnative conferido e E4 humano2026-10-05 da configuração nesseSHA. FechamentoCI/nota pública/liberação pendentes. Requisitos globais/parciais e futuros positivos/plugins/paths/CSP/finanças/setup.exe permanecem pendentes; não promover segurança total.

## SEC-002 — prova parcial de SEC-008 no scaffold
[Spec](../../planning/specs/SEC-002.md)/[harness real](../../src-tauri/src/security_tests.rs)/[handoff](../../planning/handoffs/SEC-002.md) registram E3 local2026-10-07: config parsed sem fontes remotas/wildcard/eval/inline produção, dev exceções apenas locais; WebView2 real custom-protocol/is_dev=false com React/CSS local e5tentativas script/style/img/frame/fetch bloqueadas por securitypolicyviolation enforce para URI do coletor;HTTP0/TCPpreconnect1. csp=null temporário:5HTTP e falha nos mesmos asserts, configuração restaurada byte a byte.5Rustpassed0ignored incluem regressões SEC001/junction; audits/quality/nativebuild/checks e limites no handoff. CI/review/nota pública pendentes nesta fase; sem claim de execução remoteprobe/HMR/firewall/todasAPIs ou conclusão de SEC008 global/OS/plugins/backup/finanças/setup.exe.

SEC-002 bootstrap done/passestrue/E3 após5Rust/CSP-runtime-positive-negative/fontsparsed/REDnull-restauração/gates/review/docs e CI entrega37711001108-SHAb307c2c success19steps/1coverageartifact/nativeJSON conferido. CI final/43nota-ref-liberação pendentes; não concluir SEC008global/plugins/OS/backup/finanças/setup.exe.

## SEC-003 — política; enforcement pendente

[Política](LOGGING_POLICY.md) define campos diagnósticos permitidos, classes proibidas, fontes não confiáveis/fallback, destinos, audit_event distinto e oráculos SEC014/015/BKP. [Spec](../../planning/specs/SEC-003.md)/[handoff](../../planning/handoffs/SEC-003.md) registram política/reviewE2 e checks documentaisE3 quando executados; não são redaction de runtime. Histórico SEC003: startupRust usava expect com erroTauri bruto; SEC014 abaixo registra a correção/prova limitada. Inexistência de logger ou probes isolados não prova sanitização global. Linha globalSEC003 mantém enforcementE0 até código/testes do caminho real, inclusive erro de backup/diagnóstico útil preservado.

SEC-003 tarefa de política done/passestrue após revisãoE2/validatorsdocsE3/metadata/sourceintacto/commit e CI37712913326-SHA539e674 success10success8skipped0artifact. Requisito globalSEC003/enforcementreal continuamE0;CIfinal/nota44-liberação pendentes. Não confundir campos permitidos escritos com sanitizador/captura de sinks executados.

## SEC-004 — lista de plugins (executor documental)

[Inventário](TAURI_CAPABILITIES_POLICY.md#sec-004--inventário-de-plugins-e-necessidade-do-mvp) → executor [SEC-004](../../planning/specs/SEC-004.md)/[handoff](../../planning/handoffs/SEC-004.md), para requisitos SEC-001/008. Lista do scaffold vazia; core/CLI não são plugins. SQLite/IO no core, dialog condicional, log e POCchave têm fontes/executores; sem adoção, permissões ou garantia de gate runtime. SEC-005 deverá bloquear plugin sem ADR, e cada adotante deverá testar operação necessária funcional e negações reais com escopo mínimo. Não alterar os oito requisitos/oráculos globais nem promover a segurança E0 por inventário. Requisito SEC-004 (arquivos externos) é namespace distinto. ReviewE2 sem achados materiais e checks documentaisE3 aprovados; CI/nota final/liberação pendentes. Sem promover controles globais.

ExecutorSEC004 done/passestrueE3 documental após review/checks/commit/CI37714626133-SHA7f0ff510b488e5537b48ca21a418c26b0468d9cd success10success8skipped0artifact. CI final/nota45-liberação pendentes/reserva8ATIVA; requisitos globaisSEC001/008 parciais/gateSEC005 futuro, sem nova capability.

## SEC-005 — ADR estático de plugins
RequisitosSEC001/008 → [gate](../../scripts/check_tauri_plugins.py)/[registro](TAURI_PLUGIN_REGISTER.json)/[fixtures](../../scripts/test-tauri-plugins.py) → [SEC005](../../planning/specs/SEC-005.md)/[handoff](../../planning/handoffs/SEC-005.md). ADR específico aceito/identidade igual obrigatório nas superfícies estáticas documentadas; positivos-negativos sintéticos, sem plugin instalado/grantnovo. Runtime/ACL/macros/crypto/backup globais continuam pendentes, E4 não reivindicado. RED/checks locais registrados na spec; review/CI/nota46/liberação pendentes.

Executor SEC-005 done/passes true/E3 estático: 37 fixtures e 2 casos CLI, check real, re-review sem achados e entrega corrigida 8f35b78e5ba6dc0b9545791bb6758fcc491356c1 / [CI 37717088022](https://github.com/millennium42/millani-artes/actions/runs/37717088022) success, 19 etapas/uma job. Alias e identidade real exigidos; nenhum plugin/grant instalado. Requisitos globais SEC-001/008 continuam parciais, runtime e E4 não ampliados. Reserva13 ATIVA até CI final/nota46 preservando45/refs/Git limpo. META ATIVA, próximo SEC-006 não iniciado.

## SEC-006 — instalação local de Gitleaks
Executor [SEC006](../../planning/specs/SEC-006.md)/[handoff](../../planning/handoffs/SEC-006.md) prepara scanner local do requisito SEC007; origem/digest/licença registrados, instalação/probes/review/CI documental/nota47-liberação pendentes. Não confundir executor SEC006 com requisito SEC006 de atomicidade financeira. Gate de scanner no CI será SEC007; scan não prova redaction runtime de logs ou segurança global.

InstalaçãoSEC006 E3 local: versão8.30.1/hash-origem-licença/PE/probes clean0,github-pat1,invalid126 e redaction100 verificados;zero plugin/grant novo. Scannerhistórico/gateCISEC007/segurança global não comprovados;review/CI documental/nota47-liberação pendentes.

Executor SEC006 done/passestrue/E3 instalação portátil Gitleaks8.30.1, hash/licença/PE/version/probes locais e scan do diff, review/checksroot/entregad54eee5a5d09bf67eb5af3ef8420b0fc3266d750/[CI37719495570](https://github.com/millennium42/millani-artes/actions/runs/37719495570) success10success8skip0artifact. Gate de scanner remotoSEC007 e segurança global continuam pendentes, sem E4 novo. Reserva10ATIVA atéCI final/nota47preserva46/publicrefs-Gitlimpo/liberação. PróximoSEC007 não iniciado/META ATIVA.

## SEC-007 — Gitleaks no CI
Executor [SEC007](../../planning/specs/SEC-007.md)/[handoff](../../planning/handoffs/SEC-007.md) E3 local parcial:15checks/RED válido/149commits disponíveis zeroachados; pin8.30.1/integridade/configdefault/redaction/failclosed, uma etapa obrigatória antesbuild na job existente. Review/checks/scan diff/CIexactSHA/nota48-liberação pendentes;passesfalse/reserva12ATIVA/METAativa. RequisitoSEC007 de scanners permanece parcial (OSV/npm/cargo/SAST/outros executores), SEC003 logs/chaves/runtime não provado; sem ampliar E4.

SEC007 CORRECT: P2 de cache extra reproduzido (RED8.977s) e corrigido antes execução. Cache aceita exatamente os três arquivos obrigatórios e, opcionalmente, somente o ZIP oficial validado de SEC006.16casos passaram/9.135s;scan149commits zeroachados/1.494s. Re-review/checks/diffscan/CI exato pendentes;passesfalse/reserva12ATIVA/METAativa/SEC008nãoiniciado.

SEC007 re-review E2 semP0P1P2 após corrigirP2cache;root16checks/149commits0achados/4docsvalidators verdes;12paths49prévios245tuplas preservados. Entrega/CIexato ainda pendentes;passesfalse/reserva12ATIVA/METAativa/SEC008nãoiniciado.

SEC007 executor done/passestrue/E3:16checks/root,reviewP2corrigido/re-reviewsemachados,entrega2597cf4047b870ee30e9bca8f029f9884269c6cf/[CI37768499862](https://github.com/millennium42/millani-artes/actions/runs/37768499862) success20etapas/1coverageartifact. Gate7 obrigatório/103commits públicos disponíveis/0achados;local pré-entrega149inclui notas. Requisito globalSEC007 continua parcial, sem E4 novo. CI final/nota48preserva47/refsGitclean/liberação pendentes/reserva12ATIVA/METAativa/SEC008nãoiniciado.

## SEC-008 — instalação local de OSV-Scanner
Executor [SEC008](../../planning/specs/SEC-008.md)/[handoff](../../planning/handoffs/SEC-008.md) E3 local parcial:OSV2.6.0/WindowsAMD64/hash-checksum-licenseApache2-PE/version-help-erro conferidos;4probes somente CLI, sem scan de dependências. Requisito globalSEC007 continua parcial;gateOSV éSEC009, não instalação. NotSigned/SLSA não verificado/telemetria não medida;semE4novo. Review/docs/diffsecurity/CIexato/nota49-liberação pendentes;passesfalse/reserva10ATIVA/METAativa.

SEC008 guard documental corrigido:versão exata/hashes antes-depois;6casos reais+mockexplícito verificados pelo root/P2resolvido/re-reviewE2semachados. CLI original e3hashes íntegros,semscanOSV. CIentrega-final/nota49-liberação pendentes;passesfalse/reserva10ATIVA.

## SEC-008 — CI de entrega / fechamento documental
Entrega pública c6b4a27bae6ca31e17e80eacd984c5f07163d1d2 / [CI37774382975](https://github.com/millennium42/millani-artes/actions/runs/37774382975), job113301471481/attempt1/main/SHAexato/success19etapas(11success8skip)/zeroartifacts. Root conferiu API dos runs/jobs/steps/artifacts;etapa Scan Git history for secrets obrigatória success antesbuild. Seletor real false baseefb25a13→entrega;nenhum OSV remoto/download/scan integrado,novo job/action/cache/upload,rerun ou CI negativo. Descoberta ghapi indisponível;credentialmanagerGit existente reutilizado somente em memória para API GitHub,sem log/credencial publicado e sem redirects autenticados.
Executor SEC008 done/passestrue/E3 somente instalação local:ausênciaRED/3arquivos oficiais fixos/version2.6.0/4probesCLI/AuthenticodeNotSigned/6casos documentais apósP2/3hashes originais preservados/cleanup concluído/re-reviewsemP0P1P2/checks docs/diffsegredos/CI entrega verde. GatesOSV/dependências sãoSEC009 e posteriores,semclaimzero vulnerabilidades/E4novo. Root4validadores finais/10paths/50JSONanteriores intactos245tuplas/51items48done240files271links;log208642base preservado,entrega213920. Scansegredos diff finalentrega exit0/0achados/0.447s/SHA2564adb72f5e9421b59f32a88a4eac22a2a2e638a7709e2f9eff4f2f3384b95fb05. Reserva10ATIVA até CI do fechamento noSHAexato/nota49preserva48/publicrefs-Gitlimpo/liberação. Recibo final em notas Git evita commit recursivo. Semproduto/manifests/pins/locks/grants/thresholds/workflow/seletor alterado. METAativa:finanças/SQLite/backup/setup.exe/release pendentes. PróximoSEC009←SEC008,INF010;não iniciado neste ciclo.

## SEC-009 — gate OSV em preparação
[Spec](../../planning/specs/SEC-009.md)/[handoff](../../planning/handoffs/SEC-009.md)/[checker](../../scripts/check_osv.py)/[testes](../../scripts/test-osv.py):29casosRootE3/CLIofflinefictícia+fronteiras;OSVonline537packages/3registros2avisos emCargo. PropostaWindows vinculada aosinputs/versões/IDs/expiração estáapprovedfalse,semE4;review/checks/CI/nota50-liberação pendentes/passfalse/reserva13ATIVA. Requisito globalSEC007 permaneceparcial/demaisgates pendentes/METAativa.

SEC-009: 34 casos locais passaram; P1 do vínculo entre aceite e conteúdo corrigido, re-review pendente. Ensaio online real somente com aceite sintético: 537 pacotes, 3 registros da proposta, 0 outros. Proposta Windows inativa, sem E4/CI novo; passes:false, reserva 13 ATIVA. Digest cb8a47f5d84b722cdd1f6274c7807112132bc6e008acdd073972ddeb1b8b6921. META ativa; SEC-010 não iniciado.


SEC-009 (estado vigente): 38 casos locais passaram; referência ao registro separado impede reutilizar aceite anterior. Proposta e registro desativados, sem E4/CI novo; re-review pendente. Reserva 14 ATIVA, 52 itens/48 done/247 arquivos. Digest cb8a47f5d84b722cdd1f6274c7807112132bc6e008acdd073972ddeb1b8b6921; META ativa, SEC-010 não iniciado.


SEC-009: exceção aprovada pela usuária em 2026-10-08 e ativada no escopo cb8a47f5d84b722cdd1f6274c7807112132bc6e008acdd073972ddeb1b8b6921; registro d9e4cb4821fcfbbd7630d361b1eef8bb5092c11520000ba9d796380660188152. 38 testes root verdes; scan real537pacotes/3excepted/0outros. Expira23/10/2026 00hUTC; mudanças/expiração bloqueiam. E4 somente a dispensa delimitada, passes:false até CI exactSHA e fechamento. Reserva14 ATIVA; outros gates/funcionalidades/instalador pendentes.


SEC-009: checkout dos dois inputs Cargo com LF explícito, mesmos hashes/aceite/prazo. 39 casos verdes, incluindo Git checkout Windows autocrlf=true. CI2c263c7/run37790695323 falhou no binding antes do build; correção e novo CI exactSHA pendentes, sem rerun manual. Reserva15ATIVA/passfalse; META final pendente.


SEC-009 executor done/passes:true/E4 limitado ao aceite e E3 ao gate OSV: 39 casos/inventário537/3registros aprovados/0outros; CI entrega corrigida37791933046/SHA1f16a6a3bfde511522fef8fcf5824cc8fd1b7668/21etapas verdes/nativePEAMD64. ExceçãoWindows expira23/10/2026 00hUTC, mesmosinputs/hash/IDs; globalSEC007 parcial e demaisgates pendentes. Coverage frontend atual100%, wrapperPython percentualnão registrado. FechamentoCI final/nota50/liberação15 pendentes; META final não concluída.

## SEC-010 — npm audit local

[Spec](../../planning/specs/SEC-010.md), [checker](../../scripts/check_npm_audit.py), [testes](../../scripts/test-npm-audit.py), [procedimento](../operations/NPM_AUDIT.md): E3 root38casos/4CLIreaisloopback; highcritical/erro/JSONparcial/summaryfalso/runtime/inputs/junction bloqueiam. Auditreal120dependências/0reportadas; locks/manifests preservados. Config/preload/token isolados; sem fix/scripts/upgrade. RequisitoSEC007 permanece parcial: integraçãoauditCI éSEC012, outrosgates/futurasfinanças/release pendentes. Review/checks/publicação/CIexactSHA/nota51/liberação ainda pendentes; passes:false/reserva11ATIVA. SemE4novo nem transferência da exceçãoOSV.

SEC010 VERIFY: root40casos/4npmCLIreaisloopback/7.799s, auditonline120dependências/0reportadas/1.967s, hashesinputs preservados. DoisP2review reproduzidos/corrigidos; re-reviewP-REV5.6-solmedium E2semachados. Root docs/map/schema/evidence/log/ignore261links/diffsecurity verdes;52items anteriores(em24JSONfiles)/245tuplas/prefixo234620bytes preservados. Produto/manifests/locks/pins/thresholds/workflow/seletor intactos;253tracked/53items49done previstos naentrega. P-QA6-lunalow oráculo+P-REV5.6-solmedium revisão,2delegadosreadonly/perguntasdistintas/0escalonamentos; tokens/custo/cache/latênciamodellos/retriesevitados não registrado. Integração auditCI permaneceSEC012. EntregaCIexactSHA/fechamentoCIexactSHA/nota51-liberação ainda pendentes;passesfalse/reserva11ATIVA/METAfinalpendente.

## SEC-010 — entrega comprovada / fechamento

SEC010 executor done/passes:true/E3 após entrega públicaa93b3de4bb2dc13a864c263f73a4c934647c3de9/[CI37815340184](https://github.com/millennium42/millani-artes/actions/runs/37815340184)/job113442423726/attempt1/main/success21etapas/uma job/1artifactcoverage existente. Root conferiu API/logs reais em memória: OSV39casos/3.66s/537pacotes/3registros aprovados3excepted0outros/expiry2026-10-23, Gitleaks8.30.1/111commits/0achados/0.77s, sourceSHAexato. NativePE32+AMD64/custom-protocol/8563200bytes/SHA256edcb99371fd46f252b0c53b642c9d9da39bb1c049f1f0dee8992d5f595780aee, RustCargo1.99/Node24.21/image20260925.250.1; produto/locks/pins/grants/thresholds preservados. Qualityfrontend passouformat/lint/typecheck/coverage; artifact11566902508/retention1dia solicitado(86399s) lido em memória:100%4linhas5statements1function2branches, thresholds90/90/90/85 inalterados. Isso é apenas frontendscaffold atual, nãofinanças/Rust/Python/setup.exe.
Gate npm40casos locais/4CLIreaisloopback/7.799s, auditreal120dependências/0reportadas/1.818s noSHAentrega; inputs{"package-lock.json":"bb9dd4c788dc028f426b4a7d7c3ef3be7ce02b5374ac555455382e9dfb52b284","package.json":"475d1d3c95ab6a79ae5944eba4c34d100e15fc890a5f21aafac687970d59484a"} preservados. Checker/testes npm NÃO executados remotamente; integraçãoCIaudit éSEC012. Nenhuma nova etapa/job/action/upload/cache ou rerun nesta tarefa. Review2P2corrigidos/re-reviewE2semachados; fonte/testes sem mudanças depois das verificações. Doisdelegadoseconômicos (P-QA6-lunalow/P-REV5.6-solmedium),0escalonamentos; custo/cache/tokens/latênciamodellos/retriesevitados não registrado. Contexto sem rerodar scans/testes após mudanças puramente documentais.
passes:true refere-se ao executor/CIentrega observado; LIBERADA ainda exige CI no SHAfinal/nota51 preservando50/publicrefs/Gitlimpo/cleanup e liberação11paths. Reserva11ATIVA até esse recibo externo àárvore.53items50done253tracked/52itens(em24JSONprévios)245tuplas/prefixo234620preservados. META final appfinanceiro/SQLite/backup/setup.exe/release não concluída; próximoSEC011 só após liberação/não iniciado.

## SEC-011 — gate Cargo / aceite próprio

SEC-011 E3+E4: gateCargo0.22.2 falhafechada/47cases/6CLIreais/DBGitfictícia e online real;417deps/0vulnsclassificadas/2warnings excetuados. Aceite literal próprio/recorda0698660/scope5bbcd9f7/expiração23Oct00UTC, hashesfixos; demaisachados/inputsmudados/expiração bloqueiam. Reviewativação P-REV5.6-solmedium E2semachados; executorpassesfalse atéCIentrega. [Spec](../../planning/specs/SEC-011.md)/[handoff](../../planning/handoffs/SEC-011.md) têm detalhes e limitações; CargoCISEC012pendente.

SEC-011 executor/E3+E4: 47casos/6CLIreais/gateCargo417deps0vulnsclassificadas2excepted, entrega3af54a6/CI37824390707 verde21steps. E4scope/prazo/hashes intactos; OSV537/3excepted/Gitleaks114commits0; cobertura100% apenasfrontendscaffold4linhas. Reviewsemachados/checks green/53itemsprévios245tuplas/prefixo245790preservados. passestrue=executor; liberação13 dependeCIfinal/nota52 preserva51. CargoCISEC012pendente; METAsetupfinanceiro pendente. Detalhes na spec/handoffSEC011.

## SEC-012 — interrupção do build pelos audits

[Spec](../../planning/specs/SEC-012.md) →42casos locais/14PowerShell reais, mutações de bypass e bootstrap frio/quente/negativos offline. Erros interrompem antes do marcador de build; runtime/cache/banco reparse/divergentes bloqueiam. Gates npm40/Cargo47 preservados, execução online e CI exactSHA pendentes. E4 anterior/prazo preservados; coverage Python NR. Segurança global/SAST ainda parcial.

## SEC-012 — entrega corrigida validada

Executor E3/passes:true: [entrega680e8d2](https://github.com/millennium42/millani-artes/commit/680e8d2922f8149959df53c62e9b80390651e5c0)/[CI37832138829](https://github.com/millennium42/millani-artes/actions/runs/37832138829) success23steps/1coverageartifact. Remoto:40npm+47Cargo+42integração/14PowerShell; scan120npmzero/417Cargo0vulns2excepted, OSV537/3excepted, Gitleaks117/0. E4/prazo/inputs/thresholds preservados; coverage100% sóscaffoldfrontend4linhas. FalhainicialRTK ausente reproduzida/corrigida semalterarcasos/instalarferramenta; re-reviewE2semachados. Reserva16ATIVA atéCIfinal/nota53;55items52done267files/245tuplas. Detalhes na spec/handoffSEC012, METAativa/SEC013nãoiniciado.

## SEC-013 — avaliação local de SAST

[Spec](../../planning/specs/SEC-013.md) →[avaliação](../../docs/security/SAST_EVALUATION.md) →[harness real](../../scripts/test-sast.py):15casos/12lintCLIs/3PowerShell; HTML perigoso/eval direto e alias detectados, limpos verdes, parse/config/exclusão falham, retornos0/1/2 comprovam interrupção. Biome2.5.15 instalado/config atual, overrideVCS somente fixture. E3 local não é CI verde; review/CI ainda pendentes,passesfalse/reserva11ATIVA. SAST limitado ao frontend/padrões testados; requisitoSEC007 continua parcial/RustSQLrestoreE0, METAativa/SEC014nãoiniciado. Semnova ferramenta/job/upload/threshold/aceiteE4.

## SEC-013 — VERIFY local

15casos/12CLI/3PowerShell,negativoPython-O,quality3testes/100%4linhas,42integração green; reviewP-REV5.6-solmediumE2 semP0/P1/P2, oráculoP-SEC6-lunalow usado e reproduzido root. Validators/securitydiffzero/preservação55prévios27JSON/245tuplas/log256613/inputE4 verdes. [Spec](../../planning/specs/SEC-013.md)/[handoff](../../planning/handoffs/SEC-013.md) limitam prova; CI entrega/final pending/11ATIVA/passfalse, METAativa.

## SEC-013 — entrega validada

Executor done/passes:true/E3, SHAa8e02cf7c4938855007dab4a4f3e10f087e335f2/[CI37851893560](https://github.com/millennium42/millani-artes/actions/runs/37851893560)/attempt1:15SAST/12CLI/3PS,qualidade/audits/build green. [Spec/limites](../../planning/specs/SEC-013.md#executor--learn--entrega-validada): padrãofrontend não provaRustSQLrestore/finanças/globalSEC007continua parcial. Reserva11ATIVA atéCIfinal/nota54/refsGitclean;METAativa/SEC014nãoiniciado.

## SEC-014 — telefone na fronteira real

[Spec](../../planning/specs/SEC-014.md) →[main](../../src-tauri/src/main.rs)/[tests](../../src-tauri/src/startup_tests.rs) →[handoff](../../planning/handoffs/SEC-014.md). RED verdadeiroSTARTUP_PHONE_LEAK_DETECTED exit101/12.031s, sem publicar buffers. BUILD4filhos reais green9.922s(tempoincluicompilação):sucesso0/erroTauri1/panic101/stderrfechado1; catálogo JSON estático e hook antesbuilder, sem regex/plugin/dependência produtiva. E3local parcial/CI-review pendentes/11ATIVA/passesfalse; SEC003global ainda parcial/finançasbackupsFfIfuturosE0. METAativa/SEC015nãoiniciado.

SEC014 VERIFYlocal final:buildTaurireal/faultInjectionErrorIo compartilhada com main/4filhos/6Rustpassed0ignored/hookstderrstatic+closedpipe/fmtclippy/auditqualitygreen; nenhuma simulação de falha natural da biblioteca/nenhum pluginadotado. [Spec](../../planning/specs/SEC-014.md#verify-final-local) registra RED final8.718s,sourcehashes/limites/cleanupcontrolado. ReviewCIpending/passesfalse/11ATIVA/METAativa.

SEC014 VERIFY final apósP2: cinco filhos (quatro privacidade + guard before-injection102), confirmação AtomicBool após build real impede falso positivo; seis Rusttests/zeroignored/fmtclippy green. [Prova e hashes finais](../../planning/specs/SEC-014.md#verify--correção-p2-executada). E3local/re-reviewCIpending/11ATIVA; fronteira Err com faultinjection, não falha natural da biblioteca; METAativa.

SEC014 executor done/passestrue/E3 após entrega55b481af4d394aa624d384b6d38887bbdf898a04/[CI37857821138](https://github.com/millennium42/millani-artes/actions/runs/37857821138)/attempt1/23success. Cincofilhos/seisRustlocal/P2corrigido/re-reviewsemachados; [prova/limites](../../planning/specs/SEC-014.md#ci--learn--entrega-validada). Fechamento9docs/reserva11ATIVAatéCIfinalnota55refsGitclean; SEC003global parcial/METAativa/SEC015nãoiniciado.

## SEC-015 — dados financeiros na fronteira atual

[Spec](../../planning/specs/SEC-015.md) →[tests reais compartilhados](../../src-tauri/src/startup_tests.rs) →[handoff](../../planning/handoffs/SEC-015.md). E3local: RED de sensibilidade expect antiga detectou vazamento financeiro, restauraçãofinally; cinco filhos financeiros/cincophone, seteRust0ignored/fmtclippyaudit green. Main produtivo inalterado. Centavos/saldos/preços/quantidade/conta/categoria/contextos/IDs/agregados e formato/UnicodeCRLF/nestedunknown sintéticos ausentes de stdoutstderr, saída e status estritos/injeção pósbuildreal/guard102. Não simula falha natural do Builder.run nem prova transação financeira/SQLite/backup/all-emitter/FFI; requisitoSEC003global continua parcial. Review/CI/nota56-liberação pendentes/passfalse/11ATIVA/METAativa/ARC001nãoiniciado.

SEC015 executor done/passestrue/E3 após entrega96c06e66e53f8c9776d51576bf7f467102487ad2/[CI37860308161](https://github.com/millennium42/millani-artes/actions/runs/37860308161)/attempt1/23success; [spec/prova/limites](../../planning/specs/SEC-015.md#ci--learn--entrega-validada). Cincofinance+cincophone/seteRustlocal/productionunchanged/reviewE2fontessemachados; CI não executaRustsuite. Reserva11ATIVAatéCIfinalnota56/refsGitclean/METAativa/ARC001nãoiniciado; globalSEC003parcial.


## ARC-001 — regressão após extração das camadas
[Application](../../src-tauri/src/application/mod.rs)/[infrastructure](../../src-tauri/src/infrastructure/mod.rs) preservam fronteira/hook/emissor fechado, exercitados pelo [harness](../../src-tauri/src/startup_tests.rs); main faz composição. Marker productionChanged:false removido: não era fato medido pelo teste. Probe std-only com6imports negativos verifica isolamento no SHA; não substitui scanner Rust, ARC-005 ou testes de SQL/finanças/backup. [Spec](../../planning/specs/ARC-001.md) registra execução real e limites; estado global de SEC003/SEC007 permanece parcial.


ARC-001 executor done/E3/passes:true: [entrega](https://github.com/millennium42/millani-artes/commit/e94eec776213725f72239c8d07aff2a293f6f632)/[CI37863336200](https://github.com/millennium42/millani-artes/actions/runs/37863336200) success23steps/buildWindows, std-only6controles e7Rust locais/reviewE2fontes semachados. Domínio sem regra; finanças/SQLite/backup/setup.exe pendentes/METAativa. Reserva22ATIVA atéCI final/nota57, fechamento11docs; detalhe na spec/handoffARC001. ARC002 não iniciado.

## ARC-002 — superfície fechada do erro de domínio

Executor E3/entrega93362971/CI37865774054: root10Rust/5compilernegatives, plugin0, Gitleaks0, reviewE2 sem achados de contrato. CI compila produção/audita/scaneia, sem executar Rustsuite; fechamento documental/reserva17 até nota58 no [handoff](../../planning/handoffs/ARC-002.md).
[DomainError/Result](../../src-tauri/src/domain/mod.rs) →[unitários](../../src-tauri/src/domain/tests.rs) + compilernegativesE0618(payloadtexto)/E0277(From<String>) +std-only semTauri/infra/SQLite →[spec](../../planning/specs/ARC-002.md)/[handoff](../../planning/handoffs/ARC-002.md). Biblioteca pública Rust não é IPC. Bootstrap/hook/catalogo/CSP/grants/UI intactos,7regressões native/5phone5finance passaram. Estado globalSEC003/SEC007 e invariantes financeiras continuam parciais; nenhum dado real usado.

SAL-022 E3 verifica a superfície pública de erro no finalizador: FK deferred de conta faz o commit falhar, mas o consumidor retorna apenas `SaleError::StoreFailed`, mensagem genérica e sem causa; snapshot financeiro/auditoria é preservado. Isso não substitui prova global de SEC-003 nem cobre UI/IPC.


## ARC-003 — fornecedores injetáveis
[Portas](../../src-tauri/src/domain/ports.rs) →[testes](../../src-tauri/src/domain/ports_tests.rs) substituição/16bytes/sequência/falha opaca →std-only6tests/5negativos(String/UUID15bytes/Tauri/infra/SQLite) →[spec/handoff](../../planning/handoffs/ARC-003.md). Sem IO/unsafe/plugin/IPC/log/default novo.13Rust regrediram CSP6/IPCdenied/phone5/finance5/junction após cleanup do próprio perfil do baseline. IDs/erro de adapters futuros continuam proibidos em logs; erro opaco não é sanitizado. Plugin0, scan/review/CI pendentes/inputE4prazo intactos; segurança global parcial.


ARC-003 executor E3 validado na entrega6eafaa40/[CI37963747801](https://github.com/millennium42/millani-artes/actions/runs/37963747801), root13Rust/5negativos/reviewE2 sem achados. Fechamento documental preserva fontes/thresholds/inputs; reserva14 até CI final/nota59 no [handoff](../../planning/handoffs/ARC-003.md). Portas não comprovam UUID real/unicidade/relógio monotônico/invariantes financeiras.

## ARC-005 — separação de SQL no frontend

[Spec](../../planning/specs/ARC-005.md) →[config/regra](../../scripts/no-ui-sql.grit) →[harness real](../../scripts/test-sast.py): E3 local,45casos/42CLI/3PS incluindo15casos anteriores e falhas de plugin/configuração. SQL reconhecível, imports de drivers, require e comandos SQL IPC literais recusados em src/. Comentário/JSX textual limpos passam; fragmentos/escapes e texto JS com palavra SQL inicial são limites registrados. Não comprova bindings/constraints/SQLite nem muda o requisitoSEC002 de SQLi para verde. SEC007 continua parcial. Quality100%/4linhas do scaffold; review/CI/nota61/liberação14 pendentes, E4 temporários e prazo intactos.


ARC-005 executor done/E3 após [entrega c5872e20](https://github.com/millennium42/millani-artes/commit/c5872e2026291512fd9eb879e3562200d72b7dd7)/[CI37974060505](https://github.com/millennium42/millani-artes/actions/runs/37974060505), tentativa1/23success.45casos SAST/42CLI/3PS, quality/audits/build Windows aprovados; [prova e limites](../../planning/specs/ARC-005.md#ci--learn--entrega-validada). Fechamento preserva fontes; reserva14 até CI final/nota61 pública. Guard de padrões não comprova SQLi/SQLite/finanças; META ativa/DB001 não iniciado.
## DB-002 — abertura de arquivo no core

RED/GREEN local E3 cobriu arquivo real, round-trip, rejeições e preservação de sentinela; review E2 sem P0/P1/P2. E4 materializada nos records; gates Cargo/OSV pós-aceite passaram. Houve colisão scratch em duas tentativas concorrentes; retries locais em série passaram. CI 38103942047 validou a entrega pública (23/23); DB-002 fechado: CI documental 38104862341/job114368093002 no SHA 1ef3bd833708f46e0e26188e71b3cfb83f679014 passou 16/7/0; nota63 pública. A prova não estabelece escolha segura de diretório, containment, startup, permissões, locks entre processos, finanças ou backup. [Spec/handoff](../../planning/specs/DB-002.md) detalham evidência e limites.

## DB-003 — executor transacional de migrations

E3 local: falha inicial do stub (`ApplyFailed`), harness debug/release 9/9, suíte Rust 22/22, fmt/clippy e gates Cargo/OSV passaram; SQLite 3.53.2. Review incremental E2 sem P0/P1/P2. CI entrega 38197556421 passou 23/23 sem skips e CI final documental 38198511557 passou 23/23. Erros opacos e descarte da conexão foram cobertos; não há alegação de SQL externo sanitizado, fault injection de I/O, C SQLite coberto pelos audits ou segurança global. Veja [spec/handoff](../../planning/specs/DB-003.md).

## DB-004 — coerência do ledger e migração versionada

E3 local root: harness debug 16 casos e release 16/16, suíte Rust 29; fmt/clippy. Review incremental sem P0/P1/P2 após dois P2 corrigidos. Entrega CI 38201472821/23 success; CI fechamento 38299901151/job114980000210 no SHA 94e37d38f5df375ffb9492e3f80d272f2f44b331 passou 16 success/7 skipped/0 artifact; nota 65 publicada e reserva liberada após preservação das 64 notas e refs remotos. Ledger é coerência estrutural, não autenticação, sandbox ou validação global de backup; scripts são SQL trusted. Sem claim de concorrência global, fault injection física ou cobertura de C SQLite.


## DB-005 — migration versionada vazia em arquivo

E3 local no harness: dois arquivos reais, SQL vazio pela API versionada, ledger v1/shape/check/autocommit/reabertura/integrity_check e isolamento; segundo teste rejeita chave duplicada e valores 0/-1 por constraints SQLite. RED controlado do caller, não vulnerabilidade de produção. Rust 31/31, release harness 18/18, fmt/clippy; review 5.6-sol-medium E2 sem P0/P1/P2. CI entrega 38303443795 passou 23/23/release18/18; staged scan e validators da entrega passaram. CI documental final 38304515870/job114995197572 no SHA 1d92fdd185cf24aececd1fa5b5708d9d0efeab03 passou 16 etapas/7 skipped/0 artifacts; nota66 refs 0c75692168c5c220c364733567b5dc28a01a5739 publicada, preservadas 65 notas/refs e reserva11 liberada. Controle limitado ao bootstrap de teste, sem prova de SQL injection global ou migrações de produto. [Spec](../../planning/specs/DB-005.md) e [handoff](../../planning/handoffs/DB-005.md).


## DB-006 — snapshot de fixture (escopo de teste)

SQL checked-in trusted, estático e sintético exercitado por `execute_migration`; teste valida dois arquivos isolados, metadados/constraints e estado após reopen. RED vazio e GREEN da fixture passaram conforme E3 root; source production `database.rs` permaneceu SHA-256 `242d439a2e32fb7e7f6ce12ba7f2d66fccb78ee8eab9884bd86181ffe6ab72a7`. Scans Cargo/OSV executados pelo root: 422 pacotes, 0 vulnerabilidades classificadas/2 avisos aprovados; advisory DB 1307, commit `57dcc0f2192e025138907a495c1ac9a7e1e3e123`; OSV 542 pacotes/3 achados excetuados/0 fora. Scopes E4 herdados sem alteração, alvos e prazo na [spec](../../planning/specs/DB-006.md); C SQLite fora de cobertura. CI entrega 38307470576/job115004892529 no SHA 0b8f2a7d0dc8d236d91498cc20e28b358c62dcce passou 23/23; marcadores Cargo422/0vuln-class/2avisos, advisory1307 commit57dcc0f2192e025138907a495c1ac9a7e1e3e123 e OSV542/3 aprovados/0 fora. Executor done/E3; DB-006 fechado: CI final 38308867688/job115009442508 no SHA dd60e198c7ae93bebc41ac873442e5be8193a748, 16 success/7 skipped/0 artifacts; nota67 pública nas refs 69f8ea41a260c442944713b9cff6552f39cbd4a2, 66 notas anteriores preservadas e reserva12 liberada pelo root. Isso não prova segurança de migrações de produto, SQL dinâmico, startup, backup/restore ou segurança global. [Handoff](../../planning/handoffs/DB-006.md).


## DB-007 — upgrade versionado do snapshot

[Spec](../../planning/specs/DB-007.md)/[handoff](../../planning/handoffs/DB-007.md): RED controlado ledger `[1]` vs `[1,2]`; GREEN caller versionado. Root E3 Rust33/33, release20/20 SQLite3.53.2, 19 antigos preservados; review sem P0/P1/P2. CI entrega e documental passaram; executor `done`/E3/`passes=true`. DB-007 fechado: CI final 38320496225/job115047171038 no SHA 323e336b53c74147c4b70bdec44391dfad29fe72, 16 success/7 skipped/0 artifacts; nota68 pública refs f9786502025de68b33275fff766f0428cc862bd1, 67 notas preservadas e reserva11 liberada.


## DB-008 — evidência de rollback

[Spec](../../planning/specs/DB-008.md)/[handoff](../../planning/handoffs/DB-008.md): tarefa documental sem código; cinco fixtures existentes PASS em stdout debug/release. Root fmt/clippy/Rust33/33/release20/20; review E2 sem achados. CI documental entrega 38322109825/job115052349852 no SHA 08a214888bf581d40e74a143bfdfdc48dea0878a passou 16 success/7 skipped/0 artifact (docs apenas). Executor `done`/E3/`passes=true`; DB-008 fechado: CI final38322839469/job115054651101 no SHA dfbe60893150c222b17987518f65a161d3e7ef77, 16/7/0; nota69 refs054bfc0f6bce94a795158a89ded50d20d71bd989, 68 notas preservadas/reserva10 liberada. Sem prova de falha física de rollback.


## DB-009 — ativar foreign keys ao abrir

[Spec](../../planning/specs/DB-009.md)/[handoff](../../planning/handoffs/DB-009.md): baseline ON, controle OFF fail-closed; E3 local 34/34 e release21/21; CI entrega 38324569231/job115060146427 passou 23/23. Executor `done`/E3/`passes=true`. DB-009 fechado: CI final 38325659041/job115063600279 no SHA 17f5397efc996091bfb4271a5169d4eede4249fe (16/7/0); nota70 refs 097a4650bac277404275272876296cbd5e682dbb, 69 notas preservadas e reserva12 liberada pelo root.


## DB-010 — provar enforcement de foreign keys (fechado)

[Spec](../../planning/specs/DB-010.md)/[handoff](../../planning/handoffs/DB-010.md): executor fechado no baseline SHA `083ac68e073667db3cba474722962b89ca7cdaa5`, CI `38387475775` SUCCESS 16/7/0, nota71 pública `dc2a872282bfaf9fb035cef0a0c336e6bdc8ff19`, 70 notas preservadas e reserva11 liberada. Evidência SEC-002 parcial e limitada ao schema sintético.

## DB-011 — transação SQLite

E3 local de root: quatro testes físicos passaram em debug/release; sanitização foi exercitada com arquivo real, erro real do driver e identificador sintético dentro de E. Gitleaks sinalizou a variável local do marcador; foi renomeada junto de três referências, mantendo literal, driver e oráculos; nenhuma regra/allowlist foi relaxada. Harness final SHA-256 `aec307745a413a46ba209d3a3228dd0ba14505999b309468d3ad4c5c16cc7913`. Review independente E2 e confirmação focal no hash final não encontraram P0/P1/P2. CI entrega: Gitleaks8.30.1 158 commits/0; Cargo422/0 vulnerabilidades/2 warnings excetuados; OSV542/3 excetuados/0 fora; SAST45 (42 Biome+3 PS). Root staged Gitleaks13 paths/0 findings e validators passaram. Isso não prova rollback composto nem resolve SEC-006. CI final e fechamento ainda pendentes. C SQLite permanece fora do alcance dos scanners. Consulte [spec](../../planning/specs/DB-011.md) e [handoff](../../planning/handoffs/DB-011.md).

## DB-012 — rollback genérico (E3 local)

Cinco casos físicos `db012_*` passaram debug/release e na CI de entrega `38400843616` (harness31/31). Scans remotos: Gitleaks160/0, Cargo422/0 vulnerabilidades/2 warnings aprovados, OSV542/3 excetuados/0 fora, npm120/0, SAST45 (42 Biome/3 PS; backend Rust/SQL/restore fora). Validators4/344 files/28 sources/74 items PASS; Gitleaks staged de entrega11 paths/45.749B/0 findings. Review E2 sem P0/P1/P2, limitada ao diff. CI documental final e nota73 pendentes. Não declarar SEC-006 fechado; C SQLite segue excluído dos scanners.

## DB-013 — evento técnico de auditoria (E3; entrega passou)

SQLite físico verifica metadados, tags, inválidos, duplicata, constraints, mutações, erro sanitizado e inserção ignorada. CI entrega passou: Gitleaks162/0, npm120/0, Cargo422/0 vulnerabilidades, OSV542/3 excetuados/0 fora, SAST45 (limites no handoff). Review E2 sem achados impeditivos. Não protege processo hostil do arquivo, audita payload ou prova atomicidade financeira. CI documental/nota74 pendentes; [spec](../../planning/specs/DB-013.md).


DB-014 E3: cenários SQLite temporários exercitaram atomicidade, erros sanitizados e integridade; nenhuma auditoria parcial visível. Evidência limitada ao harness, sem prova de ameaça de host/arquivo ou invariantes financeiras; CI entrega `38438041786` passou; fechamento documental pendente. [Handoff](../../planning/handoffs/DB-014.md).

## FIN-001 — superfície mínima de Account

Executor E3 concluído após CI `38444911363`/SHA `31af12adcf8bebe617539d9e5dde286a64674deb`: Gitleaks166/0, npm120/0, Cargo422/0 vulnerabilidades/2 avisos aprovados, OSV542/3 exceções aprovadas/0 fora e SAST45 (42 Biome + 3 PowerShell). Rust backend não é coberto pelo SAST; CI não executa testes de domínio/doctests/cobertura Rust. E4 existente e review E2 sem achados foram preservados. Escopo continua sem dados financeiros, validação de nome, armazenamento ou queries; não fecha controles/invariantes de produto. CI final documental pendente. [Spec/handoff](../../planning/specs/FIN-001.md).
## FIN-002 — validação de nome
E3 local: vazio e whitespace Unicode rejeitados com DomainError::InvalidInput; nome Unicode com espaços aceito sem mutação de name/id. Erro RED E0599 confirmou método ausente antes da implementação. Campos públicos permitem bypass: exigir validação no consumidor FIN-008. Cargo audit 422/0 vulnerabilidades/2 avisos aprovados; OSV 542/3 exceções/0 fora, expiração 2026-10-23T00Z. Review E2 sem P0/P1/P2. CI/Gitleaks staged ainda pendentes; nenhuma alegação de segurança de persistência.
## FIN-002 CORRECT — smoke de junction no Windows runner
E3 local: substituído wrapper RTK por pwsh direto, sem mudar guardas de path, sentinel ou deny cleanup. Binário executado com PATH restrito sem RTK: 6 passed/1 ignored/0 failed; junction real PASS e fixture removida. CI anterior falhou porque runner não tem RTK; nova entrega automática ainda pendente. Não inferir sucesso CI nem alterar teste/threshold.

Entrega CI FIN-002 38483459997: junction smoke PASS; release suite 57 ordinários+2 doctests, 0 failed/1 ignored. Scans remotos Gitleaks169/0, Cargo422/0 vulnerabilidades/2 avisos aprovados, OSV542/3 exceções/0 fora, SAST45 com limites apenas frontend/PowerShell e sem cobertura Rust. Exceções E4 expiram 2026-10-23T00Z. Falha anterior de wrapper RTK preservada como histórico, correção pwsh exercitada. CI documental final e liberação pendentes.

## FIN-003 — tipo puro de conta
E3 local: AccountType com quatro variantes sem conversores/serialização, teste preserva variantes; sem I/O, SQL, IPC ou dados sensíveis. Cargo audit422/0 vulnerabilidades/2 avisos E4 aprovados; OSV542/3 exceções/0 fora, expiry2026-10-23T00Z, inputs intactos. Review E2 sem P0/P1/P2. Nenhuma validação de fatura/taxas/cartão ou regra financeira inferida; CI e Gitleaks staged pendentes.

CI entrega38487235749: release suite58 ordinários+2 doctests, 0 falhas/1 ignored; junction test PASS. Gitleaks171/0, Cargo422/0 vuln/2 avisados, OSV542/3 exceções/0 fora; SAST45 não cobre backend Rust. Resultados são gates da entrega; não demonstram armazenamento de tipo nem operação financeira.

## FIN-004 — campo inicial i64
Tipo primitivo sem I/O ou boundary exercitada; preservação exata dos seis valores testada. Audit Cargo422/0 vulnerabilidades/2 E4 aprovados; OSV542/3 exceções/0 fora. Review E2 sem achados. Precisão de UI, consumo/persistência e política de sinal continuam não implementadas; FIN-005 deve provar tratamento como saldo inicial, não receita.


FIN-004 startup-test correction: probe exige stdout null sem parser; cinco fixtures de privacidade existentes preservados. Review Sol6 medium E2 sem P0/P1/P2. Medição global não cobre setup, persistência ou consumidores financeiros.

FIN-004 CI final: Gitleaks174 commits/0 findings; Cargo422/0 vulnerabilidades/2 aprovações E4; OSV542/3 aprovações/0 fora; npm120/0. SAST45 cobre Biome/PowerShell, não backend Rust. Nenhum desses scans prova consumidores financeiros ou setup.

CAT-001 é estrutura pura sem I/O ou trust boundary. Review focal E2 sem P0/P1/P2; Gitleaks/validators/CI primários ainda pendentes. Não inferir semântica operacional de is_active.

CAT-001 checks locais primários: review Sol6 medium E2 sem P0/P1/P2; 15 paths guard tracked intactos, cleanup de 8 artefatos e 0 processos; raízes temporárias ausentes. Gitleaks staged e CI finais ainda pendentes.

CAT-001 CI final scans: Gitleaks176/0, npm120/0, Cargo422/0 vulnerabilidades/2 aprovadas, OSV542/3 aprovadas/0 fora; SAST45 limita-se a Biome/PowerShell, sem Rust backend. Guard tests passaram; sem claim de workflow/persistência.

FIN-010 enum puro sem I/O/trust boundary; review E2 focal sem P0/P1/P2. Scans/Gitleaks do diff primário e CI exata pendentes; não alegar cobertura de consumers ou persistência.

FIN-010 local primário preservou pins/locks/approvals e 15 guards tracked; audit422/0/2, OSV542/3/0 passaram. Review primária, Gitleaks staged e CI exata pendentes; não inferir security de consumers inexistentes.

FIN-010 review primária Sol6 medium E2 concluída sem P0/P1/P2; diff focal enum/teste, sem consumidor novo. Gitleaks staged e CI final continuam pendentes. Estado histórico anterior à entrega final.

FIN-010 CI final passou Gitleaks178/0, Cargo422/0 vulnerabilidades/2 E4 aprovadas, OSV542/3 aprovadas/0 fora e SAST limitado a Biome/PowerShell; backend Rust não coberto por SAST.


## FIN-006 — constraints de migration (E3 focal)
Migration STRICT `003-accounts.sql` limita ID a BLOB16, aceita apenas quatro chaves explícitas de account_type, exige nome SQL não vazio e saldo INTEGER; o harness executou vazio/upgrade, audit preservado, reopen e integridade. CHECK `name <> ''` não implementa trim Unicode; validação de domínio pertence a consumidor futuro. Não há evidência de startup, consumer/UI, regra de receita FIN-005 ou enforcement fora do executor exercitado. Cargo/security scans e CI integrados pendentes.

FIN-006 coverage focal E3 passou os gates críticos de linhas/funções no database.rs. Não substitui security scans: Gitleaks staged, validators e CI integrada pendentes; audit/OSV locais permanecem evidência E3 registrada na spec/handoff.


FIN-006 CI: Gitleaks180/0, Cargo422/0 vulnerabilidades/2 E4 aprovadas, OSV542/3 aprovadas/0 fora. SAST cobre Biome/PowerShell, não backend Rust. Exceções vigentes até 2026-10-23T00Z.

CAT-002 executor done/E3/passes:true após CI exata 38616439285/job116032611143 SUCCESS 23/0 no SHA `3a94855181576ed354a73831dca3e1964641e79d`; Gitleaks staged 12 paths/30.621B/0; OSV542/3 aprovadas/0 fora; audit422/0 vulnerabilidades/2 aprovadas; NPM120/0 vulnerabilidades/0,700s; SAST45/4.453; review E2 sem P0/P1/P2. Reparo advisory-db e cleanup E3 preservam drafts/manifest externos (não apagar). Campos públicos exigem chamada explícita por consumers futuros; sem consumer, FIN-005, setup ou publicação final.

FIN-011 é conversão pura de `&str`, sem I/O, persistência ou trust boundary externo. Executor `done`/E3: CI source38621842911/SHA `809626e89dabbcce158fb466f63401c0a7b02c99` SUCCESS23/0; Gitleaks184/0, OSV542/3 approved/0 outside, Cargo422/0 vulnerabilidades/2 approved E4 warnings, NPM120/0 vulns, SAST45. Gitleaks staged12 paths/0 findings e quatro validators PASS. SAST não cobre backend Rust; nenhum consumer, SQL, UI/IPC ou parsing serializado provado. CI documental/nota84/liberação ACTIVE24 pendentes.

FIN-007 primary E3 valida rejeição `ConstraintViolation`, preservação e reopen em fixture SQLite temporária; migration e produção sem alterações. Audit/OSV e review E2 concluídos; staged Gitleaks, 4 validators e source CI ainda pendentes. Não prova consumer/startup ou regras comerciais.

Fechamento FIN-007: source CI `38628675813`/job `116074330758` SUCCESS23/0 no SHA `a58ad20b7664db5f7b5fb85a3cb7b0b341a46978`; Gitleaks187/0, OSV542/3 aprovadas/0 fora, Cargo audit422/0 vulnerabilidades/2 aprovadas, npm120/0, SAST45 (sem cobertura Rust), quatro validators PASS. Fixtures continuam sintéticas; sem consumer/startup inferidos.

FIN-008 primary E3: INSERTs usam parâmetros SQLite e account/audit compartilham `execute_write_transaction`; erros externos são mapeados a mensagens estáticas. Cargo audit422/0 vulnerabilidades/2 avisos E4 aprovados; OSV542/3 aprovados/0 fora; review Sol6 medium E2 sem P0/P1/P2. Isso cobre o diff e cenários focais exercitados; Gitleaks staged, quatro validators e CI source ainda pendentes. Sem claim de threat coverage global, UI/IPC, startup ou acesso ao banco da usuária.
## FIN-008 — evidência do executor (E3)
FIN-008 concluiu scanners do source: Gitleaks staged source 19 paths/0; Cargo audit 422/0 vulnerabilidades/2 aprovadas; OSV 542/3 aprovadas/0 fora. Source CI 38765481430 no SHA eea53ff05f2df34c5ff8e9aa759b70dc0a25ab57; No preparo deste recibo, o staged documental ainda não havia sido escaneado; o scan do conteúdo final será registrado na nota de fechamento.

## CAT-003 — integracao primaria 16/10/2026

Schema restrito e SQL trusted sem dados reais;preserva executor e migrations aplicadas;auditsCargo422/0vulnerabilidades/2E4,OSV542/3E4/0fora,NPM120/0,Gitleaks290/0,SAST45limitado. E4 validada nos inputs atuais ate2026-10-23T00Z;nao e seguranca global. Fonte: planning/handoffs/CAT-003.md e planning/specs/CAT-003.md, SHA7573c65899416cf180bb25545f6cfae6335bb00e; E3 local, CI de entrega em execucao, item ainda aberto.


CAT-003 executor done/E3/passestrue apos CI38907719490/job116995418974/attempt1/SHA7573c65899416cf180bb25545f6cfae6335bb00e SUCCESS23/0. Evidencia e limites em planning/handoffs/CAT-003.md;CI documental final/nota87/liberacao pendentes. FIN-012/UI-001 continuam abertos.


## FIN-012 — integração primária em 16/10/2026

Movement/MovementKind sem Debug, Display ou serde, com quatro guards compile-fail executados. Nenhum novo sink, dependência, IPC ou consumidor. Scans do código idêntico passaram; E4 existente restrita aos inputs aprovados e vigente até 2026-10-23T00Z. Fontes e evidência: planning/specs/FIN-012.md e planning/handoffs/FIN-012.md; source 7573c65, main 7cad038, produção idêntica. Item ainda aberto até fechamento documental.

FIN-012 executor done/E3/passes true após os checks e CI do código integrado, com fonte em planning/handoffs/FIN-012.md. CI documental final, nota88 e liberação pendentes. UI-001 aguarda liberação. Sem comprovação de consumidor, regras financeiras, coverage global ou instalador.


## UI-001 — verificação primária em 16/10/2026

Fonte: planning/handoffs/UI-001.md, source7573c65/main59a3789, E3 local.

```json
{
  "capabilities": "inalteradas, sem comando de negócio",
  "perfil": "WebView2 próprio confirmado e removido com containment/no-reparse após processos ausentes",
  "scans": "código idêntico já aprovado, sem nova dependência ou exceção"
}
```


UI-001 executor concluído: E3 de execução e inspeção nativa, checks do código idêntico e CI source38907719490 SUCCESS23/0; review integrado/visual E2 sem P0/P1/P2. Documentos e matrizes atualizados antes de passes true. Reserva retida até CI documental final no novo SHA, nota89/preservação88notas/Gitlimpo/WTarquivada e liberação do root. Código de saída nativo não registrado(null); sem claim exit0, WCAG numérico, mínimo de produto, jornada financeira, Rustglobal ou setup. Próximo após liberação: reconciliar elegibilidade FIN-005 no roadmap.


## FIN-009 — leitura de contas, E3 local em 17/10/2026

npm120/0 vulnerabilidades, Cargo422/0 vulnerabilidades/2E4, OSV542/3E4/0fora; SAST45casos frontend, sem scanner Rust. E4 revalidado expiry 2026-10-23. Inputs inalterados. RustSec drafts preservados. Erro opaco sem writes. Nenhuma validação nova.


FIN-009 executor done/E3/passestrue após CI39009285553/job117328105513/attempt1/SHA83e682303354c000d2319f512f8321882901aa35 SUCCESS23/0,97ordinários+8doctestsPASS/1ignored. Evidência local global459/470linhas46/49funções ecríticos95PASS não é coverage CI; statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4 atualizados; CI documental final/nota90/preservação89/liberação ainda pendentes. Sem UI/IPC/receita/setup; fonte planning/handoffs/FIN-009.md.


## FIN-013 — validação pura de entrada, E3 local em 17/10/2026

validate_entry recebe &self, não faz IO/logs/mutação/aritmética e retorna erro estático existente, sem serializar Movement. Clone/Copy fora do contrato foram removidos de Context; privacidade e construção raw preservadas. npm120/0, Cargo422/0vuln/2E4, OSV542/3E4/0fora, plugin e SAST45PASS; SAST não cobre Rust. E4 expirando23/10/2026 e hashes aprovados revalidados, sem novas exceções.


FIN-013 executor done/E3/passestrue após CI39021912847/job117365252558/attempt1/SHAa3dee6febce169d4dbac3779ce6a67c998e506ab SUCCESS23/0,102ordinários+8doctestsPASS/1ignored. Crítico domainmod24/24linhas5/5funções/controle rejeita95 é E3 local,não coverageCI/globalatual;statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4/validators atualizados;CIfinaldocumental/nota91/preservação90/liberação pendentes. SemUIIPC/persistência/receita/setup. Fonte planning/handoffs/FIN-013.md.


## FIN-014 — validação pura de saída, E3 local em 17/10/2026

Regra de saída usa &self e erro estático existente, sem payload financeiro, IO/logs/mutação ou aritmética. Nenhum derive novo; entrada e rawconstructor preservados. npm120/0, Cargo422/0vuln/2E4, OSV542/3E4/0fora, plugin e SAST45PASS (não scanner Rust). Exceções E4 revalidadas em hashes aprovados e expiram23/10/2026; nenhuma exceção nova.


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
## FIN-017 — persistência de Entry (planejado, E2)
| Superfície | Controle/verificação requerida | Estado |
|---|---|---|
| Entrada e dados | validar Entry antes de IDs/clock/store; SQL parametrizado; kind fixo; FK pela transação; exigir affected rows = 1 | pendente |
| Atomicidade e diagnóstico | movimento + audit event em `BEGIN IMMEDIATE`; abortar em qualquer falha; erros estáticos sem payload financeiro/SQL/source; adapter envenenado após falha de DB | pendente |
| Schema e exposição | migrations 002–005 imutáveis; sem IPC/UI/startup | pendente |


FIN-017 | E3 local:6unit+3SQLite/4falhas,criaçãoEntry+audit atômicas/reopen/rollback/poison/inputinválido sem efeitos;Rust121+8docs/debug+release/fmt/clippy/TS/securityE4 PASS;coveragecrítica app43/43linhas3/3fn eadapter51/51linhas5/5fn,controle0,rejeita95. Review independente concluído;CI source/final/liberação pendentes,in_progress/passesfalse. Sem jornadaUIIPC/globalRust/setup/E2E;apósFIN017METApausará,nenhum próximoID. Ver spec/handoffFIN017;custonão registrado.


FIN-017 | done/E3: source5a09ca2ec7117be7e39605bfe25848e7fc39acaf CI39090756071/job117566803340/attempt1success23/0skip/7marcadores exatos;121testesRust+8doctests. Entry+audit atômicos/6unit+3SQLite4falhas,critical app43/43linhas3/3fn+adapter51/51linhas5/5fn100%,controle0. Gateslocais/review/validators/stagedscan aprovados;docs antespassestrue. CI documental/nota95/liberação pendentes;WTarquivada,evidênciapreservada,locaisencerrados. DepoispausarMETA por decisão humana,nenhumpróximoID. ProdutoUIIPC/receitaFIN005/globalRust/setup.exe/E2E pendentes;custonão registrado.
SAL-020 | SECURITY/VERIFY local | E3 | Gate Cargo Audit 0.22.2 passou com DB RustSec `ee1a49a2769a24a596afc1c20cb33748d2769586`: 422 dependências, 0 vulnerabilidades, 2 avisos aprovados; uma invocação manual separada teve aviso de atualização do índice crates.io, sem afetar o gate pinned. OSV 2.6.0: 542 pacotes, 4 exceções aprovadas, 0 fora; validade até 2026-11-06. npm audit: 120/0; SAST 45 casos. Sem nova exceção/lock/dependência. SAST limita-se a frontend/PowerShell; não cobre SQL Rust/backend. Revisão interna sem defeito impeditivo, não independente; Gitleaks staged 0 achados, histórico, CI exata e nota pendentes.
