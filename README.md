# Millani Artes

Aplicativo desktop, local-first e offline-first para a gestão financeira pessoal e da fábrica Millani Artes. A primeira versão serve uma única usuária em um computador Windows, sem login, servidor ou cloud obrigatória. A entrega será um instalador Windows (`setup.exe`), para instalar e abrir pelo menu Iniciar.

## Comece aqui

1. Leia [AGENTS.md](AGENTS.md), [mapa do projeto](00-MAPA-DO-PROJETO.md) e [roadmap](planning/ROADMAP.md).
2. Escolha somente o primeiro work item `todo` com todas as dependências fechadas, handoff e CI comprovados.
3. Crie a spec a partir do [template](planning/SPEC_TEMPLATE.md); siga o ciclo e registre os checks, evidência e [handoff](planning/handoffs/).

O produto ainda não foi implementado. Esta revisão é exclusivamente o bootstrap documental.

## Fonte canônica

As [regras de produto](docs/product/), os [ADRs aceitos](docs/architecture/adr/) e o [planejamento](planning/) são fontes canônicas. Em conflitos, vale a hierarquia de [AGENTS.md](AGENTS.md).

## Estado verificável

- [Repositório público](https://github.com/millennium42/millani-artes) com documentação, templates, roadmap e work items versionados.
- [CI de documentação](https://github.com/millennium42/millani-artes/actions/workflows/docs.yml) já executado remotamente. Para afirmar verde, confira o run e o SHA exatos registrados no handoff e nas notas Git de evidência.
- Ainda não há dependências de produto, banco, UI, executável ou instalador para usar. O CI atual valida o bootstrap documental.

## Checks locais do bootstrap

Com PowerShell 7, Git e RTK disponíveis, execute na raiz do checkout:

```powershell
rtk proxy pwsh -NoProfile -File scripts/test-project-map.ps1
rtk proxy pwsh -NoProfile -File scripts/check-project-map.ps1
rtk proxy pwsh -NoProfile -File scripts/test-gitignore.ps1
```

Os checks validam o mapa contra a árvore Git, seus casos negativos e a exclusão de artefatos locais/sensíveis. Build, testes de produto e instalação serão definidos nas tarefas `INF-*` e `REL-*` do roadmap.
