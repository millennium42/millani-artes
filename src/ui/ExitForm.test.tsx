// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, test, vi } from "vitest";
import ExitForm from "./ExitForm";
import * as entryApi from "./entryApi";
import * as exitApi from "./exitApi";
import type { EntryAccount } from "./entryApi";

vi.mock("./entryApi", () => ({
  listAccounts: vi.fn(),
}));

vi.mock("./exitApi", () => ({
  createExit: vi.fn(),
}));

const accountId = Array.from({ length: 16 }, (_, index) => index + 1);
const account: EntryAccount = { id: accountId, name: "Conta sintética" };

afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

test("mostra carregamento e a lista vazia orienta o cadastro de conta", async () => {
  let resolveAccounts!: (value: EntryAccount[]) => void;
  vi.mocked(entryApi.listAccounts).mockReturnValue(
    new Promise((resolve) => {
      resolveAccounts = resolve;
    }),
  );
  render(<ExitForm />);

  expect(screen.getByRole("status").textContent).toContain("Carregando contas");
  resolveAccounts([]);
  expect(
    await screen.findByText("Cadastre uma conta para registrar uma saída."),
  ).toBeTruthy();
  expect(
    (
      screen.getByRole("button", {
        name: "Registrar saída",
      }) as HTMLButtonElement
    ).disabled,
  ).toBe(true);
});

test("falha ao carregar contas mostra alerta genérico e permite retry", async () => {
  const user = userEvent.setup();
  vi.mocked(entryApi.listAccounts)
    .mockRejectedValueOnce("SQL secreto")
    .mockResolvedValueOnce([account]);
  render(<ExitForm />);

  expect((await screen.findByRole("alert")).textContent).toContain(
    "Não foi possível carregar as contas. Tente novamente.",
  );
  expect(screen.queryByText("SQL secreto")).toBeNull();
  await user.click(screen.getByRole("button", { name: "Tentar novamente" }));
  expect(await screen.findByRole("combobox", { name: "Conta" })).toBeTruthy();
  expect(entryApi.listAccounts).toHaveBeenCalledTimes(2);
});

test("labels e teclado seguem conta, valor, contexto e envio", async () => {
  const user = userEvent.setup();
  vi.mocked(entryApi.listAccounts).mockResolvedValue([account]);
  render(<ExitForm />);

  const accountSelect = (await screen.findByRole("combobox", {
    name: "Conta",
  })) as HTMLSelectElement;
  const amount = screen.getByRole("textbox", {
    name: "Valor (R$)",
  }) as HTMLInputElement;
  const context = screen.getByRole("combobox", {
    name: "Contexto",
  }) as HTMLSelectElement;
  const submit = screen.getByRole("button", { name: "Registrar saída" });

  accountSelect.focus();
  expect(document.activeElement).toBe(accountSelect);
  await user.tab();
  expect(document.activeElement).toBe(amount);
  await user.tab();
  expect(document.activeElement).toBe(context);
  await user.tab();
  expect(document.activeElement).toBe(submit);
});

test("formato inválido foca o valor e não chama o backend", async () => {
  const user = userEvent.setup();
  vi.mocked(entryApi.listAccounts).mockResolvedValue([account]);
  render(<ExitForm />);
  await user.selectOptions(
    await screen.findByRole("combobox", { name: "Conta" }),
    accountId.join(","),
  );
  const amount = screen.getByRole("textbox", { name: "Valor (R$)" });
  await user.type(amount, "1.234");
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Contexto" }),
    "home",
  );
  await user.click(screen.getByRole("button", { name: "Registrar saída" }));

  expect((await screen.findByRole("alert")).textContent).toContain(
    "Use vírgula decimal",
  );
  expect(document.activeElement).toBe(amount);
  expect(exitApi.createExit).not.toHaveBeenCalled();
});

test("zero chega ao caso de uso; erro do domínio é opaco e conserva o formulário", async () => {
  const user = userEvent.setup();
  vi.mocked(entryApi.listAccounts).mockResolvedValue([account]);
  vi.mocked(exitApi.createExit).mockRejectedValueOnce("EXIT_INVALID");
  render(<ExitForm />);
  const accountSelect = (await screen.findByRole("combobox", {
    name: "Conta",
  })) as HTMLSelectElement;
  await user.selectOptions(accountSelect, accountId.join(","));
  const amount = screen.getByRole("textbox", {
    name: "Valor (R$)",
  }) as HTMLInputElement;
  await user.type(amount, "0");
  const context = screen.getByRole("combobox", {
    name: "Contexto",
  }) as HTMLSelectElement;
  await user.selectOptions(context, "home");
  await user.click(screen.getByRole("button", { name: "Registrar saída" }));

  expect(exitApi.createExit).toHaveBeenCalledWith({
    accountId,
    amountCents: "0",
    context: "home",
  });
  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toContain(
    "Não foi possível registrar a saída. Tente novamente.",
  );
  expect(alert.textContent).not.toContain("EXIT_INVALID");
  expect(amount.value).toBe("0");
  expect(context.value).toBe("home");
  await waitFor(() => expect(document.activeElement).toBe(alert));
});

test("erro de envio conserva os campos e foca somente o alerta genérico", async () => {
  const user = userEvent.setup();
  vi.mocked(entryApi.listAccounts).mockResolvedValue([account]);
  vi.mocked(exitApi.createExit).mockRejectedValueOnce("SQL secreto");
  render(<ExitForm />);
  const accountSelect = (await screen.findByRole("combobox", {
    name: "Conta",
  })) as HTMLSelectElement;
  await user.selectOptions(accountSelect, accountId.join(","));
  const amount = screen.getByRole("textbox", {
    name: "Valor (R$)",
  }) as HTMLInputElement;
  await user.type(amount, "12,34");
  const context = screen.getByRole("combobox", {
    name: "Contexto",
  }) as HTMLSelectElement;
  await user.selectOptions(context, "factory");
  await user.click(screen.getByRole("button", { name: "Registrar saída" }));

  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toContain(
    "Não foi possível registrar a saída. Tente novamente.",
  );
  expect(alert.textContent).not.toContain("SQL secreto");
  expect(accountSelect.value).toBe(accountId.join(","));
  expect(amount.value).toBe("12,34");
  expect(context.value).toBe("factory");
  await waitFor(() => expect(document.activeElement).toBe(alert));
});

test("sucesso confirmado limpa o formulário e devolve foco à conta", async () => {
  const user = userEvent.setup();
  let resolveExit!: () => void;
  vi.mocked(entryApi.listAccounts).mockResolvedValue([account]);
  vi.mocked(exitApi.createExit).mockReturnValue(
    new Promise((resolve) => {
      resolveExit = resolve;
    }),
  );
  render(<ExitForm />);
  const accountSelect = (await screen.findByRole("combobox", {
    name: "Conta",
  })) as HTMLSelectElement;
  await user.selectOptions(accountSelect, accountId.join(","));
  const amount = screen.getByRole("textbox", {
    name: "Valor (R$)",
  }) as HTMLInputElement;
  await user.type(amount, "12,34");
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Contexto" }),
    "factory",
  );
  const submit = screen.getByRole("button", { name: "Registrar saída" });
  await user.click(submit);

  expect(submit.matches(":disabled")).toBe(true);
  expect(exitApi.createExit).toHaveBeenCalledWith({
    accountId,
    amountCents: "1234",
    context: "factory",
  });
  resolveExit();
  expect(await screen.findByText("Saída registrada.")).toBeTruthy();
  expect(accountSelect.value).toBe("");
  expect(amount.value).toBe("");
  expect(
    (screen.getByRole("combobox", { name: "Contexto" }) as HTMLSelectElement)
      .value,
  ).toBe("");
  await waitFor(() => expect(document.activeElement).toBe(accountSelect));
});

test("refresh em segundo plano preserva campos e foco", async () => {
  const user = userEvent.setup();
  const secondAccount: EntryAccount = {
    id: Array.from({ length: 16 }, (_, index) => index + 30),
    name: "Outra conta sintética",
  };
  let resolveRefresh!: (value: EntryAccount[]) => void;
  vi.mocked(entryApi.listAccounts)
    .mockResolvedValueOnce([account])
    .mockReturnValueOnce(
      new Promise((resolve) => {
        resolveRefresh = resolve;
      }),
    );
  const view = render(<ExitForm accountRefreshKey={0} />);
  const accountSelect = (await screen.findByRole("combobox", {
    name: "Conta",
  })) as HTMLSelectElement;
  await user.selectOptions(accountSelect, accountId.join(","));
  const amount = screen.getByRole("textbox", {
    name: "Valor (R$)",
  }) as HTMLInputElement;
  await user.type(amount, "7,50");
  const context = screen.getByRole("combobox", {
    name: "Contexto",
  }) as HTMLSelectElement;
  await user.selectOptions(context, "home");
  context.focus();

  view.rerender(<ExitForm accountRefreshKey={1} />);
  expect(await screen.findByRole("status")).toBeTruthy();
  expect(amount.value).toBe("7,50");
  expect(context.value).toBe("home");
  expect(document.activeElement).toBe(context);
  resolveRefresh([account, secondAccount]);
  expect(
    await screen.findByRole("option", { name: "Outra conta sintética" }),
  ).toBeTruthy();
  expect(amount.value).toBe("7,50");
  expect(document.activeElement).toBe(context);
});
