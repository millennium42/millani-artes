# ADR-001 — Tauri desktop
Contexto: Windows local-first com entrega instalável. Decisão: Tauri 2 e `setup.exe` NSIS como artefato padrão. Alternativas: MSI, Electron e web. Consequência: Rust mínimo e build Windows; risco: estratégia WebView2/assinatura. Rever se suporte Windows falhar ou distribuição corporativa exigir MSI.
