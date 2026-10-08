# Audits obrigatórios no CI — SEC-012

O workflow Documentation mantém um job Windows, as três actions fixas e o upload de coverage existente com retenção de um dia. Depois do Node fixo, executa npm e Cargo antes de instalar dependências, publicar coverage ou compilar. Mudanças documentais continuam dispensando as sete etapas frontend/nativas; os audits e o Node permanecem obrigatórios. O cache npm existente só é restaurado/salvo quando o seletor exige instalação/build; em diff documental o input cache fica vazio e package-manager-cache continua false. Nenhuma nova matriz, action, cache remoto ou upload.

## Execução local

Use o Node 24.21.0/npm 11.19.0 fixo no PATH. O Cargo Audit portátil deve ser preparado antes dos testes:
```
python -B scripts/test-npm-audit.py
python -B scripts/check_npm_audit.py
python -B scripts/prepare_cargo_audit.py
python -B scripts/test-ci-audits.py
python -B scripts/test-cargo-audit.py
python -B scripts/check_cargo_audit.py
```

Cada comando é seguido no workflow por uma verificação do exit code que lança erro. Falha de preparação, fixture ou gate impede as etapas seguintes pelo comportamento normal do job. Não há condição nem continue-on-error nos audits. [npm](NPM_AUDIT.md) mantém high/critical; [Cargo](CARGO_AUDIT.md) mantém lock integral, warnings e vulnerabilidades, atualização RustSec e consulta de yanked.

## Preparação Cargo

[Release oficial 0.22.2](https://github.com/rustsec/rustsec/releases/tag/cargo-audit/v0.22.2), ZIP Windows MSVC de 6192256 bytes, SHA256 0a7316540862c13d954f648917ceacca593747baed6eec180fafa590be2710ab; EXE SHA256 0157f5ce1ce9fd4fb0a1f7c79af1229771d1f80b6c2613ddb0d9200a8ba73946. Python stdlib limita o download, valida integridade e as seis entradas antes de gravar cinco arquivos fixos, incluindo LICENSE-MIT e LICENSE-APACHE. Não usa extractall. Cache local em LOCALAPPDATA/MillaniArtesDev, mesmo runtime registrado no INF-002.

Banco ausente é clonado de https://github.com/RustSec/advisory-db.git no caminho já usado pelo gate; origem, limpeza e HEAD são validados. Runtime/cache/banco existentes divergentes ou reparse points bloqueiam. Não há substituição, remoção ou reparo automático desses caminhos globais. Uma instalação interrompida exige investigação local.

## Prova e limites

42 casos locais: forma restrita do workflow com 11 mutações de bypass rejeitadas, 14 processos PowerShell reais com corpos extraídos do arquivo e exits 0/1/2, bootstrap frio/quente e negativos. Nos erros, filhos posteriores e o marcador de build não executam. Fixtures usam apenas arquivos, ZIP oficial validado, banco Git e subprocessos próprios; download e clone remoto são substituídos somente no harness. Não fazem chamadas negativas públicas. As 40 fixtures npm e 47 Cargo existentes são executadas separadamente e continuam inalteradas.

O parser da prova não valida toda a linguagem YAML nem simula a engine GitHub. CI verde no SHA exato e logs dos dois gates são necessários para a evidência remota. Coverage Python percentual: não registrado. SourceSha do runtime preparado e de ambos os scans identifica o checkout; reports brutos e credenciais não são publicados. Segurança e cleanup limitam-se aos caminhos próprios.

Aceites OSV e Cargo anteriores permanecem separados, com mesmos inputs, escopos e prazo de 2026-10-23T00UTC (22/10 às 21h Brasília), sem renovação. Expiração, inputs divergentes e outros achados bloqueiam. Isso não conclui o aplicativo financeiro nem o instalador.

Estado/checks/review/CI e recibo de liberação: [spec](../../planning/specs/SEC-012.md) e [handoff](../../planning/handoffs/SEC-012.md).

## Correção de portabilidade

CI inicial37830906943/e982e82 falhou noRTK ausente antes do build. Root reproduziu WinError2 removendo RTK do PATH; após remover três prefixos locais, o mesmo ambiente semRTK passou40npm/47Cargo/42integração (10.144s/8.269s/20.063s). Casos e gates mantidos, nenhum RTK instalado no runner. CI corrigido ainda pendente; execução inicial falha será preservada no recibo.
