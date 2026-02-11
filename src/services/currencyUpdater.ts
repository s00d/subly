import type { Currency, Settings } from "@/schemas/appData";

// =============================================
// Fixer.io / apilayer rate fetching
// =============================================

const FIXER_BASE = "http://data.fixer.io/api/latest";
const APILAYER_BASE = "https://api.apilayer.com/fixer/latest";

interface FixerResponse {
  success: boolean;
  base: string;
  rates: Record<string, number>;
}

/**
 * Fetch exchange rates from fixer.io or apilayer.
 * Returns rates relative to the base currency (EUR for fixer free tier).
 */
export async function fetchExchangeRates(
  apiKey: string,
  provider: number,
  baseCurrencyCode: string,
  targetCodes: string[],
): Promise<Record<string, number>> {
  if (!apiKey) throw new Error("No API key provided");
  if (targetCodes.length === 0) return {};

  const symbols = targetCodes.join(",");

  let url: string;
  let options: RequestInit = {};

  if (provider === 1) {
    // apilayer.com
    url = `${APILAYER_BASE}?symbols=${symbols}`;
    options = {
      headers: { apikey: apiKey },
    };
  } else {
    // fixer.io (free tier only supports EUR base)
    url = `${FIXER_BASE}?access_key=${apiKey}&symbols=${symbols}`;
  }

  const res = await fetch(url, options);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);

  const data: FixerResponse = await res.json();
  if (!data.success) throw new Error("API returned error");

  const rates = data.rates;
  const fetchBase = data.base || "EUR";

  // If our main currency is not the API base, we need to cross-rate
  // fixer free tier always returns EUR base
  if (fetchBase !== baseCurrencyCode) {
    const baseRate = rates[baseCurrencyCode];
    if (!baseRate) {
      // Can't cross-rate, return raw
      return rates;
    }
    // Convert: rate[X] relative to our base = rate[X] / rate[ourBase]
    const crossRated: Record<string, number> = {};
    for (const [code, rate] of Object.entries(rates)) {
      crossRated[code] = rate / baseRate;
    }
    return crossRated;
  }

  return rates;
}

/**
 * Check if enough time has passed since last update (24h).
 */
export function shouldUpdateRates(lastUpdate: string): boolean {
  if (!lastUpdate) return true;
  const last = new Date(lastUpdate);
  const now = new Date();
  const diffMs = now.getTime() - last.getTime();
  return diffMs >= 24 * 60 * 60 * 1000; // 24 hours
}

/**
 * Update currency rates in-place. Returns number of updated currencies.
 */
export async function updateCurrencyRates(
  apiKey: string,
  provider: number,
  currencies: Currency[],
  mainCurrencyId: string,
  targetCurrencyIds: string[],
): Promise<{ updated: number; error?: string }> {
  try {
    const mainCurrency = currencies.find((c) => c.id === mainCurrencyId);
    if (!mainCurrency) return { updated: 0, error: "Main currency not found" };

    // Determine which currencies to update
    const targetsToUpdate = targetCurrencyIds.length > 0
      ? currencies.filter((c) => targetCurrencyIds.includes(c.id) && c.id !== mainCurrencyId)
      : currencies.filter((c) => c.id !== mainCurrencyId);

    if (targetsToUpdate.length === 0) return { updated: 0 };

    const targetCodes = targetsToUpdate.map((c) => c.code);

    const rates = await fetchExchangeRates(apiKey, provider, mainCurrency.code, targetCodes);

    let updated = 0;
    for (const cur of targetsToUpdate) {
      if (rates[cur.code] !== undefined) {
        cur.rate = rates[cur.code];
        updated++;
      }
    }

    // Ensure main currency rate is 1
    mainCurrency.rate = 1;

    return { updated };
  } catch (e) {
    return { updated: 0, error: String(e) };
  }
}
