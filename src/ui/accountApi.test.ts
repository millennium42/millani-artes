import { beforeEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { createAccount } from "./accountApi";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

beforeEach(() => {
  vi.resetAllMocks();
});

test("invoca somente create_account com centavos decimais como string", async () => {
  vi.mocked(invoke).mockResolvedValue(undefined);

  await createAccount({
    name: "Caixa",
    accountType: "cash",
    initialBalanceCents: "-9223372036854775808",
  });

  expect(invoke).toHaveBeenCalledWith("create_account", {
    name: "Caixa",
    accountType: "cash",
    initialBalanceCents: "-9223372036854775808",
  });
});
