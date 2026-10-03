# Política de evidência

Evidência registra data, ambiente, comando, versão, saída resumida, escopo e SHA. E0 é hipótese; E1 tem fonte; E2 teve fonte lida; E3 teve execução; E4 recebeu aceite humano. Nunca promova nível por inferência.

Preserve somente o necessário e nunca dados financeiros, telefones, chaves ou backups reais. Evidência de CI contém URL/run e SHA. Se um campo não existir, escreva `não registrado`.

## Estados e limite de verificação

| Estado | Exemplo de registro suficiente para esse nível |
|---|---|
| E0 | Hipótese de comportamento ainda não verificada. |
| E1 | Fonte identificada por path/URL e revisão, ainda não lida. |
| E2 | Fonte lida ou review documental; não prova execução de produto. |
| E3 | Comando executado com ambiente, versão, saída, escopo/data/SHA registrados; CI também exige run/URL e SHA exato. |
| E4 | Aceite humano explícito registrado para um escopo/revisão; não substitui checks ou CI. |

O nível pertence ao escopo comprovado: executar check documental não prova funcionalidade financeira; aceite de spec não prova execução. `passes=true` não pode se apoiar só em E0/E1/E2, e continua exigindo todos os gates humanos/canônicos, inclusive CI remoto no SHA atual.

`rtk proxy pwsh -NoProfile -File scripts/test-evidence-states.ps1` e `rtk proxy pwsh -NoProfile -File scripts/check-evidence-states.ps1` validam declarações dos JSONs. Aceitam os literais E0–E4, passes booleano e E3/E4 para true. Falham com níveis/tipos/estruturas inválidos sem imprimir conteúdo. Esses checks não comprovam a verdade de E3/E4, aceite humano, documentação, revisão ou CI; confira os respectivos registros/handoffs/notas.

PLN-002 adiciona `planning/WORK_ITEM.schema.json` e `scripts/check-work-items.ps1` com fixtures em `scripts/test-work-items.ps1`: 12 campos obrigatórios do template, objeto ou coleção não vazia, tipos/valores, IDs únicos entre arquivos e passes=true somente com done/E3/E4. O schema usa somente refs locais e o Test-Json nativo do PowerShell 7, sem dependência instalada. O template com placeholders deve ser instanciado antes de validar. Cinco registros legados recebem agentPlan explicitamente não registrado; isso não comprova plano executado. Existência/completude de specs, dependências canônicas (PLN-006), gates humanos e CI factual continuam exigindo comprovantes separados.
