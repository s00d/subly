import type { RatesProvider } from "../types";

export class CurrencyApiProvider implements RatesProvider {
  readonly type = "currencyapi" as const;
  readonly name = "CurrencyAPI";
  readonly url = "https://currencyapi.com";
  readonly requiresKey = true;
  readonly freeTierNote = "Free: 300 req/mo, all bases";

  async fetchRates(apiKey: string, baseCurrencyCode: string, targetCodes: string[]): Promise<Record<string, number>> {
    const currencies = targetCodes.join(",");
    const url = `https://api.currencyapi.com/v3/latest?base_currency=${baseCurrencyCode}&currencies=${currencies}`;

    const res = await fetch(url, {
      headers: { apikey: apiKey },
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);

    const data = await res.json();
    if (data.errors) throw new Error(JSON.stringify(data.errors));

    const result: Record<string, number> = {};
    for (const [code, info] of Object.entries(data.data as Record<string, { value: number }>)) {
      result[code] = info.value;
    }
    return result;
  }
}
