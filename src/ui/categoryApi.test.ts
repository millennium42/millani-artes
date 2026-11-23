import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, test, vi } from "vitest";
import { createCategory } from "./categoryApi";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

afterEach(() => {
  vi.resetAllMocks();
});

test("invoca somente create_category com o nome literal", async () => {
  vi.mocked(invoke).mockResolvedValue(undefined);
  const literalName = "  Categoria Açúcar 🧵  ";

  await createCategory(literalName);

  expect(invoke).toHaveBeenCalledWith("create_category", {
    name: literalName,
  });
  expect(invoke).toHaveBeenCalledTimes(1);
});
