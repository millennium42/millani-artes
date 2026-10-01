# Arquitetura

Aplicativo Windows Tauri 2 com React, TypeScript strict e Vite. SQLite é o único banco local compartilhado. A UI chama casos de uso; regras puras vivem no domínio; persistência fica em repositórios; o core Tauri expõe somente comandos/capabilities necessárias.

O artefato de distribuição do MVP será um instalador Windows `setup.exe` (NSIS), gerado no Windows. A aplicação instalada funciona sem rede; a estratégia para instalar WebView2 será testada no work item de release. O desenho é candidato aprovado, não implementação existente.
