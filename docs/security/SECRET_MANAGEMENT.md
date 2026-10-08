# Gestão de segredos

Nenhum segredo de produção é necessário no MVP local. Chaves de backup não podem ser hardcoded, commitadas ou exibidas. Use mecanismo nativo/auditado a definir em POC; fixtures e evidências usam dados sintéticos.

Chaves, tokens e credenciais também ficam fora de erros/diagnósticos e evidências conforme [política de logs](LOGGING_POLICY.md); código de armazenamento/recuperação e provas reais continuam nos executores próprios.
