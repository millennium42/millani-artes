import { useEffect, useRef, useState, type FormEvent } from "react";
import { createCategory } from "./categoryApi";

export default function CategoryForm() {
  const [name, setName] = useState("");
  const [nameError, setNameError] = useState("");
  const [formError, setFormError] = useState("");
  const [notice, setNotice] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const isSubmittingRef = useRef(false);
  const nameRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (nameError !== "" && !isSubmitting) nameRef.current?.focus();
  }, [nameError, isSubmitting]);

  function clearMessages() {
    setNameError("");
    setFormError("");
    setNotice("");
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (isSubmittingRef.current) return;

    clearMessages();
    isSubmittingRef.current = true;
    setIsSubmitting(true);
    try {
      await createCategory(name);
      setName("");
      setNotice("Categoria criada. Você pode cadastrar outra.");
    } catch (error) {
      if (error === "CATEGORY_NAME_INVALID") {
        setNameError("Digite um nome para a categoria.");
      } else {
        setFormError("Não foi possível salvar a categoria. Tente novamente.");
      }
    } finally {
      isSubmittingRef.current = false;
      setIsSubmitting(false);
    }
  }

  return (
    <form onSubmit={submit}>
      <fieldset disabled={isSubmitting}>
        <div className="field">
          <label htmlFor="category-name">Nome da categoria</label>
          <input
            ref={nameRef}
            id="category-name"
            name="name"
            type="text"
            autoComplete="off"
            required
            value={name}
            aria-invalid={nameError !== ""}
            aria-describedby={
              nameError !== "" ? "category-name-error" : undefined
            }
            onChange={(event) => {
              setName(event.target.value);
              clearMessages();
            }}
          />
          {nameError && (
            <p className="field-error" id="category-name-error" role="alert">
              {nameError}
            </p>
          )}
        </div>

        {formError && (
          <p className="form-error" role="alert">
            {formError}
          </p>
        )}

        <button type="submit">
          {isSubmitting ? "Salvando…" : "Salvar categoria"}
        </button>
      </fieldset>
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
    </form>
  );
}
