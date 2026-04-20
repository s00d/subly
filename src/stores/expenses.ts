import { defineStore } from "pinia";
import { ref } from "vue";
import type { Expense, AppData } from "@/schemas/appData";
import { parseExpense } from "@/schemas/appData";
import {
  dbLoadExpenses, dbInsertExpense, dbUpdateExpense, dbDeleteExpense,
  dbDeleteExpensesBatch, dbGetExpenseById, dbGetExpenseCount,
  type ExpenseFilter, type ExpensePage,
} from "@/services/database";
import { useSubscriptionsStore } from "./subscriptions";

export const PAGE_SIZE = 10;

export const useExpensesStore = defineStore("expenses", () => {
  const items = ref<Expense[]>([]);
  const totalCount = ref(0);
  const currentPage = ref(1);
  const filter = ref<ExpenseFilter>({});
  const loading = ref(false);
  const storeLogPrefix = "[ExpensesStore]";

  function logStoreError(scope: string, error: unknown, extra?: Record<string, unknown>) {
    console.error(`${storeLogPrefix} ${scope}`, {
      error,
      ...extra,
    });
  }

  async function fetchPage(page?: number, newFilter?: ExpenseFilter) {
    if (newFilter !== undefined) filter.value = newFilter;
    if (page !== undefined) currentPage.value = page;

    loading.value = true;
    try {
      const offset = (currentPage.value - 1) * PAGE_SIZE;
      const result: ExpensePage = await dbLoadExpenses(filter.value, PAGE_SIZE, offset);
      items.value = result.items;
      totalCount.value = result.total;
    } catch (e) {
      logStoreError("fetchPage failed", e, {
        page: currentPage.value,
        filter: filter.value,
      });
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function addExpense(raw: Partial<Expense> & { id: string; name: string; currencyId: string; date: string }) {
    try {
      const exp = parseExpense(raw);
      await dbInsertExpense(exp);
      await fetchPage();
    } catch (e) {
      logStoreError("addExpense failed", e, { id: raw.id, name: raw.name });
      throw e;
    }
  }

  async function updateExpense(raw: Partial<Expense> & { id: string }) {
    try {
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
    } catch (e) {
      logStoreError("updateExpense failed", e, { id: raw.id });
      throw e;
    }
  }

  async function deleteExpense(id: string) {
    try {
      const linked = items.value.find((e) => e.id === id) ?? await dbGetExpenseById(id);
      if (linked?.subscriptionId && linked?.paymentRecordId) {
        const sub = useSubscriptionsStore().subscriptions.find((s) => s.id === linked.subscriptionId);
        if (sub?.paymentHistory) {
          sub.paymentHistory = sub.paymentHistory.filter((r) => r.id !== linked.paymentRecordId);
        }
      }
      await dbDeleteExpense(id);
      await fetchPage();
    } catch (e) {
      logStoreError("deleteExpense failed", e, { id });
      throw e;
    }
  }

  async function batchDeleteExpenses(ids: string[]) {
    try {
      const linkedByExpense = new Map(items.value.map((e) => [e.id, e]));
      const subsStore = useSubscriptionsStore();
      for (const id of ids) {
        const linked = linkedByExpense.get(id);
        if (linked?.subscriptionId && linked?.paymentRecordId) {
          const sub = subsStore.subscriptions.find((s) => s.id === linked.subscriptionId);
          if (sub?.paymentHistory) {
            sub.paymentHistory = sub.paymentHistory.filter((r) => r.id !== linked.paymentRecordId);
          }
        }
      }
      await dbDeleteExpensesBatch(ids);
      await fetchPage();
    } catch (e) {
      logStoreError("batchDeleteExpenses failed", e, { ids });
      throw e;
    }
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
