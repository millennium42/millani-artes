# Mapa do Projeto

| Área | Fonte | Autoridade |
|---|---|---|
| Produto | `docs/product/` | regras, escopo, jornadas e invariantes |
| Arquitetura | `docs/architecture/` | limites, dados, transações, backups e ADRs |
| Qualidade | `docs/quality/` | testes, cobertura, smoke, inspeção e release |
| Segurança | `docs/security/` | ameaças, controles, scans e capabilities |
| Operação | `docs/operations/` | migrations, backup/restore e reprodução |
| Governança | `governance/` | IA, evidência, exceções e ferramentas |
| Planejamento | `planning/` | roadmap, specs, work items, handoffs e histórico |
| Automação | `.github/workflows/` | checks remotos |

Não implemente regras financeiras na UI. Caminho de escrita: UI → caso de uso → domínio → repositório → SQLite.
