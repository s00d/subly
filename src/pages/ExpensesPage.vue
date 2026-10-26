<script setup lang="ts">
import AppCheckbox from "@/components/ui/AppCheckbox.vue";
import UiViewSwitch from "@/components/ui/UiViewSwitch.vue";
import ConfirmDialog from "@/components/ui/ConfirmDialog.vue";
import AppDatePicker from "@/components/ui/AppDatePicker.vue";
import UiEmptyState from "@/components/ui/UiEmptyState.vue";
import { localYmd } from "@/composables/useLocaleFormat";
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import {
  deleteExpense,
  deleteExpensesBatch,
  getExpenseCount,
  insertExpense,
  type ExpenseFilter,
} from "@/services/expensesClient";
import { storeToRefs } from "pinia";
import type { Currency, Expense, Subscription, Category, PaymentMethod, Tag, Settings, HouseholdMember } from "@/schemas/appData";
import { expenseToIsoDate, parseExpense } from "@/schemas/appData";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { useHeaderActions, type HeaderAction } from "@/composables/useHeaderActions";
import { useToast } from "@/composables/useToast";
import { useClipboard } from "@/composables/useClipboard";
import { useScrollLock } from "@/composables/useScrollLock";
import ExpenseForm from "@/components/expenses/ExpenseForm.vue";
import ExpenseDetail from "@/components/expenses/ExpenseDetail.vue";
import AiSmartDialog from "@/components/ai/AiSmartDialog.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import UniversalListRow from "@/components/ui/UniversalListRow.vue";
import ContextActionMenu, {
  type ContextMenuRow,
  type ContextMenuExcludeRect,
} from "@/components/ui/ContextActionMenu.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import Toast from "@/components/ui/Toast.vue";
import {
  convertAmountToCurrency,
  convertToMain,
  moneyFromDecimal,
  moneyGt,
  sumMoney,
  type MoneyString,
} from "@/lib/money";
import { Plus, Search, Pencil, Trash2, ExternalLink, CopyPlus, ClipboardCopy, CheckSquare, Square, Wallet, ChevronLeft, ChevronRight, Filter, Sparkles } from "@lucide/vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { formatErrorForToast } from "@/utils/formatError";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { useSubscriptionsStore } from "@/stores/subscriptionsStore";
import { useExpensesStore } from "@/stores/expensesStore";
import { ui, avatarTone, button } from "@/lib/tv";
import { useAiConfigStore } from "@/stores/aiConfigStore";

const PAGE_SIZE = 10;
const metaStore = useAppMetaStore();
const subscriptionsStore = useSubscriptionsStore();
const expensesStore = useExpensesStore();
const metaRefs = storeToRefs(metaStore);
// Live views of the stores (sync pulls / AI imports / catalog edits show up without a remount).
const subscriptions = computed<Subscription[]>(() => subscriptionsStore.items);
const categories = computed<Category[]>(() => metaRefs.categories.value);
const paymentMethods = computed<PaymentMethod[]>(() => metaRefs.paymentMethods.value);
const currencies = computed<Currency[]>(() => metaRefs.currencies.value);
const tags = computed<Tag[]>(() => metaRefs.tags.value);
const household = computed<HouseholdMember[]>(() => metaRefs.household.value);
const settings = computed<Settings | null>(() => metaRefs.settings.value ?? null);
const expensesRefs = storeToRefs(expensesStore);
const items = expensesRefs.items;
const totalCount = expensesRefs.total;
const currentPage = expensesRefs.page;
const loading = expensesRefs.loading;
const activeFilter = expensesRefs.currentFilter;
const totalFiltered = expensesRefs.totalFiltered;
const { t } = useI18n();
const { setActions, clearActions } = useHeaderActions();
const { fmtDateMedium, fmtCurrency } = useLocaleFormat();
const { fmt } = useCurrencyFormat();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();
const { copyToClipboard } = useClipboard();
const pageLogPrefix = "[ExpensesPage]";
const showFilters = ref(false);

function logPageError(scope: string, error: unknown, extra?: Record<string, unknown>) {
  console.error(`${pageLogPrefix} ${scope}`, {
    error,
    ...extra,
  });
}

// AI smart-input state — must be declared before updateHeaderActions so the
// action builder can read the availability flag without forward refs.
const showAiSmart = ref(false);
const aiConfigStore = useAiConfigStore();
const {
  enabled: aiEnabled,
  features: aiFeatures,
  hasApiKey: aiHasApiKey,
  activeProvider: aiActiveProvider,
  loaded: aiLoaded,
} = storeToRefs(aiConfigStore);
/** Sparkles when config is loaded and any expense AI feature is on. Keyring read only on click. */
const showAiHeaderButton = computed(() => {
  if (!aiLoaded.value) return false;
  return (
    !!aiFeatures.value.expenseInput ||
    !!aiFeatures.value.receiptImport ||
    !!aiFeatures.value.statementImport
  );
});

const aiRouter = useRouter();
function openAiSettings() {
  aiRouter.push("/settings?section=ai");
}

function openAiSmart() {
  showAiSmart.value = true;
}

async function onAiAssistantClick() {
  if (!aiEnabled.value) {
    openAiSettings();
    return;
  }
  const requiresKey = !!aiActiveProvider.value?.requiresKey;
  if (requiresKey) {
    await aiConfigStore.refreshHasApiKey();
  }
  const keyOk = !requiresKey || aiHasApiKey.value;
  if (keyOk) {
    openAiSmart();
  } else {
    openAiSettings();
  }
}

function onAiImported() {
  fetchPage(currentPage.value, activeFilter.value).catch((e) => {
    logPageError("onAiImported reload failed", e);
  });
  updateTotal();
}

function updateHeaderActions() {
  const actions: HeaderAction[] = [];
  if (showAiHeaderButton.value) {
    actions.push({
      id: "ai-smart-expense",
      icon: Sparkles,
      title: t("ai_smart_open"),
      onClick: () => {
        void onAiAssistantClick();
      },
      style: "accent",
    });
  }
  actions.push({ id: "add-expense", icon: Plus, title: t("add_expense"), label: t("add"), onClick: openAdd, style: "primary" });
  setActions(actions);
}

onMounted(() => {
  updateHeaderActions();
  loadInitial().then(() => applyFilters());
  aiConfigStore.load().then(() => updateHeaderActions());
});
watch(showAiHeaderButton, () => updateHeaderActions());
onUnmounted(() => clearActions());

// ---- View mode ----
const viewMode = computed(() => settings.value?.expenseViewMode || "default");

/** Non-default filters in effect (drives the badge on the Filter button and "no results" state). */
const activeFilterCount = computed(() => (filterCategory.value ? 1 : 0) + (filterPayment.value ? 1 : 0) + (filterTag.value ? 1 : 0) + (dateFrom.value ? 1 : 0) + (dateTo.value ? 1 : 0) + (searchQuery.value.trim() ? 1 : 0));

function resetFilters() {
  searchQuery.value = "";
  filterCategory.value = "";
  filterPayment.value = "";
  filterTag.value = "";
  dateFrom.value = "";
  dateTo.value = "";
}

function setViewMode(mode: "default" | "compact" | "expanded") {
  if (!settings.value) return;
  metaStore.updateSettings({ expenseViewMode: mode });
}
watch([showFilters, viewMode], updateHeaderActions);

// ---- Filters ----
const searchQuery = ref("");
const filterCategory = ref("");
const filterPayment = ref("");
const filterTag = ref("");
const dateFrom = ref("");
const dateTo = ref("");
const sortBy = ref<string>("date_desc");

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

function buildFilter(): ExpenseFilter {
  return {
    search: searchQuery.value || undefined,
    categoryId: filterCategory.value || undefined,
    paymentMethodId: filterPayment.value || undefined,
    tag: filterTag.value || undefined,
    dateFrom: dateFrom.value || undefined,
    dateTo: dateTo.value || undefined,
    sortBy: sortBy.value as ExpenseFilter["sortBy"],
  };
}

function applyFilters() {
  selectedIds.value = new Set();
  const filter = buildFilter();
  fetchPage(1, filter).catch((e) => {
    logPageError("fetchPage failed", e, { filter, page: 1 });
    toast(formatErrorForToast(e, t), "error");
  });
  updateTotal();
}

function applyFiltersDebounced() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(applyFilters, 300);
}

watch([filterCategory, filterPayment, filterTag, dateFrom, dateTo, sortBy], applyFilters);
watch(searchQuery, applyFiltersDebounced);

// ---- Pagination ----
const totalPages = computed(() => Math.max(1, Math.ceil(totalCount.value / PAGE_SIZE)));

function goPage(p: number) {
  if (p < 1 || p > totalPages.value) return;
  selectedIds.value = new Set();
  fetchPage(p).catch((e) => {
    logPageError("goPage failed", e, { page: p });
    toast(formatErrorForToast(e, t), "error");
  });
}

// ---- Summary ----
async function updateTotal() {
  try {
    await expensesStore.refreshSummary(buildFilter());
  } catch (e) {
    logPageError("updateTotal failed", e, { filter: buildFilter() });
    toast(formatErrorForToast(e, t), "error");
  }
}

// ---- Form ----
const showForm = ref(false);
const editingExpense = ref<Expense | null>(null);

// ---- Detail panel ----
const showDetail = ref(false);
const detailExpId = ref<string | null>(null);
const detailExp = computed(() =>
  detailExpId.value ? items.value.find((e) => e.id === detailExpId.value) ?? null : null,
);

// ---- Selection for batch ----
const selectionMode = ref(false);
const selectedIds = ref<Set<string>>(new Set());
watch(selectionMode, updateHeaderActions);

function toggleSelect(id: string) {
  const s = new Set(selectedIds.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  selectedIds.value = s;
}

function selectAll() {
  selectedIds.value = new Set(items.value.map((e) => e.id));
}

function deselectAll() {
  selectedIds.value = new Set();
}

function toggleSelectionMode() {
  selectionMode.value = !selectionMode.value;
  if (!selectionMode.value) selectedIds.value = new Set();
}

// ---- Delete confirmation ----
const deleteConfirmId = ref<string | null>(null);
const batchDeleteConfirmIds = ref<string[]>([]);
const hasBlockingOverlay = computed(() => Boolean(deleteConfirmId.value) || batchDeleteConfirmIds.value.length > 0);
useScrollLock(hasBlockingOverlay);

function requestDelete(id: string) {
  showDetail.value = false;
  detailExpId.value = null;
  deleteConfirmId.value = id;
}

async function confirmDelete() {
  try {
    if (deleteConfirmId.value) {
      await deleteExpense(deleteConfirmId.value);
      await fetchPage(currentPage.value, activeFilter.value);
      toast(t("expense_deleted"));
      await updateTotal();
    }
  } catch (e) {
    logPageError("confirmDelete failed", e, { expenseId: deleteConfirmId.value });
    toast(formatErrorForToast(e, t), "error");
  } finally {
    deleteConfirmId.value = null;
  }
}

function cancelDelete() { deleteConfirmId.value = null; }

function batchDeleteSelected() {
  batchDeleteConfirmIds.value = [...selectedIds.value];
}

async function confirmBatchDelete() {
  try {
    if (batchDeleteConfirmIds.value.length > 0) {
      await deleteExpensesBatch(batchDeleteConfirmIds.value);
      await fetchPage(currentPage.value, activeFilter.value);
      toast(t("batch_deleted_expenses", { count: batchDeleteConfirmIds.value.length }));
      selectedIds.value = new Set();
      selectionMode.value = false;
      await updateTotal();
    }
  } catch (e) {
    logPageError("confirmBatchDelete failed", e, { ids: [...batchDeleteConfirmIds.value] });
    toast(formatErrorForToast(e, t), "error");
  } finally {
    batchDeleteConfirmIds.value = [];
  }
}

// ---- Computed options ----
const categoryOptions = computed<SelectOption[]>(() => [
  { label: t("category"), value: "" },
  ...[...categories.value].sort((a, b) => a.sortOrder - b.sortOrder).map((c) => ({ label: c.name, value: c.id, icon: c.icon || undefined })),
]);

const paymentOptions = computed<SelectOption[]>(() => [
  { label: t("payment_method"), value: "" },
  ...paymentMethods.value.filter((p) => p.enabled).map((p) => ({ label: p.name, value: p.id, icon: p.icon })),
]);

const tagOptions = computed<SelectOption[]>(() => [
  { label: t("filter_by_tag"), value: "" },
  ...tags.value.map((tg) => ({ label: tg.name, value: tg.name })),
]);

const sortOptions = computed<SelectOption[]>(() => [
  { label: `${t("expense_date")} ↓`, value: "date_desc" },
  { label: `${t("expense_date")} ↑`, value: "date_asc" },
  { label: `${t("expense_amount")} ↓`, value: "amount_desc" },
  { label: `${t("expense_amount")} ↑`, value: "amount_asc" },
]);

// ---- Helpers ----
function getCurrency(id: string) { return currencies.value.find((c) => c.id === id); }
function getCategory(id: string) { return categories.value.find((c) => c.id === id); }
function getPaymentMethod(id: string) { return paymentMethods.value.find((p) => p.id === id); }
function getSubscription(id: string) { return subscriptions.value.find((s) => s.id === id); }
function getExpenseIcon(exp: Expense): string {
  return (exp.subscriptionId ? getSubscription(exp.subscriptionId)?.logo : "")
    || getCategory(exp.categoryId)?.icon
    || "";
}
function formatDate(d: string) { return fmtDateMedium(d); }
function formatExpenseDate(e: Expense) { return fmtDateMedium(expenseToIsoDate(e)); }

/** Default mode groups rows under day headers (Today / Yesterday / date); compact and grid stay flat. */
const dayGroups = computed(() => {
  if (viewMode.value !== "default") return [{ key: "all", label: "", items: items.value }];
  const today = new Date();
  const yesterday = new Date(today.getFullYear(), today.getMonth(), today.getDate() - 1);
  const groups: { key: string; label: string; items: Expense[]; total?: MoneyString }[] = [];
  for (const exp of items.value) {
    const key = expenseToIsoDate(exp);
    let g = groups[groups.length - 1];
    if (!g || g.key !== key) {
      const label =
        key === localYmd(today) ? t("today") : key === localYmd(yesterday) ? t("yesterday") : fmtDateMedium(key);
      g = { key, label, items: [] };
      groups.push(g);
    }
    g.items.push(exp);
  }
  const rates = ratesRecord();
  for (const g of groups) {
    g.total = moneyFromDecimal(
      sumMoney(g.items.map((e) => moneyFromDecimal(convertToMain(e.amount, e.currencyId, rates)))),
    );
  }
  return groups;
});
function formatAmount(amount: MoneyString, currencyId: string) {
  return fmt(amount, currencyId);
}

const targetCurrencies = computed(() => {
  const mainId = settings.value?.mainCurrencyId;
  const targets = settings.value?.currencyUpdateTargets ?? [];
  const ids = new Set(targets);
  if (mainId) ids.add(mainId);
  return [...ids]
    .map((id) => currencies.value.find((c) => c.id === id))
    .filter((c): c is Currency => !!c && moneyGt(c.rate, "0"));
});
const expenseLookupData = computed(() => {
  if (!settings.value) return null;
  return {
    categories: categories.value,
    paymentMethods: paymentMethods.value,
    household: household.value,
    currencies: currencies.value,
    settings: settings.value,
  };
});
const expenseFormLookupData = computed(() => {
  if (!settings.value) return null;
  return {
    settings: settings.value,
    currencies: currencies.value,
    paymentMethods: paymentMethods.value,
    household: household.value,
    categories: categories.value,
    tags: tags.value,
    expensesCount: allExpensesCount.value,
  };
});
const mainCurrencyCode = computed(() => {
  const mainId = settings.value?.mainCurrencyId;
  return currencies.value.find((c) => c.id === mainId)?.code || "USD";
});

function ratesRecord(): Record<string, MoneyString> {
  const m: Record<string, MoneyString> = {};
  for (const c of currencies.value) m[c.id] = c.rate;
  return m;
}

function convertAmount(amount: MoneyString, fromCurId: string, toCurrency: Currency): MoneyString {
  return convertAmountToCurrency(amount, fromCurId, toCurrency, ratesRecord()).toFixed();
}

function fmtInCurrency(amount: MoneyString, currency: Currency): string {
  return fmt(amount, currency.id);
}

function getConvertedAmounts(amount: MoneyString, currencyId: string) {
  return targetCurrencies.value
    .filter((tc) => tc.id !== currencyId)
    .map((tc) => ({
      currency: tc,
      amount: convertAmount(amount, currencyId, tc),
    }));
}

// ---- Actions ----
function openAdd() { editingExpense.value = null; showForm.value = true; }

function openDetail(exp: Expense) {
  detailExpId.value = exp.id;
  showDetail.value = true;
}

function openEdit(exp: Expense) {
  showDetail.value = false;
  detailExpId.value = null;
  editingExpense.value = exp;
  showForm.value = true;
}

async function onSaved() {
  try {
    toast(editingExpense.value ? t("expense_updated") : t("expense_added"));
    await fetchPage(currentPage.value, activeFilter.value);
    await updateTotal();
  } catch (e) {
    logPageError("onSaved failed", e);
    toast(formatErrorForToast(e, t), "error");
  }
}

function onDetailEdit(exp: Expense) { openEdit(exp); }
function onDetailDelete(id: string) { requestDelete(id); }

async function handleOpenUrl(url: string) {
  const fullUrl = url.startsWith("http") ? url : `https://${url}`;
  try {
    await openUrl(fullUrl);
  } catch (e) {
    logPageError("handleOpenUrl failed", e, { url: fullUrl });
    toast(formatErrorForToast(e, t), "error");
  }
}

async function handleDuplicateExpense(exp: Expense) {
  showDetail.value = false;
  detailExpId.value = null;
  const { updatedAt: _omitUpdated, ...rest } = exp;
  const cloned = parseExpense({
    ...rest,
    id: crypto.randomUUID(),
    name: `${exp.name} (copy)`,
    // Today's local date at noon UTC (as ExpenseForm stores it); a raw UTC "now" can land on another day.
    createdAt: `${localYmd()}T12:00:00.000Z`,
    subscriptionId: "",
    paymentRecordId: "",
  });
  try {
    await insertExpense(cloned);
    toast(t("expense_added"));
    await fetchPage(currentPage.value, activeFilter.value);
    await updateTotal();
  } catch (e) {
    logPageError("handleDuplicateExpense failed", e, { id: exp.id });
    toast(formatErrorForToast(e, t), "error");
  }
}

async function handleCopyExpenseName(exp: Expense) {
  const copied = await copyToClipboard(exp.name);
  if (copied) toast(t("copied_to_clipboard"));
  else toast(t("clipboard_copy_failed"), "error");
}

function onDetailOpenUrl(url: string) { handleOpenUrl(url); }

const contextMenuOpen = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const contextMenuRows = ref<ContextMenuRow[]>([]);
const contextMenuExcludeRect = ref<ContextMenuExcludeRect | null>(null);

watch(contextMenuOpen, (open) => {
  if (!open) contextMenuExcludeRect.value = null;
});

function showContextMenu(exp: Expense, event: MouseEvent, anchorRect?: DOMRect | null) {
  if (selectionMode.value) return;

  const vx = event.clientX || window.innerWidth / 2;
  const vy = event.clientY || window.innerHeight / 3;
  contextMenuX.value = vx;
  contextMenuY.value = vy;
  contextMenuExcludeRect.value =
    anchorRect != null
      ? {
          left: anchorRect.left,
          top: anchorRect.top,
          right: anchorRect.right,
          bottom: anchorRect.bottom,
        }
      : null;

  const rows: ContextMenuRow[] = [
    {
      kind: "button",
      label: t("edit_expense"),
      icon: Pencil,
      run: () => {
        openEdit(exp);
      },
    },
    {
      kind: "button",
      label: t("clone"),
      icon: CopyPlus,
      run: () => {
        void handleDuplicateExpense(exp);
      },
    },
  ];
  if (exp.url) {
    rows.push({
      kind: "button",
      label: t("url"),
      icon: ExternalLink,
      run: () => {
        handleOpenUrl(exp.url);
      },
    });
  }
  rows.push(
    {
      kind: "button",
      label: t("copy"),
      icon: ClipboardCopy,
      run: () => {
        void handleCopyExpenseName(exp);
      },
    },
    { kind: "separator" },
    {
      kind: "button",
      label: t("delete"),
      danger: true,
      icon: Trash2,
      run: () => {
        requestDelete(exp.id);
      },
    },
  );

  contextMenuRows.value = rows;
  contextMenuOpen.value = true;
}

/** Unfiltered count: seeds the "Expense #N" default name (the filtered total gave duplicates). */
const allExpensesCount = ref(0);
watch(totalCount, () => {
  getExpenseCount().then((n) => (allExpensesCount.value = n)).catch(() => {});
}, { immediate: true });

async function loadInitial() {
  await metaStore.ensureLoaded();
  await subscriptionsStore.loadBrief({});
}

async function fetchPage(page?: number, newFilter?: ExpenseFilter) {
  if (newFilter !== undefined) activeFilter.value = newFilter;
  if (page !== undefined) currentPage.value = page;
  expensesStore.pageSize = PAGE_SIZE;
  await expensesStore.loadPage(currentPage.value, activeFilter.value);
}
</script>

<template>
  <div class="max-w-5xl mx-auto">
    <!-- Filters -->
    <!-- List toolbar: search always visible; filter / select / layout as labelled controls -->
    <div class="mb-3 flex items-center gap-2">
      <div class="relative min-w-0 flex-1">
        <Search :size="14" class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
        <input v-model="searchQuery" type="search" :placeholder="t('search')" :class="[ui.field(), 'pl-8 text-[13px]']" />
      </div>
      <button
        type="button"
        :class="button({ pressed: showFilters })"
        :aria-pressed="showFilters"
        @click="showFilters = !showFilters"
      >
        <Filter :size="14" />
        <span class="max-sm:hidden">{{ t('filter') }}</span>
        <span
          v-if="activeFilterCount > 0"
          class="inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 text-[10px] font-semibold text-white tabular-nums"
        >{{ activeFilterCount }}</span>
      </button>
      <button
        type="button"
        :class="button({ pressed: selectionMode })"
        :aria-pressed="selectionMode"
        @click="toggleSelectionMode"
      >
        <CheckSquare :size="14" />
        <span class="max-sm:hidden">{{ t('select') }}</span>
      </button>
      <UiViewSwitch :model-value="viewMode" @update:model-value="setViewMode" />
    </div>

    <Transition enter-active-class="transition ease-out duration-200" enter-from-class="opacity-0 -translate-y-2" enter-to-class="opacity-100 translate-y-0" leave-active-class="transition ease-in duration-150" leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 -translate-y-2">
      <div v-if="showFilters" class="mb-3 p-3 rounded-xl border border-border bg-surface shadow-sm space-y-2">
        <div v-if="activeFilterCount > 0" class="flex justify-end -mb-1">
          <button type="button" class="text-xs font-medium text-primary hover:text-primary-hover" @click="resetFilters">{{ t('reset_filters') }}</button>
        </div>
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-2">
          <div class="min-w-0"><AppSelect v-model="sortBy" :options="sortOptions" /></div>
          <div class="min-w-0"><AppSelect v-model="filterCategory" :options="categoryOptions" /></div>
          <div class="min-w-0"><AppSelect v-model="filterPayment" :options="paymentOptions" /></div>
          <div v-if="tags.length > 0" class="min-w-0"><AppSelect v-model="filterTag" :options="tagOptions" /></div>
          <div class="min-w-0"><AppDatePicker v-model="dateFrom" :max="dateTo || undefined" /></div>
          <div class="min-w-0"><AppDatePicker v-model="dateTo" :min="dateFrom || undefined" /></div>
        </div>
      </div>
    </Transition>

    <!-- Batch toolbar -->
    <Transition enter-active-class="transition ease-out duration-200" enter-from-class="opacity-0 -translate-y-2" enter-to-class="opacity-100 translate-y-0" leave-active-class="transition ease-in duration-150" leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 -translate-y-2">
      <div v-if="selectionMode" class="sticky top-2 z-10 flex items-center gap-1.5 mb-3 px-3 py-2 rounded-xl border border-border-strong bg-surface shadow-md overflow-x-auto">
        <span class="text-xs font-semibold text-text-primary tabular-nums whitespace-nowrap">{{ selectedIds.size }} {{ t('selected_count') }}</span>
        <button :class="button({ intent: 'ghost', size: 'iconSm' })" @click="selectAll" :title="t('select_all')">
          <CheckSquare :size="13" />
        </button>
        <button :class="button({ intent: 'ghost', size: 'iconSm' })" @click="deselectAll" :title="t('deselect_all')">
          <Square :size="13" />
        </button>
        <div class="flex-1" />
        <button @click="batchDeleteSelected" :disabled="selectedIds.size === 0"
          :class="button({ size: 'sm' })"
        >{{ t('batch_delete') }}</button>
      </div>
    </Transition>

    <!-- Summary bar -->
    <div class="flex items-end justify-between px-1 pb-3">
      <div class="flex items-center gap-1.5 text-xs text-text-muted">
        <Wallet :size="14" />
        <span>{{ totalCount }} {{ t('expenses').toLowerCase() }}</span>
      </div>
      <div class="text-xl font-semibold text-text-primary tabular-nums tracking-[-0.02em]">
        {{ fmtCurrency(totalFiltered, mainCurrencyCode) }}
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="text-center py-16">
      <div class="w-8 h-8 mx-auto border-2 border-primary border-t-transparent rounded-full animate-spin" />
    </div>

    <!-- Empty state -->
    <div v-else-if="items.length === 0" :class="ui.panel()">
      <UiEmptyState v-if="activeFilterCount > 0" :icon="Search" :title="t('nothing_found')">
        <button :class="button()" @click="resetFilters">{{ t('reset_filters') }}</button>
      </UiEmptyState>
      <UiEmptyState v-else :icon="Wallet" :title="t('no_expenses_yet')">
        <button :class="button({ intent: 'primary' })" @click="openAdd">
          <Plus :size="15" /> {{ t('add_expense') }}
        </button>
      </UiEmptyState>
    </div>

    <!-- Expense list -->
    <div
      v-if="!loading && items.length > 0"
      :class="viewMode === 'expanded' ? 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3' : `${ui.listContainer()} shadow-sm`"
    >
    <template v-for="group in dayGroups" :key="group.key">
      <!-- Day header as a separator row inside the one list (default mode only) -->
      <div
        v-if="group.label"
        class="sticky top-0 z-[1] flex items-center justify-between gap-3 px-4 py-1.5 bg-surface-secondary/95 backdrop-blur border-b border-border text-[11px] font-semibold uppercase tracking-wide text-text-secondary"
      >
        <span>{{ group.label }}</span>
        <span class="normal-case tracking-normal tabular-nums text-text-muted">{{ fmt(group.total ?? "0", settings?.mainCurrencyId ?? "") }}</span>
      </div>
      <div
        v-for="exp in group.items"
        :key="exp.id"
        class="overflow-hidden transition-colors cursor-pointer"
        :class="[
 viewMode === 'expanded' ? 'bg-surface rounded-xl border shadow-sm hover:shadow-md' : '',
          viewMode === 'expanded' && !selectedIds.has(exp.id) ? 'border-border' : '',
          selectedIds.has(exp.id) ? (viewMode === 'expanded' ? 'border-primary ring-1 ring-primary/30' : 'bg-primary-light') : '',
        ]"
      >
        <UniversalListRow
          :mode="viewMode as 'compact' | 'default' | 'expanded'"
          @click="selectionMode ? toggleSelect(exp.id) : openDetail(exp)"
          @contextmenu="(e, rect) => showContextMenu(exp, e, rect)"
        >
          <template #selection>
            <div
              v-if="selectionMode && viewMode !== 'expanded'"
              class="shrink-0"
              @click.stop
              >
                <AppCheckbox :model-value="selectedIds.has(exp.id)" :aria-label="t('select')" @update:model-value="toggleSelect(exp.id)" />
            </div>
          </template>
          <template #leading>
            <div
              v-if="viewMode !== 'expanded'"
              class="flex items-center justify-center font-semibold shrink-0 overflow-hidden"
              :class="[
 viewMode === 'compact' ? 'w-6 h-6 rounded-md text-[10px]' : 'w-9 h-9 rounded-[10px] text-sm',
                getExpenseIcon(exp) ? 'bg-surface-hover' : avatarTone(exp.name),
              ]"
            >
              <IconDisplay v-if="getExpenseIcon(exp)" :icon="getExpenseIcon(exp)" :size="viewMode === 'compact' ? 12 : 18" />
              <span v-else>{{ exp.name.charAt(0).toUpperCase() }}</span>
            </div>
          </template>
          <template #main>
            <p v-if="viewMode === 'compact'" class="text-xs font-medium text-text-primary truncate">{{ exp.name }}</p>
            <div v-else-if="viewMode === 'default'" class="min-w-0">
              <p class="text-sm font-semibold text-text-primary truncate">{{ exp.name }}</p>
              <div class="flex items-center gap-1.5 flex-wrap">
                <span class="text-xs text-text-muted">{{ getCategory(exp.categoryId)?.name || '' }}</span>
                <span v-for="tag in (exp.tags || []).slice(0, 3)" :key="tag" class="hidden sm:inline-flex items-center px-1.5 py-0 rounded text-[9px] font-medium bg-surface-hover text-text-muted border border-border">#{{ tag }}</span>
              </div>
            </div>
          </template>
          <template #meta>
            <span v-if="viewMode === 'compact'" class="text-[11px] text-text-muted shrink-0 w-[92px] text-right tabular-nums">{{ formatExpenseDate(exp) }}</span>


          </template>
          <template #value>
            <p v-if="viewMode === 'compact'" class="text-xs font-semibold text-text-primary shrink-0 w-[108px] text-right tabular-nums">{{ formatAmount(exp.amount, exp.currencyId) }}</p>
            <div v-else-if="viewMode === 'default'" class="text-right shrink-0 w-[112px] sm:w-[132px]">
              <p class="text-[15px] font-semibold text-text-primary tabular-nums tracking-[-0.01em]">{{ formatAmount(exp.amount, exp.currencyId) }}</p>
              <div v-if="getConvertedAmounts(exp.amount, exp.currencyId).length > 0" class="mt-0.5 space-y-0">
                <p v-for="cv in getConvertedAmounts(exp.amount, exp.currencyId)" :key="cv.currency.id"
                  class="text-[10px] text-text-muted tabular-nums">≈ {{ fmtInCurrency(cv.amount, cv.currency) }}</p>
              </div>
            </div>
          </template>
          <template #trailing>
            <div v-if="viewMode === 'default'" class="shrink-0 hidden sm:block" :title="getPaymentMethod(exp.paymentMethodId)?.name">
              <IconDisplay v-if="getPaymentMethod(exp.paymentMethodId)" :icon="getPaymentMethod(exp.paymentMethodId)!.icon" :size="22" />
            </div>
          </template>
          <template #expanded>
            <div class="flex items-start gap-3 mb-3">
              <div v-if="selectionMode" class="shrink-0 mt-1" @click.stop>
                <AppCheckbox :model-value="selectedIds.has(exp.id)" :aria-label="t('select')" @update:model-value="toggleSelect(exp.id)" />
              </div>
              <div class="w-11 h-11 rounded-xl flex items-center justify-center text-base font-semibold shrink-0 overflow-hidden" :class="getExpenseIcon(exp) ? 'bg-surface-hover' : avatarTone(exp.name)">
                <IconDisplay v-if="getExpenseIcon(exp)" :icon="getExpenseIcon(exp)" :size="22" />
                <span v-else>{{ exp.name.charAt(0).toUpperCase() }}</span>
              </div>
              <div class="min-w-0 flex-1">
                <p class="text-sm font-semibold text-text-primary truncate">{{ exp.name }}</p>
                <p class="text-xs text-text-muted flex items-center gap-1">
                  <IconDisplay v-if="getExpenseIcon(exp)" :icon="getExpenseIcon(exp)" :size="12" />
                  {{ getCategory(exp.categoryId)?.name || '' }}
                </p>
              </div>
            </div>
            <div class="flex items-end justify-between">
              <div>
                <p class="text-lg font-semibold text-text-primary">{{ formatAmount(exp.amount, exp.currencyId) }}</p>
                <div v-if="getConvertedAmounts(exp.amount, exp.currencyId).length > 0" class="mt-0.5 space-y-0">
                  <p v-for="cv in getConvertedAmounts(exp.amount, exp.currencyId)" :key="cv.currency.id"
                    class="text-[10px] text-text-muted tabular-nums">≈ {{ fmtInCurrency(cv.amount, cv.currency) }}</p>
                </div>
              </div>
              <div class="text-right">
                <p class="text-xs font-medium text-text-primary">{{ formatExpenseDate(exp) }}</p>
              </div>
            </div>
            <div v-if="(exp.tags || []).length > 0" class="flex items-center gap-1 mt-2 flex-wrap">
              <span v-for="tag in exp.tags" :key="tag" class="inline-flex items-center px-1.5 py-0 rounded text-[9px] font-medium bg-surface-hover text-text-muted border border-border">#{{ tag }}</span>
            </div>
            <div v-if="getPaymentMethod(exp.paymentMethodId)" class="flex items-center gap-1.5 mt-2">
              <IconDisplay :icon="getPaymentMethod(exp.paymentMethodId)!.icon" :size="16" />
              <span class="text-[10px] text-text-muted">{{ getPaymentMethod(exp.paymentMethodId)!.name }}</span>
            </div>
          </template>
        </UniversalListRow>
      </div>
    </template>
    </div>

    <!-- Pagination -->
    <div v-if="totalPages > 1" class="flex items-center justify-center gap-1 mt-4">
      <button :class="button({ size: 'icon' })" :aria-label="t('previous_page')" :disabled="currentPage <= 1" @click="goPage(currentPage - 1)">
        <ChevronLeft :size="16" />
      </button>
      <template v-for="p in totalPages" :key="p">
        <button
          v-if="p === 1 || p === totalPages || (p >= currentPage - 1 && p <= currentPage + 1)"
          :class="button({ intent: p === currentPage ? 'primary' : 'ghost', size: 'icon', class: 'tabular-nums' })"
          :aria-current="p === currentPage ? 'page' : undefined"
          @click="goPage(p)"
        >{{ p }}</button>
        <span v-else-if="p === currentPage - 2 || p === currentPage + 2" class="px-1 text-xs text-text-muted">…</span>
      </template>
      <button :class="button({ size: 'icon' })" :aria-label="t('next_page')" :disabled="currentPage >= totalPages" @click="goPage(currentPage + 1)">
        <ChevronRight :size="16" />
      </button>
    </div>

    <ExpenseDetail v-if="expenseLookupData" :show="showDetail" :expense="detailExp" :lookupData="expenseLookupData" @close="showDetail = false" @edit="onDetailEdit" @delete="onDetailDelete" @openUrl="onDetailOpenUrl" />
    <ExpenseForm
      v-if="expenseFormLookupData"
      :show="showForm"
      :editExpense="editingExpense"
      :lookupData="expenseFormLookupData"
      @close="showForm = false"
      @saved="onSaved"
    />

    <AiSmartDialog
      :show="showAiSmart"
      surface="expense"
      @close="showAiSmart = false"
      @imported="onAiImported"
    />

    <ConfirmDialog
      :show="Boolean(deleteConfirmId)"
      :title="t('delete')"
      :message="t('confirm_delete_expense')"
      @confirm="confirmDelete"
      @cancel="cancelDelete"
    />

    <ConfirmDialog
      :show="batchDeleteConfirmIds.length > 0"
      :title="t('batch_delete')"
      :message="t('batch_confirm_delete_expenses', { count: batchDeleteConfirmIds.length })"
      @confirm="confirmBatchDelete"
      @cancel="batchDeleteConfirmIds = []"
    />

    <ContextActionMenu
      v-model:open="contextMenuOpen"
      :x="contextMenuX"
      :y="contextMenuY"
      :rows="contextMenuRows"
      :exclude-rect="contextMenuExcludeRect"
    />

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </div>
</template>
