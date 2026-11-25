import { useEffect, useRef, useState, type FormEvent } from "react";
import { createProduct } from "./productApi";
import { parseEntryAmountCents } from "./entryInput";

const PRICE_HELP =
  "Use vírgula decimal, no máximo duas casas e sem separador de milhares. Zero é aceito.";

type FocusTarget = "alert" | "name" | "price" | null;

export default function ProductForm() {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [price, setPrice] = useState("");
  const [nameError, setNameError] = useState("");
  const [priceError, setPriceError] = useState("");
  const [formError, setFormError] = useState("");
  const [notice, setNotice] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [focusTarget, setFocusTarget] = useState<FocusTarget>(null);
  const isSubmittingRef = useRef(false);
  const nameRef = useRef<HTMLInputElement>(null);
  const priceRef = useRef<HTMLInputElement>(null);
  const alertRef = useRef<HTMLParagraphElement>(null);

  useEffect(() => {
    if (isSubmitting || focusTarget === null) return;
    if (focusTarget === "alert") alertRef.current?.focus();
    if (focusTarget === "name") nameRef.current?.focus();
    if (focusTarget === "price") priceRef.current?.focus();
    setFocusTarget(null);
  }, [focusTarget, isSubmitting]);

  function clearMessages() {
    setNameError("");
    setPriceError("");
    setFormError("");
    setNotice("");
    setFocusTarget(null);
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (isSubmittingRef.current) return;

    clearMessages();
    const suggestedPriceCents = parseEntryAmountCents(price);
    if (suggestedPriceCents === null) {
      setPriceError(PRICE_HELP);
      priceRef.current?.focus();
      return;
    }

    isSubmittingRef.current = true;
    setIsSubmitting(true);
    try {
      await createProduct({ name, description, suggestedPriceCents });
      setName("");
      setDescription("");
      setPrice("");
      setNotice("Produto cadastrado.");
    } catch (error) {
      if (error === "PRODUCT_NAME_INVALID") {
        setNameError("Digite um nome para o produto.");
        setFocusTarget("name");
      } else if (error === "PRODUCT_PRICE_INVALID") {
        setPriceError(PRICE_HELP);
        setFocusTarget("price");
      } else {
        setFormError("Não foi possível salvar o produto. Tente novamente.");
        setFocusTarget("alert");
      }
    } finally {
      isSubmittingRef.current = false;
      setIsSubmitting(false);
    }
  }

  return (
    <section className="setup-section" aria-labelledby="product-heading">
      <h2 id="product-heading">Cadastrar produto</h2>
      <p className="intro">
        Cadastre um modelo para usar nas vendas. O produto não controla estoque.
      </p>
      <form onSubmit={submit}>
        <fieldset disabled={isSubmitting}>
          <div className="field">
            <label htmlFor="product-name">Nome do produto</label>
            <input
              ref={nameRef}
              id="product-name"
              name="name"
              type="text"
              autoComplete="off"
              required
              value={name}
              aria-invalid={nameError !== ""}
              aria-describedby={
                nameError
                  ? "product-name-help product-name-error"
                  : "product-name-help"
              }
              onChange={(event) => {
                setName(event.target.value);
                clearMessages();
              }}
            />
            <p className="field-help" id="product-name-help">
              Informe o nome como deseja mantê-lo no cadastro.
            </p>
            {nameError && (
              <p className="field-error" id="product-name-error" role="alert">
                {nameError}
              </p>
            )}
          </div>

          <div className="field">
            <label htmlFor="product-description">Descrição do produto</label>
            <textarea
              id="product-description"
              name="description"
              autoComplete="off"
              rows={3}
              value={description}
              aria-describedby="product-description-help"
              onChange={(event) => {
                setDescription(event.target.value);
                clearMessages();
              }}
            />
            <p className="field-help" id="product-description-help">
              Opcional. O texto informado será preservado.
            </p>
          </div>

          <div className="field">
            <label htmlFor="product-price">Preço sugerido (R$)</label>
            <input
              ref={priceRef}
              id="product-price"
              name="suggestedPrice"
              type="text"
              inputMode="decimal"
              autoComplete="off"
              required
              value={price}
              aria-invalid={priceError !== ""}
              aria-describedby={
                priceError
                  ? "product-price-help product-price-error"
                  : "product-price-help"
              }
              onChange={(event) => {
                setPrice(event.target.value);
                clearMessages();
              }}
            />
            <p className="field-help" id="product-price-help">
              {PRICE_HELP}
            </p>
            {priceError && (
              <p className="field-error" id="product-price-error" role="alert">
                {priceError}
              </p>
            )}
          </div>

          {formError && (
            <p ref={alertRef} className="form-error" role="alert" tabIndex={-1}>
              {formError}
            </p>
          )}

          <button type="submit">
            {isSubmitting ? "Salvando…" : "Salvar produto"}
          </button>
        </fieldset>
      </form>
      {isSubmitting && (
        <p
          className="form-status"
          role="status"
          aria-live="polite"
          aria-atomic="true"
        >
          Salvando produto…
        </p>
      )}
      {notice && (
        <p
          className="form-status success"
          role="status"
          aria-live="polite"
          aria-atomic="true"
        >
          {notice}
        </p>
      )}
    </section>
  );
}
