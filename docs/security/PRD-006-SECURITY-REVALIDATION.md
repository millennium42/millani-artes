# Revalidação de segurança — PRD-006, 23/10/2026

Estado atual: **aceite humano recebido em 23/10/2026**, limitado aos dois
pacotes, IDs, hashes, alvo Windows e prazo abaixo. Cargo e OSV passaram após
o registro do aceite. O registro da decisão está ao final deste documento.

Texto da proposta original, preservado para contexto: a autorização
“permissão explicita para verficiações de segunraça que tenham expirado” permite
executar as verificações. Naquele momento, os registros antigos estavam expirados.

As auditorias atuais encontraram seis avisos em Vite 8.3.2 e
RUSTSEC-2026-0386 em Tokio 1.53.2. Aplicados os patches Vite 8.3.3 e Tokio
1.53.3, os novos scans deixaram somente os dois pacotes já excepcionados.
Nenhum pacote foi adicionado ou removido pelo patch npm. A atualização Cargo
alterou versão e checksum do Tokio e resolveu três referências entre pacotes
já presentes: cssparser-macros 0.7.1 passou de syn 3.0.6 para 2.0.119;
dirs-sys 0.5.0 e winapi-util 0.1.11 passaram de windows-sys 0.61.2 para 0.59.0.
Nenhum outro pacote teve sua versão alterada. Os gates afetados são reexecutados;
resultados anteriores permanecem preservados fora do worktree.

Fontes: [release Vite 8.3.3](https://github.com/vitejs/vite/releases/tag/v8.3.3),
[aviso Vite](https://github.com/vitejs/vite/security/advisories/GHSA-9jrq-w75r-8gcw),
RustSec advisory-db no commit `84d943a6deeb8cc6c6f632531aaf4f1c62275239`.

## Escopo proposto para renovação

Responsável pelo preparo: P-INT; revisão independente P-REV. A decisão final
é humana. Reutilizar os gates SEC-009/OSV 2.6.0 e SEC-011/cargo-audit 0.22.2,
somente para `x86_64-pc-windows-msvc`, até **antes de 06/11/2026 UTC**.

| Pacote exato | IDs exatos | Risco mantido |
| --- | --- | --- |
| glib 0.18.5 | RUSTSEC-2024-0429; GHSA-wrw7-89jp-8q8g | Falha de segurança de memória em iterador de strings Variant; correção upstream a partir de 0.20.0. |
| proc-macro-error 1.0.4 | RUSTSEC-2024-0370 | Dependência sem manutenção; sem patch publicado no aviso. |

O `cargo tree --locked --target x86_64-pc-windows-msvc --edges all` confirmou
que ambos estão ausentes da resolução Windows, embora constem do lockfile
multiplataforma. Isso limita a exposição do alvo atual; não prova segurança
de Linux/macOS, manutenção futura ou ausência de outras vulnerabilidades.

Hashes que vinculam a proposta:

- `src-tauri/Cargo.lock`: `dcc811e8c3dfb8c4bab85bfb28b836ccc6c1816e16775348c31aed45b4e4d012`.
- `src-tauri/Cargo.toml`: `2d8b42708311f046d0e9c80451aaa8a582a76e15ce7d2a4a0b176f973b9a5b57`.

Não adicionar Tokio, Vite, outro ID, versão, alvo ou supressão aos aceites.
Qualquer mudança desses hashes ou a chegada da data exige nova decisão.
Na proposta original, os gates oficiais Cargo/OSV permaneceriam bloqueados
até o aceite. O aceite posterior e a execução dos gates constam acima e no
registro final, sem ampliar este escopo.

Controles compensatórios: scanner e inventário fixados, scans completos sem
ignorar achados desconhecidos, resolução Windows registrada, expiração estrita,
gates locais e CI no SHA exato. Remover a exceção ao atualizar a cadeia
GTK/Tauri que ainda referencia os pacotes, em tarefa própria antes de ampliar
a plataforma; nenhum item futuro é declarado concluído aqui.

Recibos externos: `PRD-006-security-refresh-patched-20261023.json`,
`PRD-006-security-platform-proof-20261023.json`, relatórios JSON brutos e
backups anteriores à correção. Tokens/custo: não registrado.


PRD-006 | retomada autorizada23out | E4 limitado ao aceite | Humana respondeu literalmente “execução temporaria aprovada, siga com a meta em modo economia de tokens” à proposta c86bf6753ee0bb8d579e74e699e4bab108222835. Renovados somente glib0.18.5/RUSTSEC-2024-0429/GHSA-wrw7-89jp-8q8g e proc-macro-error1.0.4/RUSTSEC-2024-0370, Windows x86_64-pc-windows-msvc, hashes Cargo.lock dcc811e8c3dfb8c4bab85bfb28b836ccc6c1816e16775348c31aed45b4e4d012 e Cargo.toml2d8b42708311f046d0e9c80451aaa8a582a76e15ce7d2a4a0b176f973b9a5b57, até antes06/11/2026UTC. Nenhum ID/versão/alvo adicional. Reserva adicional prévia dos4JSON de exceção/aceite; total25paths. Backups e vínculo SHA256 preservados; somente gates Cargo/OSV serão repetidos, demais checks válidos mantidos. CIs/nota134/liberação/E2E ainda pendentes; passesfalse.
