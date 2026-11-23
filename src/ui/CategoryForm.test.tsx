// @vitest-environment jsdom

import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, test, vi } from "vitest";
import CategoryForm from "./CategoryForm";
import * as categoryApi from "./categoryApi";

vi.mock("./categoryApi", () => ({
  createCategory: vi.fn(),
}));

afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

test("expõe nome rotulado e formulário por teclado", async () => {
  const user = userEvent.setup();
  render(<CategoryForm />);

  const name = screen.getByRole("textbox", { name: "Nome da categoria" });
  const submit = screen.getByRole("button", { name: "Salvar categoria" });

  await user.tab();
  expect(document.activeElement).toBe(name);
  await user.tab();
  expect(document.activeElement).toBe(submit);
});

test("campo vazio é bloqueado sem chamar a API", async () => {
  const user = userEvent.setup();
  render(<CategoryForm />);

  const name = screen.getByRole("textbox", { name: "Nome da categoria" });
  expect((name as HTMLInputElement).required).toBe(true);
  expect((name as HTMLInputElement).validity.valueMissing).toBe(true);

  await user.click(screen.getByRole("button", { name: "Salvar categoria" }));

  expect(categoryApi.createCategory).not.toHaveBeenCalled();
});

test("envia whitespace Unicode literal e associa a validação ao campo", async () => {
  const user = userEvent.setup();
  vi.mocked(categoryApi.createCategory).mockRejectedValueOnce(
    "CATEGORY_NAME_INVALID",
  );
  render(<CategoryForm />);

  const name = screen.getByRole("textbox", { name: "Nome da categoria" });
  const invalidName = " \u00a0\u2003 ";
  await user.type(name, invalidName);
  await user.click(screen.getByRole("button", { name: "Salvar categoria" }));

  expect(categoryApi.createCategory).toHaveBeenCalledWith(invalidName);
  expect((await screen.findByRole("alert")).textContent).toBe(
    "Digite um nome para a categoria.",
  );
  expect(name.getAttribute("aria-invalid")).toBe("true");
  expect(name.getAttribute("aria-describedby")).toBe("category-name-error");
  expect(document.activeElement).toBe(name);
});

test("envia o nome literal, anuncia sucesso e limpa o campo", async () => {
  const user = userEvent.setup();
  vi.mocked(categoryApi.createCategory).mockResolvedValue(undefined);
  render(<CategoryForm />);

  const name = screen.getByRole("textbox", { name: "Nome da categoria" });
  const literalName = "  Categoria Açúcar 🧵  ";
  await user.type(name, literalName);
  await user.click(screen.getByRole("button", { name: "Salvar categoria" }));

  expect(categoryApi.createCategory).toHaveBeenCalledWith(literalName);
  expect(
    await screen.findByText("Categoria criada. Você pode cadastrar outra."),
  ).toBeTruthy();
  expect((name as HTMLInputElement).value).toBe("");
});

test("falha de persistência é genérica e não expõe path nem SQL", async () => {
  const user = userEvent.setup();
  vi.mocked(categoryApi.createCategory).mockRejectedValueOnce(
    "path=C:\\secret\\db.sqlite SQL=INSERT",
  );
  render(<CategoryForm />);

  const name = screen.getByRole("textbox", { name: "Nome da categoria" });
  await user.type(name, "Artesanato");
  await user.click(screen.getByRole("button", { name: "Salvar categoria" }));

  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toBe(
    "Não foi possível salvar a categoria. Tente novamente.",
  );
  expect(alert.textContent).not.toContain("secret");
  expect((name as HTMLInputElement).value).toBe("Artesanato");
});

test("anuncia o envio e impede uma segunda submissão enquanto salva", async () => {
  const user = userEvent.setup();
  let finish: (() => void) | undefined;
  const pending = new Promise<void>((resolve) => {
    finish = resolve;
  });
  vi.mocked(categoryApi.createCategory).mockReturnValue(pending);
  render(<CategoryForm />);

  await user.type(
    screen.getByRole("textbox", { name: "Nome da categoria" }),
    "Categoria",
  );
  await user.click(screen.getByRole("button", { name: "Salvar categoria" }));

  expect(screen.getByRole("status").textContent).toBe("Salvando…");
  expect(
    screen.getByRole("button", { name: "Salvando…" }).matches(":disabled"),
  ).toBe(true);
  await user.click(screen.getByRole("button", { name: "Salvando…" }));
  expect(categoryApi.createCategory).toHaveBeenCalledTimes(1);

  finish?.();
  expect(
    await screen.findByText("Categoria criada. Você pode cadastrar outra."),
  ).toBeTruthy();
});
