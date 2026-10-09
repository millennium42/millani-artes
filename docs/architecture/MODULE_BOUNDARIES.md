# Fronteiras de módulos

| Módulo | Responsabilidade | Não pode fazer |
|---|---|---|
| UI | interação, estado de tela, acessibilidade | SQL ou cálculo financeiro autoritativo |
| Aplicação | orquestrar caso de uso e transação | renderizar UI |
| Domínio | regras, invariantes e cálculos puros | acesso direto a SQLite/Tauri |
| Repositórios | consultas/persistência e constraints | decidir regra de negócio |
| Infraestrutura | SQLite, filesystem, criptografia, Tauri | reimplementar domínio |

Cada operação canônica de escrita — lançamento, transferência, finalizar venda, receber, pagar recorrência, devolver, reembolsar e restaurar — terá um único caso de uso.

O [modelo transacional](TRANSACTION_MODEL.md#fronteira-da-unidade--arc-004) define a propriedade da unidade financeira: caso de uso coordena validação do estado, repositórios no mesmo contexto, gravações, auditoria e resultado após commit. Repositórios não confirmam etapas independentemente; a infraestrutura implementará a transação em DB-011. Restore possui seu protocolo de arquivo e não presume proteção por rollback SQL. ARC-004 define este contrato documental, sem executor ou prova de atomicidade.

## Estrutura atual — ARC-001

[Main](../../src-tauri/src/main.rs) compõe [application](../../src-tauri/src/application/mod.rs) e [infrastructure](../../src-tauri/src/infrastructure/mod.rs). Application orquestra apenas inicialização/hook com erro opaco e callback técnico; infrastructure contém Tauri e o catálogo estático de stderr. [Domain](../../src-tauri/src/domain/mod.rs) é uma raiz compilada sem regras implementadas. [UI](../../src/ui/App.tsx) contém a tela existente, ligada pelo entrypoint React.

O probe local compila cópias exatas de domain/application somente com rustc/std e rejeita seis controles de imports Tauri/infra/SQLite. Isso verifica essas raízes no SHA observado, sem substituir proteção futura de SQL em componentes (ARC-005). Repositórios, persistência e casos de uso financeiros ainda não implementados; [spec](../../planning/specs/ARC-001.md) delimita a prova.

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
