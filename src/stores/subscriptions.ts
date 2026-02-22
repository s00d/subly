import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Subscription, PaymentRecord, AppData } from "@/schemas/appData";
import { parseSubscription } from "@/schemas/appData";
import {
  dbInsertSubscription, dbUpdateSubscription, dbDeleteSubscription,
  dbDeleteSubscriptionsBatch, dbInsertPaymentRecord, dbDeletePaymentRecord,
  dbInsertExpense, dbDeleteExpenseByPaymentRecord, dbLoadSubscriptions,
} from "@/services/database";
import { parseExpense } from "@/schemas/appData";

export const useSubscriptionsStore = defineStore("subscriptions", () => {
  const subscriptions = ref<Subscription[]>([]);

  const activeSubscriptions = computed(() =>
    subscriptions.value.filter((s) => !s.inactive),
  );

  const inactiveSubscriptions = computed(() =>
    subscriptions.value.filter((s) => s.inactive),
  );

  function $hydrate(data: Partial<AppData>) {
    if (data.subscriptions) subscriptions.value = data.subscriptions;
  }

  async function reload() {
    subscriptions.value = await dbLoadSubscriptions();
  }

  async function addSubscription(raw: Partial<Subscription> & { id: string; name: string; currencyId: string; nextPayment: string; startDate: string }) {
    const sub = parseSubscription(raw);
    subscriptions.value.push(sub);
    await dbInsertSubscription(sub);
  }

  async function updateSubscription(raw: Partial<Subscription> & { id: string }) {
    const idx = subscriptions.value.findIndex((s) => s.id === raw.id);
    if (idx !== -1) {
      const updated = parseSubscription({ ...subscriptions.value[idx], ...raw });
      subscriptions.value[idx] = updated;
      await dbUpdateSubscription(updated);
    }
  }

  async function deleteSubscription(id: string) {
    subscriptions.value = subscriptions.value.filter((s) => s.id !== id);
    subscriptions.value.forEach((s) => {
      if (s.replacementSubscriptionId === id) s.replacementSubscriptionId = null;
    });
    await dbDeleteSubscription(id);
  }

  async function cloneSubscription(id: string): Promise<Subscription | null> {
    const original = subscriptions.value.find((s) => s.id === id);
    if (!original) return null;
    const cloned = parseSubscription({
      ...original,
      id: crypto.randomUUID(),
      name: `${original.name} (copy)`,
      createdAt: new Date().toISOString(),
      paymentHistory: [],
    });
    subscriptions.value.push(cloned);
    await dbInsertSubscription(cloned);
    return cloned;
  }

  function advanceNextPayment(sub: Subscription) {
    const next = new Date(sub.nextPayment);
    switch (sub.cycle) {
      case 1: next.setDate(next.getDate() + sub.frequency); break;
      case 2: next.setDate(next.getDate() + 7 * sub.frequency); break;
      case 3: next.setMonth(next.getMonth() + sub.frequency); break;
      case 4: next.setFullYear(next.getFullYear() + sub.frequency); break;
    }
    sub.nextPayment = next.toISOString().split("T")[0];
  }

  async function renewSubscription(id: string) {
    await recordPayment(id);
  }

  async function createExpenseFromPayment(sub: Subscription, record: PaymentRecord) {
    const exp = parseExpense({
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
    await dbInsertExpense(exp);
  }

  async function recordPayment(id: string, amount?: number, note?: string) {
    const sub = subscriptions.value.find((s) => s.id === id);
    if (!sub) return;
    const record: PaymentRecord = {
      id: crypto.randomUUID(),
      date: new Date().toISOString().split("T")[0],
      amount: amount ?? sub.price,
      currencyId: sub.currencyId,
      note: note || "",
    };
    if (!sub.paymentHistory) sub.paymentHistory = [];
    sub.paymentHistory.unshift(record);
    advanceNextPayment(sub);
    await dbUpdateSubscription(sub);
    await dbInsertPaymentRecord(sub.id, record);
    await createExpenseFromPayment(sub, record);
  }

  async function addPaymentRecord(subId: string, record: PaymentRecord) {
    const sub = subscriptions.value.find((s) => s.id === subId);
    if (!sub) return;
    if (!sub.paymentHistory) sub.paymentHistory = [];
    sub.paymentHistory.push(record);
    await dbInsertPaymentRecord(subId, record);
    await createExpenseFromPayment(sub, record);
  }

  async function deletePaymentRecord(subId: string, recordId: string) {
    const sub = subscriptions.value.find((s) => s.id === subId);
    if (!sub || !sub.paymentHistory) return;
    sub.paymentHistory = sub.paymentHistory.filter((r) => r.id !== recordId);
    await dbDeletePaymentRecord(recordId);
    await dbDeleteExpenseByPaymentRecord(subId, recordId);
  }

  async function toggleFavorite(id: string) {
    const sub = subscriptions.value.find((s) => s.id === id);
    if (sub) {
      sub.favorite = !sub.favorite;
      await dbUpdateSubscription(sub);
    }
  }

  async function markNotified(subId: string, date: string) {
    const sub = subscriptions.value.find((s) => s.id === subId);
    if (sub) {
      sub.lastNotifiedDate = date;
      await dbUpdateSubscription(sub);
    }
  }

  async function batchDelete(ids: string[]) {
    subscriptions.value = subscriptions.value.filter((s) => !ids.includes(s.id));
    subscriptions.value.forEach((s) => {
      if (s.replacementSubscriptionId && ids.includes(s.replacementSubscriptionId)) {
        s.replacementSubscriptionId = null;
      }
    });
    await dbDeleteSubscriptionsBatch(ids);
  }

  async function batchSetInactive(ids: string[], inactive: boolean) {
    for (const sub of subscriptions.value) {
      if (ids.includes(sub.id)) sub.inactive = inactive;
    }
    for (const sub of subscriptions.value) {
      if (ids.includes(sub.id)) await dbUpdateSubscription(sub);
    }
  }

  async function batchSetCategory(ids: string[], categoryId: string) {
    for (const sub of subscriptions.value) {
      if (ids.includes(sub.id)) sub.categoryId = categoryId;
    }
    for (const sub of subscriptions.value) {
      if (ids.includes(sub.id)) await dbUpdateSubscription(sub);
    }
  }

  async function batchSetTags(ids: string[], tags: string[]) {
    for (const sub of subscriptions.value) {
      if (ids.includes(sub.id)) {
        sub.tags = [...new Set([...sub.tags, ...tags])];
      }
    }
    for (const sub of subscriptions.value) {
      if (ids.includes(sub.id)) await dbUpdateSubscription(sub);
    }
  }

  return {
    subscriptions,
    activeSubscriptions,
    inactiveSubscriptions,
    $hydrate,
    reload,
    addSubscription,
    updateSubscription,
    deleteSubscription,
    cloneSubscription,
    renewSubscription,
    recordPayment,
    addPaymentRecord,
    deletePaymentRecord,
    toggleFavorite,
    markNotified,
    batchDelete,
    batchSetInactive,
    batchSetCategory,
    batchSetTags,
  };
});
