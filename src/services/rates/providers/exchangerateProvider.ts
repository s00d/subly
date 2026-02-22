import type { RatesProvider } from "../types";

export class ExchangeRateProvider implements RatesProvider {
  readonly type = "exchangerate" as const;
  readonly name = "ExchangeRate-API";
  readonly url = "https://www.exchangerate-api.com";
  readonly requiresKey = true;
  readonly freeTierNote = "Free: 1500 req/mo, all bases";

  async fetchRates(apiKey: string, baseCurrencyCode: string, targetCodes: string[]): Promise<Record<string, number>> {
    const url = `https://v6.exchangerate-api.com/v6/${apiKey}/latest/${baseCurrencyCode}`;

    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);

    const data = await res.json();
    if (data.result !== "success") throw new Error(data["error-type"] || "ExchangeRate-API error");

    const allRates: Record<string, number> = data.conversion_rates;
    const filtered: Record<string, number> = {};
    for (const code of targetCodes) {
      if (allRates[code] !== undefined) filtered[code] = allRates[code];
    }
    return filtered;
  }
}
