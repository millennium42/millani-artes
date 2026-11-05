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

## Contexto de pagamentos iniciais de venda — decisão humana de 29/10/2026
Cada pagamento inicial de uma venda tem seu próprio contexto Casa/Fábrica, escolhido separadamente para classificar a entrada correspondente. Pagamentos da mesma venda podem usar contextos diferentes. Decisão registrada na resposta humana durante a reconciliação de SAL-020.

- Vencimento mensal: quando o dia escolhido não existir, usar o último dia do mês; considerar o vencimento até o fim do dia no horário de Brasília. Decisão humana registrada em 19/10/2026, na resposta à pergunta de calendário durante REC-011.

## Entradas do dia no dashboard — decisão humana de 05/11/2026

Para DSH-003, “hoje” é o dia civil de Brasília com offset fixo UTC−3, do início desse dia inclusive ao início do dia seguinte exclusivo. Conta como entrada do dia um movimento atualmente classificado como entrada cujo evento original `movement_entry_create` ocorreu nessa janela. A data é a do evento original em UTC; edições de valor ou contexto não a alteram, e uma saída convertida em entrada por `movement_update` não conta como entrada criada no dia. Decisão humana recebida em 05/11/2026 nesta conversa: “Continue com o recomendado”, em resposta à recomendação completa de DSH-003; horário exato não registrado. O offset é fixo e não aplica regras históricas de IANA/DST.

## Saídas do dia no dashboard — decisão humana de 05/11/2026

Para DSH-004, aplicar a mesma definição de “hoje” já aprovada para DSH-003: dia civil de Brasília em UTC−3 fixo e janela semiaberta. Conta como saída do dia um movimento atualmente classificado como saída cujo evento original `movement_exit_create` ocorreu nessa janela. A data é a do evento original em UTC; edições de valor/contexto não a alteram, e uma entrada convertida em saída por `movement_update` não conta como saída criada no dia. Decisão humana recebida em 05/11/2026 nesta conversa: “Continue com o recomendado”, em resposta à recomendação de reutilizar a semântica temporal já aprovada; horário exato não registrado. O offset é fixo e não aplica regras históricas de IANA/DST.


## Preço sugerido de produto — decisão humana de 19/10/2026
O preço sugerido é informado em centavos inteiros e pode ser R$ 0,00. Valores negativos são rejeitados. Esta validação não altera o produto nem depende de seu nome ou estado ativo. Não foi definido limite superior adicional ao tipo i64. Registro autorizado na resposta humana da PRD-003: “Aceitar zero; rejeitar negativos”. Persistência, criação, edição e desativação seguem seus work items próprios.


## Telefone opcional de cliente — decisão humana de 23/10/2026
Telefone ausente é aceito. Quando informado, deve seguir exatamente o modelo `(DD) 9XXXX-XXXX`, com DDD brasileiro existente, dígitos ASCII e número móvel de nove dígitos iniciado por 9. A validação preserva o texto original, o nome e a identidade do cliente; não remove espaços, pontuação ou prefixos e não verifica se a linha existe ou está ativa. Não aceita telefone fixo de oito dígitos, prefixo de país ou formato livre neste contrato. A máscara concreta é a interpretação de implementação da decisão humana “validação completa no modelo (dd) + 9 digitos”, em resposta à pergunta CUS-003 sobre formato. Cadastro e apresentação pertencem aos seus work items.
Referências técnicas: [plano brasileiro](https://www.gov.br/anatel/pt-br/regulado/numeracao/plano-de-numeracao-brasileiro), [nono dígito](https://www.gov.br/anatel/pt-br/regulado/numeracao/codigos-nacionais/nono-digito) e mapa de DDDs da mesma página, consultados em 23/10/2026. A lista de 67 códigos e a evidência da leitura constam da spec CUS-003/recibo de decisão.


## Quantidade e desconto de item — decisão humana de 24/10/2026
A quantidade de item de venda é inteira. No lançamento, o desconto pode ser informado em valor monetário (R$) ou em percentual, escolhendo uma das duas modalidades para o item. Resposta humana às duas perguntas de SAL-001: “QUANTIDADE INTEIRA / AMBOS”. O desconto incide sobre o total bruto do item (quantidade × preço unitário), como definido na pergunta respondida. O snapshot conserva o desconto monetário efetivamente aplicado em centavos, junto de quantidade, preço e total. Validação, conversão percentual e cálculo seguem SAL-002/003/004; esta decisão não fixa precisão percentual, arredondamento ou limites adicionais.


## Limites e arredondamento de desconto — decisão humana de 24/10/2026
O desconto em valor vai de R$ 0 até o total bruto do item, inclusive. O percentual vai de 0% a 100%, inclusive, com até duas casas decimais. O desconto percentual calculado é arredondado para o centavo mais próximo; exatamente meio centavo arredonda para cima. Exemplo: R$ 0,05 × 10% resulta em R$ 0,01 de desconto. A resposta humana escolheu a opção 1 (“Arredondar ao centavo mais próximo, meio para cima”); o item 2 veio vazio e não alterou a escolha. A pergunta respondida estabelecia os limites e a precisão comuns às duas opções. O desconto incide no bruto do item e não é aplicado duas vezes: no lançamento escolhe-se valor ou percentual. Não há valor negativo nem desconto acima do bruto. No domínio, percentual usa pontos-base inteiros (1 = 0,01%; 10.000 = 100%); a futura entrada textual deve rejeitar mais de duas casas, sem arredondar silenciosamente a entrada. A conta financeira usa somente inteiros.

## Rateio da devolução e prioridade da dívida — decisão humana de 02/11/2026
O valor líquido devolvido de um item com desconto é rateado pro rata cumulativamente sobre o total líquido congelado da linha; cada parcela é a diferença entre os valores cumulativos arredondados antes e depois da quantidade devolvida, usando o arredondamento já definido de centavo mais próximo, meio para cima. A parcela primeiro reduz o devido aberto do recebível; qualquer excedente permanece reservado para o fluxo RET-007 e não gera movimento nem altera conta em RET-006. Resposta humana às perguntas de RET-006: “Pro rata cumulativo” e “Dívida primeiro; excedente reservado para RET-007”.

## Reembolso parcial do excedente — decisão humana de 02/11/2026
O excedente reservado após RET-006 pode ser reembolsado em parcelas explícitas. Cada parcela fica limitada ao saldo ainda reservado para aquela devolução por item; parcelas confirmadas somadas não podem exceder o excedente reservado. O saldo disponível é o reservado por RET-006 menos os reembolsos confirmados vinculados à mesma devolução. Registrar a devolução não cria automaticamente uma saída. Cada reembolso efetivo cria uma saída em conta escolhida conforme o fluxo de devolução. Resposta humana à pergunta de RET-007: “Permitir reembolsos parciais até o saldo reservado”. Seleção de conta, persistência da saída, rollback e cancelamento permanecem nos itens RET-008/009/010/013/012.

## Cancelamento de venda quitada — decisão humana de 04/11/2026
RET-012 só cancela venda finalizada sem devido em aberto; venda aberta ou parcialmente quitada é rejeitada. O cancelamento é integral e único, registra devolução das quantidades ainda disponíveis de cada item e preserva a venda, pagamentos, devoluções, reembolsos e auditorias já existentes. Para cada devolução por item, reembolsa apenas o saldo ainda reservado (valor devolvido menos redução de dívida e reembolsos já confirmados), sem duplicar saída. Em 04/11/2026, a usuária confirmou: usar um reembolso total em uma conta escolhida explicitamente, mesmo se os recebimentos originais usaram contas diferentes. Todas as novas saídas geradas pelo cancelamento compartilham essa única conta escolhida; continuam separadas e vinculadas às reservas por devolução/item para preservar o histórico, e cada saída exige contexto explícito. Reembolsos confirmados antes do cancelamento permanecem intactos e não são movidos nem repetidos. Nenhuma conta/contexto é inferida. A operação e seu status persistido são atômicos com devoluções, saídas e auditorias; depois do cancelamento, novos recebimentos, recebíveis, devoluções e reembolsos da venda são rejeitados. Os cálculos de devolução e reserva reutilizam RET-006/007 e a seleção/saída seguem RET-008/009/010.
