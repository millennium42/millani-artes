// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import TransferForm from "./TransferForm";
import * as entryApi from "./entryApi";
import * as transferApi from "./transferApi";
import type { EntryAccount } from "./entryApi";

vi.mock("./entryApi", () => ({ listAccounts: vi.fn() }));
vi.mock("./transferApi", () => ({ createTransfer: vi.fn() }));

const sourceAccount: EntryAccount = {
  id: Array.from({ length: 16 }, (_, index) => index + 1),
  name: "Conta origem sintética",
};
const destinationAccount: EntryAccount = {
  id: Array.from({ length: 16 }, (_, index) => index + 30),
  name: "Conta destino sintética",
};
const accounts = [sourceAccount, destinationAccount];

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(entryApi.listAccounts).mockResolvedValue(accounts);
  vi.mocked(transferApi.createTransfer).mockResolvedValue();
});

afterEach(() => cleanup());

async function fillForm(
  user: ReturnType<typeof userEvent.setup>,
  context: "home" | "factory" = "factory",
) {
  const source = await screen.findByRole("combobox", {
    name: "Conta de origem",
  });
  const destination = screen.getByRole("combobox", {
    name: "Conta de destino",
  });
  const amount = screen.getByRole("textbox", { name: "Valor (R$)" });
  const contextSelect = screen.getByRole("combobox", { name: "Contexto" });
  await user.selectOptions(source, sourceAccount.id.join(","));
  await user.selectOptions(destination, destinationAccount.id.join(","));
  await user.type(amount, "12,34");
  await user.selectOptions(contextSelect, context);
  return {
    source: source as HTMLSelectElement,
    destination: destination as HTMLSelectElement,
    amount: amount as HTMLInputElement,
    contextSelect: contextSelect as HTMLSelectElement,
  };
}

describe("TransferForm", () => {
  test("anuncia carregamento inicial e depois oferece os campos", async () => {
    let resolveAccounts!: (value: EntryAccount[]) => void;
    vi.mocked(entryApi.listAccounts).mockReturnValueOnce(
      new Promise((resolve) => {
        resolveAccounts = resolve;
      }),
    );
    render(<TransferForm />);

    expect(screen.getByRole("status").textContent).toContain(
      "Carregando contas",
    );
    resolveAccounts(accounts);
    expect(
      await screen.findByRole("combobox", { name: "Conta de origem" }),
    ).toBeTruthy();
  });

  test("não revela erro no carregamento e permite tentar novamente", async () => {
    const user = userEvent.setup();
    vi.mocked(entryApi.listAccounts).mockRejectedValueOnce(
      "SENSITIVE_DATABASE_PATH",
    );
    render(<TransferForm />);

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain(
      "Não foi possível carregar as contas. Tente novamente.",
    );
    expect(alert.textContent).not.toContain("SENSITIVE_DATABASE_PATH");
    await user.click(screen.getByRole("button", { name: "Tentar novamente" }));
    expect(
      await screen.findByRole("combobox", { name: "Conta de origem" }),
    ).toBeTruthy();
  });

  test("mostra orientação quando há menos de duas contas", async () => {
    vi.mocked(entryApi.listAccounts).mockResolvedValueOnce([sourceAccount]);
    render(<TransferForm />);

    expect(
      await screen.findByText("Cadastre ao menos duas contas para transferir."),
    ).toBeTruthy();
    expect(
      screen.queryByRole("button", { name: "Registrar transferência" }),
    ).toBeNull();
  });

  test("recupera erro ao atualizar lista com menos de duas contas", async () => {
    const user = userEvent.setup();
    vi.mocked(entryApi.listAccounts)
      .mockResolvedValueOnce([sourceAccount])
      .mockRejectedValueOnce("SENSITIVE_DATABASE_PATH")
      .mockResolvedValueOnce(accounts);
    const view = render(<TransferForm accountRefreshKey={0} />);

    expect(
      await screen.findByText("Cadastre ao menos duas contas para transferir."),
    ).toBeTruthy();
    view.rerender(<TransferForm accountRefreshKey={1} />);
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain(
      "Não foi possível atualizar as contas. Tente novamente.",
    );
    expect(alert.textContent).not.toContain("SENSITIVE_DATABASE_PATH");
    await user.click(screen.getByRole("button", { name: "Tentar novamente" }));
    expect(
      await screen.findByRole("combobox", { name: "Conta de origem" }),
    ).toBeTruthy();
  });

  test("envia IDs, centavos exatos e contexto e devolve foco após sucesso", async () => {
    const user = userEvent.setup();
    render(<TransferForm />);

    const fields = await fillForm(user);
    await user.click(
      screen.getByRole("button", { name: "Registrar transferência" }),
    );

    await waitFor(() =>
      expect(transferApi.createTransfer).toHaveBeenCalledWith({
        sourceAccountId: sourceAccount.id,
        destinationAccountId: destinationAccount.id,
        amountCents: "1234",
        context: "factory",
      }),
    );
    expect((await screen.findByRole("status")).textContent).toContain(
      "Transferência registrada.",
    );
    await waitFor(() => expect(document.activeElement).toBe(fields.source));
    expect(fields.source.value).toBe("");
    expect(fields.destination.value).toBe("");
    expect(fields.amount.value).toBe("");
    expect(fields.contextSelect.value).toBe("");
  });

  test("valida sintaxe local do valor, associa a mensagem e move foco", async () => {
    const user = userEvent.setup();
    render(<TransferForm />);
    const fields = await fillForm(user);

    await user.clear(fields.amount);
    await user.type(fields.amount, "12.34");
    await user.click(
      screen.getByRole("button", { name: "Registrar transferência" }),
    );

    expect(screen.getByRole("alert").textContent).toContain(
      "Use vírgula decimal",
    );
    expect(fields.amount.getAttribute("aria-invalid")).toBe("true");
    expect(fields.amount.getAttribute("aria-describedby")).toContain(
      "transfer-amount-error",
    );
    expect(document.activeElement).toBe(fields.amount);
    expect(transferApi.createTransfer).not.toHaveBeenCalled();
  });

  test("mantém campos e não revela erro interno quando salvar falha", async () => {
    const user = userEvent.setup();
    vi.mocked(transferApi.createTransfer).mockRejectedValueOnce(
      "SENSITIVE_SQL_AND_ACCOUNT",
    );
    render(<TransferForm />);
    const fields = await fillForm(user, "home");

    await user.click(
      screen.getByRole("button", { name: "Registrar transferência" }),
    );
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain(
      "Não foi possível registrar a transferência. Tente novamente.",
    );
    expect(alert.textContent).not.toContain("SENSITIVE_SQL_AND_ACCOUNT");
    await waitFor(() => expect(document.activeElement).toBe(alert));
    expect(fields.source.value).toBe(sourceAccount.id.join(","));
    expect(fields.destination.value).toBe(destinationAccount.id.join(","));
    expect(fields.amount.value).toBe("12,34");
    expect(fields.contextSelect.value).toBe("home");
  });

  test("mantém valores e foco enquanto atualiza a lista em segundo plano", async () => {
    const user = userEvent.setup();
    let resolveRefresh!: (value: EntryAccount[]) => void;
    const extraAccount: EntryAccount = {
      id: Array.from({ length: 16 }, (_, index) => index + 80),
      name: "Conta extra sintética",
    };
    vi.mocked(entryApi.listAccounts)
      .mockResolvedValueOnce(accounts)
      .mockReturnValueOnce(
        new Promise((resolve) => {
          resolveRefresh = resolve;
        }),
      );
    const view = render(<TransferForm accountRefreshKey={0} />);
    const fields = await fillForm(user, "home");
    fields.contextSelect.focus();

    view.rerender(<TransferForm accountRefreshKey={1} />);
    expect(await screen.findByText("Atualizando contas…")).toBeTruthy();
    expect(fields.amount.value).toBe("12,34");
    expect(fields.contextSelect.value).toBe("home");
    expect(document.activeElement).toBe(fields.contextSelect);

    resolveRefresh([...accounts, extraAccount]);
    expect(
      await screen.findAllByRole("option", { name: "Conta extra sintética" }),
    ).toBeTruthy();
    expect(fields.amount.value).toBe("12,34");
    expect(document.activeElement).toBe(fields.contextSelect);
  });

  test("recupera falha de atualização sem desmontar nem apagar o formulário", async () => {
    const user = userEvent.setup();
    vi.mocked(entryApi.listAccounts)
      .mockResolvedValueOnce(accounts)
      .mockRejectedValueOnce("SENSITIVE_DATABASE_PATH")
      .mockResolvedValueOnce(accounts);
    const view = render(<TransferForm accountRefreshKey={0} />);
    const fields = await fillForm(user, "home");
    fields.contextSelect.focus();

    view.rerender(<TransferForm accountRefreshKey={1} />);
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain(
      "Não foi possível atualizar as contas. Tente novamente.",
    );
    expect(alert.textContent).not.toContain("SENSITIVE_DATABASE_PATH");
    expect(fields.amount.value).toBe("12,34");
    expect(fields.contextSelect.value).toBe("home");
    expect(document.activeElement).toBe(fields.contextSelect);

    await user.click(screen.getByRole("button", { name: "Tentar novamente" }));
    expect(
      await screen.findByRole("combobox", { name: "Conta de origem" }),
    ).toBeTruthy();
    expect(fields.amount.value).toBe("12,34");
    expect(fields.contextSelect.value).toBe("home");
  });

  test("a ordem de teclado segue origem, destino, valor, contexto e ação", async () => {
    const user = userEvent.setup();
    render(<TransferForm />);
    const fields = await fillForm(user);
    fields.source.focus();

    await user.tab();
    expect(document.activeElement).toBe(fields.destination);
    await user.tab();
    expect(document.activeElement).toBe(fields.amount);
    await user.tab();
    expect(document.activeElement).toBe(fields.contextSelect);
    await user.tab();
    expect(document.activeElement).toBe(
      screen.getByRole("button", { name: "Registrar transferência" }),
    );
  });

  test("ignora um segundo envio enquanto a primeira chamada está pendente", async () => {
    const user = userEvent.setup();
    let resolveSave!: () => void;
    vi.mocked(transferApi.createTransfer).mockReturnValueOnce(
      new Promise((resolve) => {
        resolveSave = () => resolve();
      }),
    );
    render(<TransferForm />);
    await fillForm(user);

    const form = (
      screen.getByRole("button", {
        name: "Registrar transferência",
      }) as HTMLButtonElement
    ).form;
    if (form === null) throw new Error("UI007_FORM_MISSING");
    fireEvent.submit(form);
    fireEvent.submit(form);
    expect(transferApi.createTransfer).toHaveBeenCalledOnce();
    expect(await screen.findByText("Registrando transferência…")).toBeTruthy();
    resolveSave();
    expect(await screen.findByText("Transferência registrada.")).toBeTruthy();
  });
});
