import { callCommand } from "@/services/commandClient";

export type RatesProviderType =
  | "frankfurter"
  | "fixer"
  | "apilayer"
  | "exchangerate"
  | "openexchangerates"
  | "currencyapi";

export interface RatesProviderMeta {
  readonly type: RatesProviderType;
  readonly name: string;
  readonly requiresKey: boolean;
  readonly freeTierNote: string;
}

export interface RateHistoryPoint {
  rate: number;
  recordedAt: string;
}

export async function saveRateSnapshot(currencyId: string, rate: number): Promise<void> {
  return callCommand("rate_history_save_snapshot", { currencyId, rate });
}

export async function getRateHistory(currencyId: string, days = 30): Promise<RateHistoryPoint[]> {
  return callCommand("rate_history_get", { currencyId, days });
}

export async function getRateHistoryBatch(currencyIds: string[], days = 30): Promise<Record<string, RateHistoryPoint[]>> {
  return callCommand("get_rate_history_widget", { targetIds: currencyIds, days });
}

export async function pruneRateHistory(keepDays: number): Promise<number> {
  return callCommand("rate_history_prune", { keepDays });
}

export async function clearRateHistory(): Promise<void> {
  return callCommand("rate_history_clear");
}

export async function rateHistoryCount(): Promise<number> {
  return callCommand("rate_history_count");
}

export async function getExpenseAggregations(monthPrefix: string, yearPrefix: string): Promise<{ monthTotal: number; yearTotal: number; recentExpenses: import("@/schemas/appData").Expense[] }> {
  return callCommand("expenses_aggregations", { monthPrefix, yearPrefix });
}

export interface RatesUpdateOptions {
  historyEnabled?: boolean;
  historyDays?: number;
}

export async function ratesShouldUpdate(lastUpdate: string): Promise<boolean> {
  return callCommand("rates_should_update", { lastUpdate });
}

export async function ratesGetProviders(): Promise<RatesProviderMeta[]> {
  return callCommand("rates_get_providers");
}

export async function getCurrencyFlags(codes: string[]): Promise<Record<string, string>> {
  return callCommand("currency_get_flags", { codes });
}

export async function ratesUpdateWithFallback(
  providerType: RatesProviderType,
  apiKey: string,
  mainCurrencyId: string,
  targetCurrencyIds: string[],
  opts?: RatesUpdateOptions,
): Promise<{ updated: number; error?: string }> {
  return callCommand("rates_update_with_fallback", {
    providerType,
    apiKey,
    mainCurrencyId,
    targetCurrencyIds,
    opts,
  });
}

export async function ratesRunBackendUpdate(): Promise<{ updated: number; error?: string }> {
  console.log("[ratesClient] invoke rates_run_backend_update");
  try {
    const result = await callCommand<{ updated: number; error?: string }>("rates_run_backend_update");
    console.log("[ratesClient] rates_run_backend_update result", result);
    return result;
  } catch (error) {
    console.error("[ratesClient] rates_run_backend_update failed", error);
    throw error;
  }
}

