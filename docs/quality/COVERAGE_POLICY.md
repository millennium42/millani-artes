# Política de cobertura

Quando Vitest existir: global lines/statements/functions ≥90%, branches ≥85%. Arquivos críticos de saldo, transferência, venda, recebimento, devolução, reembolso, recorrência, transação e backup/restore: 95/95/95/90, aplicado por glob/per-file. Não reduzir para passar; exigir exceção humana/ADR.

Se Rust contiver domínio, medir cobertura de linhas e habilitar branches se o toolchain permitir; documentar limitação se não permitir. Sempre rodar `cargo fmt --check`, clippy com warnings erro, test e audit.


## Aplicação no inventário — INF-007

O inventário TS desta revisão contém App.tsx/main.tsx de bootstrap, testes e configs; não contém cálculo de saldo, transferência, venda, recebimento, devolução, reembolso, recorrência, transação ou backup/restore. Logo há zero arquivos críticos atuais, sem aplicar o limite crítico ao bootstrap. main.rs também contém apenas inicialização Tauri, sem domínio para medir coverageRust.

vitest.config.ts aplica o global90/90/90/85 a todo src TS/TSX, inclusive não importado, excluindo somente testes. npm run coverage emite texto no console e grava json-summary, LCOV e HTML em coverage/; npm test conserva coverage habilitada. Relatórios são locais/ignorados, não são aprovacao financeira, e uploadCI segue INF013.

Cada work item que introduzir lógica crítica deve, antes de passes=true:
- identificar os paths reais pela função de negócio/implementação observada, incluindo infraestrutura de backup/restore quando aplicável;
- acrescentar esses paths ou globs comprovadamente abrangentes em coverage.thresholds, com perFile:true e lines95/statements95/functions95/branches90; não depender só de média do grupo;
- registrar classificação, testes explícitos de invariantes e evidência na matriz/spec/handoff, verificar que todos os arquivos críticos estão abrangidos e executar os gates;
- para domínio Rust, aplicar a regra Rust acima e documentar a capacidade/limitação real do toolchain.

ARCHITECTURE/MODULE_BOUNDARIES define responsabilidades, sem fixar diretórios TS; ARC001 e implementações posteriores definirão paths. Não pré-cadastrar caminhos fictícios. Quando surgir arquivo crítico, a ausência do gate porarquivo é pendência daquele item, não exceção implícita. Limiares acima permanecem canônicos.

O mecanismo nativo Vitest5.0.3 foi exercitado com arquivos sintéticos temporários: média global/glob≥95 não rejeita um arquivo mal coberto; perFile:true identifica/rejeita esse arquivo abaixo95/95/95/90, e cobertura completa passa. Esses probes e seu glob não persistem; não afirmam implantação financeira. [Reprodução](../operations/REPRODUCING.md) e [handoff](../../planning/handoffs/INF-007.md).
