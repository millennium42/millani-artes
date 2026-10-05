# Millani Artes

Aplicativo desktop, local-first e offline-first para a gestão financeira pessoal e da fábrica Millani Artes. A primeira versão serve uma única usuária em um computador Windows, sem login, servidor ou cloud obrigatória. A entrega será um instalador Windows (`setup.exe`), para instalar e abrir pelo menu Iniciar.

## Comece aqui

1. Leia [AGENTS.md](AGENTS.md), [mapa do projeto](00-MAPA-DO-PROJETO.md) e [roadmap](planning/ROADMAP.md).
2. Escolha somente o primeiro work item `todo` com todas as dependências fechadas, handoff e CI comprovados.
3. Crie a spec a partir do [template](planning/SPEC_TEMPLATE.md); siga o ciclo e registre os checks, evidência e [handoff](planning/handoffs/).

O scaffold Tauri/React foi compilado e executado no Windows; evidências e fechamento na [INF-002](planning/handoffs/INF-002.md). As funcionalidades financeiras e o instalador ainda não foram implementados.

## Fonte canônica

As [regras de produto](docs/product/), os [ADRs aceitos](docs/architecture/adr/) e o [planejamento](planning/) são fontes canônicas. Em conflitos, vale a hierarquia de [AGENTS.md](AGENTS.md).

## Estado verificável

- [Repositório público](https://github.com/millennium42/millani-artes) com documentação, templates, roadmap e work items versionados.
- [CI de documentação](https://github.com/millennium42/millani-artes/actions/workflows/docs.yml) já executado remotamente. Para afirmar verde, confira o run e o SHA exatos registrados no handoff e nas notas Git de evidência.
- Há scaffold mínimo e dependências de build; o estado de execução fica no handoff INF-002. Banco, funcionalidades financeiras e instalador ainda pendentes. O workflow valida documentos/registro e executa a qualidade do frontend em alterações relevantes; comandos, execuções e limites no [handoff INF-010](planning/handoffs/INF-010.md).

## Checks locais do bootstrap

Com PowerShell 7, Git e RTK disponíveis, execute na raiz do checkout:

```powershell
rtk proxy pwsh -NoProfile -File scripts/test-project-map.ps1
rtk proxy pwsh -NoProfile -File scripts/check-project-map.ps1
rtk proxy pwsh -NoProfile -File scripts/test-gitignore.ps1
rtk proxy pwsh -NoProfile -File scripts/test-evidence-states.ps1
rtk proxy pwsh -NoProfile -File scripts/check-evidence-states.ps1
rtk proxy pwsh -NoProfile -File scripts/test-work-items.ps1
rtk proxy pwsh -NoProfile -File scripts/check-work-items.ps1
rtk proxy pwsh -NoProfile -File scripts/test-progress-log.ps1
rtk proxy pwsh -NoProfile -File scripts/check-progress-log.ps1 -BaseRef HEAD -TargetRef INDEX
```

Os checks validam o mapa contra a árvore Git, seus casos negativos, a exclusão de artefatos locais/sensíveis, o schema/declarações dos work items e o progress log append-only. Para o log, adicione suas mudanças ao índice antes de comparar com HEAD; para commits, informe a base e o alvo explicitamente. A comparação usa os bytes dos blobs Git e falha se base/histórico/arquivo não estiverem disponíveis; CI usa push.before ou PR.base.sha contra HEAD do checkout. Schema/log válido não comprova fatos, aceite humano ou CI; consulte os registros. Build, testes de produto e instalação serão definidos nas tarefas `INF-*` e `REL-*` do roadmap.

## Executar o scaffold

Siga [reprodução](docs/operations/REPRODUCING.md) para ativar Node/Rust e conferir os pré-requisitos Windows. Com os lockfiles presentes: `rtk proxy npm ci --ignore-scripts`, `rtk proxy npm run build` e `rtk proxy npm run tauri -- build --no-bundle`. Para desenvolvimento local: `rtk proxy npm run tauri -- dev`. O scaffold não contém operações financeiras; MSVC/SDK instalados, build de produção e execução/fechamento passaram; a usuária confirmou a janela. O recibo INF-002 registra o CI no SHA final.
