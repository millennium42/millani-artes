# Inventário do escopo essencial

Base documental: DOC-005, revisão de produto no SHA `0bca4a5`. São **13 grupos funcionais** enumerados em [SCOPE](../product/SCOPE.md), não 13 requisitos atômicos nem funcionalidades implementadas. Cada termo da lista conta uma vez; devolução/reembolso e backup/restore permanecem os grupos compostos da fonte. IDs MVP-* identificam o inventário e não acrescentam regras ao produto.

| ID | Grupo do escopo | Comportamento já definido / fonte |
|---|---|---|
| MVP-01 | contas | Saldo inicial fora de receita; saldo posterior derivado de movimentos ([negócio](../product/BUSINESS_RULES.md), INV-FIN-001/002). [Rastreabilidade planejada TRC-001](REQUIREMENTS_MATRIX.md#mvp-01--contas-rastreabilidade-planejada), produto E0/testes ainda não registrados. |
| MVP-02 | categorias | Criar, renomear e desativar preservando referência histórica ([negócio](../product/BUSINESS_RULES.md)). [Rastreabilidade financeira planejada TRC-002](REQUIREMENTS_MATRIX.md#mvp-02-a-mvp-06--financeiro-rastreabilidade-planejada), produtoE0/testes não registrados. |
| MVP-03 | entradas | Movimento de entrada com contexto Casa/Fábrica ([negócio](../product/BUSINESS_RULES.md), [glossário](../product/GLOSSARY.md)). [Rastreabilidade financeira planejada TRC-002](REQUIREMENTS_MATRIX.md#mvp-02-a-mvp-06--financeiro-rastreabilidade-planejada), produtoE0/testes não registrados. |
| MVP-04 | saídas | Movimento de saída com contexto Casa/Fábrica ([negócio](../product/BUSINESS_RULES.md), [glossário](../product/GLOSSARY.md)). [Rastreabilidade financeira planejada TRC-002](REQUIREMENTS_MATRIX.md#mvp-02-a-mvp-06--financeiro-rastreabilidade-planejada), produtoE0/testes não registrados. |
| MVP-05 | transferências | Alterar contas sem alterar receita, despesa ou resultado ([negócio](../product/BUSINESS_RULES.md), INV-FIN-003). [Rastreabilidade financeira planejada TRC-002](REQUIREMENTS_MATRIX.md#mvp-02-a-mvp-06--financeiro-rastreabilidade-planejada), produtoE0/testes não registrados. |
| MVP-06 | recorrências | Fixa/variável, estados previstos e pagamento com saída única ([negócio](../product/BUSINESS_RULES.md), INV-REC-001). [Rastreabilidade financeira planejada TRC-002](REQUIREMENTS_MATRIX.md#mvp-02-a-mvp-06--financeiro-rastreabilidade-planejada), produtoE0/testes não registrados. |
| MVP-07 | produtos | Template ativo; alteração não muda o item de venda congelado ([negócio](../product/BUSINESS_RULES.md), INV-SAL-005). |
| MVP-08 | vendas | Vários itens/pagamentos, item livre sem cadastro; pendência exige cliente/vencimento e não aumenta saldo ([negócio](../product/BUSINESS_RULES.md), INV-SAL-001/002/003/006). |
| MVP-09 | clientes mínimos | Cadastro mínimo; cliente identificado quando a venda fica pendente ([jornadas](../product/USER_JOURNEYS.md), [negócio](../product/BUSINESS_RULES.md), INV-SAL-002). |
| MVP-10 | recebimentos | Parcial/total na conta escolhida, sem exceder o devido sem operação explícita ([jornadas](../product/USER_JOURNEYS.md), [negócio](../product/BUSINESS_RULES.md), INV-FIN-004 e INV-SAL-004). |
| MVP-11 | devoluções/reembolsos | Respeitar quantidade restante; reduzir dívida ou reembolsar dinheiro em uma conta escolhida ([negócio](../product/BUSINESS_RULES.md), INV-RET-001 e INV-FIN-005). |
| MVP-12 | dashboard | Projetar a fonte de verdade sem agregados editáveis ([negócio](../product/BUSINESS_RULES.md)). [Parcela financeira planejada TRC-002](REQUIREMENTS_MATRIX.md#mvp-02-a-mvp-06--financeiro-rastreabilidade-planejada); parcela comercial aguardaTRC-003, produtoE0. |
| MVP-13 | backup/restore local criptografado | Backup manual/automático e restore validado, íntegro, preservando banco atual em falha ([jornadas](../product/USER_JOURNEYS.md), [segurança](../security/BACKUP_SECURITY.md), INV-BKP-001/002). |

## Conjuntos que não devem ser somados

- [INVARIANTS](../product/INVARIANTS.md): **17 IDs**; regras atravessam os grupos.
- [USER_JOURNEYS](../product/USER_JOURNEYS.md): **7 jornadas**; uma jornada pode usar vários grupos.
- [BUSINESS_RULES](../product/BUSINESS_RULES.md): **9 bullets**; um bullet pode conter várias regras.
- [SECURITY_REQUIREMENTS](../security/SECURITY_REQUIREMENTS.md): **8 IDs**; controles transversais, não capacidades adicionais.
- [Matriz](REQUIREMENTS_MATRIX.md): oito linhas-síntese de produto agrupam capacidades; o detalhe MVP-01 remete a testes ainda planejados; linhas DOC-* são evidência documental. A decomposição requisito → invariante → teste será feita nos TRC-* do [roadmap](../../planning/ROADMAP.md).

## Restrições essenciais da entrega

Estas restrições continuam obrigatórias e não entram na contagem dos 13 grupos:

- Windows, instalação por setup.exe NSIS e abertura no menu Iniciar: [visão](../product/VISION.md), [ADR-001](../architecture/adr/ADR-001-tauri-desktop.md).
- Operação offline, SQLite local único, banco compartilhado, uma usuária sem login e sem servidor remoto: [ADR-003](../architecture/adr/ADR-003-sqlite-source-of-truth.md), [004](../architecture/adr/ADR-004-single-database.md), [005](../architecture/adr/ADR-005-local-first.md), [006](../architecture/adr/ADR-006-single-user-no-login.md), [007](../architecture/adr/ADR-007-no-remote-server.md).
- Casa/Fábrica são contextos; dinheiro é inteiro de centavos; regras ficam no domínio/casos de uso: [ADR-008](../architecture/adr/ADR-008-context-not-account.md), [013](../architecture/adr/ADR-013-money-in-cents.md), [017](../architecture/adr/ADR-017-domain-separated-from-ui.md).
- Escritas compostas atômicas e rollback sem meia operação: [negócio](../product/BUSINESS_RULES.md), INV-FIN-006/007.
- Backup criptografado/versionado e entrada externa validada; capabilities mínimas, logs sanitizados e scans proporcionais: [ADR-015](../architecture/adr/ADR-015-validated-encrypted-backups.md), [ADR-021](../architecture/adr/ADR-021-tauri-least-privilege.md), [requisitos de segurança](../security/SECURITY_REQUIREMENTS.md).
- Gates de coverage, testes, smoke, instalação e CI no SHA: [cobertura](../quality/COVERAGE_POLICY.md), [prontidão de release](../quality/RELEASE_READINESS.md). Contar fontes não prova esses gates.

Fora de escopo permanece a lista de SCOPE. Não há banco, UI, executável ou setup.exe nesta revisão; este inventário não concede passes=true a nenhum item de produto.

## Check reproduzível da contagem

Na raiz do checkout, execute este Python stdlib com `rtk proxy python -c` (PowerShell aceita o bloco em uma here-string). Ele confere o inventário contra a lista canônica; a interpretação dos comportamentos/fontes exige review E2.

```python
import pathlib, re
root = pathlib.Path.cwd()
read = lambda p: (root / p).read_text(encoding="utf-8")
scope = next(x for x in read("docs/product/SCOPE.md").splitlines() if x.startswith("Inclui "))
groups = re.split(r",\s*| e (?=backup/restore)", scope.removeprefix("Inclui ").removesuffix("."))
catalog = read("docs/traceability/ESSENTIAL_REQUIREMENTS.md")
rows = re.findall(r"^\| (MVP-\d{2}) \| ([^|]+) \|", catalog, re.M)
assert [label.strip() for _, label in rows] == groups, "Grupos divergem do SCOPE"
assert [key for key, _ in rows] == [f"MVP-{n:02d}" for n in range(1, len(groups) + 1)], "IDs duplicados/fora de ordem"
invariants = re.findall(r"^\| (INV-[A-Z]+-\d{3}) \|", read("docs/product/INVARIANTS.md"), re.M)
security = re.findall(r"^- (SEC-\d{3}):", read("docs/security/SECURITY_REQUIREMENTS.md"), re.M)
assert len(invariants) == len(set(invariants)) and len(security) == len(set(security)), "IDs canônicos duplicados"
counts = (len(groups), len(invariants), len(re.findall(r"^\d+\. ", read("docs/product/USER_JOURNEYS.md"), re.M)), len(re.findall(r"^- ", read("docs/product/BUSINESS_RULES.md"), re.M)), len(security))
assert counts == (13, 17, 7, 9, 8), "Revisar totais documentados após mudança canônica"
print("PASS: grupos/invariantes/jornadas/bullets/segurança =", counts)
```
