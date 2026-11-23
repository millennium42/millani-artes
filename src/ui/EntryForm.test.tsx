// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, test, vi } from "vitest";
import EntryForm from "./EntryForm";
import * as entryApi from "./entryApi";

vi.mock("./entryApi", () => ({
  listAccounts: vi.fn(),
  createEntry: vi.fn(),
}));

const accountId = Array.from({ length: 16 }, (_, index) => index + 1);
const account = { id: accountId, name: "Conta sintética" };

afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

test("mostra carregamento e a lista vazia orienta o cadastro de conta", async () => {
  let resolveAccounts!: (value: (typeof account)[]) => void;
  vi.mocked(entryApi.listAccounts).mockReturnValue(
    new Promise((resolve) => {
      resolveAccounts = resolve;
    }),
  );
  render(<EntryForm />);

  expect(screen.getByRole("status").textContent).toContain("Carregando contas");
  resolveAccounts([]);
  expect(
    await screen.findByText("Cadastre uma conta para registrar uma entrada."),
  ).toBeTruthy();
  expect(
    (
      screen.getByRole("button", {
        name: "Registrar entrada",
      }) as HTMLButtonElement
    ).disabled,
  ).toBe(true);
});

test("falha de leitura mostra alerta genérico e permite tentar novamente", async () => {
  const user = userEvent.setup();
  vi.mocked(entryApi.listAccounts)
    .mockRejectedValueOnce("SQL secreto")
    .mockResolvedValueOnce([account]);
  render(<EntryForm />);

  expect((await screen.findByRole("alert")).textContent).toContain(
    "Não foi possível carregar as contas. Tente novamente.",
  );
  expect(screen.queryByText("SQL secreto")).toBeNull();
  await user.click(screen.getByRole("button", { name: "Tentar novamente" }));
  expect(
    (
      (await screen.findByRole("combobox", {
        name: "Conta",
      })) as HTMLSelectElement
    ).disabled,
  ).toBe(false);
  expect(entryApi.listAccounts).toHaveBeenCalledTimes(2);
});

test("labels e teclado seguem conta, valor, contexto e envio", async () => {
  const user = userEvent.setup();
  vi.mocked(entryApi.listAccounts).mockResolvedValue([account]);
  render(<EntryForm />);

  const accountSelect = (await screen.findByRole("combobox", {
    name: "Conta",
  })) as HTMLSelectElement;
  const amount = screen.getByRole("textbox", {
    name: "Valor (R$)",
  }) as HTMLInputElement;
  const context = screen.getByRole("combobox", {
    name: "Contexto",
  }) as HTMLSelectElement;
  const submit = screen.getByRole("button", { name: "Registrar entrada" });

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
  render(<EntryForm />);
  const accountSelect = (await screen.findByRole("combobox", {
    name: "Conta",
  })) as HTMLSelectElement;
  await user.selectOptions(accountSelect, accountId.join(","));
  const amount = screen.getByRole("textbox", {
    name: "Valor (R$)",
  }) as HTMLInputElement;
  await user.type(amount, "1.234");
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Contexto" }),
    "home",
  );
  await user.click(screen.getByRole("button", { name: "Registrar entrada" }));

  expect((await screen.findByRole("alert")).textContent).toContain(
    "Use vírgula decimal",
  );
  expect(document.activeElement).toBe(amount);
  expect(entryApi.createEntry).not.toHaveBeenCalled();
});

test("erro backend conserva os campos e foca somente o alerta genérico", async () => {
  const user = userEvent.setup();
  vi.mocked(entryApi.listAccounts).mockResolvedValue([account]);
  vi.mocked(entryApi.createEntry).mockRejectedValueOnce("SQL secreto");
  render(<EntryForm />);
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
  await user.click(screen.getByRole("button", { name: "Registrar entrada" }));

  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toContain(
    "Não foi possível registrar a entrada. Tente novamente.",
  );
  expect(alert.textContent).not.toContain("SQL secreto");
  expect(accountSelect.value).toBe(accountId.join(","));
  expect(amount.value).toBe("12,34");
  expect(context.value).toBe("factory");
  await waitFor(() => expect(document.activeElement).toBe(alert));
});

test("sucesso confirmado limpa o formulário e devolve foco à conta", async () => {
  const user = userEvent.setup();
  let resolveEntry!: () => void;
  vi.mocked(entryApi.listAccounts).mockResolvedValue([account]);
  vi.mocked(entryApi.createEntry).mockReturnValue(
    new Promise((resolve) => {
      resolveEntry = resolve;
    }),
  );
  render(<EntryForm />);
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
    "home",
  );
  const submit = screen.getByRole("button", { name: "Registrar entrada" });
  await user.click(submit);

  expect(submit.matches(":disabled")).toBe(true);
  expect(entryApi.createEntry).toHaveBeenCalledWith({
    accountId,
    amountCents: "1234",
    context: "home",
  });
  resolveEntry();
  expect(await screen.findByText("Entrada registrada.")).toBeTruthy();
  await waitFor(() => expect(document.activeElement).toBe(accountSelect));
  expect(accountSelect.value).toBe("");
  expect(amount.value).toBe("");
});

test("atualiza opções quando uma conta nova é criada", async () => {
  vi.mocked(entryApi.listAccounts)
    .mockResolvedValueOnce([])
    .mockResolvedValueOnce([account]);
  const { rerender } = render(<EntryForm accountRefreshKey={0} />);
  expect(
    await screen.findByText("Cadastre uma conta para registrar uma entrada."),
  ).toBeTruthy();

  rerender(<EntryForm accountRefreshKey={1} />);
  expect(
    await screen.findByRole("option", { name: "Conta sintética" }),
  ).toBeTruthy();
  expect(entryApi.listAccounts).toHaveBeenCalledTimes(2);
});

test("falha de envio durante atualização das contas mantém o alerta focável", async () => {
  const user = userEvent.setup();
  let rejectEntry!: (error: unknown) => void;
  let resolveAccounts!: (value: (typeof account)[]) => void;
  vi.mocked(entryApi.listAccounts)
    .mockResolvedValueOnce([account])
    .mockReturnValueOnce(
      new Promise((resolve) => {
        resolveAccounts = resolve;
      }),
    );
  vi.mocked(entryApi.createEntry).mockReturnValue(
    new Promise((_, reject) => {
      rejectEntry = reject;
    }),
  );
  const { rerender } = render(<EntryForm accountRefreshKey={0} />);
  const accountSelect = (await screen.findByRole("combobox", {
    name: "Conta",
  })) as HTMLSelectElement;
  await user.selectOptions(accountSelect, accountId.join(","));
  await user.type(screen.getByRole("textbox", { name: "Valor (R$)" }), "12,34");
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Contexto" }),
    "factory",
  );
  await user.click(screen.getByRole("button", { name: "Registrar entrada" }));

  rerender(<EntryForm accountRefreshKey={1} />);
  rejectEntry(new Error("falha sintética"));
  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toContain("Não foi possível registrar a entrada.");
  await waitFor(() => expect(document.activeElement).toBe(alert));

  resolveAccounts([account]);
  expect(
    await screen.findByRole("option", { name: "Conta sintética" }),
  ).toBeTruthy();
});

test("falha de atualização em segundo plano mantém o formulário e permite tentar novamente", async () => {
  const user = userEvent.setup();
  vi.mocked(entryApi.listAccounts)
    .mockResolvedValueOnce([account])
    .mockRejectedValueOnce("SQL secreto")
    .mockResolvedValueOnce([account]);
  const { rerender } = render(<EntryForm accountRefreshKey={0} />);
  expect(await screen.findByRole("combobox", { name: "Conta" })).toBeTruthy();

  rerender(<EntryForm accountRefreshKey={1} />);
  expect((await screen.findByRole("alert")).textContent).toContain(
    "Não foi possível atualizar as contas. Tente novamente.",
  );
  expect(screen.getByRole("combobox", { name: "Conta" })).toBeTruthy();
  expect(screen.queryByText("SQL secreto")).toBeNull();
  await user.click(screen.getByRole("button", { name: "Tentar novamente" }));
  expect(await screen.findByRole("combobox", { name: "Conta" })).toBeTruthy();
  expect(entryApi.listAccounts).toHaveBeenCalledTimes(3);
});
