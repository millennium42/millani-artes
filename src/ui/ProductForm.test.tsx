// @vitest-environment jsdom

import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, test, vi } from "vitest";
import ProductForm from "./ProductForm";
import * as productApi from "./productApi";

vi.mock("./productApi", () => ({
  createProduct: vi.fn(),
}));

type User = ReturnType<typeof userEvent.setup>;

async function fillForm(
  user: User,
  values: { name?: string; description?: string; price?: string } = {},
) {
  await user.type(
    screen.getByRole("textbox", { name: "Nome do produto" }),
    values.name ?? "Modelo sintético",
  );
  const description = values.description ?? "Descrição sintética";
  if (description !== "") {
    await user.type(
      screen.getByRole("textbox", { name: "Descrição do produto" }),
      description,
    );
  }
  await user.type(
    screen.getByRole("textbox", { name: "Preço sugerido (R$)" }),
    values.price ?? "12,34",
  );
}

afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

test("rótulos e teclado seguem nome, descrição, preço e envio", async () => {
  const user = userEvent.setup();
  render(<ProductForm />);

  const name = screen.getByRole("textbox", {
    name: "Nome do produto",
  }) as HTMLInputElement;
  const description = screen.getByRole("textbox", {
    name: "Descrição do produto",
  });
  const price = screen.getByRole("textbox", {
    name: "Preço sugerido (R$)",
  }) as HTMLInputElement;
  const submit = screen.getByRole("button", { name: "Salvar produto" });

  expect(name.getAttribute("aria-describedby")).toContain("product-name-help");
  expect(description.getAttribute("aria-describedby")).toContain(
    "product-description-help",
  );
  expect(price.getAttribute("aria-describedby")).toContain(
    "product-price-help",
  );
  await user.tab();
  expect(document.activeElement).toBe(name);
  await user.tab();
  expect(document.activeElement).toBe(description);
  await user.tab();
  expect(document.activeElement).toBe(price);
  await user.tab();
  expect(document.activeElement).toBe(submit);
});

test("preserva texto literal e aceita preço zero após persistência", async () => {
  const user = userEvent.setup();
  vi.mocked(productApi.createProduct).mockResolvedValue(undefined);
  render(<ProductForm />);
  await fillForm(user, {
    name: "  Modelo Árvore 🪴  ",
    description: "  Feito à mão  ",
    price: "0,00",
  });

  await user.click(screen.getByRole("button", { name: "Salvar produto" }));

  expect(productApi.createProduct).toHaveBeenCalledWith({
    name: "  Modelo Árvore 🪴  ",
    description: "  Feito à mão  ",
    suggestedPriceCents: "0",
  });
  expect(await screen.findByText("Produto cadastrado.")).toBeTruthy();
  expect(
    (
      screen.getByRole("textbox", {
        name: "Nome do produto",
      }) as HTMLInputElement
    ).value,
  ).toBe("");
  expect(
    (
      screen.getByRole("textbox", {
        name: "Descrição do produto",
      }) as HTMLTextAreaElement
    ).value,
  ).toBe("");
});

test("converte decimal exato para centavos textuais", async () => {
  const user = userEvent.setup();
  vi.mocked(productApi.createProduct).mockResolvedValue(undefined);
  render(<ProductForm />);
  await fillForm(user, { price: "123456789012345,67" });

  await user.click(screen.getByRole("button", { name: "Salvar produto" }));

  expect(productApi.createProduct).toHaveBeenCalledWith({
    name: "Modelo sintético",
    description: "Descrição sintética",
    suggestedPriceCents: "12345678901234567",
  });
  expect(await screen.findByText("Produto cadastrado.")).toBeTruthy();
});

test("aceita o maior valor i64 em centavos e rejeita overflow", async () => {
  const user = userEvent.setup();
  vi.mocked(productApi.createProduct).mockResolvedValue(undefined);
  render(<ProductForm />);
  await fillForm(user, { price: "92233720368547758,07" });

  await user.click(screen.getByRole("button", { name: "Salvar produto" }));

  expect(productApi.createProduct).toHaveBeenCalledWith({
    name: "Modelo sintético",
    description: "Descrição sintética",
    suggestedPriceCents: "9223372036854775807",
  });
  expect(await screen.findByText("Produto cadastrado.")).toBeTruthy();

  cleanup();
  vi.resetAllMocks();
  render(<ProductForm />);
  await fillForm(user, { price: "92233720368547758,08" });
  await user.click(screen.getByRole("button", { name: "Salvar produto" }));

  expect(await screen.findByRole("alert")).toBeTruthy();
  expect(productApi.createProduct).not.toHaveBeenCalled();
});

test("preço inválido ou fora de i64 foca o campo e não chama o comando", async () => {
  const user = userEvent.setup();
  render(<ProductForm />);
  await fillForm(user, { price: "1.234" });

  await user.click(screen.getByRole("button", { name: "Salvar produto" }));

  const error = await screen.findByRole("alert");
  const price = screen.getByRole("textbox", { name: "Preço sugerido (R$)" });
  expect(error.textContent).toContain("Use vírgula decimal");
  expect(price.getAttribute("aria-invalid")).toBe("true");
  expect(document.activeElement).toBe(price);
  expect(productApi.createProduct).not.toHaveBeenCalled();
});

test("erro de nome Unicode do servidor associa mensagem e mantém o texto", async () => {
  const user = userEvent.setup();
  vi.mocked(productApi.createProduct).mockRejectedValue("PRODUCT_NAME_INVALID");
  render(<ProductForm />);
  await fillForm(user, { name: "\u2003\u2003", description: "", price: "0" });

  await user.click(screen.getByRole("button", { name: "Salvar produto" }));

  const error = await screen.findByRole("alert");
  const name = screen.getByRole("textbox", {
    name: "Nome do produto",
  }) as HTMLInputElement;
  expect(error.textContent).toContain("Digite um nome para o produto.");
  expect(name.getAttribute("aria-invalid")).toBe("true");
  expect(name.value).toBe("\u2003\u2003");
  await waitFor(() => expect(document.activeElement).toBe(name));
});

test("erro estático de preço do servidor associa mensagem e permite corrigir", async () => {
  const user = userEvent.setup();
  vi.mocked(productApi.createProduct).mockRejectedValue(
    "PRODUCT_PRICE_INVALID",
  );
  render(<ProductForm />);
  await fillForm(user);

  await user.click(screen.getByRole("button", { name: "Salvar produto" }));

  const error = await screen.findByRole("alert");
  const price = screen.getByRole("textbox", {
    name: "Preço sugerido (R$)",
  }) as HTMLInputElement;
  expect(error.textContent).toContain("Use vírgula decimal");
  expect(price.getAttribute("aria-invalid")).toBe("true");
  await waitFor(() => expect(document.activeElement).toBe(price));
  expect(price.value).toBe("12,34");
});

test("falha de armazenamento é genérica, focável e preserva campos", async () => {
  const user = userEvent.setup();
  vi.mocked(productApi.createProduct).mockRejectedValue("SQL e path secreto");
  render(<ProductForm />);
  await fillForm(user);

  await user.click(screen.getByRole("button", { name: "Salvar produto" }));

  const error = await screen.findByRole("alert");
  expect(error.textContent).toContain(
    "Não foi possível salvar o produto. Tente novamente.",
  );
  expect(error.textContent).not.toContain("SQL e path secreto");
  expect(
    (
      screen.getByRole("textbox", {
        name: "Nome do produto",
      }) as HTMLInputElement
    ).value,
  ).toBe("Modelo sintético");
  expect(
    (
      screen.getByRole("textbox", {
        name: "Descrição do produto",
      }) as HTMLTextAreaElement
    ).value,
  ).toBe("Descrição sintética");
  expect(
    (
      screen.getByRole("textbox", {
        name: "Preço sugerido (R$)",
      }) as HTMLInputElement
    ).value,
  ).toBe("12,34");
  await waitFor(() => expect(document.activeElement).toBe(error));
});

test("anuncia salvamento e ignora submissão duplicada", async () => {
  const user = userEvent.setup();
  let resolveCreate!: () => void;
  vi.mocked(productApi.createProduct).mockReturnValue(
    new Promise((resolve) => {
      resolveCreate = resolve;
    }),
  );
  render(<ProductForm />);
  await fillForm(user);
  const form = screen
    .getByRole("button", { name: "Salvar produto" })
    .closest("form");
  expect(form).not.toBeNull();

  await user.click(screen.getByRole("button", { name: "Salvar produto" }));
  expect(screen.getByRole("status").textContent).toContain("Salvando produto");
  fireEvent.submit(form as HTMLFormElement);
  expect(productApi.createProduct).toHaveBeenCalledTimes(1);

  resolveCreate();
  expect(await screen.findByText("Produto cadastrado.")).toBeTruthy();
});
