import { invoke } from "@tauri-apps/api/core";
import type { CreateEntryInput } from "./entryApi";

export type CreateExitInput = CreateEntryInput;

export function createExit(input: CreateExitInput): Promise<void> {
  return invoke("create_exit", {
    accountId: input.accountId,
    amountCents: input.amountCents,
    context: input.context,
  });
}
