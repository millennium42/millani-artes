# Requisitos de segurança

- SEC-001: negar capabilities Tauri por padrão.
- SEC-002: usar queries parametrizadas e constraints.
- SEC-003: não registrar dados pessoais/financeiros ou segredos em logs.
- SEC-004: tratar arquivo externo como não confiável; validar antes de uso.
- SEC-005: restore não substitui banco atual antes de validação e integridade.
- SEC-006: operações financeiras compostas são atômicas.
- SEC-007: scans de segredo e dependência são gates de CI quando tooling existir.
- SEC-008: conteúdo remoto, shell e filesystem amplo exigem ADR e capability específica.
