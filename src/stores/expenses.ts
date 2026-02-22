import { defineStore } from "pinia";
import { ref } from "vue";
import type { Expense, AppData } from "@/schemas/appData";
import { parseExpense } from "@/schemas/appData";
import {
  dbLoadExpenses, dbInsertExpense, dbUpdateExpense, dbDeleteExpense,
  dbDeleteExpensesBatch, dbGetExpenseById, dbGetExpenseCount,
  type ExpenseFilter, type ExpensePage,
} from "@/services/database";

export const PAGE_SIZE = 10;

export const useExpensesStore = defineStore("expenses", () => {
  const items = ref<Expense[]>([]);
  const totalCount = ref(0);
  const currentPage = ref(1);
  const filter = ref<ExpenseFilter>({});
  const loading = ref(false);

  async function fetchPage(page?: number, newFilter?: ExpenseFilter) {
    if (newFilter !== undefined) filter.value = newFilter;
    if (page !== undefined) currentPage.value = page;

    loading.value = true;
    try {
      const offset = (currentPage.value - 1) * PAGE_SIZE;
      const result: ExpensePage = await dbLoadExpenses(filter.value, PAGE_SIZE, offset);
      items.value = result.items;
      totalCount.value = result.total;
    } finally {
      loading.value = false;
    }
  }

  async function addExpense(raw: Partial<Expense> & { id: string; name: string; currencyId: string; date: string }) {
    const exp = parseExpense(raw);
    await dbInsertExpense(exp);
    await fetchPage();
  }

  async function updateExpense(raw: Partial<Expense> & { id: string }) {
    const existing = items.value.find((e) => e.id === raw.id);
    if (existing) {
      const updated = parseExpense({ ...existing, ...raw });
      await dbUpdateExpense(updated);
      await fetchPage();
    } else {
      const fromDb = await dbGetExpenseById(raw.id);
      if (fromDb) {
        const updated = parseExpense({ ...fromDb, ...raw });
        await dbUpdateExpense(updated);
        await fetchPage();
      }
    }
  }

  async function deleteExpense(id: string) {
    await dbDeleteExpense(id);
    await fetchPage();
  }

  async function batchDeleteExpenses(ids: string[]) {
    await dbDeleteExpensesBatch(ids);
    await fetchPage();
  }

  async function getCount(): Promise<number> {
    return dbGetExpenseCount();
  }

  function $hydrate(data: Partial<AppData>) {
    if (data.expenses) {
      items.value = data.expenses.slice(0, PAGE_SIZE);
      totalCount.value = data.expenses.length;
    }
  }

  return {
    items,
    totalCount,
    currentPage,
    filter,
    loading,
    fetchPage,
    addExpense,
    updateExpense,
    deleteExpense,
    batchDeleteExpenses,
    getCount,
    $hydrate,
    // backward compat getter
    get expenses() { return items.value; },
  };
});
