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
| Automação | `.github/workflows/` | checks remotos |
| Checks locais | `scripts/` | validação reproduzível do mapa |

Diretórios agrupam seus arquivos internos; o mapa não duplica o inventário completo. O estado atual contém documentação e checks, sem banco, UI, executável ou instalador. Verifique os paths e a cobertura da árvore com `rtk proxy pwsh -NoProfile -File scripts/check-project-map.ps1`.

Não implemente regras financeiras na UI. Caminho de escrita: UI → caso de uso → domínio → repositório → SQLite.
