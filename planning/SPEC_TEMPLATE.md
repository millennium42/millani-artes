# Spec — ID

## Objetivo observável
## Motivo
## Escopo / Fora de escopo
## Dependências e rastreabilidade
## Plano de agentes: `P-INT` sempre; persona de cada delegado; modelo/esforço; paths; motivo do paralelismo
Reserva de escrita antes da edição: ID | responsável | base/revisão | paths exatos (sem glob) | modo read/write | estado | conflitos e reconciliação | condição de liberação. Checkout compartilhado: único escritor P-INT; delegados read-only. Ampliar paths exige atualizar reserva antes de escrever. Reserva ativa até CI final no SHA exato e nota publicada; passes=true após entrega não libera. Handoff/recibo registram liberação; interrupção exige reconciliação, sem liberação automática. Paralelismo de escrita segue MULTI_AGENT_POLICY.
## Contrato: entradas, saídas, invariantes, erros esperados
## Critérios de aceite
## Testes previstos / coverage afetada
## Segurança / smoke ou E2E / evidência necessária
## Riscos / gate humano
## Definição de concluído
