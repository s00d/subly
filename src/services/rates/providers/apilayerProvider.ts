import type { RatesProvider } from "../types";

export class ApiLayerRatesProvider implements RatesProvider {
  readonly type = "apilayer" as const;
  readonly name = "APILayer (Fixer)";
  readonly url = "https://apilayer.com";
  readonly requiresKey = true;
  readonly freeTierNote = "Free: 100 req/mo";

  async fetchRates(apiKey: string, baseCurrencyCode: string, targetCodes: string[]): Promise<Record<string, number>> {
    const allCodes = new Set(targetCodes);
    allCodes.add(baseCurrencyCode);
    const symbols = [...allCodes].join(",");
    const url = `https://api.apilayer.com/fixer/latest?symbols=${symbols}&base=${baseCurrencyCode}`;

    const res = await fetch(url, { headers: { apikey: apiKey } });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);

    const data = await res.json();
    if (!data.success) throw new Error(data.error?.info || "APILayer error");

    const fetchBase = data.base || baseCurrencyCode;
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
