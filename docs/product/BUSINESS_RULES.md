# Regras de negócio

- Contas têm saldo inicial; ele inicia o histórico e não é receita. Depois, saldo é derivado de movimentos e não editado diretamente.
- Entradas, saídas e transferências têm contexto Casa ou Fábrica. Transferência própria altera contas, nunca receita/despesa ou resultado.
- Categorias podem ser criadas, renomeadas e desativadas; categoria histórica permanece referenciável.
- Recorrência fixa cria obrigação preenchida; variável fica `aguardando valor`. Estados: aguardando valor, a pagar, pago, vencido. Pagar cria uma saída única.
- Produto é template ativo com nome, descrição e preço sugerido. Item de venda congela descrição, quantidade, preço, desconto e total; item livre não cria produto.
- Venda pode ter vários itens e pagamentos. Venda não é recebimento: apenas valor efetivamente recebido cria entrada. Saldo pendente exige cliente e vencimento; recebimentos parciais não excedem o devido sem operação explícita.
- Devolução parcial respeita a quantidade restante. Se reduzir dívida aberta, reduz somente o a receber; se houver dinheiro, cria reembolso em uma conta escolhida.
- Dashboard projeta a fonte de verdade, sem agregados editáveis.
- Escritas financeiras compostas são atômicas e deixam histórico técnico suficiente para diagnóstico.

- Vencimento mensal: quando o dia escolhido não existir, usar o último dia do mês; considerar o vencimento até o fim do dia no horário de Brasília. Decisão humana registrada em 19/10/2026, na resposta à pergunta de calendário durante REC-011.


## Preço sugerido de produto — decisão humana de 19/10/2026
O preço sugerido é informado em centavos inteiros e pode ser R$ 0,00. Valores negativos são rejeitados. Esta validação não altera o produto nem depende de seu nome ou estado ativo. Não foi definido limite superior adicional ao tipo i64. Registro autorizado na resposta humana da PRD-003: “Aceitar zero; rejeitar negativos”. Persistência, criação, edição e desativação seguem seus work items próprios.
