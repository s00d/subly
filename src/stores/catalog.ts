import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Category, Currency, HouseholdMember, PaymentMethod, Tag, AppData } from "@/schemas/appData";
import { parseCategory, parseCurrency, parseHouseholdMember, parsePaymentMethod, parseTag } from "@/schemas/appData";
import { translate } from "@/i18n";
import { useSettingsStore } from "./settings";
import { useSubscriptionsStore } from "./subscriptions";
import {
  dbUpsertCategory, dbDeleteCategory,
  dbUpsertCurrency, dbDeleteCurrency, dbUpdateCurrencyRates,
  dbUpsertHouseholdMember, dbDeleteHouseholdMember,
  dbUpsertPaymentMethod, dbDeletePaymentMethod,
  dbUpsertTag, dbDeleteTag,
  dbUpdateExpenseTagsBatch, dbRemoveExpenseTagBatch,
  dbUpdateSubscription, dbMaxSortOrder,
} from "@/services/database";

export const useCatalogStore = defineStore("catalog", () => {
  const categories = ref<Category[]>([]);
  const currencies = ref<Currency[]>([]);
  const household = ref<HouseholdMember[]>([]);
  const paymentMethods = ref<PaymentMethod[]>([]);
  const tags = ref<Tag[]>([]);

  const sortedCategories = computed(() => categories.value);

  const sortedTags = computed(() => tags.value);

  const favoriteTags = computed(() =>
    tags.value.filter((t) => t.favorite),
  );

  const enabledPaymentMethods = computed(() =>
    paymentMethods.value.filter((p) => p.enabled),
  );

  const mainCurrency = computed(() => {
    const stgs = useSettingsStore();
    return currencies.value.find((c) => c.id === stgs.settings.mainCurrencyId) || currencies.value[0];
  });

  function $hydrate(data: Partial<AppData>) {
    if (data.categories) categories.value = data.categories;
    if (data.currencies) currencies.value = data.currencies;
    if (data.household) household.value = data.household;
    if (data.paymentMethods) paymentMethods.value = data.paymentMethods;
    if (data.tags) tags.value = data.tags;
  }

  // ---- Categories ----
  async function addCategory(name: string): Promise<Category> {
    const maxOrder = await dbMaxSortOrder("categories");
    const cat = parseCategory({ id: crypto.randomUUID(), name, sortOrder: maxOrder + 1 });
    categories.value.push(cat);
    await dbUpsertCategory(cat);
    return cat;
  }

  async function updateCategory(id: string, name: string, icon?: string) {
    const cat = categories.value.find((c) => c.id === id);
    if (cat) {
      cat.name = name;
      if (icon !== undefined) cat.icon = icon;
      await dbUpsertCategory(cat);
    }
  }

  async function deleteCategory(id: string): Promise<boolean> {
    if (id === "cat-1") return false;
    const subs = useSubscriptionsStore();
    if (subs.subscriptions.some((s) => s.categoryId === id)) return false;
    categories.value = categories.value.filter((c) => c.id !== id);
    await dbDeleteCategory(id);
    return true;
  }

  async function reorderCategories(ids: string[]) {
    ids.forEach((id, i) => {
      const cat = categories.value.find((c) => c.id === id);
      if (cat) cat.sortOrder = i;
    });
    for (const cat of categories.value) {
      await dbUpsertCategory(cat);
    }
  }

  // ---- Currencies ----
  async function addCurrency(name: string, symbol: string, code: string): Promise<Currency> {
    const maxOrder = await dbMaxSortOrder("currencies");
    const cur = parseCurrency({ id: crypto.randomUUID(), name, symbol, code, rate: 1, sortOrder: maxOrder + 1 });
    currencies.value.push(cur);
    await dbUpsertCurrency(cur);
    return cur;
  }

  async function updateCurrency(id: string, updates: Partial<Currency>) {
    const cur = currencies.value.find((c) => c.id === id);
    if (cur) {
      Object.assign(cur, updates);
      await dbUpsertCurrency(cur);
    }
  }

  async function deleteCurrency(id: string): Promise<boolean> {
    const stgs = useSettingsStore();
    if (id === stgs.settings.mainCurrencyId) return false;
    const subs = useSubscriptionsStore();
    if (subs.subscriptions.some((s) => s.currencyId === id)) return false;
    currencies.value = currencies.value.filter((c) => c.id !== id);
    await dbDeleteCurrency(id);
    return true;
  }

  async function reorderCurrencies(ids: string[]) {
    ids.forEach((id, i) => {
      const cur = currencies.value.find((c) => c.id === id);
      if (cur) cur.sortOrder = i;
    });
    for (const cur of currencies.value) {
      await dbUpsertCurrency(cur);
    }
  }

  async function updateCurrencyRates(updates: { id: string; rate: number }[]) {
    for (const u of updates) {
      const cur = currencies.value.find((c) => c.id === u.id);
      if (cur) cur.rate = u.rate;
    }
    await dbUpdateCurrencyRates(updates);
  }

  // ---- Household ----
  async function addHouseholdMember(name: string): Promise<HouseholdMember> {
    const maxOrder = await dbMaxSortOrder("householdMembers");
    const member = parseHouseholdMember({ id: crypto.randomUUID(), name, email: "", sortOrder: maxOrder + 1 });
    household.value.push(member);
    await dbUpsertHouseholdMember(member);
    return member;
  }

  async function updateHouseholdMember(id: string, name: string, email: string) {
    const m = household.value.find((h) => h.id === id);
    if (m) {
      m.name = name;
      m.email = email;
      await dbUpsertHouseholdMember(m);
    }
  }

  async function deleteHouseholdMember(id: string): Promise<boolean> {
    if (household.value.length <= 1) return false;
    if (id === household.value[0]?.id) return false;
    const subs = useSubscriptionsStore();
    if (subs.subscriptions.some((s) => s.payerUserId === id)) return false;
    household.value = household.value.filter((h) => h.id !== id);
    await dbDeleteHouseholdMember(id);
    return true;
  }

  // ---- Payment Methods ----
  async function addPaymentMethod(name: string, icon: string): Promise<PaymentMethod> {
    const maxOrder = await dbMaxSortOrder("paymentMethods");
    const pm = parsePaymentMethod({ id: crypto.randomUUID(), name, icon, enabled: true, sortOrder: maxOrder + 1 });
    paymentMethods.value.push(pm);
    await dbUpsertPaymentMethod(pm);
    return pm;
  }

  async function updatePaymentMethod(id: string, updates: Partial<PaymentMethod>) {
    const pm = paymentMethods.value.find((p) => p.id === id);
    if (pm) {
      Object.assign(pm, updates);
      await dbUpsertPaymentMethod(pm);
    }
  }

  async function deletePaymentMethod(id: string): Promise<boolean> {
    const subs = useSubscriptionsStore();
    if (subs.subscriptions.some((s) => s.paymentMethodId === id)) return false;
    paymentMethods.value = paymentMethods.value.filter((p) => p.id !== id);
    await dbDeletePaymentMethod(id);
    return true;
  }

  async function togglePaymentMethod(id: string) {
    const pm = paymentMethods.value.find((p) => p.id === id);
    if (!pm) return;
    const subs = useSubscriptionsStore();
    if (subs.subscriptions.some((s) => s.paymentMethodId === id) && pm.enabled) return;
    pm.enabled = !pm.enabled;
    await dbUpsertPaymentMethod(pm);
  }

  async function reorderPaymentMethods(ids: string[]) {
    ids.forEach((id, i) => {
      const pm = paymentMethods.value.find((p) => p.id === id);
      if (pm) pm.sortOrder = i;
    });
    for (const pm of paymentMethods.value) {
      await dbUpsertPaymentMethod(pm);
    }
  }

  // ---- Tags ----
  async function addTag(name: string): Promise<Tag> {
    const n = name.trim();
    const maxOrder = await dbMaxSortOrder("tags");
    const tag = parseTag({ id: crypto.randomUUID(), name: n, sortOrder: maxOrder + 1, favorite: true });
    tags.value.push(tag);
    await dbUpsertTag(tag);
    return tag;
  }

  async function updateTag(id: string, updates: Partial<Tag>) {
    const tag = tags.value.find((t) => t.id === id);
    if (tag) {
      if (updates.name && updates.name !== tag.name) {
        const oldName = tag.name;
        const newName = updates.name;
        const subs = useSubscriptionsStore();
        for (const sub of subs.subscriptions) {
          const si = sub.tags.indexOf(oldName);
          if (si !== -1) {
            sub.tags[si] = newName;
            await dbUpdateSubscription(sub);
          }
        }
        await dbUpdateExpenseTagsBatch(oldName, newName);
      }
      Object.assign(tag, updates);
      await dbUpsertTag(tag);
    }
  }

  async function deleteTag(id: string) {
    const tag = tags.value.find((t) => t.id === id);
    if (!tag) return;
    const name = tag.name;
    tags.value = tags.value.filter((t) => t.id !== id);
    const subs = useSubscriptionsStore();
    for (const sub of subs.subscriptions) {
      if (sub.tags.includes(name)) {
        sub.tags = sub.tags.filter((t) => t !== name);
        await dbUpdateSubscription(sub);
      }
    }
    await dbRemoveExpenseTagBatch(name);
    await dbDeleteTag(id);
  }

  async function reorderTags(ids: string[]) {
    ids.forEach((id, i) => {
      const tag = tags.value.find((t) => t.id === id);
      if (tag) tag.sortOrder = i;
    });
    for (const tag of tags.value) {
      await dbUpsertTag(tag);
    }
  }

  async function toggleTagFavorite(id: string) {
    const tag = tags.value.find((t) => t.id === id);
    if (tag) {
      tag.favorite = !tag.favorite;
      await dbUpsertTag(tag);
    }
  }

  async function ensureTag(name: string): Promise<string> {
    const n = name.trim();
    if (!n) return "";
    if (!tags.value.find((t) => t.name === n)) await addTag(n);
    return n;
  }

  async function retranslateDefaults() {
    const subs = useSubscriptionsStore();

    for (const cat of categories.value) {
      if (cat.i18nKey) {
        const translated = translate(cat.i18nKey);
        if (translated !== cat.i18nKey) cat.name = translated;
      }
    }
    for (const cur of currencies.value) {
      if (cur.i18nKey) {
        const translated = translate(cur.i18nKey);
        if (translated !== cur.i18nKey) cur.name = translated;
      }
    }
    for (const pm of paymentMethods.value) {
      if (pm.i18nKey) {
        const translated = translate(pm.i18nKey);
        if (translated !== pm.i18nKey) pm.name = translated;
      }
    }
    for (const tag of tags.value) {
      if (tag.i18nKey) {
        const translated = translate(tag.i18nKey);
        if (translated !== tag.i18nKey) {
          const oldName = tag.name;
          const newName = translated;
          if (oldName !== newName) {
            for (const sub of subs.subscriptions) {
              const idx = sub.tags.indexOf(oldName);
              if (idx !== -1) sub.tags[idx] = newName;
            }
            tag.name = newName;
          }
        }
      }
    }
  }

  return {
    categories, currencies, household, paymentMethods, tags,
    sortedCategories, sortedTags, favoriteTags, enabledPaymentMethods, mainCurrency,
    $hydrate,
    addCategory, updateCategory, deleteCategory, reorderCategories,
    addCurrency, updateCurrency, deleteCurrency, reorderCurrencies, updateCurrencyRates,
    addHouseholdMember, updateHouseholdMember, deleteHouseholdMember,
    addPaymentMethod, updatePaymentMethod, deletePaymentMethod, togglePaymentMethod, reorderPaymentMethods,
    addTag, updateTag, deleteTag, reorderTags, toggleTagFavorite, ensureTag,
    retranslateDefaults,
  };
});
