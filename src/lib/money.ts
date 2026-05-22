import Decimal from "decimal.js";

/** Canonical money amount from IPC / storage (decimal string, e.g. `"12.50"`). */
export type MoneyString = string;

const MONEY_RE = /^-?\d+(\.\d+)?$/;

/** Parse a validated money string (or internal `Decimal` from calculations). */
export function parseMoney(value: MoneyString | Decimal): Decimal {
  if (value instanceof Decimal) return value;
  const t = String(value ?? "").trim();
  if (!t || !MONEY_RE.test(t)) return new Decimal(0);
  return new Decimal(t);
}

/** Boundary: HTML number inputs / legacy JSON numbers → IPC string. */
export function moneyFromNumber(value: number): MoneyString {
  if (!Number.isFinite(value)) return "0";
  return new Decimal(value).toFixed();
}

/** Boundary: unknown IPC / import values → money string. */
export function asMoneyString(v: unknown, fallback: MoneyString = "0"): MoneyString {
  if (typeof v === "string") {
    const t = v.trim();
    if (t && MONEY_RE.test(t)) return t;
    if (!t) return fallback;
    try {
      return new Decimal(t).toFixed();
    } catch {
      return fallback;
    }
  }
  if (typeof v === "number" && Number.isFinite(v)) {
    return new Decimal(v).toFixed();
  }
  return fallback;
}

/** Serialize calculated `Decimal` back to IPC / `fmt` input. */
export function moneyFromDecimal(d: Decimal): MoneyString {
  return d.toFixed();
}

export function add(a: MoneyString | Decimal, b: MoneyString | Decimal): Decimal {
  return parseMoney(a).plus(parseMoney(b));
}

export function sub(a: MoneyString | Decimal, b: MoneyString | Decimal): Decimal {
  return parseMoney(a).minus(parseMoney(b));
}

export function mul(a: MoneyString | Decimal, b: MoneyString | Decimal): Decimal {
  return parseMoney(a).times(parseMoney(b));
}

export function div(a: MoneyString | Decimal, b: MoneyString | Decimal): Decimal {
  const divisor = parseMoney(b);
  if (divisor.isZero()) return parseMoney(a);
  return parseMoney(a).dividedBy(divisor);
}

export function sumMoney(values: Iterable<MoneyString>): Decimal {
  let total = new Decimal(0);
  for (const v of values) {
    total = total.plus(parseMoney(v));
  }
  return total;
}

export function convertToMain(
  amount: MoneyString | Decimal,
  currencyId: string,
  rates: Map<string, MoneyString> | Record<string, MoneyString>,
): Decimal {
  const map = rates instanceof Map ? rates : new Map(Object.entries(rates));
  const rate = parseMoney(map.get(currencyId) ?? "1");
  const amt = parseMoney(amount);
  if (rate.lte(0)) return amt;
  return amt.dividedBy(rate);
}

/** Only place that yields `number` for Intl / ECharts axes. */
export function toDisplayNumber(d: Decimal, fractionDigits = 2): number {
  return d.toDecimalPlaces(fractionDigits, Decimal.ROUND_HALF_UP).toNumber();
}

export function moneyGt(a: MoneyString, b: MoneyString): boolean {
  return parseMoney(a).greaterThan(parseMoney(b));
}

export function moneyLt(a: MoneyString, b: MoneyString): boolean {
  return parseMoney(a).lessThan(parseMoney(b));
}

export function moneySign(a: MoneyString): -1 | 0 | 1 {
  const d = parseMoney(a);
  if (d.isZero()) return 0;
  return d.isNegative() ? -1 : 1;
}

export function convertAmountToCurrency(
  amount: MoneyString,
  fromCurrencyId: string,
  toCurrency: { id: string; rate: MoneyString },
  rates: Record<string, MoneyString>,
): Decimal {
  const inMain = convertToMain(amount, fromCurrencyId, rates);
  const toRate = parseMoney(toCurrency.rate);
  if (toRate.lte(0)) return inMain;
  return inMain.times(toRate);
}
