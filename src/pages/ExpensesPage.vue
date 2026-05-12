<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import {
  deleteExpense,
  deleteExpensesBatch,
  insertExpense,
  type ExpenseFilter,
} from "@/services/expensesClient";
import { storeToRefs } from "pinia";
import type { Currency, Expense, Subscription, Category, PaymentMethod, Tag, Settings, HouseholdMember } from "@/schemas/appData";
import { expenseToIsoDate, parseExpense } from "@/schemas/appData";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { useHeaderActions, type HeaderAction } from "@/composables/useHeaderActions";
import { useToast } from "@/composables/useToast";
import { useClipboard } from "@/composables/useClipboard";
import { useScrollLock } from "@/composables/useScrollLock";
import ExpenseForm, { type ExpensePrefill } from "@/components/expenses/ExpenseForm.vue";
import ExpenseDetail from "@/components/expenses/ExpenseDetail.vue";
import AiQuickAddExpense from "@/components/ai/AiQuickAddExpense.vue";
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
  Plus,
  Search,
  Pencil,
  Trash2,
  ExternalLink,
  CopyPlus,
  ClipboardCopy,
  CheckSquare,
  Square,
  Wallet,
  AlertTriangle,
  ChevronLeft,
  ChevronRight,
  Rows3,
  LayoutList,
  LayoutGrid,
  Filter,
  Sparkles,
} from "@lucide/vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { formatErrorForToast } from "@/utils/formatError";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { useSubscriptionsStore } from "@/stores/subscriptionsStore";
import { useExpensesStore } from "@/stores/expensesStore";
import { ui } from "@/lib/tv";
import { type ExpenseDraft } from "@/services/aiClient";
import { useAiConfigStore } from "@/stores/aiConfigStore";

const PAGE_SIZE = 10;
const subscriptions = ref<Subscription[]>([]);
const categories = ref<Category[]>([]);
const paymentMethods = ref<PaymentMethod[]>([]);
const currencies = ref<Currency[]>([]);
const tags = ref<Tag[]>([]);
const household = ref<HouseholdMember[]>([]);
const settings = computed<Settings | null>(() => metaRefs.settings.value ?? null);
const metaStore = useAppMetaStore();
const subscriptionsStore = useSubscriptionsStore();
const expensesStore = useExpensesStore();
const metaRefs = storeToRefs(metaStore);
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

// AI quick-add state — must be declared before updateHeaderActions so the
// action builder can read the availability flag without forward refs.
const showAiQuickAdd = ref(false);
const aiConfigStore = useAiConfigStore();
const {
  enabled: aiEnabled,
  features: aiFeatures,
  hasApiKey: aiHasApiKey,
  activeProvider: aiActiveProvider,
  loaded: aiLoaded,
} = storeToRefs(aiConfigStore);
/**
 * Header AI button state. The expense surface treats `expenseInput` as the
 * primary action and only goes into "setup" mode when neither of the two
 * supported features is even toggled in config — in that case we still
 * want a clear path back to settings.
 */
const aiHeaderState = computed<"ready" | "setup" | "hidden">(() => {
  if (!aiLoaded.value) return "hidden";
  const wantsAnyFeature =
    aiFeatures.value.expenseInput ||
    aiFeatures.value.receiptImport ||
    aiFeatures.value.statementImport;
  if (!wantsAnyFeature) return "hidden";
  const requiresKey = !!aiActiveProvider.value?.requiresKey;
  const keyOk = !requiresKey || aiHasApiKey.value;
  if (aiEnabled.value && keyOk) return "ready";
  return "setup";
});

const aiRouter = useRouter();
function openAiSettings() {
  aiRouter.push("/settings?section=ai");
}

function openAiQuickAdd() {
  showAiQuickAdd.value = true;
}

function updateHeaderActions() {
  const viewIcon = viewMode.value === "compact" ? Rows3 : viewMode.value === "expanded" ? LayoutGrid : LayoutList;
  const nextViewMode = viewMode.value === "compact" ? "default" : viewMode.value === "default" ? "expanded" : "compact";
  const currentViewTitle = viewMode.value === "compact" ? t("view_compact") : viewMode.value === "expanded" ? t("view_expanded") : t("view_default");
  const nextViewTitle = nextViewMode === "compact" ? t("view_compact") : nextViewMode === "expanded" ? t("view_expanded") : t("view_default");

  const actions: HeaderAction[] = [
    { id: "toggle-expense-filters", icon: Filter, title: showFilters.value ? `${t("filter")} ✓` : `${t("filter")} ✕`, onClick: () => { showFilters.value = !showFilters.value; }, style: showFilters.value ? "warning" : "neutral" },
    { id: "cycle-expense-view", icon: viewIcon, title: `${currentViewTitle} → ${nextViewTitle}`, onClick: () => setViewMode(nextViewMode), style: "accent" },
    { id: "expense-selection-mode", icon: CheckSquare, title: selectionMode.value ? `${t("select")} ✓` : `${t("select")} ✕`, onClick: toggleSelectionMode, style: selectionMode.value ? "success" : "neutral" },
  ];
  if (aiHeaderState.value === "ready") {
    actions.push({ id: "ai-add-expense", icon: Sparkles, title: t("ai_quick_add_expense"), onClick: openAiQuickAdd, style: "accent" });
  } else if (aiHeaderState.value === "setup") {
    actions.push({ id: "ai-setup-expense", icon: Sparkles, title: t("ai_setup_assistant"), onClick: openAiSettings, style: "neutral" });
  }
  actions.push({ id: "add-expense", icon: Plus, title: t("add_expense"), onClick: openAdd, style: "primary" });
  setActions(actions);
}

onMounted(() => {
  updateHeaderActions();
  loadInitial().then(() => applyFilters());
  aiConfigStore.load().then(() => updateHeaderActions());
});
watch(aiHeaderState, () => updateHeaderActions());
onUnmounted(() => clearActions());

// ---- View mode ----
const viewMode = computed(() => settings.value?.expenseViewMode || "default");

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
const formPrefill = ref<ExpensePrefill | null>(null);

function applyAiDraft(draft: ExpenseDraft) {
  const prefill: ExpensePrefill = {};
  if (draft.name) prefill.name = draft.name;
  if (draft.amount > 0) prefill.amount = draft.amount;
  if (draft.currencyId) prefill.currencyId = draft.currencyId;
  if (draft.date) prefill.date = draft.date;
  if (draft.categoryId) prefill.categoryId = draft.categoryId;
  if (draft.paymentMethodId) prefill.paymentMethodId = draft.paymentMethodId;
  if (draft.tags?.length) prefill.tags = [...draft.tags];
  if (draft.notes) prefill.notes = draft.notes;
  if (draft.url) prefill.url = draft.url;

  editingExpense.value = null;
  formPrefill.value = prefill;
  showForm.value = true;

  if (draft.warnings.length) {
    toast(t("ai_draft_warnings", { count: draft.warnings.length }));
  }
}

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
      toast(t("batch_deleted_expenses").replace("{count}", String(batchDeleteConfirmIds.value.length)));
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
function formatAmount(amount: number, currencyId: string) {
  const cur = getCurrency(currencyId);
  return fmtCurrency(amount, cur?.code || "USD");
}

const targetCurrencies = computed(() => {
  const mainId = settings.value?.mainCurrencyId;
  const targets = settings.value?.currencyUpdateTargets ?? [];
  const ids = new Set(targets);
  if (mainId) ids.add(mainId);
  return [...ids]
    .map((id) => currencies.value.find((c) => c.id === id))
    .filter((c): c is Currency => !!c && c.rate > 0);
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
    expensesCount: totalCount.value,
  };
});
const mainCurrencyCode = computed(() => {
  const mainId = settings.value?.mainCurrencyId;
  return currencies.value.find((c) => c.id === mainId)?.code || "USD";
});

function convertAmount(amount: number, fromCurId: string, toCurrency: Currency): number {
  const fromCur = currencies.value.find((c) => c.id === fromCurId);
  if (!fromCur || fromCur.rate <= 0) return 0;
  return (amount / fromCur.rate) * toCurrency.rate;
}

function fmtCur(amount: number, currency: Currency): string {
  return fmtCurrency(amount, currency.code);
}

function getConvertedAmounts(amount: number, currencyId: string) {
  return targetCurrencies.value
    .filter((tc) => tc.id !== currencyId)
    .map((tc) => ({
      currency: tc,
      amount: convertAmount(amount, currencyId, tc),
    }));
}

// ---- Actions ----
function openAdd() { editingExpense.value = null; formPrefill.value = null; showForm.value = true; }

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
    createdAt: new Date().toISOString(),
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

async function loadInitial() {
  await metaStore.ensureLoaded();
  categories.value = metaRefs.categories.value;
  paymentMethods.value = metaRefs.paymentMethods.value;
  currencies.value = metaRefs.currencies.value;
  tags.value = metaRefs.tags.value;
  household.value = metaRefs.household.value;
  await subscriptionsStore.loadBrief();
  subscriptions.value = subscriptionsStore.items;
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
    <Transition enter-active-class="transition ease-out duration-200" enter-from-class="opacity-0 -translate-y-2" enter-to-class="opacity-100 translate-y-0" leave-active-class="transition ease-in duration-150" leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 -translate-y-2">
      <div v-if="showFilters" class="mb-3 p-2.5 sm:p-4 rounded-xl border border-border bg-surface space-y-2">
        <div class="relative min-w-0">
          <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('search')"
            class="w-full pl-8 pr-3 py-2 rounded-lg border border-border bg-surface text-xs text-text-primary placeholder-text-muted hover:border-text-muted focus:outline-none focus:ring-2 focus:ring-primary transition-shadow"
          />
        </div>
        <div class="flex items-center gap-2 overflow-x-auto pb-1 -mx-1 px-1 scrollbar-none">
          <div class="w-28 shrink-0"><AppSelect v-model="sortBy" :options="sortOptions" size="sm" /></div>
          <div class="w-28 shrink-0"><AppSelect v-model="filterCategory" :options="categoryOptions" size="sm" /></div>
          <div class="w-28 shrink-0"><AppSelect v-model="filterPayment" :options="paymentOptions" size="sm" /></div>
          <div v-if="tags.length > 0" class="w-28 shrink-0"><AppSelect v-model="filterTag" :options="tagOptions" size="sm" /></div>
          <input v-model="dateFrom" type="date" class="shrink-0 w-28 px-2 py-1 text-[11px] rounded-lg bg-surface border border-border text-text-primary focus:outline-none focus:ring-2 focus:ring-primary" />
          <input v-model="dateTo" type="date" class="shrink-0 w-28 px-2 py-1 text-[11px] rounded-lg bg-surface border border-border text-text-primary focus:outline-none focus:ring-2 focus:ring-primary" />
        </div>
      </div>
    </Transition>

    <!-- Batch toolbar -->
    <Transition enter-active-class="transition ease-out duration-200" enter-from-class="opacity-0 -translate-y-2" enter-to-class="opacity-100 translate-y-0" leave-active-class="transition ease-in duration-150" leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 -translate-y-2">
      <div v-if="selectionMode" class="flex items-center gap-1.5 sm:gap-2 mb-3 px-2 sm:px-3 py-2 rounded-lg bg-surface-secondary border border-border overflow-x-auto">
        <span class="text-xs font-medium text-text-primary">{{ selectedIds.size }} {{ t('selected_count') }}</span>
        <button @click="selectAll" class="p-1 rounded text-text-secondary hover:bg-surface transition-colors" :title="t('select_all')">
          <CheckSquare :size="13" />
        </button>
        <button @click="deselectAll" class="p-1 rounded text-text-muted hover:bg-surface-hover transition-colors" :title="t('deselect_all')">
          <Square :size="13" />
        </button>
        <div class="flex-1" />
        <button @click="batchDeleteSelected" :disabled="selectedIds.size === 0"
          class="px-2.5 py-1 rounded-md text-[11px] font-medium border border-border text-text-secondary hover:bg-surface disabled:opacity-30 transition-colors"
        >{{ t('batch_delete') }}</button>
      </div>
    </Transition>

    <!-- Summary bar -->
    <div class="flex items-center justify-between px-3 py-2 mb-3 rounded-lg bg-surface border border-border">
      <div class="flex items-center gap-2 text-xs text-text-muted">
        <Wallet :size="14" />
        <span>{{ totalCount }} {{ t('expenses').toLowerCase() }}</span>
      </div>
      <div class="text-sm font-semibold text-text-primary">
        {{ fmtCurrency(totalFiltered, mainCurrencyCode) }}
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="text-center py-16">
      <div class="w-8 h-8 mx-auto border-2 border-primary border-t-transparent rounded-full animate-spin" />
    </div>

    <!-- Empty state -->
    <div v-else-if="items.length === 0" class="text-center py-16">
      <div class="w-20 h-20 mx-auto mb-4 rounded-full bg-surface-hover flex items-center justify-center">
        <Wallet :size="36" class="text-text-muted" />
      </div>
      <p class="text-text-muted mb-4">{{ t('no_expenses_yet') }}</p>
      <button @click="openAdd"
        class="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover"
      ><Plus :size="18" /> {{ t('add_expense') }}</button>
    </div>

    <!-- Expense list -->
    <div v-if="!loading && items.length > 0" :class="viewMode === 'expanded' ? 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2 sm:gap-3' : 'space-y-1.5 sm:space-y-2'">
      <div
        v-for="exp in items"
        :key="exp.id"
        class="bg-surface rounded-xl border overflow-hidden transition-colors cursor-pointer"
        :class="selectedIds.has(exp.id) ? 'border-primary ring-1 ring-primary/30' : 'border-border'"
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
              @click.stop="toggleSelect(exp.id)"
            >
              <div
                class="rounded border-2 flex items-center justify-center transition-colors cursor-pointer"
                :class="[
                  viewMode === 'compact' ? 'w-4 h-4' : 'w-5 h-5',
                  selectedIds.has(exp.id) ? 'bg-primary border-primary text-white' : 'border-border hover:border-primary',
                ]"
              >
                <svg v-if="selectedIds.has(exp.id)" :width="viewMode === 'compact' ? 10 : 12" :height="viewMode === 'compact' ? 10 : 12" viewBox="0 0 12 12" fill="none"><path d="M2 6L5 9L10 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
              </div>
            </div>
          </template>
          <template #leading>
            <div
              v-if="viewMode !== 'expanded'"
              class="rounded bg-primary-light flex items-center justify-center font-bold text-primary shrink-0 overflow-hidden"
              :class="viewMode === 'compact' ? 'w-6 h-6 text-[10px]' : 'w-9 h-9 sm:w-10 sm:h-10 text-xs sm:text-sm'"
            >
              <IconDisplay v-if="getExpenseIcon(exp)" :icon="getExpenseIcon(exp)" :size="viewMode === 'compact' ? 12 : 18" />
              <span v-else>{{ exp.name.charAt(0).toUpperCase() }}</span>
            </div>
          </template>
          <template #main>
            <p v-if="viewMode === 'compact'" class="text-xs font-medium text-text-primary truncate">{{ exp.name }}</p>
            <div v-else-if="viewMode === 'default'" class="min-w-0">
              <p class="text-xs sm:text-sm font-medium text-text-primary truncate">{{ exp.name }}</p>
              <div class="flex items-center gap-1.5 flex-wrap">
                <span class="text-[10px] sm:text-xs text-text-muted">{{ getCategory(exp.categoryId)?.name || '' }}</span>
                <span v-for="tag in (exp.tags || []).slice(0, 3)" :key="tag" class="hidden sm:inline-flex items-center px-1.5 py-0 rounded text-[9px] font-medium bg-surface-hover text-text-muted border border-border">#{{ tag }}</span>
              </div>
            </div>
          </template>
          <template #meta>
            <span v-if="viewMode === 'compact'" class="text-[10px] text-text-muted shrink-0 w-[86px] text-right tabular-nums">{{ formatExpenseDate(exp) }}</span>
            <div v-else-if="viewMode === 'default'" class="text-right shrink-0 w-[104px] sm:w-[120px]">
              <p class="text-xs sm:text-sm font-medium text-text-primary">
                <span class="hidden sm:inline tabular-nums">{{ formatExpenseDate(exp) }}</span>
                <span class="sm:hidden text-[10px] text-text-muted tabular-nums">{{ formatExpenseDate(exp) }}</span>
              </p>
            </div>
          </template>
          <template #value>
            <p v-if="viewMode === 'compact'" class="text-xs font-semibold text-text-primary shrink-0 w-[108px] text-right tabular-nums">{{ formatAmount(exp.amount, exp.currencyId) }}</p>
            <div v-else-if="viewMode === 'default'" class="text-right shrink-0 w-[112px] sm:w-[132px]">
              <p class="text-xs sm:text-sm font-semibold text-text-primary tabular-nums">{{ formatAmount(exp.amount, exp.currencyId) }}</p>
              <div v-if="getConvertedAmounts(exp.amount, exp.currencyId).length > 0" class="mt-0.5 space-y-0">
                <p v-for="cv in getConvertedAmounts(exp.amount, exp.currencyId)" :key="cv.currency.id"
                  class="text-[10px] text-text-muted tabular-nums">≈ {{ fmtCur(cv.amount, cv.currency) }}</p>
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
              <div v-if="selectionMode" class="shrink-0 mt-1" @click.stop="toggleSelect(exp.id)">
                <div class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors cursor-pointer"
                  :class="selectedIds.has(exp.id) ? 'bg-primary border-primary text-white' : 'border-border hover:border-primary'"
                ><svg v-if="selectedIds.has(exp.id)" width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 6L5 9L10 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg></div>
              </div>
              <div class="w-12 h-12 rounded-lg bg-primary-light flex items-center justify-center text-lg font-bold text-primary shrink-0 overflow-hidden">
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
                <p class="text-lg font-bold text-text-primary">{{ formatAmount(exp.amount, exp.currencyId) }}</p>
                <div v-if="getConvertedAmounts(exp.amount, exp.currencyId).length > 0" class="mt-0.5 space-y-0">
                  <p v-for="cv in getConvertedAmounts(exp.amount, exp.currencyId)" :key="cv.currency.id"
                    class="text-[10px] text-text-muted tabular-nums">≈ {{ fmtCur(cv.amount, cv.currency) }}</p>
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
    </div>

    <!-- Pagination -->
    <div v-if="totalPages > 1" class="flex items-center justify-center gap-1 mt-4">
      <button @click="goPage(currentPage - 1)" :disabled="currentPage <= 1"
        class="p-1.5 rounded-lg border border-border text-text-muted hover:bg-surface-hover disabled:opacity-30 transition-colors">
        <ChevronLeft :size="16" />
      </button>
      <template v-for="p in totalPages" :key="p">
        <button v-if="p === 1 || p === totalPages || (p >= currentPage - 1 && p <= currentPage + 1)"
          @click="goPage(p)"
          class="min-w-8 h-8 rounded-lg text-xs font-medium transition-colors"
          :class="p === currentPage ? 'bg-primary text-white' : 'text-text-secondary hover:bg-surface-hover'"
        >{{ p }}</button>
        <span v-else-if="p === currentPage - 2 || p === currentPage + 2" class="text-xs text-text-muted">…</span>
      </template>
      <button @click="goPage(currentPage + 1)" :disabled="currentPage >= totalPages"
        class="p-1.5 rounded-lg border border-border text-text-muted hover:bg-surface-hover disabled:opacity-30 transition-colors">
        <ChevronRight :size="16" />
      </button>
    </div>

    <ExpenseDetail v-if="expenseLookupData" :show="showDetail" :expense="detailExp" :lookupData="expenseLookupData" @close="showDetail = false" @edit="onDetailEdit" @delete="onDetailDelete" @openUrl="onDetailOpenUrl" />
    <ExpenseForm
      v-if="expenseFormLookupData"
      :show="showForm"
      :editExpense="editingExpense"
      :prefill="formPrefill"
      :lookupData="expenseFormLookupData"
      @close="showForm = false; formPrefill = null"
      @saved="onSaved"
    />

    <AiQuickAddExpense
      :show="showAiQuickAdd"
      @close="showAiQuickAdd = false"
      @draft="applyAiDraft"
    />

    <!-- Delete Confirmation -->
    <Teleport to="body">
      <Transition name="app-modal">
        <div v-if="deleteConfirmId" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
          <div class="app-modal-backdrop absolute inset-0 bg-black/50" @click="cancelDelete" />
          <div class="app-modal-panel relative bg-surface w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-sm p-4 sm:p-6">
            <div class="flex items-center gap-3 mb-3 sm:mb-4">
              <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center shrink-0"><AlertTriangle :size="18" class="text-red-500" /></div>
              <h3 :class="ui.sectionTitle()">{{ t('delete') }}</h3>
            </div>
            <p class="text-xs sm:text-sm text-text-secondary mb-4 sm:mb-6">{{ t('confirm_delete_expense') }}</p>
            <div class="flex justify-end gap-2 sm:gap-3">
              <button @click="cancelDelete" class="px-3 sm:px-4 py-2 rounded-lg border border-border text-xs sm:text-sm font-medium text-text-secondary hover:bg-surface-hover">{{ t('cancel') }}</button>
              <button @click="confirmDelete" class="px-3 sm:px-4 py-2 rounded-lg bg-red-600 text-white text-xs sm:text-sm font-medium hover:bg-red-700">{{ t('delete') }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Batch Delete Confirmation -->
    <Teleport to="body">
      <Transition name="app-modal">
        <div v-if="batchDeleteConfirmIds.length > 0" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
          <div class="app-modal-backdrop absolute inset-0 bg-black/50" @click="batchDeleteConfirmIds = []" />
          <div class="app-modal-panel relative bg-surface w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-sm p-4 sm:p-6">
            <div class="flex items-center gap-3 mb-3 sm:mb-4">
              <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center shrink-0"><AlertTriangle :size="18" class="text-red-500" /></div>
              <h3 :class="ui.sectionTitle()">{{ t('batch_delete') }}</h3>
            </div>
            <p class="text-xs sm:text-sm text-text-secondary mb-4 sm:mb-6">{{ t('batch_confirm_delete_expenses').replace('{count}', String(batchDeleteConfirmIds.length)) }}</p>
            <div class="flex justify-end gap-2 sm:gap-3">
              <button @click="batchDeleteConfirmIds = []" class="px-3 sm:px-4 py-2 rounded-lg border border-border text-xs sm:text-sm font-medium text-text-secondary hover:bg-surface-hover">{{ t('cancel') }}</button>
              <button @click="confirmBatchDelete" class="px-3 sm:px-4 py-2 rounded-lg bg-red-600 text-white text-xs sm:text-sm font-medium hover:bg-red-700">{{ t('delete') }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

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
