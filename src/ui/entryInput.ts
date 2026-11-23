const I64_MAX = 9_223_372_036_854_775_807n;
const DECIMAL_INPUT = /^[0-9]+(?:,[0-9]{1,2})?$/;

export function parseEntryAmountCents(input: string): string | null {
  if (!DECIMAL_INPUT.test(input)) return null;

  const [whole, fraction = ""] = input.split(",");
  const normalizedWhole = whole.replace(/^0+(?=\d)/, "");
  if (normalizedWhole.length > 17) return null;

  const cents =
    BigInt(normalizedWhole) * 100n + BigInt(fraction.padEnd(2, "0"));
  if (cents > I64_MAX) return null;

  return cents.toString();
}
