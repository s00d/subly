import { defineStore } from "pinia";
import { ref } from "vue";
import { parseExpense, parseSubscription, parseSubscriptionListItem, type PaymentRecord, type Subscription, type SubscriptionListItem } from "@/schemas/appData";
import type { SubscriptionFilter } from "@/services/subscriptionsClient";
import {
  deleteSubscription,
  deleteSubscriptionsBatch,
  getNextCycleDate,
  insertPaymentRecord,
  insertSubscription,
  listSubscriptionsFiltered,
  listSubscriptionsPage,
  updateSubscription,
} from "@/services/subscriptionsClient";
import { insertExpense } from "@/services/expensesClient";

export const useSubscriptionsStore = defineStore("subscriptions", () => {
  function requireSubscription(id: string): SubscriptionListItem {
    const sub = items.value.find((s) => s.id === id);
    if (!sub) {
      throw new Error(`Subscription '${id}' not found`);
    }
    return sub;
  }

  const items = ref<SubscriptionListItem[]>([]);
  const loading = ref(false);
  /** Первая успешная загрузка списка завершена (для скелетона при входе на страницу). */
  const initialListLoaded = ref(false);
  const currentFilter = ref<SubscriptionFilter>({});

  async function loadBrief(filter?: SubscriptionFilter) {
    if (filter) currentFilter.value = filter;
    loading.value = true;
    try {
      const rows = filter ? await listSubscriptionsFiltered(currentFilter.value) : await listSubscriptionsPage();
      items.value = rows.map((raw) => parseSubscriptionListItem(raw));
    } finally {
      loading.value = false;
      initialListLoaded.value = true;
    }
  }

  async function updateOne(subscription: Subscription) {
    await updateSubscription(subscription);
    const idx = items.value.findIndex((s) => s.id === subscription.id);
    if (idx >= 0) {
      const prev = items.value[idx];
      items.value[idx] = { ...prev, ...subscription };
    }
  }

  async function deleteById(id: string) {
    await deleteSubscription(id);
    items.value = items.value.filter((s) => s.id !== id);
  }

  async function cloneById(id: string) {
    const original = requireSubscription(id);
    const { credentials: _omitCreds, ...rest } = original;
    const cloned = parseSubscription({
      ...rest,
      id: crypto.randomUUID(),
      name: `${original.name} (copy)`,
      createdAt: new Date().toISOString(),
      paymentHistory: [],
    });
    await insertSubscription(cloned);
    await loadBrief();
  }

  async function toggleFavorite(id: string) {
    const sub = requireSubscription(id);
    await updateOne({ ...sub, favorite: !sub.favorite });
  }

  async function recordPayment(id: string) {
    const sub = requireSubscription(id);
    const record: PaymentRecord = {
      id: crypto.randomUUID(),
      date: new Date().toISOString().split("T")[0],
      amount: sub.price,
      currencyId: sub.currencyId,
      note: "",
    };
    const nextPayment = await getNextCycleDate(sub.nextPayment, sub.cycle, sub.frequency);
    const updated: Subscription = {
      ...sub,
      nextPayment,
      paymentHistory: [record, ...(sub.paymentHistory || [])],
    };
    await updateSubscription(updated);
    await insertPaymentRecord(sub.id, record);
    const expense = parseExpense({
      id: crypto.randomUUID(),
      name: sub.name,
      amount: record.amount,
      currencyId: record.currencyId,
      date: record.date,
      categoryId: sub.categoryId,
      paymentMethodId: sub.paymentMethodId,
      payerUserId: sub.payerUserId,
      tags: [...sub.tags],
      notes: record.note,
      subscriptionId: sub.id,
      paymentRecordId: record.id,
    });
    await insertExpense(expense);
    await updateOne(updated);
  }

  async function batchDelete(ids: string[]) {
    await deleteSubscriptionsBatch(ids);
    items.value = items.value.filter((s) => !ids.includes(s.id));
  }

  async function batchSetInactive(ids: string[], inactive: boolean) {
    for (const id of ids) {
      const sub = requireSubscription(id);
      await updateOne({ ...sub, inactive });
    }
  }

  async function batchSetCategory(ids: string[], categoryId: string) {
    for (const id of ids) {
      const sub = requireSubscription(id);
      await updateOne({ ...sub, categoryId });
    }
  }

  return {
    items,
    loading,
    initialListLoaded,
    currentFilter,
    loadBrief,
    updateOne,
    deleteById,
    cloneById,
    toggleFavorite,
    recordPayment,
    batchDelete,
    batchSetInactive,
    batchSetCategory,
  };
});

