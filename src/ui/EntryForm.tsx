import { useEffect, useRef, useState, type FormEvent } from "react";
import { createEntry, listAccounts, type EntryAccount } from "./entryApi";
import { parseEntryAmountCents } from "./entryInput";

const AMOUNT_HELP =
  "Use vírgula decimal, no máximo duas casas e sem separador de milhares.";

type EntryFormProps = { accountRefreshKey?: number };

export default function EntryForm({ accountRefreshKey = 0 }: EntryFormProps) {
  const [accounts, setAccounts] = useState<EntryAccount[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);
  const [reloadKey, setReloadKey] = useState(0);
  const [accountId, setAccountId] = useState("");
  const [amount, setAmount] = useState("");
  const [context, setContext] = useState<"home" | "factory" | "">("");
  const [amountError, setAmountError] = useState("");
  const [formError, setFormError] = useState("");
  const [notice, setNotice] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [focusTarget, setFocusTarget] = useState<"alert" | "account" | null>(
    null,
  );
  const isSubmittingRef = useRef(false);
  const currentLoadKeyRef = useRef("");
  const hasLoadedAccountsRef = useRef(false);
  const accountRef = useRef<HTMLSelectElement>(null);
  const amountRef = useRef<HTMLInputElement>(null);
  const alertRef = useRef<HTMLParagraphElement>(null);

  useEffect(() => {
    const loadKey = `${accountRefreshKey}:${reloadKey}`;
    currentLoadKeyRef.current = loadKey;
    let active = true;
    setIsLoading(true);
    setLoadError(false);
    void listAccounts()
      .then((result) => {
        if (active && currentLoadKeyRef.current === loadKey) {
          setAccounts(result);
          hasLoadedAccountsRef.current = true;
        }
      })
      .catch(() => {
        if (active && currentLoadKeyRef.current === loadKey) setLoadError(true);
      })
      .finally(() => {
        if (active && currentLoadKeyRef.current === loadKey)
          setIsLoading(false);
      });
    return () => {
      active = false;
    };
  }, [accountRefreshKey, reloadKey]);

  useEffect(() => {
    if (isSubmitting || focusTarget === null) return;
    if (focusTarget === "alert") alertRef.current?.focus();
    if (focusTarget === "account") accountRef.current?.focus();
    setFocusTarget(null);
  }, [focusTarget, isSubmitting]);

  function clearMessages() {
    setAmountError("");
    setFormError("");
    setNotice("");
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (isSubmittingRef.current || accountId === "" || context === "") return;

    clearMessages();
    const amountCents = parseEntryAmountCents(amount);
    if (amountCents === null) {
      setAmountError(AMOUNT_HELP);
      amountRef.current?.focus();
      return;
    }

    const selectedAccount = accounts.find(
      (account) => account.id.join(",") === accountId,
    );
    if (!selectedAccount) return;

    isSubmittingRef.current = true;
    setIsSubmitting(true);
    try {
      await createEntry({
        accountId: selectedAccount.id,
        amountCents,
        context,
      });
      setAccountId("");
      setAmount("");
      setContext("");
      setNotice("Entrada registrada.");
      setFocusTarget("account");
    } catch {
      setFormError("Não foi possível registrar a entrada. Tente novamente.");
      setFocusTarget("alert");
    } finally {
      isSubmittingRef.current = false;
      setIsSubmitting(false);
    }
  }

  if (isLoading && !hasLoadedAccountsRef.current) {
    return (
      <section className="setup-section" aria-labelledby="entry-heading">
        <h2 id="entry-heading">Registrar uma entrada</h2>
        <p className="form-status" role="status" aria-live="polite">
          Carregando contas…
        </p>
      </section>
    );
  }

  if (loadError && !hasLoadedAccountsRef.current) {
    return (
      <section className="setup-section" aria-labelledby="entry-heading">
        <h2 id="entry-heading">Registrar uma entrada</h2>
        <p className="form-error" role="alert">
          Não foi possível carregar as contas. Tente novamente.
        </p>
        <button type="button" onClick={() => setReloadKey((key) => key + 1)}>
          Tentar novamente
        </button>
      </section>
    );
  }

  if (accounts.length === 0) {
    return (
      <section className="setup-section" aria-labelledby="entry-heading">
        <h2 id="entry-heading">Registrar uma entrada</h2>
        {isLoading && (
          <p className="form-status" role="status" aria-live="polite">
            Atualizando contas…
          </p>
        )}
        {loadError && (
          <>
            <p className="form-error" role="alert">
              Não foi possível atualizar as contas. Tente novamente.
            </p>
            <button
              type="button"
              onClick={() => setReloadKey((key) => key + 1)}
            >
              Tentar novamente
            </button>
          </>
        )}
        <p className="form-status" role="status">
          Cadastre uma conta para registrar uma entrada.
        </p>
        <button type="button" disabled>
          Registrar entrada
        </button>
      </section>
    );
  }

  return (
    <section className="setup-section" aria-labelledby="entry-heading">
      <h2 id="entry-heading">Registrar uma entrada</h2>
      {isLoading && (
        <p className="form-status" role="status" aria-live="polite">
          Atualizando contas…
        </p>
      )}
      {loadError && (
        <>
          <p className="form-error" role="alert">
            Não foi possível atualizar as contas. Tente novamente.
          </p>
          <button type="button" onClick={() => setReloadKey((key) => key + 1)}>
            Tentar novamente
          </button>
        </>
      )}
      <form onSubmit={submit}>
        <fieldset disabled={isSubmitting}>
          <div className="field">
            <label htmlFor="entry-account">Conta</label>
            <select
              ref={accountRef}
              id="entry-account"
              name="account"
              required
              value={accountId}
              onChange={(event) => {
                setAccountId(event.target.value);
                clearMessages();
              }}
            >
              <option value="" disabled>
                Selecione uma conta
              </option>
              {accounts.map((account) => (
                <option key={account.id.join(",")} value={account.id.join(",")}>
                  {account.name}
                </option>
              ))}
            </select>
          </div>

          <div className="field">
            <label htmlFor="entry-amount">Valor (R$)</label>
            <input
              ref={amountRef}
              id="entry-amount"
              name="amount"
              type="text"
              inputMode="decimal"
              autoComplete="off"
              required
              value={amount}
              aria-invalid={amountError !== ""}
              aria-describedby={
                amountError
                  ? "entry-amount-help entry-amount-error"
                  : "entry-amount-help"
              }
              onChange={(event) => {
                setAmount(event.target.value);
                clearMessages();
              }}
            />
            <p className="field-help" id="entry-amount-help">
              {AMOUNT_HELP}
            </p>
            {amountError && (
              <p className="field-error" id="entry-amount-error" role="alert">
                {amountError}
              </p>
            )}
          </div>

          <div className="field">
            <label htmlFor="entry-context">Contexto</label>
            <select
              id="entry-context"
              name="context"
              required
              value={context}
              onChange={(event) => {
                setContext(event.target.value as "home" | "factory" | "");
                clearMessages();
              }}
            >
              <option value="" disabled>
                Selecione um contexto
              </option>
              <option value="home">Casa</option>
              <option value="factory">Fábrica</option>
            </select>
          </div>

          {formError && (
            <p ref={alertRef} className="form-error" role="alert" tabIndex={-1}>
              {formError}
            </p>
          )}
          <button type="submit">Registrar entrada</button>
        </fieldset>
      </form>
      {isSubmitting && (
        <p className="form-status" role="status" aria-live="polite">
          Registrando entrada…
        </p>
      )}
      {notice && (
        <p className="form-status success" role="status" aria-live="polite">
          {notice}
        </p>
      )}
    </section>
  );
}
