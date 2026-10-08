# Cargo audit — gate local

Na raiz, com Python 3.12 ou superior e cargo-audit 0.22.2 portátil instalado pelo INF-002:
```powershell
rtk proxy python -B scripts/test-cargo-audit.py
rtk proxy python -B scripts/check_cargo_audit.py
```

O checker confere tamanho, hash e versão do binário, lê o Cargo.lock completo e cria configuração própria, sem ignores ou filtro de plataforma. Atualiza o banco oficial RustSec por HTTPS e exige origem Git oficial, checkout limpo e commit correspondente ao relatório. Vulnerabilidades, warnings sem aprovação, erros, relatório incoerente, timeout e mudança nos inputs bloqueiam. Não modifica dependências. Publica somente metadados públicos e códigos estáticos; relatório bruto e stderr ficam em memória.

A [CLI 0.22.2](https://github.com/rustsec/rustsec/blob/cargo-audit%2Fv0.22.2/cargo-audit/src/commands/audit.rs) nega warnings explicitamente. A [política de dependências](../security/DEPENDENCY_SECURITY.md) exige correção ou exceção humana temporária.

## Exceção humana vigente

A [configuração](../security/CARGO_WINDOWS_EXCEPTION.json) e o [registro próprio](../security/CARGO_WINDOWS_APPROVAL.json) registram approved=true e a decisão humana recebida em 2026-10-08. Escopo:

- glib 0.18.5: unsound / RUSTSEC-2024-0429;
- proc-macro-error 1.0.4: unmaintained / RUSTSEC-2024-0370;
- gate SEC-011/cargo-audit/0.22.2, target x86_64-pc-windows-msvc, hashes exatos de Cargo.lock e Cargo.toml;
- expiração em 2026-10-23 às 00h UTC: 22/10/2026 às 21h de Brasília, sem renovação automática.

ScopeSHA256: 5bbcd9f74bb6d831b11ecd337df33cb487e05637d6daa247ced096f7a081cca8. Somente estes warnings podem ser dispensados; vulnerabilidades, outros warnings ou inputs diferentes continuam bloqueados. A aprovação OSV SEC-009 não ativa esta proposta.

O registro ativo exige a mensagem humana literal “execução temporaria aprovada, siga com a meta em modo economia de tokens” e referência no formato “Codex user reply to SEC-011 proposal <SHA de 40 caracteres>”. O Integrador registra somente a resposta humana realmente recebida à proposta publicada. O checker valida frase, formato, scopehash e conteúdo do registro; a origem humana é evidência do chat e do histórico de revisão, sem autenticação criptográfica por este script. Texto sintético, recusa e referência de fixture são rejeitados em produção.

## Evidência e limites

Root após E4: 47 casos, seis execuções reais da CLI, 7.544s, fixtures removidas. Gate local: exit0, 417 dependências, zero vulnerabilidades classificadas, dois warnings excetuados, 2.598s. O teste adicional rejeita a frase apenas sugerida na pergunta, que não foi digitada pelo usuário. Execução sobre o working diff; audit no commit de entrega e CI permanecem necessários. Histórico antes do aceite: O scan online encontrou zero vulnerabilidades classificadas e dois warnings em 417 dependências; o gate real bloqueou com CARGO_WARNINGS_UNAPPROVED. Banco: 1295 advisories, commit 550efd3d587a29b2e2c2b21b17a440da4fede999, atualizado em 2026-10-08T16:47:14+02:00. Metadata Windows locked/offline/custom-protocol, Rust 1.99.0: 247 nós e ambas as dependências ausentes; hashes preservados. Isto não prova segurança geral das dependências.

As fixtures usam banco Git sintético local, --no-fetch e --no-yanked somente nos testes. A CLI offline omite commit/data do banco; o driver anota apenas estes dois campos usando o Git da própria fixture. O corpo dos achados vem da CLI real. A atualização online e consulta de yanked são verificadas pelo scan real separado. A aprovação sintética positiva exige patch explícito de constantes no harness; após removê-lo, o mesmo registro é rejeitado. Nenhum modo de teste existe no checker.

Revisão corrigiu P1 de aceite textual livre; RED reproduziu a aceitação indevida e 46 casos passaram após a correção. Segunda revisão focal P-REV 5.6-sol medium, E2: sem achados. Proposta pública 0e99e4c recebeu E4 própria com scope e prazo inalterados. Revisão focal da ativação P-REV 5.6-sol medium, E2: sem achados. CI de entrega ainda pendente; integração dos audits ao CI pertence ao SEC-012. Coverage Python percentual, telemetria e licenças transitivas: não registrado.

Scratch e fixtures ficam em artifacts/sec011-check e artifacts/sec011-tests, com proteção contra symlink/junction e limpeza somente dos caminhos próprios. Banco global e runtime não são removidos nem publicados no aplicativo.

## Entrega validada

[Commit 3af54a6](https://github.com/millennium42/millani-artes/commit/3af54a6dbcc6ca8d44bce9b4632a10aa97254458) / [CI 37824390707](https://github.com/millennium42/millani-artes/actions/runs/37824390707): 21 etapas verdes, build Windows verificado e coverage frontend do scaffold 100% (apenas quatro linhas). O gate Cargo local no mesmo SHA passou com 417 dependências, zero vulnerabilidades classificadas e os dois warnings aprovados, 2.595s. Os 47 casos locais passaram; checker/testes Cargo ainda não rodam no CI, integração prevista no SEC-012. Fechamento documental e recibo externo de liberação permanecem necessários. Nenhum código executável foi alterado depois dessas verificações.

## Integração SEC-012

O workflow passa a preparar o mesmo runtime 0.22.2 e banco oficial e a executar seus 47 casos/gate antes do build, inclusive em diff documental. [Operação integrada](CI_AUDITS.md) documenta bootstrap, provas offline e limites; CI exactSHA ainda pendente neste estado. Checker, hashes dos inputs, aceite próprio e prazo preservados.

## CI SEC-012 observado

[CI37832138829](https://github.com/millennium42/millani-artes/actions/runs/37832138829) noSHA680e8d2922f8149959df53c62e9b80390651e5c0/attempt1 passou23etapas. Gates reais npm120depszero eCargo417deps0vulns2excepted,40/47/42casos incluindo14PowerShell ebootstrapfrio verdadeiro. Falha inicialRTK local foi reproduzida e corrigida comcmd/git nativos, casos preservados. Reserva16ATIVA atéCIfinal/nota53; não háclaim de domíniofinanceiro/setup.exe. Histórico anterior mantém estado observado àépoca; estado vigente/details no handoffSEC012.
