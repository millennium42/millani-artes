import { useEffect, useRef, useState, type FormEvent } from "react";
import { createAccount, type AccountTypeKey } from "./accountApi";
import { parseInitialBalanceCents } from "./accountInput";
import CategoryForm from "./CategoryForm";

const accountTypes: ReadonlyArray<{ value: AccountTypeKey; label: string }> = [
  { value: "cash", label: "Dinheiro" },
  { value: "bank_pix", label: "Banco ou Pix" },
  { value: "card", label: "Cartão" },
  { value: "other", label: "Outro" },
];

const BALANCE_HELP =
  "Use vírgula decimal, no máximo duas casas e sem separador de milhares.";

export default function App() {
  const [name, setName] = useState("");
  const [accountType, setAccountType] = useState<AccountTypeKey | "">("");
  const [balance, setBalance] = useState("");
  const [nameError, setNameError] = useState("");
  const [balanceError, setBalanceError] = useState("");
  const [formError, setFormError] = useState("");
  const [notice, setNotice] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const isSubmittingRef = useRef(false);
  const nameRef = useRef<HTMLInputElement>(null);
  const balanceRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (nameError !== "" && !isSubmitting) nameRef.current?.focus();
  }, [nameError, isSubmitting]);

  function clearMessages() {
    setNameError("");
    setBalanceError("");
    setFormError("");
    setNotice("");
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (isSubmittingRef.current || accountType === "") return;

    clearMessages();
    const initialBalanceCents = parseInitialBalanceCents(balance);
    if (initialBalanceCents === null) {
      setBalanceError(BALANCE_HELP);
      balanceRef.current?.focus();
      return;
    }

    isSubmittingRef.current = true;
    setIsSubmitting(true);
    try {
      await createAccount({ name, accountType, initialBalanceCents });
      setName("");
      setAccountType("");
      setBalance("");
      setNotice("Conta criada. Você pode cadastrar outra.");
    } catch (error) {
      if (error === "ACCOUNT_NAME_INVALID") {
        setNameError("Digite um nome para a conta.");
      } else {
        setFormError("Não foi possível salvar a conta. Tente novamente.");
      }
    } finally {
      isSubmittingRef.current = false;
      setIsSubmitting(false);
    }
  }

  return (
    <>
      <header>Millani Artes</header>
      <main>
        <h1>Configure suas contas</h1>
        <p className="intro">Cadastre uma ou mais contas para começar.</p>
        <form onSubmit={submit}>
          <fieldset disabled={isSubmitting}>
            <div className="field">
              <label htmlFor="account-name">Nome da conta</label>
              <input
                ref={nameRef}
                id="account-name"
                name="name"
                type="text"
                autoComplete="off"
                required
                value={name}
                aria-invalid={nameError !== ""}
                aria-describedby={nameError ? "name-error" : undefined}
                onChange={(event) => {
                  setName(event.target.value);
                  clearMessages();
                }}
              />
              {nameError && (
                <p className="field-error" id="name-error" role="alert">
                  {nameError}
                </p>
              )}
            </div>

            <div className="field">
              <label htmlFor="account-type">Tipo de conta</label>
              <select
                id="account-type"
                name="accountType"
                required
                value={accountType}
                onChange={(event) => {
                  setAccountType(event.target.value as AccountTypeKey | "");
                  clearMessages();
                }}
              >
                <option value="" disabled>
                  Selecione um tipo
                </option>
                {accountTypes.map((type) => (
                  <option key={type.value} value={type.value}>
                    {type.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="field">
              <label htmlFor="initial-balance">Saldo inicial (R$)</label>
              <input
                ref={balanceRef}
                id="initial-balance"
                name="initialBalance"
                type="text"
                inputMode="decimal"
                autoComplete="off"
                required
                value={balance}
                aria-invalid={balanceError !== ""}
                aria-describedby={
                  balanceError ? "balance-help balance-error" : "balance-help"
                }
                onChange={(event) => {
                  setBalance(event.target.value);
                  clearMessages();
                }}
              />
              <p className="field-help" id="balance-help">
                {BALANCE_HELP}
              </p>
              {balanceError && (
                <p className="field-error" id="balance-error" role="alert">
                  {balanceError}
                </p>
              )}
            </div>

            {formError && (
              <p className="form-error" role="alert">
                {formError}
              </p>
            )}

            <button type="submit">
              {isSubmitting ? "Salvando…" : "Salvar conta"}
            </button>
          </fieldset>
        </form>
        {isSubmitting && (
          <p className="form-status" role="status" aria-live="polite">
            Salvando…
          </p>
        )}
        {notice && (
          <p className="form-status success" role="status" aria-live="polite">
            {notice}
          </p>
        )}
        <section className="setup-section">
          <h2>Categorias</h2>
          <p className="intro">
            Cadastre categorias para organizar seus registros.
          </p>
          <CategoryForm />
        </section>
      </main>
    </>
  );
}
