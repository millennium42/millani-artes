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
