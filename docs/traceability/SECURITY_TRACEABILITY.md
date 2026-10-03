# Rastreabilidade de segurança

TRC-006 liga os oito requisitosSEC001..008 de [SECURITY_REQUIREMENTS](../security/SECURITY_REQUIREMENTS.md) à [matriz de controles/testes planejados](../security/SECURITY_TEST_MATRIX.md). Contagem8 incluiSEC008; não soma ameaças/linhas de roadmap/invariantes. Ameaças e superfícies vêm do [threat model](../security/THREAT_MODEL.md), decisões de [ADR020](../architecture/adr/ADR-020-security-policy.md)/[ADR021](../architecture/adr/ADR-021-tauri-least-privilege.md).

## IDs de requisito e de executor

SEC-* na primeira coluna identifica requisito do catálogo. SEC-* na coluna executores identifica tarefa no [roadmap](../../planning/ROADMAP.md), que tem intenções diferentes: requisitoSEC002=SQL/constraints, tarefaSEC002=CSP; requisitoSEC007=gates de scans, tarefaSEC007=Gitleaks no CI. Não inferir mapping por número. BKP/DB/FIN/SAL/RCV/RET/REC/UI/INF/ARC-* são outros executores concretos quando o controle cruza camadas; suas dependências/gates continuam pendentes. LegadosFIN001/BKP001 ficam paraPLN006, sem canonicalizar/liberar. Não inventar novo controle ou funcionalidade.

## Oráculos e evidência necessária

**Planejado/E0 produto**: código/configurações/testes/scanners/comandos/saídas/coverage/CI de segurança não registrados. Matriz descreve positivos/negativos por superfície: mínimo permitido versus negado; SQL literal/constraints; logs sanitizados; arquivo externo/path; restore preservado; transação/audit/falhas/repetição por contrato; scanners/gates; CSP/ADR/permissões específicas. [Invariantes globais](INVARIANT_TEST_MATRIX.md#operações-compostas--testes-globais-planejados) e [backup](REQUIREMENTS_MATRIX.md#mvp-13--backuprestore-rastreabilidade-planejada) são planosE0, não implementações.

Cada executor deve vincular requisito→ameaça→controle/config/código→teste/scanner→resultado: path/teste/fixture sintética/comando/versão/saída resumida sanitizada/escopo/data/SHA/coverage/run-conclusão quando remoto. ADR/capability/exceção humana registrados quando exigidos. Não armazenar dados reais/telefone/chave/backup em fixtures/handoff/evidência. Scanner não prova redaction/SQL/restore/CSP; texto de config não prova execução runtime. Erro de scanner/indisponibilidade/scan vazio não permite concluir segurança verde.

## Limite do fechamento documental

TRC006 pode ter checksE3/reviewE2/CI documental; isso não promove controles/implementação a E3 nem autoriza permissões. Oito status da matriz continuamtodo/E0. Gates reais seguem fontes/políticas, com ferramenta aprovada/escopo pertinente/achados corrigidos ou exceção humana temporária. [Coverage](../quality/COVERAGE_POLICY.md), [testes](../quality/TEST_STRATEGY.md) e [smoke](../quality/SMOKE_TEST_POLICY.md) preservados; nenhuma proteção foi executada pelo produto nesta revisão.
