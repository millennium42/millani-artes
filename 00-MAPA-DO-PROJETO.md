# Mapa do Projeto

| Área | Fonte | Autoridade |
|---|---|---|
| Entrada | `README.md`, `AGENTS.md`, `00-MAPA-DO-PROJETO.md`, `CONTRIBUTING.md` | início, regras, navegação e contribuição |
| Política pública | `SECURITY.md`, `.gitignore` | reporte de segurança e exclusão de artefatos locais/sensíveis |
| Produto | `docs/product/` | regras, escopo, jornadas e invariantes |
| Arquitetura | `docs/architecture/` | limites, dados, transações, backups e ADRs |
| Qualidade | `docs/quality/` | testes, cobertura, smoke, inspeção e release |
| Segurança | `docs/security/` | ameaças, controles, scans e capabilities |
| Operação | `docs/operations/` | migrations, backup/restore e reprodução |
| Rastreabilidade | `docs/traceability/` | requisito → work item → teste e evidência |
| Governança | `governance/` | IA, evidência, exceções e ferramentas |
| Planejamento | `planning/` | roadmap, specs, work items, handoffs e histórico |
| Runtimes | `.node-version`, `rust-toolchain.toml` | pins dev Windows; reprodução e evidência em operação |
| Frontend | `package.json`, `package-lock.json`, `tsconfig.json`, `vite.config.ts`, `index.html`, `src/` | scaffold React/TypeScript/Vite; sem regra financeira |
| Core desktop | `src-tauri/` | scaffold Tauri Windows e lockfile Rust |
| Automação | `.github/workflows/` | checks remotos |
| Checks locais | `scripts/` | validação reproduzível do mapa |

Diretórios agrupam seus arquivos internos; o mapa não duplica o inventário completo. O estado atual contém documentação, checks e scaffold Tauri/React em verificação. Banco, regras financeiras e instalador ainda não implementados; builds/estado factual estão no handoff INF-002. Verifique os paths e a cobertura da árvore com `rtk proxy pwsh -NoProfile -File scripts/check-project-map.ps1`.

Não implemente regras financeiras na UI. Caminho de escrita: UI → caso de uso → domínio → repositório → SQLite.
