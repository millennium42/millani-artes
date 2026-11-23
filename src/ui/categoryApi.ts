import { invoke } from "@tauri-apps/api/core";

export function createCategory(name: string): Promise<void> {
  return invoke<void>("create_category", { name });
}
