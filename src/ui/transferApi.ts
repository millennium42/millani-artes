import { invoke } from "@tauri-apps/api/core";

export type TransferContext = "home" | "factory";
export type CreateTransferInput = {
  sourceAccountId: number[];
  destinationAccountId: number[];
  amountCents: string;
  context: TransferContext;
};

export function createTransfer(input: CreateTransferInput): Promise<void> {
  return invoke("create_transfer", {
    sourceAccountId: input.sourceAccountId,
    destinationAccountId: input.destinationAccountId,
    amountCents: input.amountCents,
    context: input.context,
  });
}
