import { describe, expect, test } from "vitest";
import { parseEntryAmountCents } from "./entryInput";

describe("parseEntryAmountCents", () => {
  test.each([
    ["0", "0"],
    ["12", "1200"],
    ["1,2", "120"],
    ["1,23", "123"],
    ["0001,05", "105"],
    ["92233720368547758,07", "9223372036854775807"],
  ])("converte %s exatamente para %s centavos", (text, cents) => {
    expect(parseEntryAmountCents(text)).toBe(cents);
  });

  test.each([
    "",
    "-1",
    "1.00",
    "1.234",
    "1,234",
    "1e2",
    "1\n",
    "1\r",
    "1\u2028",
    "1\u2029",
    "92233720368547758,08",
  ])("rejeita sintaxe ou overflow sem arredondar: %s", (text) => {
    expect(parseEntryAmountCents(text)).toBeNull();
  });
});
