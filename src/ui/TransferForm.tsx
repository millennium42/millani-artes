import { useEffect, useRef, useState, type FormEvent } from "react";
import { listAccounts, type EntryAccount } from "./entryApi";
import { createTransfer, type TransferContext } from "./transferApi";
import { parseEntryAmountCents } from "./entryInput";

const AMOUNT_HELP =
  "Use vírgula decimal, no máximo duas casas e sem separador de milhares.";

type TransferFormProps = { accountRefreshKey?: number };
type ContextValue = TransferContext | "";

export default function TransferForm({
  accountRefreshKey = 0,
}: TransferFormProps) {
  const [accounts, setAccounts] = useState<EntryAccount[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);
  const [reloadKey, setReloadKey] = useState(0);
  const [sourceAccountId, setSourceAccountId] = useState("");
  const [destinationAccountId, setDestinationAccountId] = useState("");
  const [amount, setAmount] = useState("");
  const [context, setContext] = useState<ContextValue>("");
  const [amountError, setAmountError] = useState("");
  const [formError, setFormError] = useState("");
  const [notice, setNotice] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [focusTarget, setFocusTarget] = useState<"alert" | "source" | null>(
    null,
  );
  const isSubmittingRef = useRef(false);
  const currentLoadKeyRef = useRef("");
  const hasLoadedAccountsRef = useRef(false);
  const sourceAccountRef = useRef<HTMLSelectElement>(null);
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
    if (focusTarget === "source") sourceAccountRef.current?.focus();
    setFocusTarget(null);
  }, [focusTarget, isSubmitting]);

  function clearMessages() {
    setAmountError("");
    setFormError("");
    setNotice("");
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (
      isSubmittingRef.current ||
      sourceAccountId === "" ||
      destinationAccountId === "" ||
      context === ""
    ) {
      return;
    }

    clearMessages();
    const amountCents = parseEntryAmountCents(amount);
    if (amountCents === null) {
      setAmountError(AMOUNT_HELP);
      amountRef.current?.focus();
      return;
    }

    const sourceAccount = accounts.find(
      (account) => account.id.join(",") === sourceAccountId,
    );
    const destinationAccount = accounts.find(
      (account) => account.id.join(",") === destinationAccountId,
    );
    if (!sourceAccount || !destinationAccount) return;

    isSubmittingRef.current = true;
    setIsSubmitting(true);
    try {
      await createTransfer({
        sourceAccountId: sourceAccount.id,
        destinationAccountId: destinationAccount.id,
        amountCents,
        context,
      });
      setSourceAccountId("");
      setDestinationAccountId("");
      setAmount("");
      setContext("");
      setNotice("Transferência registrada.");
      setFocusTarget("source");
    } catch {
      setFormError(
        "Não foi possível registrar a transferência. Tente novamente.",
      );
      setFocusTarget("alert");
    } finally {
      isSubmittingRef.current = false;
      setIsSubmitting(false);
    }
  }

  if (isLoading && !hasLoadedAccountsRef.current) {
    return (
      <section className="setup-section" aria-labelledby="transfer-heading">
        <h2 id="transfer-heading">Transferir entre contas</h2>
        <p className="form-status" role="status" aria-live="polite">
          Carregando contas…
        </p>
      </section>
    );
  }

  if (loadError && !hasLoadedAccountsRef.current) {
    return (
      <section className="setup-section" aria-labelledby="transfer-heading">
        <h2 id="transfer-heading">Transferir entre contas</h2>
        <p className="form-error" role="alert">
          Não foi possível carregar as contas. Tente novamente.
        </p>
        <button type="button" onClick={() => setReloadKey((key) => key + 1)}>
          Tentar novamente
        </button>
      </section>
    );
  }

  if (accounts.length < 2) {
    return (
      <section className="setup-section" aria-labelledby="transfer-heading">
        <h2 id="transfer-heading">Transferir entre contas</h2>
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
          Cadastre ao menos duas contas para transferir.
        </p>
      </section>
    );
  }

  return (
    <section className="setup-section" aria-labelledby="transfer-heading">
      <h2 id="transfer-heading">Transferir entre contas</h2>
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
            <label htmlFor="transfer-source">Conta de origem</label>
            <select
              ref={sourceAccountRef}
              id="transfer-source"
              name="sourceAccountId"
              required
              value={sourceAccountId}
              onChange={(event) => {
                setSourceAccountId(event.target.value);
                clearMessages();
              }}
            >
              <option value="" disabled>
                Selecione a conta de origem
              </option>
              {accounts.map((account) => (
                <option key={account.id.join(",")} value={account.id.join(",")}>
                  {account.name}
                </option>
              ))}
            </select>
          </div>

          <div className="field">
            <label htmlFor="transfer-destination">Conta de destino</label>
            <select
              id="transfer-destination"
              name="destinationAccountId"
              required
              value={destinationAccountId}
              onChange={(event) => {
                setDestinationAccountId(event.target.value);
                clearMessages();
              }}
            >
              <option value="" disabled>
                Selecione a conta de destino
              </option>
              {accounts.map((account) => (
                <option key={account.id.join(",")} value={account.id.join(",")}>
                  {account.name}
                </option>
              ))}
            </select>
          </div>

          <div className="field">
            <label htmlFor="transfer-amount">Valor (R$)</label>
            <input
              ref={amountRef}
              id="transfer-amount"
              name="amount"
              type="text"
              inputMode="decimal"
              autoComplete="off"
              required
              value={amount}
              aria-invalid={amountError !== ""}
              aria-describedby={
                amountError
                  ? "transfer-amount-help transfer-amount-error"
                  : "transfer-amount-help"
              }
              onChange={(event) => {
                setAmount(event.target.value);
                clearMessages();
              }}
            />
            <p className="field-help" id="transfer-amount-help">
              {AMOUNT_HELP}
            </p>
            {amountError && (
              <p
                className="field-error"
                id="transfer-amount-error"
                role="alert"
              >
                {amountError}
              </p>
            )}
          </div>

          <div className="field">
            <label htmlFor="transfer-context">Contexto</label>
            <select
              id="transfer-context"
              name="context"
              required
              value={context}
              onChange={(event) => {
                setContext(event.target.value as ContextValue);
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

          <button type="submit">
            {isSubmitting ? "Registrando…" : "Registrar transferência"}
          </button>
        </fieldset>
      </form>
      {isSubmitting && (
        <p className="form-status" role="status" aria-live="polite">
          Registrando transferência…
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
