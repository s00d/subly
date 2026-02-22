<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useExpensesStore, PAGE_SIZE } from "@/stores/expenses";
import { useCatalogStore } from "@/stores/catalog";
import { useSettingsStore } from "@/stores/settings";
import { useI18n } from "vue-i18n";
import { formatCurrency } from "@/services/calculations";
import { dbGetExpenseTotalFiltered, type ExpenseFilter } from "@/services/database";
import type { Currency, Expense } from "@/schemas/appData";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { useHeaderActions } from "@/composables/useHeaderActions";
import { useToast } from "@/composables/useToast";
import ExpenseForm from "@/components/expenses/ExpenseForm.vue";
import ExpenseDetail from "@/components/expenses/ExpenseDetail.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import Toast from "@/components/ui/Toast.vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import {
  Plus, Search, Trash2, CheckSquare,
  Wallet, AlertTriangle, ChevronLeft, ChevronRight,
  Rows3, LayoutList, LayoutGrid,
} from "lucide-vue-next";
import { Menu } from "@tauri-apps/api/menu";
import type { MenuItemOptions, PredefinedMenuItemOptions } from "@tauri-apps/api/menu";
import { openUrl } from "@tauri-apps/plugin-opener";

const expsStore = useExpensesStore();
const catalogStore = useCatalogStore();
const settingsStore = useSettingsStore();
const { t } = useI18n();
const { setActions, clearActions } = useHeaderActions();
const { fmtDateMedium, fmtCurrency } = useLocaleFormat();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();

onMounted(() => {
  setActions([{ id: "add-expense", icon: Plus, title: t("add_expense"), onClick: openAdd }]);
  applyFilters();
});
onUnmounted(() => clearActions());

// ---- View mode ----
const viewMode = computed(() => settingsStore.settings.expenseViewMode || "default");

function setViewMode(mode: "default" | "compact" | "expanded") {
  settingsStore.updateSettings({ expenseViewMode: mode });
}

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
  expsStore.fetchPage(1, buildFilter());
  updateTotal();
}

function applyFiltersDebounced() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(applyFilters, 300);
}

watch([filterCategory, filterPayment, filterTag, dateFrom, dateTo, sortBy], applyFilters);
watch(searchQuery, applyFiltersDebounced);

// ---- Pagination ----
const totalPages = computed(() => Math.max(1, Math.ceil(expsStore.totalCount / PAGE_SIZE)));

function goPage(p: number) {
  if (p < 1 || p > totalPages.value) return;
  expsStore.fetchPage(p);
}

// ---- Summary ----
const totalFiltered = ref(0);
async function updateTotal() {
  totalFiltered.value = await dbGetExpenseTotalFiltered(buildFilter());
}

// ---- Form ----
const showForm = ref(false);
const editingExpense = ref<Expense | null>(null);

// ---- Detail panel ----
const showDetail = ref(false);
const detailExpId = ref<string | null>(null);
const detailExp = computed(() =>
  detailExpId.value ? expsStore.items.find((e) => e.id === detailExpId.value) ?? null : null,
);

// ---- Selection for batch ----
const selectionMode = ref(false);
const selectedIds = ref<Set<string>>(new Set());

function toggleSelect(id: string) {
  const s = new Set(selectedIds.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  selectedIds.value = s;
}

function selectAll() {
  selectedIds.value = new Set(expsStore.items.map((e) => e.id));
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

function requestDelete(id: string) {
  showDetail.value = false;
  detailExpId.value = null;
  deleteConfirmId.value = id;
}

async function confirmDelete() {
  if (deleteConfirmId.value) {
    await expsStore.deleteExpense(deleteConfirmId.value);
    toast(t("expense_deleted"));
    await updateTotal();
  }
  deleteConfirmId.value = null;
}

function cancelDelete() { deleteConfirmId.value = null; }

function batchDeleteSelected() {
  batchDeleteConfirmIds.value = [...selectedIds.value];
}

async function confirmBatchDelete() {
  if (batchDeleteConfirmIds.value.length > 0) {
    await expsStore.batchDeleteExpenses(batchDeleteConfirmIds.value);
    toast(t("batch_deleted_expenses").replace("{count}", String(batchDeleteConfirmIds.value.length)));
    selectedIds.value = new Set();
    selectionMode.value = false;
    await updateTotal();
  }
  batchDeleteConfirmIds.value = [];
}

// ---- Computed options ----
const categoryOptions = computed<SelectOption[]>(() => [
  { label: t("category"), value: "" },
  ...catalogStore.sortedCategories.map((c) => ({ label: c.name, value: c.id, icon: c.icon || undefined })),
]);

const paymentOptions = computed<SelectOption[]>(() => [
  { label: t("payment_method"), value: "" },
  ...catalogStore.enabledPaymentMethods.map((p) => ({ label: p.name, value: p.id, icon: p.icon })),
]);

const tagOptions = computed<SelectOption[]>(() => [
  { label: t("filter_by_tag"), value: "" },
  ...catalogStore.tags.map((tg) => ({ label: tg.name, value: tg.name })),
]);

const sortOptions = computed<SelectOption[]>(() => [
  { label: `${t("expense_date")} ↓`, value: "date_desc" },
  { label: `${t("expense_date")} ↑`, value: "date_asc" },
  { label: `${t("expense_amount")} ↓`, value: "amount_desc" },
  { label: `${t("expense_amount")} ↑`, value: "amount_asc" },
]);

// ---- Helpers ----
function getCurrency(id: string) { return catalogStore.currencies.find((c) => c.id === id); }
function getCategory(id: string) { return catalogStore.categories.find((c) => c.id === id); }
function getPaymentMethod(id: string) { return catalogStore.paymentMethods.find((p) => p.id === id); }
function formatDate(d: string) { return fmtDateMedium(d); }
function formatAmount(amount: number, currencyId: string) {
  const cur = getCurrency(currencyId);
  return fmtCurrency(amount, cur?.code || "USD");
}

const targetCurrencies = computed(() => {
  const mainId = settingsStore.settings.mainCurrencyId;
  const targets = settingsStore.settings.currencyUpdateTargets ?? [];
  const ids = new Set(targets);
  if (mainId) ids.add(mainId);
  return [...ids]
    .map((id) => catalogStore.currencies.find((c) => c.id === id))
    .filter((c): c is Currency => !!c && c.rate > 0);
});

function convertAmount(amount: number, fromCurId: string, toCurrency: Currency): number {
  const fromCur = catalogStore.currencies.find((c) => c.id === fromCurId);
  if (!fromCur || fromCur.rate <= 0) return 0;
  return (amount / fromCur.rate) * toCurrency.rate;
}

function fmtCur(amount: number, currency: Currency): string {
  return formatCurrency(amount, currency.code, currency.symbol);
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
  toast(editingExpense.value ? t("expense_updated") : t("expense_added"));
  await expsStore.fetchPage();
  await updateTotal();
}

function onDetailEdit(exp: Expense) { openEdit(exp); }
function onDetailDelete(id: string) { requestDelete(id); }

async function handleOpenUrl(url: string) {
  const fullUrl = url.startsWith("http") ? url : `https://${url}`;
  try { await openUrl(fullUrl); } catch (e) { console.error("Failed to open URL:", e); }
}

function onDetailOpenUrl(url: string) { handleOpenUrl(url); }

async function showContextMenu(exp: Expense, event: MouseEvent) {
  if (selectionMode.value) return;

  const items: (MenuItemOptions | PredefinedMenuItemOptions)[] = [
    { id: "edit", text: t("edit"), action: () => openEdit(exp) },
  ];
  if (exp.url) {
    items.push({ id: "url", text: t("url"), action: () => handleOpenUrl(exp.url) });
  }
  items.push(
    { item: "Separator" },
    { id: "delete", text: t("delete"), action: () => requestDelete(exp.id) },
  );

  try {
    const menu = await Menu.new({ items });
    await menu.popup();
  } catch (e) {
    console.warn("Context menu failed:", e);
  }
}
</script>

<template>
  <div class="max-w-5xl mx-auto">
    <!-- Search row -->
    <div class="flex items-center gap-2 mb-2">
      <div class="relative flex-1 min-w-0">
        <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
        <input
          v-model="searchQuery"
          type="text"
          :placeholder="t('search')"
          class="w-full pl-8 pr-3 py-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] text-xs text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] hover:border-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] transition-shadow"
        />
      </div>
      <div class="flex items-center border border-[var(--color-border)] rounded-lg overflow-hidden shrink-0">
        <Tooltip :text="t('view_compact')">
          <button @click="setViewMode('compact')" class="p-1.5 transition-colors" :class="viewMode === 'compact' ? 'bg-[var(--color-primary-light)] text-[var(--color-primary)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]'"><Rows3 :size="15" /></button>
        </Tooltip>
        <Tooltip :text="t('view_default')">
          <button @click="setViewMode('default')" class="p-1.5 transition-colors border-x border-[var(--color-border)]" :class="viewMode === 'default' ? 'bg-[var(--color-primary-light)] text-[var(--color-primary)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]'"><LayoutList :size="15" /></button>
        </Tooltip>
        <Tooltip :text="t('view_expanded')">
          <button @click="setViewMode('expanded')" class="p-1.5 transition-colors" :class="viewMode === 'expanded' ? 'bg-[var(--color-primary-light)] text-[var(--color-primary)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]'"><LayoutGrid :size="15" /></button>
        </Tooltip>
      </div>
      <Tooltip :text="t('select')">
        <button
          @click="toggleSelectionMode"
          class="p-1.5 rounded-lg border transition-colors shrink-0"
          :class="selectionMode ? 'border-[var(--color-primary)] bg-[var(--color-primary-light)] text-[var(--color-primary)]' : 'border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]'"
        >
          <CheckSquare :size="16" />
        </button>
      </Tooltip>
    </div>

    <!-- Filters row -->
    <div class="flex items-center gap-2 mb-3 overflow-x-auto pb-1 -mx-1 px-1 scrollbar-none">
      <div class="w-28 shrink-0"><AppSelect v-model="sortBy" :options="sortOptions" size="sm" /></div>
      <div class="w-28 shrink-0"><AppSelect v-model="filterCategory" :options="categoryOptions" size="sm" /></div>
      <div class="w-28 shrink-0"><AppSelect v-model="filterPayment" :options="paymentOptions" size="sm" /></div>
      <div v-if="catalogStore.tags.length > 0" class="w-28 shrink-0"><AppSelect v-model="filterTag" :options="tagOptions" size="sm" /></div>
      <input v-model="dateFrom" type="date" class="shrink-0 w-28 px-2 py-1 text-[11px] rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]" />
      <input v-model="dateTo" type="date" class="shrink-0 w-28 px-2 py-1 text-[11px] rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]" />
    </div>

    <!-- Batch toolbar -->
    <Transition enter-active-class="transition ease-out duration-200" enter-from-class="opacity-0 -translate-y-2" enter-to-class="opacity-100 translate-y-0" leave-active-class="transition ease-in duration-150" leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 -translate-y-2">
      <div v-if="selectionMode" class="flex items-center gap-1.5 sm:gap-2 mb-3 px-2 sm:px-3 py-2 rounded-lg bg-[var(--color-primary-light)] border border-[var(--color-primary)]/20 overflow-x-auto">
        <span class="text-xs font-medium text-[var(--color-primary)]">{{ selectedIds.size }} {{ t('selected_count') }}</span>
        <button @click="selectAll" class="text-[10px] font-medium text-[var(--color-primary)] hover:underline">{{ t('select_all') }}</button>
        <button @click="deselectAll" class="text-[10px] font-medium text-[var(--color-text-muted)] hover:underline">{{ t('deselect_all') }}</button>
        <div class="flex-1" />
        <button @click="batchDeleteSelected" :disabled="selectedIds.size === 0"
          class="px-2.5 py-1 rounded-md text-[11px] font-medium bg-red-100 text-red-700 hover:bg-red-200 dark:bg-red-900/30 dark:text-red-400 dark:hover:bg-red-900/50 disabled:opacity-30 transition-colors"
        >{{ t('batch_delete') }}</button>
      </div>
    </Transition>

    <!-- Summary bar -->
    <div class="flex items-center justify-between px-3 py-2 mb-3 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
      <div class="flex items-center gap-2 text-xs text-[var(--color-text-muted)]">
        <Wallet :size="14" />
        <span>{{ expsStore.totalCount }} {{ t('expenses').toLowerCase() }}</span>
      </div>
      <div class="text-sm font-semibold text-[var(--color-text-primary)]">
        {{ fmtCurrency(totalFiltered, catalogStore.mainCurrency?.code || 'USD') }}
      </div>
    </div>

    <!-- Loading -->
    <div v-if="expsStore.loading" class="text-center py-16">
      <div class="w-8 h-8 mx-auto border-2 border-[var(--color-primary)] border-t-transparent rounded-full animate-spin" />
    </div>

    <!-- Empty state -->
    <div v-else-if="expsStore.items.length === 0" class="text-center py-16">
      <div class="w-20 h-20 mx-auto mb-4 rounded-full bg-[var(--color-surface-hover)] flex items-center justify-center">
        <Wallet :size="36" class="text-[var(--color-text-muted)]" />
      </div>
      <p class="text-[var(--color-text-muted)] mb-4">{{ t('no_expenses_yet') }}</p>
      <button @click="openAdd"
        class="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:bg-[var(--color-primary-hover)]"
      ><Plus :size="18" /> {{ t('add_expense') }}</button>
    </div>

    <!-- Expense list -->
    <div v-else :class="viewMode === 'expanded' ? 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2 sm:gap-3' : 'space-y-1.5 sm:space-y-2'">
      <div
        v-for="exp in expsStore.items"
        :key="exp.id"
        class="bg-[var(--color-surface)] rounded-xl border overflow-hidden transition-colors cursor-pointer"
        :class="selectedIds.has(exp.id) ? 'border-[var(--color-primary)] ring-1 ring-[var(--color-primary)]/30' : 'border-[var(--color-border)]'"
      >
        <!-- COMPACT VIEW -->
        <div v-if="viewMode === 'compact'" class="flex items-center gap-2 px-3 py-2" @click="selectionMode ? toggleSelect(exp.id) : openDetail(exp)" @contextmenu.prevent="showContextMenu(exp, $event)">
          <div v-if="selectionMode" class="shrink-0" @click.stop="toggleSelect(exp.id)">
            <div class="w-4 h-4 rounded border-2 flex items-center justify-center transition-colors cursor-pointer"
              :class="selectedIds.has(exp.id) ? 'bg-[var(--color-primary)] border-[var(--color-primary)] text-white' : 'border-[var(--color-border)] hover:border-[var(--color-primary)]'"
            ><svg v-if="selectedIds.has(exp.id)" width="10" height="10" viewBox="0 0 12 12" fill="none"><path d="M2 6L5 9L10 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg></div>
          </div>
          <div class="w-6 h-6 rounded bg-[var(--color-primary-light)] flex items-center justify-center text-[10px] font-bold text-[var(--color-primary)] shrink-0 overflow-hidden">
            <IconDisplay v-if="getCategory(exp.categoryId)?.icon" :icon="getCategory(exp.categoryId)!.icon" :size="12" />
            <span v-else>{{ exp.name.charAt(0).toUpperCase() }}</span>
          </div>
          <p class="text-xs font-medium text-[var(--color-text-primary)] truncate min-w-0 flex-1">{{ exp.name }}</p>
          <span class="text-[10px] text-[var(--color-text-muted)] shrink-0">{{ formatDate(exp.date) }}</span>
          <p class="text-xs font-semibold text-[var(--color-text-primary)] shrink-0">{{ formatAmount(exp.amount, exp.currencyId) }}</p>
        </div>

        <!-- EXPANDED VIEW (card) -->
        <template v-else-if="viewMode === 'expanded'">
          <div class="p-4" @click="selectionMode ? toggleSelect(exp.id) : openDetail(exp)" @contextmenu.prevent="showContextMenu(exp, $event)">
            <div class="flex items-start gap-3 mb-3">
              <div v-if="selectionMode" class="shrink-0 mt-1" @click.stop="toggleSelect(exp.id)">
                <div class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors cursor-pointer"
                  :class="selectedIds.has(exp.id) ? 'bg-[var(--color-primary)] border-[var(--color-primary)] text-white' : 'border-[var(--color-border)] hover:border-[var(--color-primary)]'"
                ><svg v-if="selectedIds.has(exp.id)" width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 6L5 9L10 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg></div>
              </div>
              <div class="w-12 h-12 rounded-lg bg-[var(--color-primary-light)] flex items-center justify-center text-lg font-bold text-[var(--color-primary)] shrink-0 overflow-hidden">
                <IconDisplay v-if="getCategory(exp.categoryId)?.icon" :icon="getCategory(exp.categoryId)!.icon" :size="22" />
                <span v-else>{{ exp.name.charAt(0).toUpperCase() }}</span>
              </div>
              <div class="min-w-0 flex-1">
                <p class="text-sm font-semibold text-[var(--color-text-primary)] truncate">{{ exp.name }}</p>
                <p class="text-xs text-[var(--color-text-muted)] flex items-center gap-1">
                  <IconDisplay v-if="getCategory(exp.categoryId)?.icon" :icon="getCategory(exp.categoryId)!.icon" :size="12" />
                  {{ getCategory(exp.categoryId)?.name || '' }}
                </p>
              </div>
            </div>
            <div class="flex items-end justify-between">
              <div>
                <p class="text-lg font-bold text-[var(--color-text-primary)]">{{ formatAmount(exp.amount, exp.currencyId) }}</p>
                <div v-if="getConvertedAmounts(exp.amount, exp.currencyId).length > 0" class="mt-0.5 space-y-0">
                  <p v-for="cv in getConvertedAmounts(exp.amount, exp.currencyId)" :key="cv.currency.id"
                    class="text-[10px] text-[var(--color-text-muted)] tabular-nums">≈ {{ fmtCur(cv.amount, cv.currency) }}</p>
                </div>
              </div>
              <div class="text-right">
                <p class="text-xs font-medium text-[var(--color-text-primary)]">{{ formatDate(exp.date) }}</p>
              </div>
            </div>
            <div v-if="(exp.tags || []).length > 0" class="flex items-center gap-1 mt-2 flex-wrap">
              <span v-for="tag in exp.tags" :key="tag" class="inline-flex items-center px-1.5 py-0 rounded text-[9px] font-medium bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] border border-[var(--color-border)]">#{{ tag }}</span>
            </div>
            <div v-if="getPaymentMethod(exp.paymentMethodId)" class="flex items-center gap-1.5 mt-2">
              <IconDisplay :icon="getPaymentMethod(exp.paymentMethodId)!.icon" :size="16" />
              <span class="text-[10px] text-[var(--color-text-muted)]">{{ getPaymentMethod(exp.paymentMethodId)!.name }}</span>
            </div>
          </div>
        </template>

        <!-- DEFAULT VIEW -->
        <template v-else>
          <div
            class="flex items-center gap-2 sm:gap-3 p-3 sm:p-4"
            @click="selectionMode ? toggleSelect(exp.id) : openDetail(exp)"
            @contextmenu.prevent="showContextMenu(exp, $event)"
          >
            <div v-if="selectionMode" class="shrink-0" @click.stop="toggleSelect(exp.id)">
              <div class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors cursor-pointer"
                :class="selectedIds.has(exp.id) ? 'bg-[var(--color-primary)] border-[var(--color-primary)] text-white' : 'border-[var(--color-border)] hover:border-[var(--color-primary)]'"
              ><svg v-if="selectedIds.has(exp.id)" width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 6L5 9L10 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg></div>
            </div>

            <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-lg bg-[var(--color-primary-light)] flex items-center justify-center text-xs sm:text-sm font-bold text-[var(--color-primary)] shrink-0 overflow-hidden">
              <IconDisplay v-if="getCategory(exp.categoryId)?.icon" :icon="getCategory(exp.categoryId)!.icon" :size="18" />
              <span v-else>{{ exp.name.charAt(0).toUpperCase() }}</span>
            </div>

            <div class="min-w-0 flex-1">
              <p class="text-xs sm:text-sm font-medium text-[var(--color-text-primary)] truncate">{{ exp.name }}</p>
              <div class="flex items-center gap-1.5 flex-wrap">
                <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)]">{{ getCategory(exp.categoryId)?.name || '' }}</span>
                <span v-for="tag in (exp.tags || []).slice(0, 3)" :key="tag" class="hidden sm:inline-flex items-center px-1.5 py-0 rounded text-[9px] font-medium bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] border border-[var(--color-border)]">#{{ tag }}</span>
              </div>
            </div>

            <div class="text-right shrink-0">
              <p class="text-xs sm:text-sm font-medium text-[var(--color-text-primary)]">
                <span class="hidden sm:inline">{{ formatDate(exp.date) }}</span>
                <span class="sm:hidden text-[10px] text-[var(--color-text-muted)]">{{ formatDate(exp.date) }}</span>
              </p>
            </div>

            <div class="text-right shrink-0">
              <p class="text-xs sm:text-sm font-semibold text-[var(--color-text-primary)]">{{ formatAmount(exp.amount, exp.currencyId) }}</p>
              <div v-if="getConvertedAmounts(exp.amount, exp.currencyId).length > 0" class="mt-0.5 space-y-0">
                <p v-for="cv in getConvertedAmounts(exp.amount, exp.currencyId)" :key="cv.currency.id"
                  class="text-[10px] text-[var(--color-text-muted)] tabular-nums">≈ {{ fmtCur(cv.amount, cv.currency) }}</p>
              </div>
            </div>

            <div class="shrink-0 hidden sm:block" :title="getPaymentMethod(exp.paymentMethodId)?.name">
              <IconDisplay v-if="getPaymentMethod(exp.paymentMethodId)" :icon="getPaymentMethod(exp.paymentMethodId)!.icon" :size="22" />
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- Pagination -->
    <div v-if="totalPages > 1" class="flex items-center justify-center gap-1 mt-4">
      <button @click="goPage(expsStore.currentPage - 1)" :disabled="expsStore.currentPage <= 1"
        class="p-1.5 rounded-lg border border-[var(--color-border)] text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] disabled:opacity-30 transition-colors">
        <ChevronLeft :size="16" />
      </button>
      <template v-for="p in totalPages" :key="p">
        <button v-if="p === 1 || p === totalPages || (p >= expsStore.currentPage - 1 && p <= expsStore.currentPage + 1)"
          @click="goPage(p)"
          class="min-w-[2rem] h-8 rounded-lg text-xs font-medium transition-colors"
          :class="p === expsStore.currentPage ? 'bg-[var(--color-primary)] text-white' : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]'"
        >{{ p }}</button>
        <span v-else-if="p === expsStore.currentPage - 2 || p === expsStore.currentPage + 2" class="text-xs text-[var(--color-text-muted)]">…</span>
      </template>
      <button @click="goPage(expsStore.currentPage + 1)" :disabled="expsStore.currentPage >= totalPages"
        class="p-1.5 rounded-lg border border-[var(--color-border)] text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] disabled:opacity-30 transition-colors">
        <ChevronRight :size="16" />
      </button>
    </div>

    <ExpenseDetail :show="showDetail" :expense="detailExp" @close="showDetail = false" @edit="onDetailEdit" @delete="onDetailDelete" @openUrl="onDetailOpenUrl" />
    <ExpenseForm :show="showForm" :editExpense="editingExpense" @close="showForm = false" @saved="onSaved" />

    <!-- Delete Confirmation -->
    <Teleport to="body">
      <Transition enter-active-class="transition ease-out duration-200" enter-from-class="opacity-0" enter-to-class="opacity-100" leave-active-class="transition ease-in duration-150" leave-from-class="opacity-100" leave-to-class="opacity-0">
        <div v-if="deleteConfirmId" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
          <div class="absolute inset-0 bg-black/50" @click="cancelDelete" />
          <div class="relative bg-[var(--color-surface)] w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-sm p-4 sm:p-6">
            <div class="flex items-center gap-3 mb-3 sm:mb-4">
              <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center shrink-0"><AlertTriangle :size="18" class="text-red-500" /></div>
              <h3 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ t('delete') }}</h3>
            </div>
            <p class="text-xs sm:text-sm text-[var(--color-text-secondary)] mb-4 sm:mb-6">{{ t('confirm_delete_expense') }}</p>
            <div class="flex justify-end gap-2 sm:gap-3">
              <button @click="cancelDelete" class="px-3 sm:px-4 py-2 rounded-lg border border-[var(--color-border)] text-xs sm:text-sm font-medium text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]">{{ t('cancel') }}</button>
              <button @click="confirmDelete" class="px-3 sm:px-4 py-2 rounded-lg bg-red-600 text-white text-xs sm:text-sm font-medium hover:bg-red-700">{{ t('delete') }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Batch Delete Confirmation -->
    <Teleport to="body">
      <Transition enter-active-class="transition ease-out duration-200" enter-from-class="opacity-0" enter-to-class="opacity-100" leave-active-class="transition ease-in duration-150" leave-from-class="opacity-100" leave-to-class="opacity-0">
        <div v-if="batchDeleteConfirmIds.length > 0" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
          <div class="absolute inset-0 bg-black/50" @click="batchDeleteConfirmIds = []" />
          <div class="relative bg-[var(--color-surface)] w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-sm p-4 sm:p-6">
            <div class="flex items-center gap-3 mb-3 sm:mb-4">
              <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center shrink-0"><AlertTriangle :size="18" class="text-red-500" /></div>
              <h3 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ t('batch_delete') }}</h3>
            </div>
            <p class="text-xs sm:text-sm text-[var(--color-text-secondary)] mb-4 sm:mb-6">{{ t('batch_confirm_delete_expenses').replace('{count}', String(batchDeleteConfirmIds.length)) }}</p>
            <div class="flex justify-end gap-2 sm:gap-3">
              <button @click="batchDeleteConfirmIds = []" class="px-3 sm:px-4 py-2 rounded-lg border border-[var(--color-border)] text-xs sm:text-sm font-medium text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]">{{ t('cancel') }}</button>
              <button @click="confirmBatchDelete" class="px-3 sm:px-4 py-2 rounded-lg bg-red-600 text-white text-xs sm:text-sm font-medium hover:bg-red-700">{{ t('delete') }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </div>
</template>
