# Roadmap microgranular

Status: todos `todo`; IDs estáveis. Cada item ganha spec e JSON a partir do template antes de execução. Dependência implícita: fundação documental → ambiente/segurança → banco → domínio → UI/smoke/release.

## Fundação

- DOC-002 validar mapa contra árvore real
- DOC-003 revisar README de entrada
- GOV-001 revisar AGENTS por sessão
- GOV-002 validar estados de evidência
- GOV-003 revisar tool register antes de instalar
- GOV-004 criar processo de exceção aprovado
- GOV-005 aplicar política de agentes, modelos e esforço
- INF-001 fixar versões Node/Rust
- INF-002 inicializar frontend Tauri mínimo
- INF-003 adicionar lockfiles
- INF-004 configurar Biome e TypeScript strict
- INF-005 configurar Vitest e coverage
- INF-006 configurar CI Windows
- INF-007 adicionar Gitleaks
- INF-008 adicionar OSV-Scanner
- INF-009 adicionar npm audit gate
- INF-010 adicionar cargo audit gate
- INF-011 validar build limpo
- SEC-001 definir capabilities iniciais
- SEC-002 definir CSP inicial
- SEC-003 definir política de logs
- SEC-004 POC de formato de backup
- SEC-005 revisar superfície de plugins
- DB-001 criar harness SQLite temporário
- DB-002 definir executor de migration
- DB-003 testar banco vazio
- DB-004 testar upgrade de migration
- DB-005 registrar schema version
- DB-006 testar rollback/falha de migration

## Financeiro e contas

- FIN-001 definir tipo Account
- FIN-002 validar nome de conta
- FIN-003 definir tipo de conta
- FIN-004 definir saldo inicial em centavos
- FIN-005 provar saldo inicial fora de receita
- FIN-006 definir contexto Casa/Fábrica
- FIN-007 rejeitar contexto inválido
- FIN-008 definir movimento financeiro
- FIN-009 validar entrada positiva
- FIN-010 validar saída positiva
- FIN-011 definir transferência com origem/destino
- FIN-012 provar neutralidade da transferência
- FIN-013 calcular saldo por conta
- FIN-014 provar saldo derivado de movimentos
- FIN-015 criar migration de contas
- FIN-016 testar constraint de conta
- FIN-017 criar migration de movimentos
- FIN-018 testar constraint de movimento
- FIN-019 persistir entrada
- FIN-020 persistir saída
- FIN-021 persistir transferência atômica
- FIN-022 provar rollback de transferência
- FIN-023 listar extrato por conta
- FIN-024 editar lançamento manual com audit event
- FIN-025 validar que edição não altera saldo por atalho
- FIN-026 testar cálculo de resultado por contexto

## Categorias e recorrências

- CAT-001 definir categoria ativa
- CAT-002 validar nome de categoria
- CAT-003 criar migration de categoria
- CAT-004 criar categoria
- CAT-005 renomear categoria
- CAT-006 desativar categoria
- CAT-007 preservar referência histórica desativada
- REC-001 definir recorrência fixa
- REC-002 definir recorrência variável
- REC-003 definir estado aguardando valor
- REC-004 definir estado a pagar
- REC-005 definir estado pago
- REC-006 definir estado vencido
- REC-007 criar migration de recorrência
- REC-008 gerar obrigação fixa mensal
- REC-009 gerar pendência variável
- REC-010 informar valor pendente
- REC-011 pagar recorrência por caso canônico
- REC-012 provar saída única em pagamento
- REC-013 provar rollback ao falhar pagamento

## Produtos, vendas e clientes

- PRD-001 definir produto template
- PRD-002 validar produto ativo
- PRD-003 criar migration de produto
- PRD-004 criar produto
- PRD-005 alterar produto
- PRD-006 provar snapshot histórico imutável
- SAL-001 definir item de venda congelado
- SAL-002 validar quantidade positiva
- SAL-003 validar desconto não negativo
- SAL-004 calcular total de item em centavos
- SAL-005 aceitar item livre
- SAL-006 provar item livre não cria produto
- SAL-007 definir venda
- SAL-008 criar migration de venda
- SAL-009 criar migration de item
- SAL-010 adicionar item de produto
- SAL-011 adicionar item livre
- SAL-012 remover item antes de finalizar
- SAL-013 calcular total da venda
- SAL-014 definir pagamento parcial
- SAL-015 provar venda pendente não muda saldo
- CUS-001 definir cliente mínimo
- CUS-002 validar nome obrigatório
- CUS-003 criar migration de cliente
- CUS-004 criar cliente
- CUS-005 associar cliente à pendência
- SAL-016 exigir cliente em venda pendente
- SAL-017 exigir vencimento em venda pendente
- SAL-018 finalizar venda em transação
- SAL-019 provar rollback de finalização

## Recebimento, devolução e dashboard

- RCV-001 definir conta a receber
- RCV-002 criar migration de recebível
- RCV-003 calcular saldo devido
- RCV-004 registrar recebimento parcial
- RCV-005 criar entrada ao receber
- RCV-006 provar uma conta alterada
- RCV-007 rejeitar recebimento acima do devido
- RCV-008 fechar recebido integral
- RCV-009 provar rollback de recebimento
- RET-001 definir devolução por item
- RET-002 calcular quantidade disponível
- RET-003 rejeitar devolução excedente
- RET-004 criar migration de devolução
- RET-005 reduzir dívida sem reembolso
- RET-006 definir reembolso
- RET-007 criar saída ao reembolsar
- RET-008 permitir conta diferente da entrada
- RET-009 provar uma conta reduzida
- RET-010 testar cancelamento integral pago
- DSH-001 definir projeção de saldo total
- DSH-002 definir projeção por conta
- DSH-003 definir entrou/saiu hoje
- DSH-004 definir vendas do mês
- DSH-005 definir a receber/a pagar
- DSH-006 definir resultado fábrica/gastos casa
- DSH-007 provar dashboard sem escrita agregada

## Backup, UI, smoke e release

- BKP-001 definir metadados/versionamento backup
- BKP-002 escolher biblioteca criptográfica por POC
- BKP-003 definir gestão de chave
- BKP-004 criar backup manual consistente
- BKP-005 criar backup automático local
- BKP-006 validar backup recém-criado
- BKP-007 rejeitar arquivo malformado
- BKP-008 restaurar em staging
- BKP-009 preservar banco ativo antes de restore
- BKP-010 executar integrity check pós-restore
- BKP-011 provar restore inválido não substitui ativo
- BKP-012 testar path traversal
- UI-001 criar shell acessível
- UI-002 criar fluxo de configuração inicial
- UI-003 criar formulário de conta
- UI-004 criar formulário de movimento
- UI-005 criar formulário de transferência
- UI-006 criar formulário de produto
- UI-007 criar carrinho de venda
- UI-008 criar formulário de recebimento
- UI-009 criar fluxo de recorrência
- UI-010 criar fluxo de devolução/reembolso
- UI-011 criar tela de dashboard
- UI-012 criar tela de backup/restore
- SMK-001 iniciar build de produção
- SMK-002 abrir banco novo
- SMK-003 exercitar configuração/conta/categoria
- SMK-004 exercitar financeiro/transferência
- SMK-005 exercitar venda e recebimento
- SMK-006 exercitar pendência e recebimento posterior
- SMK-007 exercitar recorrência
- SMK-008 exercitar devolução/reembolso
- SMK-009 exercitar backup/restore controlado
- REL-001 revisar matriz funcional de entrega
- REL-002 executar inspeção visual
- REL-003 revisar P0/P1/P2
- REL-004 verificar CI remoto no SHA
- REL-005 preparar handoff/release
- REL-006 gerar instalador NSIS `setup.exe` no Windows
- REL-007 testar instalação limpa, abertura pelo menu Iniciar e desinstalação
- REL-008 decidir e registrar estratégia WebView2 (bootstrap ou offline)
- REL-009 decidir e registrar assinatura de código antes de distribuição pública
