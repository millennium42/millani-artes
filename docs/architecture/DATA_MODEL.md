# Modelo de dados inicial

Entidades previstas: `account`, `category`, `financial_movement`, `recurrence`, `product`, `sale`, `sale_item`, `customer`, `receivable`, `receipt`, `return`, `refund`, `audit_event`, `schema_migration` e metadados de backup.

Dinheiro será inteiro em centavos (ADR-013); valores agregados são consultas/projeções. Chaves, constraints, índices e migrations concretos ficam para `DB-*`, depois de uma spec e testes de banco vazio/upgrade.


DB-013 implementa `audit_event` técnico com IDs opacos de 16 bytes, instante Unix ms UTC e tags estáticas de operação/entidade. Não armazena payload financeiro nem prova invariantes financeiras; detalhes e limites em [spec](../../planning/specs/DB-013.md).

FIN-001 executor done/E3 após CI `38444911363` no SHA `31af12adcf8bebe617539d9e5dde286a64674deb`; `Account` contém somente `id: [u8; 16]` e `name: String`, com teste de preservação exata. Não define validação, tipo, saldo inicial, schema SQLite ou persistência. CI final documental e liberação pendentes; invariantes financeiras E0. [Spec e handoff](../../planning/specs/FIN-001.md).

FIN-003 VERIFY local E3: Account agora inclui account_type: AccountType com Cash, BankPix, Card e Other; representação pública em memória sem derive/default/tag numérica/conversor. Tipo não é contexto Casa/Fábrica; sem persistência/schema/serde definido. Hashes e limites no [handoff](../../planning/handoffs/FIN-003.md); CI/Gitleaks staged/validators finais pendentes.

CI de entrega FIN-003 (02c86f9): AccountType e campo compilaram e os testes de tipo/name passaram. Contrato continua apenas representação em memória; sem schema, serde, persistência ou semântica de saldo. FIN-004 futuro trata saldo inicial; contexto permanece separado.

FIN-004 VERIFY local E3: Account armazena initial_balance_cents como i64, centavos sem conversão/cálculo. Valores negativos/zero são representáveis, não uma regra comercial aprovada. Sem schema/persistência; FIN-005 avalia classificação do saldo inicial.


FIN-004 — medição de qualidade: campo initial_balance_cents permanece i64 em memória, sem cálculo ou classificação financeira. A medição Rust global E3 alcançou 301/312 linhas e 30/33 funções; não prova persistência, consumidores financeiros ou setup.

FIN-004 CI final de entrega no SHA 9e2068b passou; initial_balance_cents continua apenas valor i64 em memória. Sem consumidor real de receita, persistência ou cálculo demonstrado; FIN-005 permanece sem prova.

CAT-001 integração: Category representa somente id [u8;16], name String e is_active bool. O booleano não demonstra workflow de ativação/desativação, histórico, schema ou persistência. Integração primária ainda aguarda suíte e CI.

CAT-001 integração primária VERIFY: Category mantém somente ID, nome e bool; 61 testes+4 compile_fail doctests passaram, sem provar fluxo de ativação, persistência, histórico ou schema.

CAT-001 CI final c4def86 confirma Category como representação em memória (ID, nome, bool) e testes associados; não prova workflow, schema, persistência ou história de categorias. MVP permanece parcial.

FIN-010 representa contexto somente como enum Home/Factory no domínio; não decide campo em Account, serialização ou persistência. Evidência local standalone não confirma integração/testes primários.

FIN-010 local primário: Context Home/Factory compilou e suíte62+4 doctests passou. Enum continua somente classificação; sem consumer, Account.context, conversão externa ou persistência.

FIN-010 E3: Context Home/Factory permanece enum classificatório puro. CI run38512646302 passou; não prova consumer, persistência ou conversão financeira.


## FIN-006 — persistência inicial de contas (E3 local)
A migration `003-accounts.sql` registra quatro campos: ID opaco BLOB16, nome TEXT, chave técnica `account_type` (`cash`, `bank_pix`, `card`, `other`) e saldo inicial em centavos INTEGER. Testes físicos confirmaram round-trip, inclusive nome Unicode com prefixo NUL e extremos i64, vazio sem seed e upgrade com audit preservado; 43 PASS em SQLite 3.53.2. As chaves não são ordinais e o inteiro não aprova regra financeira. Sem consumer/startup/UI demonstrado; FIN-005 receita continua sem prova.

FIN-006 coverage focal E3 confirma gates ADR-019 na produção database.rs: 276/280 linhas e 25/25 funções; statements não medidos, sem vetor/global claim. O teste físico não demonstra app consumer, startup ou receita.


FIN-006 CI final 38570028370 passou testes da migration e do executor. O registro inicial de conta não demonstra consumo pela aplicação, startup, UI ou regra de receita FIN-005.

CAT-002 executor done/E3/passes:true: CI 38616439285 no SHA `3a94855181576ed354a73831dca3e1964641e79d` passou 67 testes +4 compile-fail doctests; 1 WebView ignored; guard/category tests PASS. `validate_name` permanece validação explícita em memória. Não prova consumer, persistência, ativação, FIN-005 ou setup; publicação final pendente.

## FIN-011 — conversão interna de contexto (E3 focal WT)
`Context` continua enum classificatório `Home`/`Factory`, separado de conta (ADR-008). Conversão interna `TryFrom<&str>` aceita exatamente `home`/`factory`; não estabelece storage key, contrato externo, contexto em Account ou persistência. Executor FIN-011 `done`/E3: primary source CI 38621842911 no SHA `809626e89dabbcce158fb466f63401c0a7b02c99` passou 70 testes+4 doctests/1 WebView ignored; três testes Context/junction passaram. Sem inferir consumo financeiro. CI documental e liberação ACTIVE24 pendentes.

FIN-007 primary E3 exercitou constraints da migration 003 imutável: 16 classes de linha inválida e ID duplicado; 45 SQL tests PASS no primary. Nenhuma mudança de schema ou regra de produto; source CI ainda pendente.

FIN-007 source CI `38628675813` no SHA `a58ad20b7664db5f7b5fb85a3cb7b0b341a46978` passou; registro acima descreve os testes do executor, sem inferir regra/consumer. Publicação documental permanece pendente.

## FIN-008 — gravação de conta e auditoria na aplicação (E3 primary)
O caso de uso valida nome, injeta IDs/relógio e usa adapter SQLite parametrizado para persistir os quatro campos de Account junto de um audit_event na mesma transação. Cargo executou 48 testes SQL; casos FIN-008 cobrem reopen e rollback de account/audit. Nenhuma migration/schema foi alterada. Isso prova o caminho focal exercitado, não startup, UI/IPC, projeção de saldo/receita, INV-FIN-001 ou consumer posterior; Gitleaks staged, validators e CI ainda pendentes.
## FIN-008 — evidência do executor (E3)
FIN-008 confirma a persistência dos quatro campos Account sem inferir receita do saldo inicial. Source CI 38765481430 no SHA eea53ff05f2df34c5ff8e9aa759b70dc0a25ab57; INV-FIN-001 segue sem prova.

## CAT-003 — integracao primaria 16/10/2026

Tabela main.categories STRICT v4 com idBLOB16/nameTEXTnaovazio NUL-safe/is_activeINTEGER0|1; constantes registradas e executor/migrations001-003 preservados. Fonte: planning/handoffs/CAT-003.md e planning/specs/CAT-003.md, SHA7573c65899416cf180bb25545f6cfae6335bb00e; E3 local, CI de entrega em execucao, item ainda aberto.


CAT-003 executor done/E3/passestrue apos CI38907719490/job116995418974/attempt1/SHA7573c65899416cf180bb25545f6cfae6335bb00e SUCCESS23/0. Evidencia e limites em planning/handoffs/CAT-003.md;CI documental final/nota87/liberacao pendentes. FIN-012/UI-001 continuam abertos.
