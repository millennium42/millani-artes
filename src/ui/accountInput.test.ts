import { describe, expect, test } from "vitest";
import { parseInitialBalanceCents } from "./accountInput";

describe("parseInitialBalanceCents", () => {
  test.each([
    ["0", "0"],
    ["12", "1200"],
    ["12,3", "1230"],
    ["12,30", "1230"],
    ["001,05", "105"],
    ["-0,01", "-1"],
    ["-92233720368547758,08", "-9223372036854775808"],
    ["92233720368547758,07", "9223372036854775807"],
  ])("converte %s exatamente para %s centavos", (text, cents) => {
    expect(parseInitialBalanceCents(text)).toBe(cents);
  });

  test.each([
    "",
    " ",
    ",50",
    "1,",
    "1,234",
    "1.234,56",
    "+1,00",
    "1e2",
    "123456789012345678901",
    "92233720368547758,08",
    "-92233720368547758,09",
  ])("rejeita entrada sem arredondar: %s", (text) => {
    expect(parseInitialBalanceCents(text)).toBeNull();
  });
});
