import { computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { formatCurrency, convertPrice } from "@/services/calculations";

export function useCurrencyFormat() {
  const store = useAppStore();

  const mainCurrency = computed(() => store.mainCurrency.value);

  function fmt(amount: number, currencyId?: string): string {
    if (currencyId) {
      const c = store.state.currencies.find((cur) => cur.id === currencyId);
      return formatCurrency(amount, c?.code || "USD", c?.symbol);
    }
    return formatCurrency(amount, mainCurrency.value?.code || "USD", mainCurrency.value?.symbol);
  }

  function getCurrencyRate(currencyId: string): number {
    return store.state.currencies.find((c) => c.id === currencyId)?.rate || 1;
  }

  function toMainCurrency(price: number, currencyId: string): number {
    return convertPrice(price, getCurrencyRate(currencyId));
  }

  return { mainCurrency, fmt, getCurrencyRate, toMainCurrency };
}
