import { invoke } from "@tauri-apps/api/core";

export type CreateProductInput = {
  name: string;
  description: string;
  suggestedPriceCents: string;
};

export function createProduct(input: CreateProductInput): Promise<void> {
  return invoke("create_product", {
    name: input.name,
    description: input.description,
    suggestedPriceCents: input.suggestedPriceCents,
  });
}
