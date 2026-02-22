import { computed } from "vue";
import { useCatalogStore } from "@/stores/catalog";
import { formatCurrency, convertPrice } from "@/services/calculations";

export function useCurrencyFormat() {
  const catalogStore = useCatalogStore();

  const mainCurrency = computed(() => catalogStore.mainCurrency);

  function fmt(amount: number, currencyId?: string): string {
    if (currencyId) {
      const c = catalogStore.currencies.find((cur) => cur.id === currencyId);
      return formatCurrency(amount, c?.code || "USD", c?.symbol);
    }
    return formatCurrency(amount, mainCurrency.value?.code || "USD", mainCurrency.value?.symbol);
  }

  function getCurrencyRate(currencyId: string): number {
    return catalogStore.currencies.find((c) => c.id === currencyId)?.rate || 1;
  }

  function toMainCurrency(price: number, currencyId: string): number {
    return convertPrice(price, getCurrencyRate(currencyId));
  }

  return { mainCurrency, fmt, getCurrencyRate, toMainCurrency };
}
