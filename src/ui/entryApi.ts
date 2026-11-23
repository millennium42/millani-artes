import { invoke } from "@tauri-apps/api/core";

export type EntryAccount = { id: number[]; name: string };
export type CreateEntryInput = {
  accountId: number[];
  amountCents: string;
  context: "home" | "factory";
};

export async function listAccounts(): Promise<EntryAccount[]> {
  const rows = await invoke<[number[], string][]>("list_accounts");
  return rows.map(([id, name]) => ({ id, name }));
}

export function createEntry(input: CreateEntryInput): Promise<void> {
  return invoke("create_entry", {
    accountId: input.accountId,
    amountCents: input.amountCents,
    context: input.context,
  });
}
