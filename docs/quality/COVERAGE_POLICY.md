# Política de cobertura

Quando Vitest existir: global lines/statements/functions ≥90%, branches ≥85%. Arquivos críticos de saldo, transferência, venda, recebimento, devolução, reembolso, recorrência, transação e backup/restore: 95/95/95/90, aplicado por glob/per-file. Não reduzir para passar; exigir exceção humana/ADR.

Se Rust contiver domínio, medir cobertura de linhas e habilitar branches se o toolchain permitir; documentar limitação se não permitir. Sempre rodar `cargo fmt --check`, clippy com warnings erro, test e audit.
