import { defineStore } from "pinia";
import { ref } from "vue";
import { parseSubscription, parseSubscriptionListItem, type Subscription, type SubscriptionListItem } from "@/schemas/appData";
import type { SubscriptionFilter } from "@/services/subscriptionsClient";
import {
  deleteSubscription,
  deleteSubscriptionsBatch,
  insertSubscription,
  listSubscriptionsFiltered,
  recordSubscriptionPayment,
  updateSubscription,
} from "@/services/subscriptionsClient";

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

  /**
   * `filter` replaces the active filter (`{}` = full list, as Calendar/Expenses need);
   * omitted = reload with the current one (after clone / record payment / data-changed).
   */
  async function loadBrief(filter?: SubscriptionFilter) {
    if (filter !== undefined) currentFilter.value = filter;
    loading.value = true;
    try {
      const rows = await listSubscriptionsFiltered(currentFilter.value);
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
    const updated = parseSubscription(await recordSubscriptionPayment(id));
    const idx = items.value.findIndex((s) => s.id === id);
    if (idx >= 0) items.value[idx] = { ...items.value[idx], ...updated };
    // List-only fields (daysLeft, cycleProgress, urgency) are backend-computed: refresh the row set.
    await loadBrief();
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

