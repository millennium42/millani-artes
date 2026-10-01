# Arquitetura

Aplicativo Windows Tauri 2 com React, TypeScript strict e Vite. SQLite é o único banco local compartilhado. A UI chama casos de uso; regras puras vivem no domínio; persistência fica em repositórios; o core Tauri expõe somente comandos/capabilities necessárias.

O desenho é candidato aprovado, não implementação existente. Toda dependência será confirmada no work item de ambiente correspondente.
