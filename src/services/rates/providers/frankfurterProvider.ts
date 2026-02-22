import type { RatesProvider } from "../types";

export class FrankfurterProvider implements RatesProvider {
  readonly type = "frankfurter" as const;
  readonly name = "Frankfurter";
  readonly url = "https://frankfurter.dev";
  readonly requiresKey = false;
  readonly freeTierNote = "Free & open-source, no key needed";

  async fetchRates(_apiKey: string, baseCurrencyCode: string, targetCodes: string[]): Promise<Record<string, number>> {
    const symbols = targetCodes.join(",");
    const url = `https://api.frankfurter.dev/v1/latest?base=${baseCurrencyCode}&symbols=${symbols}`;

    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);

    const data = await res.json();
    return data.rates as Record<string, number>;
  }
}
