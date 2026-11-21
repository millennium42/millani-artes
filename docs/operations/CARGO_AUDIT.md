# Cargo audit — gate local

Na raiz, com Python 3.12 ou superior e cargo-audit 0.22.2 portátil instalado pelo INF-002:
```powershell
rtk proxy python -B scripts/test-cargo-audit.py
rtk proxy python -B scripts/check_cargo_audit.py
```

O checker confere tamanho, hash e versão do binário, lê o Cargo.lock completo e cria configuração própria, sem ignores ou filtro de plataforma. Atualiza o banco oficial RustSec por HTTPS e exige origem Git oficial, checkout limpo e commit correspondente ao relatório. Vulnerabilidades, warnings sem aprovação, erros, relatório incoerente, timeout e mudança nos inputs bloqueiam. Não modifica dependências. Publica somente metadados públicos e códigos estáticos; relatório bruto e stderr ficam em memória.

A [CLI 0.22.2](https://github.com/rustsec/rustsec/blob/cargo-audit%2Fv0.22.2/cargo-audit/src/commands/audit.rs) nega warnings explicitamente. A [política de dependências](../security/DEPENDENCY_SECURITY.md) exige correção ou exceção humana temporária.

## Estado atual e histórico da exceção

Revalidação local de DSH-009 em 2026-11-12 UTC: a configuração atual conserva somente glib 0.18.5 (`RUSTSEC-2024-0429`) e proc-macro-error 1.0.4 (`RUSTSEC-2024-0370`), para o target Windows e hashes atuais de `Cargo.lock`/`Cargo.toml`; `expiresBeforeUTC` é `2026-11-14`. Cargo Audit 0.22.2 consultou 422 dependências no clone RustSec oficial limpo `fd6f39814c4d3c359fdbe223d0280dec4933f78f`: zero vulnerabilidades e apenas esses dois avisos. A autorização humana registrada não acrescenta findings ou exceções. Os três findings OSV permanecem no registro OSV separado e não são cobertos por este gate Cargo.

O registro abaixo documenta o aceite original e é histórico; a configuração/registro atuais nos JSON vinculados acima são a fonte de validade corrente.

A [configuração](../security/CARGO_WINDOWS_EXCEPTION.json) e o [registro próprio](../security/CARGO_WINDOWS_APPROVAL.json) registram approved=true e a decisão humana recebida em 2026-10-08. Escopo:

- glib 0.18.5: unsound / RUSTSEC-2024-0429;
- proc-macro-error 1.0.4: unmaintained / RUSTSEC-2024-0370;
- gate SEC-011/cargo-audit/0.22.2, target x86_64-pc-windows-msvc, hashes exatos de Cargo.lock e Cargo.toml;
- validade antes de 2026-11-06 UTC, após revalidação humana de 2026-10-23 documentada em PRD-006-SECURITY-REVALIDATION.md; sem renovação automática.

ScopeSHA256: 4741323d5774eaa101b5ccaf8a03123eeeb64f1238d89169eb7c44b24eb2d3f2; approval and inputs in CARGO_WINDOWS_APPROVAL.json and CARGO_WINDOWS_EXCEPTION.json. Somente estes warnings podem ser dispensados; vulnerabilidades, outros warnings ou inputs diferentes continuam bloqueados. A aprovação OSV SEC-009 não ativa esta proposta.

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

### UI-001 status-sync — SECURITY recheck (2026-11-20)

Root/Windows, Cargo Audit 0.22.2 (`0157f5ce1ce9fd4fb0a1f7c79af1229771d1f80b6c2613ddb0d9200a8ba73946`), executado em checkout dirty com `HEAD=79382b38c9df8014277c88a6eadc58dec6b9d39d`; não foi execução desse commit isolado. No review foi capturado índice candidato `f397eb0d2ad352f9752d0eef339387036b9607b6`; checker, aprovações, exceções e lockfiles lidos pelo scan têm os mesmos blobs nesse índice. Alterações posteriores de docs/testes não foram entradas do scanner; não se alega que o scan certifica o tree inteiro. Inputs Cargo.lock `68d2fc69b4b1c87ca1b30b1eec879e5d9726570d013aa68fa271a84131a47433` e Cargo.toml `8bf107e28bb0646142adac0c4099c6d7dd00586e42b3acb313badd320eb8ff5e`. O runner padrão foi bloqueado por `CARGO_DATABASE_DIRTY`: o banco global tinha 6 entradas untracked, preservadas. Repetição E3 via `check_cargo_audit.scan()` e clone oficial isolado/limpo do `RustSec/advisory-db`, HEAD `f291af9b1cc0d21dfcea1709d6b037963cef262b`, cobriu 422 crates e saiu bloqueada por `CARGO_WARNINGS_UNAPPROVED`: as duas identidades já aprovadas (glib 0.18.5 / RUSTSEC-2024-0429; proc-macro-error 1.0.4 / RUSTSEC-2024-0370) e novo warning não aprovado chrono 0.4.45 / RUSTSEC-2026-0543 / unmaintained. Cargo foi localizado em `C:\Users\milla\.cargo\bin\cargo.exe`: metadata Windows locked/offline resolveu os 422 crates; `serde_with` não habilita a feature `chrono`, e `cargo tree --locked --offline --target x86_64-pc-windows-msvc -i chrono` não mostrou caminho ativo. `chrono` continua no Cargo.lock como dependência opcional declarada por `serde_with`; isso não reduz o inventário integral que Cargo Audit avalia e não há política de exclusão por feature/target. Nenhum lock, dependência ou exceção para chrono foi alterado. O resultado é local, não CI remoto; telemetria não registrado.

### UI-001 status-sync — autorização e recheck atual (2026-11-20)

A decisão direta da usuária “autorizo explicitamente o que for preciso para avançar até ui 016” nesta thread (01a1e070-899d-7b71-8021-abc1b05dd5d6) foi vinculada ao escopo exato por CARGO_WINDOWS_APPROVAL.json: somente chrono 0.4.45/RUSTSEC-2026-0543 do tipo não mantido, preservando os avisos históricos de glib 0.18.5/RUSTSEC-2024-0429 e proc-macro-error 1.0.4/RUSTSEC-2024-0370, target x86_64-pc-windows-msvc, hashes atuais de Cargo.lock/Toml e validade até 2026-11-26 UTC (expiresBeforeUTC=2026-11-27). Digest do escopo: 5176762e94f46f5a83c6891210a96576d5e26e2dfc58b2a4397cf91e5d3d030c. A autorização não declara que chrono é seguro nem que o advisory é vulnerabilidade explorável. O registro vincula a instrução ampla da usuária a esta dispensa, mas não registra confirmação humana separada e específica do risco de falta de manutenção; esse risco permanece.

Root/Windows executou scripts.check_cargo_audit.scan() com Cargo Audit 0.22.2 (0157f5ce1ce9fd4fb0a1f7c79af1229771d1f80b6c2613ddb0d9200a8ba73946) e clone oficial limpo RustSec/advisory-db no commit f291af9b1cc0d21dfcea1709d6b037963cef262b: 422 crates, 1555 advisories, 3 warnings nas identidades aprovadas, 0 vulnerabilidades; exit 0. Inputs: Cargo.lock 68d2fc69b4b1c87ca1b30b1eec879e5d9726570d013aa68fa271a84131a47433, Cargo.toml 8bf107e28bb0646142adac0c4099c6d7dd00586e42b3acb313badd320eb8ff5e. Execução local no Windows sobre checkout dirty HEAD=79382b38c9df8014277c88a6eadc58dec6b9d39d; os blobs security relevantes estavam no tree staged  c4f883c74ec3da9ce9bf7e90a81393bb86bcd7be. Não é CI nem prova do SHA final de release; telemetria não registrado. O banco global dirty com 6 entradas do usuário foi preservado.
