# Segurança de dependências

Commitar lockfiles. Antes de adotar ferramenta/dependência, registrar finalidade e fonte. CI futuro executa `npm audit` (high/critical gate), `cargo audit`, OSV-Scanner, Gitleaks e SAST que tenha sinal real. Achados exigem correção ou exceção temporária aprovada.
