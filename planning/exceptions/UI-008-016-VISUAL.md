# Exceção temporária — inspeção visual da Meta UI-008–UI-016

- **ID / data / responsável:** EXC-UI-008-016-VISUAL / 2026-11-25 / decisão da usuária, registrada pelo P-INT.
- **Regra dispensada:** execução e inspeção visual manual da tela crítica de cada entrega UI, conforme `docs/quality/VISUAL_INSPECTION.md` e o critério visual de cada spec individual.
- **Motivo e risco aceito:** a usuária instruiu: “ignore verificação visual, conclua logo toda a UI”. Ela aceita que problemas de layout, contraste, overflow, foco e estados de interface possam passar sem detecção visual antes da liberação de cada item.
- **Controles compensatórios:** manter testes de comportamento React/API, semântica e acessibilidade verificável por teste, integração Tauri/SQLite, segurança, revisão, CI nos SHAs exatos e recibo de cada item. Os snapshots existentes são suplementares; não representam inspeção nativa. Não declarar conformidade WCAG nem prontidão visual global.
- **Escopo / revisão / expiração:** somente UI-008–UI-016 desta Meta; revisar a aplicação desta exceção em cada recibo de release. Expira em 2026-12-31 ou quando UI-016 for liberada, o que ocorrer primeiro.
- **Aprovação humana explícita / referência:** mensagem da usuária nesta conversa em 2026-11-25: “ignore verificação visual, conclua logo toda a UI”. E4 restrito à dispensa descrita neste registro.
- **Work item para remover a exceção:** UI-016; a exceção não se estende a outros IDs ou entregas.
