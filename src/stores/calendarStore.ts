import { defineStore } from "pinia";
import { ref } from "vue";
import type { Expense } from "@/schemas/appData";
import { getExpensesForMonth } from "@/services/expensesClient";
import { getPaymentDatesInMonth } from "@/services/subscriptionsClient";

export const useCalendarStore = defineStore("calendar", () => {
  const monthExpenses = ref<Expense[]>([]);
  const paymentDaysBySubId = ref<Record<string, number[]>>({});
  const loading = ref(false);
  const loadedYear = ref<number | null>(null);
  const loadedMonth = ref<number | null>(null);

  async function loadMonth(year: number, month1to12: number) {
    loading.value = true;
    try {
      const [expenses, paymentDays] = await Promise.all([
        getExpensesForMonth(year, month1to12),
        getPaymentDatesInMonth(year, month1to12),
      ]);
      monthExpenses.value = expenses;
      paymentDaysBySubId.value = paymentDays;
      loadedYear.value = year;
      loadedMonth.value = month1to12;
    } finally {
      loading.value = false;
    }
  }

  async function reloadCurrentMonth() {
    if (loadedYear.value && loadedMonth.value) {
      await loadMonth(loadedYear.value, loadedMonth.value);
    }
  }

  return {
    monthExpenses,
    paymentDaysBySubId,
    loading,
    loadedYear,
    loadedMonth,
    loadMonth,
    reloadCurrentMonth,
  };
});

