# Segurança de backup

Backup precisa de confidencialidade, integridade, versão e validação. Restore verifica metadados/autenticidade/formato antes de tocar no banco ativo, trabalha em cópia controlada e só promove após `integrity_check` e testes de leitura. Falha preserva o banco anterior e registra erro sanitizado.
