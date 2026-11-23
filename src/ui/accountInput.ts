const I64_MIN = -9_223_372_036_854_775_808n;
const I64_MAX = 9_223_372_036_854_775_807n;
const DECIMAL_INPUT = /^-?[0-9]+(?:,[0-9]{1,2})?$/;

export function parseInitialBalanceCents(input: string): string | null {
  if (!DECIMAL_INPUT.test(input)) return null;

  const isNegative = input.startsWith("-");
  const unsigned = isNegative ? input.slice(1) : input;
  const [whole, fraction = ""] = unsigned.split(",");
  const centsDigits =
    `${whole}${fraction.padEnd(2, "0")}`.replace(/^0+/, "") || "0";
  if (centsDigits.length > 19) return null;

  const magnitude = BigInt(centsDigits);
  const cents = isNegative ? -magnitude : magnitude;
  if (cents < I64_MIN || cents > I64_MAX) return null;

  return cents.toString();
}
