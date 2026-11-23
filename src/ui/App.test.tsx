// @vitest-environment jsdom

import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, expect, test, vi } from "vitest";
import App from "./App";
import * as accountApi from "./accountApi";

vi.mock("./accountApi", () => ({
  createAccount: vi.fn(),
}));

afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

test("apresenta a marca e a configuração de contas", () => {
  const markup = renderToStaticMarkup(<App />);

  expect(markup).toContain("<header>Millani Artes</header>");
  expect(markup).toContain("<h1>Configure suas contas</h1>");
  expect(markup).not.toContain("Aplicativo local em construção.");
});

test("expõe campos rotulados e percorre o formulário por teclado", async () => {
  const user = userEvent.setup();
  render(<App />);

  const name = screen.getByRole("textbox", { name: "Nome da conta" });
  const type = screen.getByRole("combobox", { name: "Tipo de conta" });
  const balance = screen.getByRole("textbox", { name: "Saldo inicial (R$)" });
  const submit = screen.getByRole("button", { name: "Salvar conta" });

  expect(type.querySelectorAll("option")).toHaveLength(5);
  await user.tab();
  expect(document.activeElement).toBe(name);
  await user.tab();
  expect(document.activeElement).toBe(type);
  await user.tab();
  expect(document.activeElement).toBe(balance);
  await user.tab();
  expect(document.activeElement).toBe(submit);
});

test("envia o nome literal e os centavos exatos; anuncia e reinicia após sucesso", async () => {
  const user = userEvent.setup();
  vi.mocked(accountApi.createAccount).mockResolvedValue(undefined);
  render(<App />);

  await user.type(
    screen.getByRole("textbox", { name: "Nome da conta" }),
    "  Banco Árvore 🪴  ",
  );
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Tipo de conta" }),
    "bank_pix",
  );
  await user.type(
    screen.getByRole("textbox", { name: "Saldo inicial (R$)" }),
    "123456789012345,67",
  );
  await user.click(screen.getByRole("button", { name: "Salvar conta" }));

  expect(accountApi.createAccount).toHaveBeenCalledWith({
    name: "  Banco Árvore 🪴  ",
    accountType: "bank_pix",
    initialBalanceCents: "12345678901234567",
  });
  expect(
    (
      await screen.findByText("Conta criada. Você pode cadastrar outra.")
    ).closest("[role='status']"),
  ).not.toBeNull();
  expect(
    (screen.getByRole("textbox", { name: "Nome da conta" }) as HTMLInputElement)
      .value,
  ).toBe("");
});

test("anuncia o envio e impede uma segunda submissão enquanto salva", async () => {
  const user = userEvent.setup();
  let finish: (() => void) | undefined;
  const pending = new Promise<void>((resolve) => {
    finish = resolve;
  });
  vi.mocked(accountApi.createAccount).mockReturnValue(pending);
  render(<App />);

  await user.type(
    screen.getByRole("textbox", { name: "Nome da conta" }),
    "Conta",
  );
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Tipo de conta" }),
    "cash",
  );
  await user.type(
    screen.getByRole("textbox", { name: "Saldo inicial (R$)" }),
    "1,00",
  );
  await user.click(screen.getByRole("button", { name: "Salvar conta" }));

  expect(screen.getByRole("status").textContent).toContain("Salvando…");
  expect(
    screen.getByRole("button", { name: "Salvando…" }).matches(":disabled"),
  ).toBe(true);
  expect(
    screen.getByRole("textbox", { name: "Nome da conta" }).matches(":disabled"),
  ).toBe(true);
  await user.click(screen.getByRole("button", { name: "Salvando…" }));
  expect(accountApi.createAccount).toHaveBeenCalledTimes(1);
  finish?.();
  expect(
    await screen.findByText("Conta criada. Você pode cadastrar outra."),
  ).toBeTruthy();
});

test("recusa sintaxe monetária inválida sem chamar o serviço", async () => {
  const user = userEvent.setup();
  render(<App />);

  await user.type(
    screen.getByRole("textbox", { name: "Nome da conta" }),
    "Conta teste",
  );
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Tipo de conta" }),
    "cash",
  );
  await user.type(
    screen.getByRole("textbox", { name: "Saldo inicial (R$)" }),
    "1,234",
  );
  await user.click(screen.getByRole("button", { name: "Salvar conta" }));

  expect((await screen.findByRole("alert")).textContent).toContain(
    "Use vírgula decimal, no máximo duas casas e sem separador de milhares.",
  );
  expect(document.activeElement).toBe(
    screen.getByRole("textbox", { name: "Saldo inicial (R$)" }),
  );
  expect(accountApi.createAccount).not.toHaveBeenCalled();
});

test("mostra erro de validação do nome e falha de persistência sem expor detalhes", async () => {
  const user = userEvent.setup();
  vi.mocked(accountApi.createAccount).mockRejectedValueOnce(
    "ACCOUNT_NAME_INVALID",
  );
  render(<App />);

  await user.type(
    screen.getByRole("textbox", { name: "Nome da conta" }),
    "   ",
  );
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Tipo de conta" }),
    "other",
  );
  await user.type(
    screen.getByRole("textbox", { name: "Saldo inicial (R$)" }),
    "0,00",
  );
  await user.click(screen.getByRole("button", { name: "Salvar conta" }));
  expect((await screen.findByRole("alert")).textContent).toContain(
    "Digite um nome para a conta.",
  );
  expect(document.activeElement).toBe(
    screen.getByRole("textbox", { name: "Nome da conta" }),
  );

  vi.mocked(accountApi.createAccount).mockRejectedValueOnce(
    "path=C:\\secret\\db.sqlite SQL=INSERT",
  );
  await user.clear(screen.getByRole("textbox", { name: "Nome da conta" }));
  await user.type(
    screen.getByRole("textbox", { name: "Nome da conta" }),
    "Conta",
  );
  await user.click(screen.getByRole("button", { name: "Salvar conta" }));
  expect((await screen.findByRole("alert")).textContent).toContain(
    "Não foi possível salvar a conta. Tente novamente.",
  );
  expect(screen.getByRole("alert").textContent).not.toContain("secret");
});
