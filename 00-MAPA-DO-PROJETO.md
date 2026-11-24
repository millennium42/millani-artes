# Mapa do Projeto

| Área | Fonte | Autoridade |
|---|---|---|
| Entrada | `README.md`, `AGENTS.md`, `00-MAPA-DO-PROJETO.md`, `CONTRIBUTING.md` | início, regras, navegação e contribuição |
| Política pública | `SECURITY.md`, `.gitignore`, `.gitattributes` | reporte de segurança e exclusão de artefatos locais/sensíveis |
| Produto | `docs/product/` | regras, escopo, jornadas e invariantes |
| Arquitetura | `docs/architecture/` | limites, dados, transações, backups e ADRs |
| Qualidade | `docs/quality/` | testes, cobertura, smoke, inspeção e release |
| Segurança | `docs/security/` | ameaças, controles, scans e capabilities |
| Operação | `docs/operations/` | migrations, backup/restore e reprodução |
| Rastreabilidade | `docs/traceability/` | requisito → work item → teste e evidência |
| Governança | `governance/` | IA, evidência, exceções e ferramentas |
| Planejamento | `planning/` | roadmap, specs, work items, handoffs e histórico |
| Runtimes | `.node-version`, `rust-toolchain.toml` | pins dev Windows; reprodução e evidência em operação |
| Frontend | `package.json`, `package-lock.json`, `tsconfig.json`, `biome.json`, `vite.config.ts`, `vitest.config.ts`, `index.html`, `src/` | scaffold React/TypeScript/Vite; sem regra financeira |
| Core desktop | `src-tauri/` | scaffold Tauri Windows e lockfile Rust |
| Automação | `.github/workflows/` | checks remotos |
| Checks locais | `scripts/` | validação reproduzível do mapa |

Diretórios agrupam seus arquivos internos; o mapa não duplica o inventário completo. Além da documentação, checks e scaffold Tauri/React, o core contém domínio, SQLite/migrations, criação de conta (FIN-008) e finalização transacional de venda com entradas e recebível pendente (SAL-020, concluído como E3 local). A UI/IPC/startup tem liberações E3 focais em UI-003 (cadastro local de contas), UI-004 (categorias via CAT-004), UI-005 (registro local de entradas) e UI-006 (formulário local de saídas), cada uma comprovada por seu próprio recibo remoto; isso não comprova outras jornadas, invariantes globais, cobertura Rust global, E2E amplo, instalador ou produção. Veja os handoffs UI-003–UI-006 para escopo e limites. Verifique paths e cobertura da árvore com rtk proxy pwsh -NoProfile -File scripts/check-project-map.ps1.

Não implemente regras financeiras na UI. Caminho de escrita: UI → caso de uso → domínio → repositório → SQLite.
