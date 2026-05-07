import { computed } from "vue";
import { storeToRefs } from "pinia";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { useAppMetaStore } from "@/stores/appMetaStore";

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

  function fmt(amount: number, currencyId?: string): string {
    if (currencyId) {
      const c = currencies.value.find((cur) => cur.id === currencyId);
      return fmtCurrency(amount, c?.code || "USD");
    }
    return fmtCurrency(amount, mainCurrency.value?.code || "USD");
  }

  function getCurrencyRate(currencyId: string): number {
    return currencies.value.find((c) => c.id === currencyId)?.rate || 1;
  }

  function toMainCurrency(price: number, currencyId: string): number {
    const rate = getCurrencyRate(currencyId);
    if (rate <= 0) return price;
    return price / rate;
  }

  return { mainCurrency, fmt, getCurrencyRate, toMainCurrency };
}
