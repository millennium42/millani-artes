import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, test, vi } from "vitest";
import { createProduct } from "./productApi";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

afterEach(() => {
  vi.resetAllMocks();
});

test("envia somente nome, descrição e centavos textuais ao comando local", async () => {
  vi.mocked(invoke).mockResolvedValue(undefined);

  await createProduct({
    name: "  Modelo Árvore 🪴  ",
    description: "  Descrição preservada  ",
    suggestedPriceCents: "1234",
  });

  expect(invoke).toHaveBeenCalledWith("create_product", {
    name: "  Modelo Árvore 🪴  ",
    description: "  Descrição preservada  ",
    suggestedPriceCents: "1234",
  });
});

test("propaga o erro estático do limite Tauri para tratamento acessível", async () => {
  vi.mocked(invoke).mockRejectedValue("PRODUCT_NAME_INVALID");

  await expect(
    createProduct({
      name: "   ",
      description: "",
      suggestedPriceCents: "0",
    }),
  ).rejects.toBe("PRODUCT_NAME_INVALID");
});
