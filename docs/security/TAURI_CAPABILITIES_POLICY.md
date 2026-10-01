# Política Tauri capabilities

Começar deny-by-default. Não habilitar `shell`, rede, updater ou filesystem arbitrário por conveniência. Dialog e filesystem futuro devem limitar comandos, janelas e diretórios necessários; cada capability referencia um `SEC-*`, ADR e teste. CSP bloqueia conteúdo remoto por padrão.
