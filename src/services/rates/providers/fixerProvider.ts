import type { RatesProvider } from "../types";

export class FixerRatesProvider implements RatesProvider {
  readonly type = "fixer" as const;
  readonly name = "Fixer.io";
  readonly url = "https://fixer.io";
  readonly requiresKey = true;
  readonly freeTierNote = "Free: 100 req/mo, EUR base only";

  async fetchRates(apiKey: string, baseCurrencyCode: string, targetCodes: string[]): Promise<Record<string, number>> {
    const allCodes = new Set(targetCodes);
    if (baseCurrencyCode !== "EUR") allCodes.add(baseCurrencyCode);
    const symbols = [...allCodes].join(",");
    const url = `http://data.fixer.io/api/latest?access_key=${apiKey}&base=${baseCurrencyCode}&symbols=${symbols}`;

    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);

    const data = await res.json();
    if (!data.success) throw new Error(data.error?.info || "Fixer API error");

    const rates = this.crossRate(data.rates, data.base || "EUR", baseCurrencyCode);
    delete rates[baseCurrencyCode];
    return rates;
  }

  private crossRate(rates: Record<string, number>, fetchBase: string, targetBase: string): Record<string, number> {
    if (fetchBase === targetBase) return rates;
    const baseRate = rates[targetBase];
    if (!baseRate) return rates;
    const result: Record<string, number> = {};
    for (const [code, rate] of Object.entries(rates)) {
      result[code] = rate / baseRate;
    }
    return result;
  }
}
