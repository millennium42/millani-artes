import { invoke } from "@tauri-apps/api/core";

export type AccountTypeKey = "cash" | "bank_pix" | "card" | "other";

export type CreateAccountInput = {
  name: string;
  accountType: AccountTypeKey;
  initialBalanceCents: string;
};

export function createAccount(input: CreateAccountInput): Promise<void> {
  return invoke<void>("create_account", {
    name: input.name,
    accountType: input.accountType,
    initialBalanceCents: input.initialBalanceCents,
  });
}
