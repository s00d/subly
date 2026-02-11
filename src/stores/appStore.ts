import { reactive, ref, computed } from "vue";
import type { AppData, Subscription, PaymentRecord, Expense, Tag, Category, Currency, HouseholdMember, PaymentMethod, Settings } from "@/schemas/appData";
import {
  SubscriptionSchema,
  ExpenseSchema,
  TagSchema,
  CategorySchema,
  CurrencySchema,
  HouseholdMemberSchema,
  PaymentMethodSchema,
  SettingsSchema,
  AppDataSchema,
  parseSubscription,
  parseExpense,
  parseTag,
  parseCategory,
  parseCurrency,
  parseHouseholdMember,
  parsePaymentMethod,
  parseSettings,
} from "@/schemas/appData";
import { loadAppData, saveAppData } from "@/services/storage";
import { setLanguage, translate } from "@/i18n";

const state = reactive<AppData>(AppDataSchema.parse({
  subscriptions: [],
  categories: [],
  currencies: [],
  household: [],
  paymentMethods: [],
  settings: SettingsSchema.parse({}),
  fixerApiKey: "",
  fixerProvider: 0,
  initialized: false,
}));

const isLoading = ref(true);
let saveTimeout: ReturnType<typeof setTimeout> | null = null;

function debouncedSave() {
  if (saveTimeout) clearTimeout(saveTimeout);
  saveTimeout = setTimeout(() => {
    saveAppData({ ...state });
  }, 300);
}

export function useAppStore() {
  // ---- Init ----
  async function init() {
    isLoading.value = true;
    try {
      const data = await loadAppData();
      Object.assign(state, data);
      setLanguage(state.settings.language);
      // Retranslate defaults to match current language
      if (state.settings.language !== "en") {
        retranslateDefaults(state.settings.language);
      }
    } catch (e) {
      console.error("Failed to load data:", e);
    } finally {
      isLoading.value = false;
    }
  }

  // ---- Subscriptions ----
  function addSubscription(raw: Partial<Subscription> & { id: string; name: string; currencyId: string; nextPayment: string; startDate: string }) {
    const sub = parseSubscription(raw);
    state.subscriptions.push(sub);
    debouncedSave();
  }

  function updateSubscription(raw: Partial<Subscription> & { id: string }) {
    const idx = state.subscriptions.findIndex((s) => s.id === raw.id);
    if (idx !== -1) {
      // Merge existing with updates, then validate
      const merged = { ...state.subscriptions[idx], ...raw };
      state.subscriptions[idx] = parseSubscription(merged);
      debouncedSave();
    }
  }

  function deleteSubscription(id: string) {
    state.subscriptions = state.subscriptions.filter((s) => s.id !== id);
    // Clear replacement references
    state.subscriptions.forEach((s) => {
      if (s.replacementSubscriptionId === id) {
        s.replacementSubscriptionId = null;
      }
    });
    debouncedSave();
  }

  function cloneSubscription(id: string): Subscription | null {
    const original = state.subscriptions.find((s) => s.id === id);
    if (!original) return null;
    const cloned = parseSubscription({
      ...original,
      id: crypto.randomUUID(),
      name: `${original.name} (copy)`,
      createdAt: new Date().toISOString(),
    });
    state.subscriptions.push(cloned);
    debouncedSave();
    return cloned;
  }

  function renewSubscription(id: string) {
    const sub = state.subscriptions.find((s) => s.id === id);
    if (!sub) return;
    const next = new Date(sub.nextPayment);
    switch (sub.cycle) {
      case 1: next.setDate(next.getDate() + sub.frequency); break;
      case 2: next.setDate(next.getDate() + 7 * sub.frequency); break;
      case 3: next.setMonth(next.getMonth() + sub.frequency); break;
      case 4: next.setFullYear(next.getFullYear() + sub.frequency); break;
    }
    sub.nextPayment = next.toISOString().split("T")[0];
    debouncedSave();
  }

  // ---- Payment history ----

  /**
   * Record a payment: add to history AND advance nextPayment by one cycle.
   */
  function recordPayment(id: string, amount?: number, note?: string) {
    const sub = state.subscriptions.find((s) => s.id === id);
    if (!sub) return;

    const record: PaymentRecord = {
      id: crypto.randomUUID(),
      date: new Date().toISOString().split("T")[0],
      amount: amount ?? sub.price,
      currencyId: sub.currencyId,
      note: note || "",
    };

    if (!sub.paymentHistory) sub.paymentHistory = [];
    sub.paymentHistory.unshift(record); // newest first

    // Advance next payment
    const next = new Date(sub.nextPayment);
    switch (sub.cycle) {
      case 1: next.setDate(next.getDate() + sub.frequency); break;
      case 2: next.setDate(next.getDate() + 7 * sub.frequency); break;
      case 3: next.setMonth(next.getMonth() + sub.frequency); break;
      case 4: next.setFullYear(next.getFullYear() + sub.frequency); break;
    }
    sub.nextPayment = next.toISOString().split("T")[0];

    debouncedSave();
  }

  /**
   * Add a manual historical payment record (without advancing nextPayment).
   */
  function addPaymentRecord(subId: string, record: PaymentRecord) {
    const sub = state.subscriptions.find((s) => s.id === subId);
    if (!sub) return;
    if (!sub.paymentHistory) sub.paymentHistory = [];
    sub.paymentHistory.push(record);
    // Sort newest first
    sub.paymentHistory.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());
    debouncedSave();
  }

  /**
   * Delete a payment record from history.
   */
  function deletePaymentRecord(subId: string, recordId: string) {
    const sub = state.subscriptions.find((s) => s.id === subId);
    if (!sub || !sub.paymentHistory) return;
    sub.paymentHistory = sub.paymentHistory.filter((r) => r.id !== recordId);
    debouncedSave();
  }

  // ---- Expenses ----
  function addExpense(raw: Partial<Expense> & { id: string; name: string; currencyId: string; date: string }) {
    const exp = parseExpense(raw);
    state.expenses.push(exp);
    debouncedSave();
  }

  function updateExpense(raw: Partial<Expense> & { id: string }) {
    const idx = state.expenses.findIndex((e) => e.id === raw.id);
    if (idx !== -1) {
      const merged = { ...state.expenses[idx], ...raw };
      state.expenses[idx] = parseExpense(merged);
      debouncedSave();
    }
  }

  function deleteExpense(id: string) {
    state.expenses = state.expenses.filter((e) => e.id !== id);
    debouncedSave();
  }

  function batchDeleteExpenses(ids: string[]) {
    state.expenses = state.expenses.filter((e) => !ids.includes(e.id));
    debouncedSave();
  }

  // ---- Categories ----
  function addCategory(name: string): Category {
    const maxOrder = state.categories.reduce((m, c) => Math.max(m, c.order), 0);
    const cat = parseCategory({ id: crypto.randomUUID(), name, order: maxOrder + 1 });
    state.categories.push(cat);
    debouncedSave();
    return cat;
  }

  function updateCategory(id: string, name: string, icon?: string) {
    const cat = state.categories.find((c) => c.id === id);
    if (cat) {
      cat.name = name;
      if (icon !== undefined) cat.icon = icon;
      debouncedSave();
    }
  }

  function deleteCategory(id: string): boolean {
    if (id === "cat-1") return false;
    const inUse = state.subscriptions.some((s) => s.categoryId === id) || state.expenses.some((e) => e.categoryId === id);
    if (inUse) return false;
    state.categories = state.categories.filter((c) => c.id !== id);
    debouncedSave();
    return true;
  }

  function reorderCategories(ids: string[]) {
    ids.forEach((id, i) => {
      const cat = state.categories.find((c) => c.id === id);
      if (cat) cat.order = i;
    });
    debouncedSave();
  }

  // ---- Currencies ----
  function addCurrency(name: string, symbol: string, code: string): Currency {
    const maxOrder = state.currencies.reduce((m, c) => Math.max(m, c.order), 0);
    const cur = parseCurrency({ id: crypto.randomUUID(), name, symbol, code, rate: 1, order: maxOrder + 1 });
    state.currencies.push(cur);
    debouncedSave();
    return cur;
  }

  function updateCurrency(id: string, updates: Partial<Currency>) {
    const cur = state.currencies.find((c) => c.id === id);
    if (cur) { Object.assign(cur, updates); debouncedSave(); }
  }

  function deleteCurrency(id: string): boolean {
    if (id === state.settings.mainCurrencyId) return false;
    const inUse = state.subscriptions.some((s) => s.currencyId === id) || state.expenses.some((e) => e.currencyId === id);
    if (inUse) return false;
    state.currencies = state.currencies.filter((c) => c.id !== id);
    debouncedSave();
    return true;
  }

  function reorderCurrencies(ids: string[]) {
    ids.forEach((id, i) => {
      const cur = state.currencies.find((c) => c.id === id);
      if (cur) cur.order = i;
    });
    debouncedSave();
  }

  // ---- Household ----
  function addHouseholdMember(name: string): HouseholdMember {
    const maxOrder = state.household.reduce((m, h) => Math.max(m, h.order), 0);
    const member = parseHouseholdMember({ id: crypto.randomUUID(), name, email: "", order: maxOrder + 1 });
    state.household.push(member);
    debouncedSave();
    return member;
  }

  function updateHouseholdMember(id: string, name: string, email: string) {
    const m = state.household.find((h) => h.id === id);
    if (m) { m.name = name; m.email = email; debouncedSave(); }
  }

  function deleteHouseholdMember(id: string): boolean {
    if (state.household.length <= 1) return false;
    if (id === state.household[0]?.id) return false;
    const inUse = state.subscriptions.some((s) => s.payerUserId === id);
    if (inUse) return false;
    state.household = state.household.filter((h) => h.id !== id);
    debouncedSave();
    return true;
  }

  // ---- Payment Methods ----
  function addPaymentMethod(name: string, icon: string): PaymentMethod {
    const maxOrder = state.paymentMethods.reduce((m, p) => Math.max(m, p.order), 0);
    const pm = parsePaymentMethod({ id: crypto.randomUUID(), name, icon, enabled: true, order: maxOrder + 1 });
    state.paymentMethods.push(pm);
    debouncedSave();
    return pm;
  }

  function updatePaymentMethod(id: string, updates: Partial<PaymentMethod>) {
    const pm = state.paymentMethods.find((p) => p.id === id);
    if (pm) { Object.assign(pm, updates); debouncedSave(); }
  }

  function deletePaymentMethod(id: string): boolean {
    const inUse = state.subscriptions.some((s) => s.paymentMethodId === id) || state.expenses.some((e) => e.paymentMethodId === id);
    if (inUse) return false;
    state.paymentMethods = state.paymentMethods.filter((p) => p.id !== id);
    debouncedSave();
    return true;
  }

  function togglePaymentMethod(id: string) {
    const pm = state.paymentMethods.find((p) => p.id === id);
    if (!pm) return;
    const inUse = state.subscriptions.some((s) => s.paymentMethodId === id);
    if (inUse && pm.enabled) return; // Can't disable if in use
    pm.enabled = !pm.enabled;
    debouncedSave();
  }

  function reorderPaymentMethods(ids: string[]) {
    ids.forEach((id, i) => {
      const pm = state.paymentMethods.find((p) => p.id === id);
      if (pm) pm.order = i;
    });
    debouncedSave();
  }

  // ---- Settings ----
  function updateSettings(updates: Partial<Settings>) {
    const merged = parseSettings({ ...state.settings, ...updates });
    Object.assign(state.settings, merged);
    if (updates.language) {
      setLanguage(updates.language);
      retranslateDefaults(updates.language);
    }
    debouncedSave();
  }

  /**
   * Retranslate all default items (categories, currencies, payment methods, tags, household)
   * when the language changes. Only items with an i18nKey are affected.
   */
  function retranslateDefaults(lang: string) {
    // Categories
    for (const cat of state.categories) {
      if (cat.i18nKey) {
        const translated = translate(cat.i18nKey);
        if (translated !== cat.i18nKey) cat.name = translated;
      }
    }

    // Currencies
    for (const cur of state.currencies) {
      if (cur.i18nKey) {
        const translated = translate(cur.i18nKey);
        if (translated !== cur.i18nKey) cur.name = translated;
      }
    }

    // Payment methods
    for (const pm of state.paymentMethods) {
      if (pm.i18nKey) {
        const translated = translate(pm.i18nKey);
        if (translated !== pm.i18nKey) pm.name = translated;
      }
    }

    // Tags
    for (const tag of state.tags) {
      if (tag.i18nKey) {
        const translated = translate(tag.i18nKey);
        if (translated !== tag.i18nKey) {
          const oldName = tag.name;
          const newName = translated;
          if (oldName !== newName) {
            // Rename in all subscriptions/expenses
            for (const sub of state.subscriptions) {
              const idx = sub.tags.indexOf(oldName);
              if (idx !== -1) sub.tags[idx] = newName;
            }
            for (const exp of state.expenses) {
              const idx = exp.tags.indexOf(oldName);
              if (idx !== -1) exp.tags[idx] = newName;
            }
            tag.name = newName;
          }
        }
      }
    }

    debouncedSave();
  }

  function setFixerApiKey(key: string, provider: number) {
    state.fixerApiKey = key;
    state.fixerProvider = provider;
    debouncedSave();
  }

  function setTelegramConfig(botToken: string, chatId: string, enabled: boolean) {
    state.telegramBotToken = botToken;
    state.telegramChatId = chatId;
    state.telegramEnabled = enabled;
    debouncedSave();
  }

  // ---- Notification tracking ----
  function markNotified(subId: string, date: string) {
    const sub = state.subscriptions.find((s) => s.id === subId);
    if (sub) {
      sub.lastNotifiedDate = date;
      debouncedSave();
    }
  }

  function updateCurrencyRates(updates: { id: string; rate: number }[]) {
    for (const u of updates) {
      const cur = state.currencies.find((c) => c.id === u.id);
      if (cur) cur.rate = u.rate;
    }
    debouncedSave();
  }

  // ---- Tags ----
  function addTag(name: string): Tag {
    const n = name.trim();
    const maxOrder = state.tags.reduce((m, t) => Math.max(m, t.order), 0);
    const tag = parseTag({ id: crypto.randomUUID(), name: n, order: maxOrder + 1, favorite: true });
    state.tags.push(tag);
    debouncedSave();
    return tag;
  }

  function updateTag(id: string, updates: Partial<Tag>) {
    const tag = state.tags.find((t) => t.id === id);
    if (tag) {
      // If name changed, rename in subscriptions/expenses
      if (updates.name && updates.name !== tag.name) {
        const oldName = tag.name;
        const newName = updates.name;
        for (const sub of state.subscriptions) {
          const si = sub.tags.indexOf(oldName);
          if (si !== -1) sub.tags[si] = newName;
        }
        for (const exp of state.expenses) {
          const ei = exp.tags.indexOf(oldName);
          if (ei !== -1) exp.tags[ei] = newName;
        }
      }
      Object.assign(tag, updates);
      debouncedSave();
    }
  }

  function deleteTag(id: string) {
    const tag = state.tags.find((t) => t.id === id);
    if (!tag) return;
    const name = tag.name;
    state.tags = state.tags.filter((t) => t.id !== id);
    // Remove from all subscriptions and expenses
    for (const sub of state.subscriptions) {
      sub.tags = sub.tags.filter((t) => t !== name);
    }
    for (const exp of state.expenses) {
      exp.tags = exp.tags.filter((t) => t !== name);
    }
    debouncedSave();
  }

  function reorderTags(ids: string[]) {
    ids.forEach((id, i) => {
      const tag = state.tags.find((t) => t.id === id);
      if (tag) tag.order = i;
    });
    debouncedSave();
  }

  function toggleTagFavorite(id: string) {
    const tag = state.tags.find((t) => t.id === id);
    if (tag) { tag.favorite = !tag.favorite; debouncedSave(); }
  }

  /** Ensure a tag name exists in global tags; return the name */
  function ensureTag(name: string): string {
    const n = name.trim();
    if (!n) return "";
    const exists = state.tags.find((t) => t.name === n);
    if (!exists) addTag(n);
    return n;
  }

  // ---- Favorite ----
  function toggleFavorite(id: string) {
    const sub = state.subscriptions.find((s) => s.id === id);
    if (sub) {
      sub.favorite = !sub.favorite;
      debouncedSave();
    }
  }

  // ---- Batch actions ----
  function batchDelete(ids: string[]) {
    state.subscriptions = state.subscriptions.filter((s) => !ids.includes(s.id));
    state.subscriptions.forEach((s) => {
      if (s.replacementSubscriptionId && ids.includes(s.replacementSubscriptionId)) {
        s.replacementSubscriptionId = null;
      }
    });
    debouncedSave();
  }

  function batchSetInactive(ids: string[], inactive: boolean) {
    for (const sub of state.subscriptions) {
      if (ids.includes(sub.id)) sub.inactive = inactive;
    }
    debouncedSave();
  }

  function batchSetCategory(ids: string[], categoryId: string) {
    for (const sub of state.subscriptions) {
      if (ids.includes(sub.id)) sub.categoryId = categoryId;
    }
    debouncedSave();
  }

  function batchSetTags(ids: string[], tags: string[]) {
    for (const sub of state.subscriptions) {
      if (ids.includes(sub.id)) {
        // Merge tags (add new, keep existing)
        const merged = [...new Set([...sub.tags, ...tags])];
        sub.tags = merged;
      }
    }
    debouncedSave();
  }

  function importData(data: AppData) {
    Object.assign(state, data);
    setLanguage(state.settings.language);
    debouncedSave();
  }

  function getExportData(): AppData {
    return { ...state };
  }

  // ---- Computed helpers ----
  const activeSubscriptions = computed(() =>
    state.subscriptions.filter((s) => !s.inactive)
  );

  const inactiveSubscriptions = computed(() =>
    state.subscriptions.filter((s) => s.inactive)
  );

  const mainCurrency = computed(() =>
    state.currencies.find((c) => c.id === state.settings.mainCurrencyId) || state.currencies[0]
  );

  const enabledPaymentMethods = computed(() =>
    state.paymentMethods
      .filter((p) => p.enabled)
      .sort((a, b) => a.order - b.order)
  );

  const sortedCategories = computed(() =>
    [...state.categories].sort((a, b) => a.order - b.order)
  );

  const sortedTags = computed(() =>
    [...state.tags].sort((a, b) => a.order - b.order)
  );

  const favoriteTags = computed(() =>
    sortedTags.value.filter((t) => t.favorite)
  );

  return {
    state,
    isLoading,
    init,
    // Subscriptions
    addSubscription,
    updateSubscription,
    deleteSubscription,
    cloneSubscription,
    renewSubscription,
    // Payment history
    recordPayment,
    addPaymentRecord,
    deletePaymentRecord,
    // Expenses
    addExpense,
    updateExpense,
    deleteExpense,
    batchDeleteExpenses,
    // Categories
    addCategory,
    updateCategory,
    deleteCategory,
    reorderCategories,
    // Currencies
    addCurrency,
    updateCurrency,
    deleteCurrency,
    reorderCurrencies,
    // Household
    addHouseholdMember,
    updateHouseholdMember,
    deleteHouseholdMember,
    // Payment Methods
    addPaymentMethod,
    updatePaymentMethod,
    deletePaymentMethod,
    togglePaymentMethod,
    reorderPaymentMethods,
    // Settings
    updateSettings,
    setFixerApiKey,
    setTelegramConfig,
    // Notification tracking
    markNotified,
    updateCurrencyRates,
    // Tags
    addTag,
    updateTag,
    deleteTag,
    reorderTags,
    toggleTagFavorite,
    ensureTag,
    // Favorite
    toggleFavorite,
    // Batch
    batchDelete,
    batchSetInactive,
    batchSetCategory,
    batchSetTags,
    // Import/Export
    importData,
    getExportData,
    // Computed
    activeSubscriptions,
    inactiveSubscriptions,
    mainCurrency,
    enabledPaymentMethods,
    sortedCategories,
    sortedTags,
    favoriteTags,
  };
}
