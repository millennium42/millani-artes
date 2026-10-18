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
