# Registro de ferramentas

| ferramenta | finalidade | versão | fonte/licença | status | dados/telemetria | risco | alternativa | remoção | evidência |
|---|---|---:|---|---|---|---|---|---|---|
| Git | versionamento | não registrado | git-scm, GPL-2.0 | E3 | arquivos locais; sem telemetria conhecida | perda por comando destrutivo | nenhum | desinstalar SO | `git init` executado |
| RTK | reduzir ruído de comandos | 0.48.0 | rtk-ai; licença a verificar | E3 | comandos/saída local | filtragem pode ocultar detalhe | shell direto | desinstalar perfil | `rtk --version` |
| project-kickoff | kickoff de projeto | revisão 62face4 | millennium42/about-my-tool-box; licença a verificar | E2 | download GitHub | instruções desatualizadas | regras locais | remover diretório de skills | fonte lida; instalador confirmou |
| granular-delivery | entrega auditável | revisão 62face4 | millennium42/about-my-tool-box; licença a verificar | E2 | download GitHub | instruções desatualizadas | regras locais | remover diretório de skills | fonte lida; instalador confirmou |
| honest-verification | evitar alegações sem prova | revisão 62face4 | millennium42/about-my-tool-box; licença a verificar | E2 | download GitHub | instruções desatualizadas | regras locais | remover diretório de skills | fonte lida; instalador confirmou |
| Node/npm, Rust/Cargo, Tauri, Vitest, Biome, Playwright | build/qualidade futura | não registrado | fontes oficiais; licença a verificar | E0 | a avaliar | supply chain | n/a | a definir | não instalados |
| Gitleaks, OSV-Scanner, cargo-audit, npm audit, Semgrep | scans futuros | não registrado | fontes oficiais; licença a verificar | E0 | repositório/lockfiles; rede conforme ferramenta | falso positivo/dados em relatório | controles complementares | a definir | não instalados |
