# Backup e restore

Operação futura: criar cópia consistente → criptografar → validar → gravar destino local. Restore: selecionar arquivo → validar sem substituir ativo → copiar ativo para preservação → restaurar em staging → `integrity_check` e leitura mínima → promoção atômica. Qualquer falha preserva o ativo.
