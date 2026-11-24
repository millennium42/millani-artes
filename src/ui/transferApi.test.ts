import { beforeEach, describe, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { createTransfer } from "./transferApi";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

beforeEach(() => vi.clearAllMocks());

describe("createTransfer", () => {
  test("invoca somente o comando com IDs, centavos textuais e contexto", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const input = {
      sourceAccountId: Array.from({ length: 16 }, (_, index) => index + 1),
      destinationAccountId: Array.from(
        { length: 16 },
        (_, index) => index + 30,
      ),
      amountCents: "1234",
      context: "factory" as const,
    };

    await createTransfer(input);

    expect(invoke).toHaveBeenCalledOnce();
    expect(invoke).toHaveBeenCalledWith("create_transfer", input);
  });
});
