import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, test, vi } from "vitest";
import { createExit } from "./exitApi";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

afterEach(() => {
  vi.resetAllMocks();
});

test("envia somente ID, centavos textuais e contexto para create_exit", async () => {
  vi.mocked(invoke).mockResolvedValue(undefined);
  const accountId = Array.from({ length: 16 }, (_, index) => 255 - index);

  await createExit({
    accountId,
    amountCents: "9223372036854775807",
    context: "factory",
  });

  expect(invoke).toHaveBeenCalledWith("create_exit", {
    accountId,
    amountCents: "9223372036854775807",
    context: "factory",
  });
  expect(invoke).toHaveBeenCalledTimes(1);
});
