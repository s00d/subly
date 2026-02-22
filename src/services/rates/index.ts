import type { Currency } from "@/schemas/appData";
import type { RatesProviderType, RatesProviderMeta } from "./types";
import type { RatesProvider } from "./types";
import { dbSaveRateSnapshot, dbPruneRateHistory } from "@/services/database";
import { FixerRatesProvider } from "./providers/fixerProvider";
import { ApiLayerRatesProvider } from "./providers/apilayerProvider";
import { ExchangeRateProvider } from "./providers/exchangerateProvider";
import { OpenExchangeRatesProvider } from "./providers/openExchangeRatesProvider";
import { CurrencyApiProvider } from "./providers/currencyapiProvider";
import { FrankfurterProvider } from "./providers/frankfurterProvider";

export type { RatesProviderType, RatesProviderMeta } from "./types";

const providers: Record<RatesProviderType, RatesProvider> = {
  frankfurter: new FrankfurterProvider(),
  fixer: new FixerRatesProvider(),
  apilayer: new ApiLayerRatesProvider(),
  exchangerate: new ExchangeRateProvider(),
  openexchangerates: new OpenExchangeRatesProvider(),
  currencyapi: new CurrencyApiProvider(),
};

export function getRatesProviders(): RatesProviderMeta[] {
  return Object.values(providers).map((p) => ({
    type: p.type,
    name: p.name,
    url: p.url,
    requiresKey: p.requiresKey,
    freeTierNote: p.freeTierNote,
  }));
}

export function getRatesProvider(type: RatesProviderType): RatesProvider {
  return providers[type];
}

export function shouldUpdateRates(lastUpdate: string): boolean {
  if (!lastUpdate) return true;
  const last = new Date(lastUpdate);
  const now = new Date();
  return now.getTime() - last.getTime() >= 24 * 60 * 60 * 1000;
}

export interface RateUpdateOptions {
  historyEnabled?: boolean;
  historyDays?: number;
}

export async function updateCurrencyRates(
  providerType: RatesProviderType,
  apiKey: string,
  currencies: Currency[],
  mainCurrencyId: string,
  targetCurrencyIds: string[],
  opts?: RateUpdateOptions,
): Promise<{ updated: number; error?: string }> {
  try {
    const provider = providers[providerType];
    if (!provider) return { updated: 0, error: "Unknown provider" };

    const mainCurrency = currencies.find((c) => c.id === mainCurrencyId);
    if (!mainCurrency) return { updated: 0, error: "Main currency not found" };

    const targetsToUpdate = targetCurrencyIds.length > 0
      ? currencies.filter((c) => targetCurrencyIds.includes(c.id) && c.id !== mainCurrencyId)
      : currencies.filter((c) => c.id !== mainCurrencyId);

    if (targetsToUpdate.length === 0) return { updated: 0 };

    const targetCodes = targetsToUpdate.map((c) => c.code);
    const rates = await provider.fetchRates(apiKey, mainCurrency.code, targetCodes);

    let updated = 0;
    const saveHistory = opts?.historyEnabled !== false;
    const historyPromises: Promise<void>[] = [];
    for (const cur of targetsToUpdate) {
      if (rates[cur.code] !== undefined) {
        cur.rate = rates[cur.code];
        if (saveHistory) historyPromises.push(dbSaveRateSnapshot(cur.id, cur.rate));
        updated++;
      }
    }

    mainCurrency.rate = 1;
    if (saveHistory) {
      historyPromises.push(dbSaveRateSnapshot(mainCurrency.id, 1));
      const keepDays = opts?.historyDays ?? 90;
      historyPromises.push(dbPruneRateHistory(keepDays).then(() => {}));
    }
    await Promise.all(historyPromises).catch(() => {});
    return { updated };
  } catch (e) {
    return { updated: 0, error: String(e) };
  }
}
