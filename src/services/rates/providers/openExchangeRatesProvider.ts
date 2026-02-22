import type { RatesProvider } from "../types";

export class OpenExchangeRatesProvider implements RatesProvider {
  readonly type = "openexchangerates" as const;
  readonly name = "Open Exchange Rates";
  readonly url = "https://openexchangerates.org";
  readonly requiresKey = true;
  readonly freeTierNote = "Free: 1000 req/mo, USD base only";

  async fetchRates(apiKey: string, baseCurrencyCode: string, targetCodes: string[]): Promise<Record<string, number>> {
    const allCodes = new Set(targetCodes);
    if (baseCurrencyCode !== "USD") allCodes.add(baseCurrencyCode);
    const symbols = [...allCodes].join(",");
    const url = `https://openexchangerates.org/api/latest.json?app_id=${apiKey}&base=${baseCurrencyCode}&symbols=${symbols}`;

    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);

    const data = await res.json();
    if (data.error) throw new Error(data.description || "Open Exchange Rates error");

    const fetchBase = data.base || "USD";
    const rates = this.crossRate(data.rates, fetchBase, baseCurrencyCode);
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
