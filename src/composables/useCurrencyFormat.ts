import { computed } from "vue";
import { storeToRefs } from "pinia";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { useAppMetaStore } from "@/stores/appMetaStore";
import {
  convertToMain,
  parseMoney,
  toDisplayNumber,
  type MoneyString,
} from "@/lib/money";

let ensureRequested = false;

export function useCurrencyFormat() {
  const metaStore = useAppMetaStore();
  const { settings, currencies } = storeToRefs(metaStore);
  if (!ensureRequested) {
    ensureRequested = true;
    void metaStore.ensureLoaded();
  }
  const { fmtCurrency } = useLocaleFormat();

  const mainCurrency = computed(() => {
    const mainId = settings.value?.mainCurrencyId;
    return currencies.value.find((c) => c.id === mainId) || null;
  });

  const ratesMap = computed(() => {
    const m: Record<string, MoneyString> = {};
    for (const c of currencies.value) {
      m[c.id] = c.rate;
    }
    return m;
  });

  function fmt(amount: MoneyString, currencyId?: string): string {
    const n = toDisplayNumber(parseMoney(amount));
    if (currencyId) {
      const c = currencies.value.find((cur) => cur.id === currencyId);
      return fmtCurrency(n, c?.code || "USD");
    }
    return fmtCurrency(n, mainCurrency.value?.code || "USD");
  }

  function getCurrencyRate(currencyId: string): MoneyString {
    return currencies.value.find((c) => c.id === currencyId)?.rate || "1";
  }

  function toMainCurrency(price: MoneyString, currencyId: string) {
    return convertToMain(price, currencyId, ratesMap.value);
  }

  return { mainCurrency, fmt, getCurrencyRate, toMainCurrency, ratesMap };
}
