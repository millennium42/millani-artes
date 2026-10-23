# Fronteiras de módulos

| Módulo | Responsabilidade | Não pode fazer |
|---|---|---|
| UI | interação, estado de tela, acessibilidade | SQL ou cálculo financeiro autoritativo |
| Aplicação | orquestrar caso de uso e transação | renderizar UI |
| Domínio | regras, invariantes e cálculos puros | acesso direto a SQLite/Tauri |
| Repositórios | consultas/persistência e constraints | decidir regra de negócio |
| Infraestrutura | SQLite, filesystem, criptografia, Tauri | reimplementar domínio |

Cada operação canônica de escrita — lançamento, transferência, finalizar venda, receber, pagar recorrência, devolver, reembolsar e restaurar — terá um único caso de uso.

O [modelo transacional](TRANSACTION_MODEL.md#fronteira-da-unidade--arc-004) define a propriedade da unidade financeira: caso de uso coordena validação do estado, repositórios no mesmo contexto, gravações, auditoria e resultado após commit. Repositórios não confirmam etapas independentemente; DB-011 fornece `execute_write_transaction` na infraestrutura, com BEGIN IMMEDIATE. E3 local e CI de entrega passaram; review independente E2 sem achados impeditivos. Os testes de DB-011 provam o contrato básico SQLite, não rollback de operação composta nem atomicidade financeira. Restore possui seu protocolo de arquivo e não presume proteção por rollback SQL. ARC-004 continua sendo a fronteira arquitetural da unidade; CI final/nota72 ainda pendentes.

## Estrutura atual — ARC-001

[Main](../../src-tauri/src/main.rs) continua compondo o bootstrap por [application/mod.rs](../../src-tauri/src/application/mod.rs) e [infrastructure/mod.rs](../../src-tauri/src/infrastructure/mod.rs); esses arquivos não mudaram. FIN-008 está exposto pela biblioteca em [lib.rs](../../src-tauri/src/lib.rs), nas namespaces `application::create_account` e `infrastructure::accounts`, implementadas em módulos próprios. Esse caso de uso não está ligado ao startup do binário nem à UI/IPC. [Domain](../../src-tauri/src/domain/mod.rs) mantém tipos/validação puros, sem projeção financeira implementada; [UI](../../src/ui/App.tsx) permanece sem fluxo FIN-008.

O probe local de ARC-001 compilou cópias de domain/application no SHA daquela tarefa e rejeitou seis controles de imports Tauri/infra/SQLite; não caracteriza o boundary atual de FIN-008 nem protege consultas em componentes (ARC-005). FIN-008 agora exercita um caso de uso e adapter de conta com SQLite real; outros consumers, startup e fluxo UI/IPC permanecem não demonstrados. [Spec ARC-001](../../planning/specs/ARC-001.md) delimita a evidência histórica.

ARC-001 executor done/E3/passes:true: [entrega](https://github.com/millennium42/millani-artes/commit/e94eec776213725f72239c8d07aff2a293f6f632)/[CI37863336200](https://github.com/millennium42/millani-artes/actions/runs/37863336200) success23steps/buildWindows, std-only6controles e7Rust locais/reviewE2fontes semachados. Domínio sem regra; finanças/SQLite/backup/setup.exe pendentes/METAativa. Reserva22ATIVA atéCI final/nota57, fechamento11docs; detalhe na spec/handoffARC001. ARC002 não iniciado.

## Contrato de erros — ARC-002

Executor E3 validado na entrega93362971/[CI37865774054](https://github.com/millennium42/millani-artes/actions/runs/37865774054); fechamento documental/reserva17 no [handoff](../../planning/handoffs/ARC-002.md). O contrato não demonstra invariantes financeiras.
[Lib](../../src-tauri/src/lib.rs) exporta domain, em vez de declaração duplicada no main. [DomainError/Result](../../src-tauri/src/domain/mod.rs) são biblioteca pura: InvalidInput sem payload e DisplayDOMAIN_INVALID_INPUT, Debug nome estático, Error semsource. Tipo padrãoResult preservaOk/propagaErr. [Testes](../../src-tauri/src/domain/tests.rs)/[spec](../../planning/specs/ARC-002.md) provam o contrato atual; não há produtor/validação financeira, erroSQLite ou mapperUI/IPC. Failure bootstrap permanece separado. Biblioteca pública Rust não concede comandoTauri.


## Portas de tempo e UUID — ARC-003
[domain::ports](../../src-tauri/src/domain/ports.rs) publica Clock/SystemTime e UuidGenerator/16bytes/erro associado opaco. Fornecedores substituíveis/dyn, sem geração global/default/IO/formatador. [Testes](../../src-tauri/src/domain/ports_tests.rs) conservam instantes, sequência e falha sem fallback. Adapter real pertence à infraestrutura futura, com prova de versão/unicidade/falhas; calendário/vencimento ainda não definido. [Spec/handoff](../../planning/handoffs/ARC-003.md): E3local/reviewCIpendentes/14ATIVA, sem caso financeiro.


ARC-003 executor E3 validado na entrega6eafaa40/[CI37963747801](https://github.com/millennium42/millani-artes/actions/runs/37963747801), root13Rust/5negativos/reviewE2 sem achados. Fechamento documental preserva fontes/thresholds/inputs; reserva14 até CI final/nota59 no [handoff](../../planning/handoffs/ARC-003.md). Portas não comprovam UUID real/unicidade/relógio monotônico/invariantes financeiras.

## Guard de SQL no frontend — ARC-005

[Configuração](../../biome.json), [regra local](../../scripts/no-ui-sql.grit) e [fixtures](../../scripts/test-sast.py) reutilizam Biome2.5.15. O lint rejeita imports de drivers reconhecidos, require (CommonJS), strings/templates com prefixos SQL e comandos literais plugin:sql| no frontend. Isso inclui componentes, entrypoint, helpers e testes sob src/. A regra Grit usa **/src/**: o filtro src/** não ativou o plugin no probe com o pin instalado; a fixture aninhada evita aceitar esse falso verde.

[Spec](../../planning/specs/ARC-005.md) registra RED/GREEN, padrões e limites. Não é análise de fluxo: SQL fragmentado/escapado passa nos controles conhecidos; texto JS iniciado por palavra SQL é rejeitado conservadoramente, enquanto comentário e JSX textual são permitidos. O gate não autoriza os contornos nem prova consultas parametrizadas, SQLite ou invariantes financeiras. Testes reais de repositórios continuam no plano DB. E3 local; review/CI/nota61 e liberação14 pendentes.


ARC-005 executor done/E3 após [entrega c5872e20](https://github.com/millennium42/millani-artes/commit/c5872e2026291512fd9eb879e3562200d72b7dd7)/[CI37974060505](https://github.com/millennium42/millani-artes/actions/runs/37974060505), tentativa1/23success.45casos SAST/42CLI/3PS, quality/audits/build Windows aprovados; [prova e limites](../../planning/specs/ARC-005.md#ci--learn--entrega-validada). Fechamento preserva fontes; reserva14 até CI final/nota61 pública. Guard de padrões não comprova SQLi/SQLite/finanças; META ativa/DB001 não iniciado.
## FIN-008 — evidência do executor (E3)
FIN-008 mantém criação em application e persistência/auditoria no adapter; UI, IPC e startup não integrados. Source CI 38765481430 no SHA eea53ff05f2df34c5ff8e9aa759b70dc0a25ab57.

## FIN-009 — leitura de contas, E3 local em 17/10/2026

O módulo porta AccountReadStore lista contas via adapter SQLite com implementação read_all_accounts. Consumer futuro sem implementação. Interface retorna Result<Vec<Account>>. Escopo limitado à leitura sem UI/IPC/startup. Adapter ganhou funcionalidade de leitura.


FIN-009 executor done/E3/passestrue após CI39009285553/job117328105513/attempt1/SHA83e682303354c000d2319f512f8321882901aa35 SUCCESS23/0,97ordinários+8doctestsPASS/1ignored. Evidência local global459/470linhas46/49funções ecríticos95PASS não é coverage CI; statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4 atualizados; CI documental final/nota90/preservação89/liberação ainda pendentes. Sem UI/IPC/receita/setup; fonte planning/handoffs/FIN-009.md.


## FIN-013 — validação pura de entrada, E3 local em 17/10/2026

Movement::validate_entry é regra pura no domínio: Entry positivo retorna Ok(()); outros kinds ou valor não positivo retornam DomainError::InvalidInput existente. Não valida UUID/conta/contexto nem escreve ou faz aritmética. Consumidores atuais: cinco testes de domínio; aplicação, persistência e UI/IPC ainda não chamam esse método.


FIN-013 executor done/E3/passestrue após CI39021912847/job117365252558/attempt1/SHAa3dee6febce169d4dbac3779ce6a67c998e506ab SUCCESS23/0,102ordinários+8doctestsPASS/1ignored. Crítico domainmod24/24linhas5/5funções/controle rejeita95 é E3 local,não coverageCI/globalatual;statementsnão medidos/branchesstableN/A. Docs/contratos/testes/review/scans/E4/validators atualizados;CIfinaldocumental/nota91/preservação90/liberação pendentes. SemUIIPC/persistência/receita/setup. Fonte planning/handoffs/FIN-013.md.


## FIN-014 — validação pura de saída, E3 local em 17/10/2026

Movement::validate_exit reutiliza o padrão puro de validate_entry: Exit positivo retorna Ok(()); zero, negativos, Entry e Transfer retornam DomainError::InvalidInput existente. Sem nova validação de UUID/conta/contexto, IO/logs/mutação/aritmética. Consumidores atuais são cinco testes de domínio; aplicação/persistência/UIIPC não chamam o novo método.


FIN-014 | CI source aprovado / fechamento funcional E3 | SHA c27b4eaf6c7c8462b0301b3489218ee927a83a89; run39026610317/job117378726603/attempt1 success,23 passos aprovados/0 ignorados,7 marcadores sourceSha exatos. Rust remoto107 ordinários(43lib+7bin+57SQLite)+8doctests PASS/1WebViewignored; artifact frontend único scaffold. PE AMD64/custom-protocol8.536.064 bytes SHA256 5c46f6e9944536be390c2937aebc1d4bb80aa1ece7b215124573fbef227eae51; não é instalador. Local fmt/clippy/debug/release/quality/build/scans/E4 aprovados; crítico domainmod29/29linhas6/6funções e controle24/29linhas5/6funções rejeita95. Revisão independente Sol6medium semP0/P1/P2. Quatro validadores415arquivos/28fontes/92itens PASS; stagedGitleaks14arquivos/703969bytes/0achados/framedSHA2562a94609b975ca6224b734c68833cd0a216a0ab410796d9f1ce31c6e6b2933602. E4 exata até23/10/2026, sem exceção nova. Evidência focal preservada em operacao/FIN-014-focal-evidence-20261017,9arquivos com hashes; WT arquivada recuperavelmente, scratch próprio removido. Sem código ou teste corrigido por GPT; Laguna local P-QA/P-DOM, integrador reproduziu gates. Custo não registrado. Esta entrega é validação pura de saída; não comprova jornada UI/IPC/persistência/receita, global Rust atual ou setup.exe. CI final/nota92 preservando91/97branches/checkpoint/Gitlimpo/liberação ainda pendentes; reserva ATIVA e próximo ID bloqueado até esse fechamento.
## FIN-017 — caso de uso e adapter (planejado, E2)
`application::create_entry` cria `Movement` de Entry, chama `validate_entry` antes de efeitos, obtém IDs/instante pelas portas existentes e delega a `EntryCreationStore`; devolve o movimento somente depois do commit. `infrastructure::movements::AdapterSqliteMovementStore` valida novamente o contrato antes de consumir a conexão, grava movimento e audit event na unidade SQLite existente. Não adiciona IPC, UI, startup nem regra de saldo.


FIN-017 | E3 local:6unit+3SQLite/4falhas,criaçãoEntry+audit atômicas/reopen/rollback/poison/inputinválido sem efeitos;Rust121+8docs/debug+release/fmt/clippy/TS/securityE4 PASS;coveragecrítica app43/43linhas3/3fn eadapter51/51linhas5/5fn,controle0,rejeita95. Review independente concluído;CI source/final/liberação pendentes,in_progress/passesfalse. Sem jornadaUIIPC/globalRust/setup/E2E;apósFIN017METApausará,nenhum próximoID. Ver spec/handoffFIN017;custonão registrado.


FIN-017 | done/E3: source5a09ca2ec7117be7e39605bfe25848e7fc39acaf CI39090756071/job117566803340/attempt1success23/0skip/7marcadores exatos;121testesRust+8doctests. Entry+audit atômicos/6unit+3SQLite4falhas,critical app43/43linhas3/3fn+adapter51/51linhas5/5fn100%,controle0. Gateslocais/review/validators/stagedscan aprovados;docs antespassestrue. CI documental/nota95/liberação pendentes;WTarquivada,evidênciapreservada,locaisencerrados. DepoispausarMETA por decisão humana,nenhumpróximoID. ProdutoUIIPC/receitaFIN005/globalRust/setup.exe/E2E pendentes;custonão registrado.


CAT-004: application::create_category/CategoryCreationStore e infrastructure::categories::AdapterSqliteCategoryStore expostos na lib. Criação ativa com validação antes de efeitos e categoria+audit na mesma transação existente; erroDB descartaadapter. Sem novo caminho de escrita UI/IPC/startup. E3 local;CI source/final pendentes; detalhes planning/handoffs/CAT-004.md.


CAT-004 executor done/E3/passestrue após CI39094721622 no SHAcbf40d8392e66a38146ebfd6abb61cd42766d47e success23/0 e130Rust+8docs; detalhes e limites no handoff CAT-004. CI documental final/nota96/liberação pendentes. MVP02 criação backend parcial; semUIIPC/E2E/setup. MetaE2Eativa.


CAT-005: application::rename_category/CategoryRenamingStore reutilizam validação e portas existentes. AdapterSqliteCategoryStore lê is_active e atualiza somente name com audit na mesma transação; preserva ID/estado e retorna após commit. NotFound permite reuso, falha DB descarta adapter. E3 local; sem alteração na criação ou novo caminho UI/IPC. Fonte: planning/handoffs/CAT-005.md.


CAT-005 executor done/E3/passestrue após CI39098498169 no SHAaef71f2308dbfa1d11f4960568fc0b041d1e7221 success23/0 e141Rust+8doctests, incluindo8unit+3SQLite. Gates locais, cobertura crítica >=95 e controle, revisão independente e validators/stagedscan aprovados; detalhes no handoff CAT-005. CI documental final/nota97/liberação pendentes. MVP02 renomeação backend parcial; UIIPC/desativação/E2E/setup pendentes. Meta E2E ativa.


CAT-006: application::deactivate_category/CategoryDeactivationStore → AdapterSqliteCategoryStore → SELECT name e UPDATE somente is_active=0 + audit atômico. Preserva ID/nome/linha, já inativa aceita, NotFound permite reuso e erroDB descarta conexão; criação/rename idênticos. E3 local; sem UIIPC. Fonte: handoffCAT006.


CAT-006 done/E3/passestrue após CI39128570976 no SHA25311c0fb01d066e39d557e3779287d680553c7f,23success/0skip e151Rust+8docs;7unit+3SQLite e gateslocais/review/validators/stagedscan PASS. Fonte: handoffCAT006. CI documental final/nota98/liberação pendentes. MetaE2Eativa; UIIPC/histórico consumidor/setup/E2E pendentes.


FIN-018: create_exit/ExitCreationStore→mesmoAdapterSqliteMovementStore→movimentoExit+audit atômicos;validação antes de portas/consumo,2IDs/Clockchecked,row1/rollback/poison/invalidreuse. ExitStoreError próprio mantém Entry intacto;sem cálculo de saldo/UIIPC. E3local;handoffFIN018.


FIN-018 done/E3/passestrue após CI39134383058/SHA4f7e7703793ac8ef5632935ade79badd3f9d44fasuccess23/0skip e162Rust+8docs,6unit+3SQLite. Saída+audit atômicos/rollback4faults/poison e gates/review/validators/stagedscan PASS. CI documental final/nota100/liberação pendentes;MetaE2Eativa. Sem saldo/receitaFIN005/UIIPC/globalRust/setup/E2E. Fonte:handoffFIN018.


FIN-019 Movement::validate_transfer puro: Transfer positivo=>Ok,demaisDomainErrorInvalidInput existente;sem mutação/IO/cálculo. Endpoints são metadados,FIN020/021/022 separados. APIEntry/Exit idêntica. E3local;handoffFIN019.


FIN-019 done/E3/passestrue após CI39136146173/SHA7ac497730b507c5576b05bff8f285169215b21cbsuccess23/0skip e167Rust+8docs,5unit. Validação pura Transferpositivo e gates/review/validators/stagedscan PASS. CI documental final/nota101/liberação pendentes;MetaE2Eativa. FIN020/021/022/UIIPC/globalRust/setup/E2E pendentes. Fonte:handoffFIN019.


FIN-020 amplia somenteguard de Movement::validate_transfer:amount>0 eorigem!=destino nos16bytes,erro existente;semIO/mutação/cálculo. Contratosanteriores intactos;FIN021/022/023 separados. E3local;handoffFIN020.


FIN-020 done/E3/passestrue após CI39137975946/SHA78beb2f62b024899d923b136251d3907b937b82asuccess23/0skip/7marcadores,Rust169+8docs/7transferunit(2novos). Contasiguaisrejeitadas/contasdistintas/metadados preservados,gates/review/validators/stagedscanPASS. CI final/nota102/liberaçãopendentes;MetaE2Eativa;FIN021/022/023/UIIPC/globalRust/setup/E2Ependentes. Fonte:handoffFIN020.


FIN-021 application::create_transfer→TransferCreationStore→AdapterSqliteMovementStore,Move+Audit mesmatransação/2contas/account+categoryNULL;domínio validaantesportas,semcálculos/migrationnova. Entry/Exitintactos;E3local;handoffFIN021.


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


PRD-004 VERIFY E3:MVP07 migration8ProductSTRICT5campos/zero válido/negativosrejeitados/2constantesRustzeroexec/5SQLitefísicosREDGREEN+2mutantes/gates/security/reviewPASS. LLVMdatabase>=95controle semTODOSSQLite rejeitado;6SQLanteriores+v1vazia/executores/domíniointactos. CIs/nota132/liberaçãopendentes;Metaativa;fonte:handoffPRD-004.


PRD-004 done/E3/passestrue apósCI39292809030/SHAc93e947e0a1b4d2fdea9793aa02a25276022fef5success23/0skip/7marcadores,Rust327+20docs/5SQLnovos. Migration8productsSTRICT/zero válido/2constantesRustzeroexec/REDGREEN+2mutantes/LLVMdatabasecomcontrole/gates/security/reviewPASS;executores/domínio/6SQLanteriores+v1vaziaintactos;CIfinal/nota132/liberaçãopendentes;PRD005..007(criar/alterar/desativar),vendas/snapshots/INVSAL005,UIIPC,coberturaRustglobal,releaseWindows/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. SQLname<>vazio não replicaUnicodeblank do domínio. STRICT permite coerções sem perda;sem percentualSQL/globalRust/CI/statements/branchesstable.;Metaativa;fonte:handoffPRD-004.


PRD-005 VERIFY E3:MVP07 criaçãoProductativo/validaçãonomepreçoantesportas/SQL+auditIMMEDIATEparametrizado/rows1/poisoner-freshrecovery/7unit4SQLnovosREDGREEN/gates/security/reviewPASS. LLVM2arquivos>=95controleomitenovosrejeitado;7SQLmigrationsanteriores+v1vazia/domíniointactos. CIs/nota133/liberaçãopendentes;Metaativa;fonte:handoffPRD-005.


PRD-005 done/E3/passestrue apósCI39298191906/SHA0ecf5a59905791a2eae452716c0b8c2220207246success23/0skip/7marcadores,Rust338+20docs/7unit4SQLnovos. CriaçãoProductativo/validaçõesantesportas/SQL+auditIMMEDIATE/rows1/poisoner-freshrecovery/REDGREEN/LLVM2arquivoscomcontrole/gates/security/reviewPASS;7migrationsSQL+v1vazia/domainintactos;CIfinal/nota133/liberaçãopendentes;PRD006/007(editar/desativar),vendas/snapshots/INVSAL005,UIIPC,coberturaRustglobal,releaseWindows/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. UUIDversão/uniqueness pertencem ao adapterUuidGenerator;zeroaudit/entity/timeinválidos são rejeitados porappend_audit_event e revertidosSQL. Stablebranches/statements/global/CI Rustcoverage não medidos.;Metaativa;fonte:handoffPRD-005.


PRD-006 E3 local: application/edit_product.rs depende somente de Product/UuidGenerator/Clock/ProductEditingStore. Valida nome/preço antes das portas, retorna NotFound/erro opaco. AdapterSqliteProductStore implementa edição e preserva criação PRD-005; UPDATE parametrizado e product_edit audit compartilham transação IMMEDIATE. UI/IPC continuam pendentes. Fonte: planning/handoffs/PRD-006.md; item não liberado, aceite de segurança e CIs pendentes.


PRD-006 VERIFY E3 local aprovado23out: ediçãoProduct/9unit5SQL/352Rust+20docs/LLVM2arquivos100%controle rejeitado/gates/security/reviewPASS. Vite8.3.3/Tokio1.53.3;E4somente2exceçõesWindows/hashescorrentes/antes06novUTC conforme aceite da proposta c86bf6753ee0bb8d579e74e699e4bab108222835. ProduçãoGPTfallback0saídaslocaisusadas;2P2docs corrigidos. CIs/nota134/liberação pendentes/passesfalse;INVSAL005SAL008/E2Ependentes;fonte:handoffPRD-006.


PRD-006 done/E3/passestrue apósCI39998764928/SHA46ddaf405b7dc4a4ab7b7b0683d87a6fd47d9c0esuccess23/0skip/7marcadores,Rust352+20docs/9unit5SQLnovos. EdiçãoProduct/validaçõesantesportas/IDativopreservados/UPDATE+auditIMMEDIATE/rows1/NotFound/poisoner-freshrecovery/REDGREEN/LLVMapp+adaptercomcontrole/gates/security/reviewPASS;PRD005/migrations/domainintactos;CIfinal/nota134/liberaçãopendentes;PRD007(desativar),vendas/snapshotsreais/INVSAL005provaSAL008,UIIPC,coberturaRustglobal,releaseWindows/instalação/atualização/backuprestauração/hashassinatura/E2Eoficial pendentes. Metadataeditpreservaativo inclusiveinativo,padrãoCategoryrename;nenhumasnapshotfictíciausada. Stablebranches/statements/global/CI Rustcoverage não medidos.;Metaativa;fonte:handoffPRD-006.


PRD-007 VERIFY E3: desativação Product/7 unit+5 SQL/364 Rust+20 doctests/LLVM dois arquivos 100% e controle rejeitado/gates de qualidade e segurança PASS; P-REV E2 sem achados. Uma inferência local P-DAT/121,752s usada sem correção de comportamento; app por reuso determinístico de Category. CIs/nota135/liberação pendentes; passesfalse; E2E oficial pendente; fonte: handoff PRD-007.


PRD-007 done/E3/passestrue após CI 40004684967/SHA b861d473dc1b4fb26771ea7d600923a5d1b397fc, success23/0 skip/7 marcadores; Rust364+20 docs/7 unit+5 SQL novos. Desativação preserva metadados, aceita já inativo com nova auditoria; UPDATE+audit IMMEDIATE/rows1/NotFound/rollback/poisoner/fresh recovery; criação/edição anteriores preservadas. RED/GREEN/LLVM app+adapter com controle/gates/security/review PASS. CI final/nota135/liberação pendentes; Meta ativa. Snapshots reais de venda/INV-SAL-005 dependem de SAL-008; jornadas UI/IPC, cobertura Rust global e instalador Windows com instalação/atualização/backup/restauração/hash/assinatura/E2E oficial pendentes. LLVM local mede somente linhas/funções dos dois arquivos inteiros; branches/statements/global/CI Rust coverage não medidos. Fonte: handoff PRD-007.
