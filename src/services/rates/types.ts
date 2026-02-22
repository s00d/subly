export type RatesProviderType = "fixer" | "apilayer" | "exchangerate" | "openexchangerates" | "currencyapi" | "frankfurter";

export interface RatesProviderMeta {
  readonly type: RatesProviderType;
  readonly name: string;
  readonly url: string;
  readonly requiresKey: boolean;
  readonly freeTierNote: string;
}

export interface RatesProvider extends RatesProviderMeta {
  fetchRates(apiKey: string, baseCurrencyCode: string, targetCodes: string[]): Promise<Record<string, number>>;
}
