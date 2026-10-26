import { defineStore } from "pinia";
import { ref } from "vue";
import type { Expense } from "@/schemas/appData";
import type { ExpenseFilter } from "@/services/expensesClient";
import { getExpenseTotalFiltered, listExpensesFiltered } from "@/services/expensesClient";

export const useExpensesStore = defineStore("expenses", () => {
  const items = ref<Expense[]>([]);
  const total = ref(0);
  const totalFiltered = ref(0);
  const loading = ref(false);
  const currentFilter = ref<ExpenseFilter>({});
  const page = ref(1);
  const pageSize = ref(10);

  async function loadPage(nextPage = 1, filter?: ExpenseFilter) {
    if (filter) currentFilter.value = filter;
    page.value = nextPage;
    loading.value = true;
    try {
      const offset = (page.value - 1) * pageSize.value;
      const res = await listExpensesFiltered(currentFilter.value, pageSize.value, offset);
      items.value = res.items;
      total.value = res.total;
      // Deleting the last row of the last page: fall back to the new last page.
      const lastPage = Math.max(1, Math.ceil(res.total / pageSize.value));
      if (res.items.length === 0 && page.value > lastPage) {
        page.value = lastPage;
        const back = await listExpensesFiltered(currentFilter.value, pageSize.value, (lastPage - 1) * pageSize.value);
        items.value = back.items;
        total.value = back.total;
      }
    } finally {
      loading.value = false;
    }
  }

  async function refreshSummary(filter?: ExpenseFilter) {
    if (filter) currentFilter.value = filter;
    totalFiltered.value = await getExpenseTotalFiltered(currentFilter.value);
  }

  return { items, total, totalFiltered, loading, currentFilter, page, pageSize, loadPage, refreshSummary };
});

