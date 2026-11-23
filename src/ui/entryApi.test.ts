import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, test, vi } from "vitest";
import { createEntry, listAccounts } from "./entryApi";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

afterEach(() => {
  vi.resetAllMocks();
});

test("lista somente pares de ID e nome pelo comando estreito", async () => {
  const id = Array.from({ length: 16 }, (_, index) => index);
  vi.mocked(invoke).mockResolvedValue([[id, "Conta sintética"]]);

  await expect(listAccounts()).resolves.toEqual([
    { id, name: "Conta sintética" },
  ]);
  expect(invoke).toHaveBeenCalledWith("list_accounts");
  expect(invoke).toHaveBeenCalledTimes(1);
});

test("envia somente ID, centavos textuais e contexto", async () => {
  vi.mocked(invoke).mockResolvedValue(undefined);
  const accountId = Array.from({ length: 16 }, (_, index) => 255 - index);

  await createEntry({
    accountId,
    amountCents: "9223372036854775807",
    context: "factory",
  });

  expect(invoke).toHaveBeenCalledWith("create_entry", {
    accountId,
    amountCents: "9223372036854775807",
    context: "factory",
  });
  expect(invoke).toHaveBeenCalledTimes(1);
});
