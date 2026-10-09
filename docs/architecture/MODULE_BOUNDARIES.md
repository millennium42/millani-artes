# Fronteiras de módulos

| Módulo | Responsabilidade | Não pode fazer |
|---|---|---|
| UI | interação, estado de tela, acessibilidade | SQL ou cálculo financeiro autoritativo |
| Aplicação | orquestrar caso de uso e transação | renderizar UI |
| Domínio | regras, invariantes e cálculos puros | acesso direto a SQLite/Tauri |
| Repositórios | consultas/persistência e constraints | decidir regra de negócio |
| Infraestrutura | SQLite, filesystem, criptografia, Tauri | reimplementar domínio |

Cada operação canônica de escrita — lançamento, transferência, finalizar venda, receber, pagar recorrência, devolver, reembolsar e restaurar — terá um único caso de uso.

## Estrutura atual — ARC-001

[Main](../../src-tauri/src/main.rs) compõe [application](../../src-tauri/src/application/mod.rs) e [infrastructure](../../src-tauri/src/infrastructure/mod.rs). Application orquestra apenas inicialização/hook com erro opaco e callback técnico; infrastructure contém Tauri e o catálogo estático de stderr. [Domain](../../src-tauri/src/domain/mod.rs) é uma raiz compilada sem regras implementadas. [UI](../../src/ui/App.tsx) contém a tela existente, ligada pelo entrypoint React.

O probe local compila cópias exatas de domain/application somente com rustc/std e rejeita seis controles de imports Tauri/infra/SQLite. Isso verifica essas raízes no SHA observado, sem substituir proteção futura de SQL em componentes (ARC-005). Repositórios, persistência e casos de uso financeiros ainda não implementados; [spec](../../planning/specs/ARC-001.md) delimita a prova.

ARC-001 executor done/E3/passes:true: [entrega](https://github.com/millennium42/millani-artes/commit/e94eec776213725f72239c8d07aff2a293f6f632)/[CI37863336200](https://github.com/millennium42/millani-artes/actions/runs/37863336200) success23steps/buildWindows, std-only6controles e7Rust locais/reviewE2fontes semachados. Domínio sem regra; finanças/SQLite/backup/setup.exe pendentes/METAativa. Reserva22ATIVA atéCI final/nota57, fechamento11docs; detalhe na spec/handoffARC001. ARC002 não iniciado.

## Contrato de erros — ARC-002

Executor E3 validado na entrega93362971/[CI37865774054](https://github.com/millennium42/millani-artes/actions/runs/37865774054); fechamento documental/reserva17 no [handoff](../../planning/handoffs/ARC-002.md). O contrato não demonstra invariantes financeiras.
[Lib](../../src-tauri/src/lib.rs) exporta domain, em vez de declaração duplicada no main. [DomainError/Result](../../src-tauri/src/domain/mod.rs) são biblioteca pura: InvalidInput sem payload e DisplayDOMAIN_INVALID_INPUT, Debug nome estático, Error semsource. Tipo padrãoResult preservaOk/propagaErr. [Testes](../../src-tauri/src/domain/tests.rs)/[spec](../../planning/specs/ARC-002.md) provam o contrato atual; não há produtor/validação financeira, erroSQLite ou mapperUI/IPC. Failure bootstrap permanece separado. Biblioteca pública Rust não concede comandoTauri.


## Portas de tempo e UUID — ARC-003
[domain::ports](../../src-tauri/src/domain/ports.rs) publica Clock/SystemTime e UuidGenerator/16bytes/erro associado opaco. Fornecedores substituíveis/dyn, sem geração global/default/IO/formatador. [Testes](../../src-tauri/src/domain/ports_tests.rs) conservam instantes, sequência e falha sem fallback. Adapter real pertence à infraestrutura futura, com prova de versão/unicidade/falhas; calendário/vencimento ainda não definido. [Spec/handoff](../../planning/handoffs/ARC-003.md): E3local/reviewCIpendentes/14ATIVA, sem caso financeiro.
