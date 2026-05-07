<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useHeaderActions } from "@/composables/useHeaderActions";
import {
  type SubscriptionFilter,
} from "@/services/subscriptionsClient";
import { storeToRefs } from "pinia";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { useToast } from "@/composables/useToast";
import { useScrollLock } from "@/composables/useScrollLock";
import { useClipboard } from "@/composables/useClipboard";
import { type Subscription, type Settings, type SubscriptionListItem } from "@/schemas/appData";
import SubscriptionForm from "@/components/subscriptions/SubscriptionForm.vue";
import SubscriptionDetail from "@/components/subscriptions/SubscriptionDetail.vue";
import Toast from "@/components/ui/Toast.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import UniversalListRow from "@/components/ui/UniversalListRow.vue";
import { Plus, Search, Pencil, Trash2, Copy, RefreshCw, ExternalLink, CreditCard, AlertTriangle, Star, CheckSquare, Square, Hash, CircleDollarSign, LayoutList, LayoutGrid, Rows3, FolderOpen, Filter } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Menu } from "@tauri-apps/api/menu";
import type { MenuItemOptions, PredefinedMenuItemOptions } from "@tauri-apps/api/menu";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { useSubscriptionsStore } from "@/stores/subscriptionsStore";
import { useNowStore } from "@/stores/nowStore";
import { ui } from "@/lib/tv";
import { formatErrorForToast } from "@/utils/formatError";

const route = useRoute();
const vueRouter = useRouter();
const { t } = useI18n();
const { setActions, clearActions } = useHeaderActions();
const { fmt } = useCurrencyFormat();
const { fmtDateMedium } = useLocaleFormat();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();
const { copyToClipboard } = useClipboard();
const nowStore = useNowStore();
const { now } = storeToRefs(nowStore);
const showFilters = ref(false);
const metaStore = useAppMetaStore();
const subscriptionsStore = useSubscriptionsStore();
const metaRefs = storeToRefs(metaStore);
const settings = computed(() => metaRefs.settings.value);
const categories = computed(() => metaRefs.categories.value ?? []);
const paymentMethods = computed(() => metaRefs.paymentMethods.value ?? []);
const tags = computed(() => metaRefs.tags.value ?? []);
const currencies = computed(() => metaRefs.currencies.value ?? []);
const household = computed(() => metaRefs.household.value ?? []);
const subscriptions = computed(() => subscriptionsStore.items);

// Register header action
function updateHeaderActions() {
  const viewIcon = viewMode.value === "compact" ? Rows3 : viewMode.value === "expanded" ? LayoutGrid : LayoutList;
  const nextViewMode = viewMode.value === "compact" ? "default" : viewMode.value === "default" ? "expanded" : "compact";
  const currentViewTitle = viewMode.value === "compact" ? t("view_compact") : viewMode.value === "expanded" ? t("view_expanded") : t("view_default");
  const nextViewTitle = nextViewMode === "compact" ? t("view_compact") : nextViewMode === "expanded" ? t("view_expanded") : t("view_default");

  setActions([
    { id: "toggle-sub-filters", icon: Filter, title: showFilters.value ? `${t("filter")} ✓` : `${t("filter")} ✕`, onClick: () => { showFilters.value = !showFilters.value; }, style: showFilters.value ? "warning" : "neutral" },
    { id: "cycle-sub-view", icon: viewIcon, title: `${currentViewTitle} → ${nextViewTitle}`, onClick: () => setViewMode(nextViewMode), style: "accent" },
    { id: "sub-selection-mode", icon: CheckSquare, title: selectionMode.value ? `${t("select")} ✓` : `${t("select")} ✕`, onClick: toggleSelectionMode, style: selectionMode.value ? "success" : "neutral" },
    { id: "add-sub", icon: Plus, title: t("new_subscription"), onClick: openAdd, style: "primary" },
  ]);
}

onMounted(() => {
  loadInitial().then(() => {
    nowStore.ensureStarted();
    updateHeaderActions();
    handleSubQueryParam();
    fetchFilteredSubs();
  });
});
onUnmounted(() => {
  clearActions();
});

// Watch for route query changes (e.g. from tray navigation)
watch(() => route.query.sub, () => handleSubQueryParam());

function handleSubQueryParam() {
  const subId = route.query.sub as string | undefined;
  if (subId) {
    const sub = subscriptions.value.find((s) => s.id === subId);
    if (sub) {
      openDetail(sub);
    }
    // Clear the query to avoid re-opening on route re-render
    vueRouter.replace({ query: {} });
  }
}

// Form
const showForm = ref(false);
const editingSub = ref<Subscription | null>(null);

// Detail panel
const showDetail = ref(false);
const detailSubId = ref<string | null>(null);
const detailSub = computed(() => {
  if (!detailSubId.value) return null;
  return filteredSubscriptions.value.find((s) => s.id === detailSubId.value)
    ?? subscriptions.value.find((s) => s.id === detailSubId.value)
    ?? null;
});

// Search & Filter
const searchQuery = ref("");
const sortBy = ref<string>("next_payment");
const filterCategory = ref<string>("");
const filterPayment = ref<string>("");
const filterState = ref<string>("");
const filterTag = ref<string>("");

// View mode & grouping
const viewMode = computed(() => settings.value?.subscriptionViewMode || "default");
const groupBy = computed(() => settings.value?.subscriptionGroupBy || "none");

function setViewMode(mode: "default" | "compact" | "expanded") {
  updateSettings({ subscriptionViewMode: mode });
}
watch([showFilters, viewMode], updateHeaderActions);

function setGroupBy(g: "none" | "category" | "payment_method") {
  updateSettings({ subscriptionGroupBy: g });
}

// Grouped subscriptions
interface SubGroup {
  key: string;
  label: string;
  subs: SubscriptionListItem[];
}

const groupedSubscriptions = computed<SubGroup[]>(() => {
  const subs = filteredSubscriptions.value;
  if (groupBy.value === "none") return [{ key: "__all", label: "", subs }];

  const map = new Map<string, { label: string; subs: SubscriptionListItem[] }>();

  for (const s of subs) {
    let key: string;
    let label: string;
    if (groupBy.value === "category") {
      key = s.categoryId;
      label = getCategoryName(s.categoryId) || t("ungrouped");
    } else {
      key = s.paymentMethodId;
      const pm = getPaymentMethod(s.paymentMethodId);
      label = pm?.name || t("ungrouped");
    }
    if (!map.has(key)) map.set(key, { label, subs: [] });
    map.get(key)!.subs.push(s);
  }

  return Array.from(map.entries()).map(([key, val]) => ({
    key,
    label: val.label,
    subs: val.subs,
  }));
});

const groupByOptions = computed<SelectOption[]>(() => [
  { value: "none", label: t("group_none") },
  { value: "category", label: t("group_category") },
  { value: "payment_method", label: t("group_payment_method") },
]);

// Batch selection
const selectionMode = ref(false);
const selectedIds = ref<Set<string>>(new Set());
const showBatchCategoryModal = ref(false);
const batchCategoryId = ref("");
const batchDeleteConfirmIds = ref<string[]>([]);
const deleteConfirmId = ref<string | null>(null);
const hasBlockingOverlay = computed(() =>
  Boolean(deleteConfirmId.value)
  || batchDeleteConfirmIds.value.length > 0
  || showBatchCategoryModal.value,
);
useScrollLock(hasBlockingOverlay);
watch(selectionMode, updateHeaderActions);

function toggleSelectionMode() {
  selectionMode.value = !selectionMode.value;
  if (!selectionMode.value) selectedIds.value = new Set();
}

function toggleSelected(id: string) {
  const s = new Set(selectedIds.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  selectedIds.value = s;
}

function selectAll() {
  selectedIds.value = new Set(filteredSubscriptions.value.map((s) => s.id));
}

function deselectAll() {
  selectedIds.value = new Set();
}

function batchDeleteSelected() {
  batchDeleteConfirmIds.value = [...selectedIds.value];
}

async function confirmBatchDelete() {
  if (batchDeleteConfirmIds.value.length > 0) {
    await subscriptionsStore.batchDelete(batchDeleteConfirmIds.value);
    toast(t("batch_deleted").replace("{count}", String(batchDeleteConfirmIds.value.length)));
    selectedIds.value = new Set();
    selectionMode.value = false;
    fetchFilteredSubs();
  }
  batchDeleteConfirmIds.value = [];
}

async function batchDeactivate() {
  const ids = [...selectedIds.value];
  await subscriptionsStore.batchSetInactive(ids, true);
  toast(t("batch_updated").replace("{count}", String(ids.length)));
  selectedIds.value = new Set();
  fetchFilteredSubs();
}

async function batchActivate() {
  const ids = [...selectedIds.value];
  await subscriptionsStore.batchSetInactive(ids, false);
  toast(t("batch_updated").replace("{count}", String(ids.length)));
  selectedIds.value = new Set();
  fetchFilteredSubs();
}

function openBatchCategoryModal() {
  batchCategoryId.value = settings.value?.defaultCategoryId || "cat-1";
  showBatchCategoryModal.value = true;
}

async function confirmBatchCategory() {
  const ids = [...selectedIds.value];
  await subscriptionsStore.batchSetCategory(ids, batchCategoryId.value);
  toast(t("batch_updated").replace("{count}", String(ids.length)));
  showBatchCategoryModal.value = false;
  selectedIds.value = new Set();
  fetchFilteredSubs();
}

// Native context menu
async function showContextMenu(sub: Subscription, event: MouseEvent) {
  if (selectionMode.value) return;

  const items: (MenuItemOptions | PredefinedMenuItemOptions)[] = [
    { id: "favorite", text: sub.favorite ? t("remove_from_favorites") : t("add_to_favorites"), action: async () => { await handleToggleFavorite(sub.id); } },
  ];
  if (!sub.inactive) {
    items.push({ id: "record_payment", text: t("record_payment"), action: () => handleRecordPayment(sub.id) });
  }
  items.push(
    { id: "edit", text: t("edit_subscription"), action: () => openEdit(sub) },
    { id: "clone", text: t("clone"), action: () => handleClone(sub.id) },
    { id: "renew", text: t("renew"), action: () => handleRenew(sub.id) },
  );
  if (sub.url) {
    items.push({ id: "url", text: t("url"), action: () => handleOpenUrl(sub.url) });
  }
  items.push(
    { id: "copy", text: t("copy"), action: () => handleCopyName(sub) },
    { item: "Separator" },
    { id: "delete", text: t("delete"), action: () => requestDelete(sub.id) },
  );

  try {
    const menu = await Menu.new({ items });
    await menu.popup();
  } catch (e) {
    console.error("[SubscriptionsPage] context menu failed", e);
    toast(formatErrorForToast(e, t), "error");
  }
}

// Filtered subscriptions from backend DTO query
const filteredSubscriptions = computed<SubscriptionListItem[]>(
  () => subscriptionsStore.items,
);
const totalSubscriptionsAmount = computed(() =>
  filteredSubscriptions.value.reduce((sum, sub) => {
    const value = Number(sub.monthlyPrice ?? sub.price ?? 0);
    return sum + (Number.isFinite(value) ? value : 0);
  }, 0),
);
const subscriptionsSummaryCurrencyId = computed(() => settings.value?.mainCurrencyId || "cur-1");

function buildSubFilter(): SubscriptionFilter {
  const f: SubscriptionFilter = {
    sortBy: (sortBy.value || "next_payment") as SubscriptionFilter["sortBy"],
    disabledToBottom: settings.value?.disabledToBottom,
    hideDisabled: settings.value?.hideDisabled,
  };
  if (filterState.value === "active" || filterState.value === "inactive") {
    f.state = filterState.value;
  }
  if (filterCategory.value) f.categoryId = filterCategory.value;
  if (filterPayment.value) f.paymentMethodId = filterPayment.value;
  if (filterTag.value) f.tag = filterTag.value;
  if (searchQuery.value) f.search = searchQuery.value;
  return f;
}

async function fetchFilteredSubs() {
  try {
    await subscriptionsStore.loadBrief(buildSubFilter());
  } catch (e) {
    console.error("[SubscriptionsPage] fetchFilteredSubs failed", {
      filter: buildSubFilter(),
      error: e,
    });
    toast(formatErrorForToast(e, t), "error");
  }
}

let subDebounce: ReturnType<typeof setTimeout> | null = null;
function fetchFilteredSubsDebounced() {
  if (subDebounce) clearTimeout(subDebounce);
  subDebounce = setTimeout(fetchFilteredSubs, 200);
}

watch([searchQuery, filterCategory, filterPayment, filterTag, filterState, sortBy], fetchFilteredSubsDebounced);

// Tag filter options
const tagFilterOptions = computed<SelectOption[]>(() => [
  { value: "", label: t("filter_by_tag") },
  ...tags.value.map((tag) => ({ value: tag.name, label: tag.name })),
]);

function getCategoryName(id: string): string {
  return categories.value.find((c) => c.id === id)?.name || "";
}

function getCategoryIcon(id: string): string {
  return categories.value.find((c) => c.id === id)?.icon || "";
}

function getPaymentMethod(id: string) {
  return paymentMethods.value.find((p) => p.id === id);
}

function getSubDaysLeft(sub: SubscriptionListItem): number {
  return Number(sub.daysLeft ?? 0);
}

function isSubOverdue(sub: SubscriptionListItem): boolean {
  return Boolean(sub.overdue);
}

function getDaysBadgeState(sub: SubscriptionListItem): "days-badge--danger" | "days-badge--warn" | "days-badge--normal" {
  const days = getSubDaysLeft(sub);
  if (days <= 3) return "days-badge--danger";
  if (days <= 7) return "days-badge--warn";
  return "days-badge--normal";
}

function billingCycleText(cycle: number, frequency: number): string {
  switch (cycle) {
    case 1: return frequency === 1 ? t("daily") : `${frequency} ${t("days")}`;
    case 2: return frequency === 1 ? t("weekly") : `${frequency} ${t("weeks")}`;
    case 3: return frequency === 1 ? t("monthly") : `${frequency} ${t("months")}`;
    case 4: return frequency === 1 ? t("yearly") : `${frequency} ${t("years")}`;
    default: return "";
  }
}

const formatDate = fmtDateMedium;
const subscriptionLookupData = computed(() => {
  if (!settings.value) return null;
  return {
    categories: categories.value,
    paymentMethods: paymentMethods.value,
    household: household.value,
    currencies: currencies.value,
    settings: settings.value,
  };
});
const subscriptionFormLookupData = computed(() => {
  if (!settings.value) return null;
  return {
    settings: settings.value,
    currencies: currencies.value,
    paymentMethods: paymentMethods.value,
    household: household.value,
    categories: categories.value,
    tags: tags.value,
  };
});

// Actions
function openAdd() {
  editingSub.value = null;
  showForm.value = true;
}

function openDetail(sub: Subscription) {
  detailSubId.value = sub.id;
  showDetail.value = true;
}

function openEdit(sub: Subscription) {
  showDetail.value = false;
  detailSubId.value = null;
  editingSub.value = sub;
  showForm.value = true;
}

function requestDelete(id: string) {
  showDetail.value = false;
  detailSubId.value = null;
  deleteConfirmId.value = id;
}

async function confirmDelete() {
  if (deleteConfirmId.value) {
    await subscriptionsStore.deleteById(deleteConfirmId.value);
    toast(t("subscription_deleted"));
    fetchFilteredSubs();
  }
  deleteConfirmId.value = null;
}

function cancelDelete() { deleteConfirmId.value = null; }

async function handleClone(id: string) {
  showDetail.value = false;
  await subscriptionsStore.cloneById(id);
  toast(t("subscription_added"));
  fetchFilteredSubs();
}

async function handleRenew(id: string) {
  showDetail.value = false;
  await subscriptionsStore.recordPayment(id);
  toast(t("payment_recorded"));
  fetchFilteredSubs();
}

async function handleRecordPayment(id: string) {
  await subscriptionsStore.recordPayment(id);
  toast(t("payment_recorded"));
  fetchFilteredSubs();
}

async function handleOpenUrl(url: string) {
  const fullUrl = url.startsWith("http") ? url : `https://${url}`;
  try {
    await openUrl(fullUrl);
  } catch (e) {
    console.error("[SubscriptionsPage] failed to open URL", { url: fullUrl, error: e });
    toast(formatErrorForToast(e, t), "error");
  }
}

async function handleCopyName(sub: Subscription) {
  const copied = await copyToClipboard(sub.name);
  if (copied) toast(t("copied_to_clipboard"));
  else toast(t("clipboard_copy_failed"), "error");
}

async function handleToggleFavorite(id: string) {
  await subscriptionsStore.toggleFavorite(id);
  await fetchFilteredSubs();
}

async function onSaved() {
  toast(editingSub.value ? t("subscription_updated") : t("subscription_added"));
  await fetchFilteredSubs();
}

// Filter options
const sortOptions = computed<SelectOption[]>(() => [
  { value: "next_payment", label: t("next_payment") },
  { value: "name", label: t("subscription_name") },
  { value: "price", label: t("price") },
]);

const categoryFilterOptions = computed<SelectOption[]>(() => [
  { value: "", label: t("category") },
  ...[...categories.value].sort((a, b) => a.sortOrder - b.sortOrder).map((c) => ({ value: c.id, label: c.name, icon: c.icon || undefined })),
]);

const paymentFilterOptions = computed<SelectOption[]>(() => [
  { value: "", label: t("payment_method") },
  ...paymentMethods.value.filter((pm) => pm.enabled).map((pm) => ({ value: pm.id, label: pm.name, icon: pm.icon })),
]);

const stateFilterOptions = computed<SelectOption[]>(() => [
  { value: "", label: t("filter") },
  { value: "active", label: t("active_subscriptions") },
  { value: "inactive", label: t("inactive_subscriptions") },
]);

// Detail panel event handlers
function onDetailEdit(sub: Subscription) { openEdit(sub); }
function onDetailClone(id: string) { handleClone(id); }
function onDetailRenew(id: string) { handleRenew(id); }
function onDetailDelete(id: string) { requestDelete(id); }
function onDetailOpenUrl(url: string) { handleOpenUrl(url); }
function onDetailRecordPayment(id: string) { handleRecordPayment(id); }
async function onDetailToggleFavorite(id: string) {
  await subscriptionsStore.toggleFavorite(id);
  fetchFilteredSubs();
}

async function updateSettings(updates: Partial<Settings>) {
  if (!settings.value) return;
  await metaStore.updateSettings({ ...settings.value, ...updates });
}

async function loadInitial() {
  await metaStore.ensureLoaded();
  await subscriptionsStore.loadBrief();
}

</script>

<template>
  <div class="max-w-5xl mx-auto">
    <!-- Filters -->
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 -translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-2"
    >
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
          <div class="w-28 shrink-0">
            <AppSelect v-model="sortBy" :options="sortOptions" size="sm" />
          </div>
          <div class="w-28 shrink-0">
            <AppSelect v-model="filterCategory" :options="categoryFilterOptions" size="sm" />
          </div>
          <div class="w-28 shrink-0">
            <AppSelect v-model="filterPayment" :options="paymentFilterOptions" size="sm" />
          </div>
          <div class="w-28 shrink-0">
            <AppSelect v-model="filterState" :options="stateFilterOptions" size="sm" />
          </div>
          <div v-if="tags.length > 0" class="w-28 shrink-0" >
            <AppSelect v-model="filterTag" :options="tagFilterOptions" size="sm" />
          </div>
          <div class="w-32 shrink-0">
            <AppSelect :modelValue="groupBy" @update:modelValue="(v: string | number) => setGroupBy(String(v) as 'none' | 'category' | 'payment_method')" :options="groupByOptions" size="sm" />
          </div>
        </div>
      </div>
    </Transition>

    <!-- Batch toolbar -->
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 -translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-2"
    >
      <div v-if="selectionMode" class="flex items-center gap-1.5 sm:gap-2 mb-3 px-2 sm:px-3 py-2 rounded-lg bg-surface-secondary border border-border overflow-x-auto">
        <span class="text-xs font-medium text-text-primary">{{ selectedIds.size }} {{ t('selected_count') }}</span>
        <button @click="selectAll" class="p-1 rounded text-text-secondary hover:bg-surface transition-colors" :title="t('select_all')">
          <CheckSquare :size="13" />
        </button>
        <button @click="deselectAll" class="p-1 rounded text-text-muted hover:bg-surface-hover transition-colors" :title="t('deselect_all')">
          <Square :size="13" />
        </button>
        <div class="flex-1" />
        <button
          @click="batchActivate"
          :disabled="selectedIds.size === 0"
          class="px-2.5 py-1 rounded-md text-[11px] font-medium border border-border text-text-secondary hover:bg-surface disabled:opacity-30 transition-colors"
        >{{ t('batch_activate') }}</button>
        <button
          @click="batchDeactivate"
          :disabled="selectedIds.size === 0"
          class="px-2.5 py-1 rounded-md text-[11px] font-medium border border-border text-text-secondary hover:bg-surface disabled:opacity-30 transition-colors"
        >{{ t('batch_deactivate') }}</button>
        <button
          @click="openBatchCategoryModal"
          :disabled="selectedIds.size === 0"
          class="px-2.5 py-1 rounded-md text-[11px] font-medium border border-border text-text-secondary hover:bg-surface disabled:opacity-30 transition-colors"
        >{{ t('batch_change_category') }}</button>
        <button
          @click="batchDeleteSelected"
          :disabled="selectedIds.size === 0"
          class="px-2.5 py-1 rounded-md text-[11px] font-medium border border-border text-text-secondary hover:bg-surface disabled:opacity-30 transition-colors"
        >{{ t('batch_delete') }}</button>
      </div>
    </Transition>

    <!-- Empty state -->
    <div v-if="filteredSubscriptions.length === 0" class="text-center py-16">
      <div class="w-20 h-20 mx-auto mb-4 rounded-full bg-surface-hover flex items-center justify-center">
        <CreditCard :size="36" class="text-text-muted" />
      </div>
      <p class="text-text-muted mb-4">{{ t('no_subscriptions_yet') }}</p>
      <button
        @click="openAdd"
        class="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover"
      >
        <Plus :size="18" />
        {{ t('add_first_subscription') }}
      </button>
    </div>

    <!-- Subscription list -->
    <div v-else>
      <!-- Summary bar -->
      <div class="flex items-center justify-between px-3 py-2 mb-3 rounded-lg bg-surface border border-border">
        <div class="flex items-center gap-2 text-xs text-text-muted">
          <CircleDollarSign :size="14" />
          <span>{{ filteredSubscriptions.length }} {{ t('subscriptions').toLowerCase() }}</span>
        </div>
        <div class="text-sm font-semibold text-text-primary tabular-nums">
          {{ fmt(totalSubscriptionsAmount, subscriptionsSummaryCurrencyId) }}
        </div>
      </div>

      <template v-for="group in groupedSubscriptions" :key="group.key">
        <!-- Group header -->
        <div v-if="groupBy !== 'none' && group.label" class="flex items-center gap-2 mb-2 mt-4 first:mt-0">
          <IconDisplay v-if="groupBy === 'category' && getCategoryIcon(group.key)" :icon="getCategoryIcon(group.key)" :size="16" />
          <FolderOpen v-else :size="14" class="text-primary" />
          <span class="text-xs font-semibold text-text-primary uppercase tracking-wide">{{ group.label }}</span>
          <span class="text-[10px] text-text-muted">({{ group.subs.length }})</span>
          <div class="flex-1 h-px bg-border" />
        </div>

        <!-- Grid for expanded mode -->
        <div :class="viewMode === 'expanded' ? 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2 sm:gap-3 mb-3' : 'space-y-1.5 sm:space-y-2 mb-3'">
          <div
            v-for="sub in group.subs"
            :key="sub.id"
            class="bg-surface rounded-xl border overflow-hidden transition-colors"
            :class="[
              sub.inactive ? 'opacity-50' : '',
              selectedIds.has(sub.id) ? 'border-primary ring-1 ring-primary/30' : 'border-border',
            ]"
          >
            <UniversalListRow
              :mode="viewMode as 'compact' | 'default' | 'expanded'"
              @click="selectionMode ? toggleSelected(sub.id) : openDetail(sub)"
              @contextmenu="showContextMenu(sub, $event)"
            >
              <template #selection>
                <div
                  v-if="selectionMode && viewMode !== 'expanded'"
                  class="shrink-0"
                  @click.stop="toggleSelected(sub.id)"
                >
                  <div
                    class="rounded border-2 flex items-center justify-center transition-colors cursor-pointer"
                    :class="[
                      viewMode === 'compact' ? 'w-4 h-4' : 'w-5 h-5',
                      selectedIds.has(sub.id) ? 'bg-primary border-primary text-white' : 'border-border hover:border-primary',
                    ]"
                  >
                    <svg v-if="selectedIds.has(sub.id)" :width="viewMode === 'compact' ? 10 : 12" :height="viewMode === 'compact' ? 10 : 12" viewBox="0 0 12 12" fill="none"><path d="M2 6L5 9L10 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                  </div>
                </div>
              </template>
              <template #leading>
                <template v-if="viewMode === 'compact'">
                  <Tooltip v-if="!selectionMode" :text="sub.favorite ? t('remove_from_favorites') : t('add_to_favorites')">
                    <button @click.stop="handleToggleFavorite(sub.id)" class="shrink-0 p-0.5 rounded transition-colors" :class="sub.favorite ? 'text-yellow-500' : 'text-border hover:text-yellow-400'">
                      <Star :size="12" :fill="sub.favorite ? 'currentColor' : 'none'" />
                    </button>
                  </Tooltip>
                  <div class="w-6 h-6 rounded bg-primary-light border border-border flex items-center justify-center text-[10px] font-bold text-primary shrink-0 overflow-hidden">
                    <img v-if="sub.logo" :src="sub.logo" class="w-full h-full object-contain" />
                    <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
                  </div>
                </template>
                <template v-else-if="viewMode === 'default'">
                  <Tooltip v-if="!selectionMode" :text="sub.favorite ? t('remove_from_favorites') : t('add_to_favorites')">
                    <button @click.stop="handleToggleFavorite(sub.id)" class="shrink-0 p-0.5 rounded transition-colors hidden sm:block" :class="sub.favorite ? 'text-yellow-500' : 'text-border hover:text-yellow-400'">
                      <Star :size="16" :fill="sub.favorite ? 'currentColor' : 'none'" />
                    </button>
                  </Tooltip>
                  <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-lg bg-primary-light border border-border flex items-center justify-center text-xs sm:text-sm font-bold text-primary shrink-0 overflow-hidden">
                    <img v-if="sub.logo" :src="sub.logo" class="w-full h-full object-contain" />
                    <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
                  </div>
                </template>
              </template>
              <template #main>
                <p v-if="viewMode === 'compact'" class="text-xs font-medium text-text-primary truncate">{{ sub.name }}</p>
                <div v-else-if="viewMode === 'default'" class="min-w-0">
                  <p class="text-xs sm:text-sm font-medium text-text-primary truncate">{{ sub.name }}</p>
                  <div class="flex items-center gap-1.5 flex-wrap">
                    <span class="text-[10px] sm:text-xs text-text-muted">
                      {{ billingCycleText(sub.cycle, sub.frequency) }}
                      <span v-if="!sub.autoRenew" class="ml-1 text-orange-500">({{ t('manual_renewal') }})</span>
                    </span>
                    <span v-for="tag in (sub.tags || []).slice(0, 3)" :key="tag" class="hidden sm:inline-flex items-center px-1.5 py-0 rounded text-[9px] font-medium bg-surface-hover text-text-muted border border-border">#{{ tag }}</span>
                    <span v-if="(sub.tags || []).length > 3" class="hidden sm:inline text-[9px] text-text-muted">+{{ sub.tags.length - 3 }}</span>
                  </div>
                </div>
              </template>
              <template #meta>
                <span
                  v-if="viewMode === 'compact' && !sub.inactive"
                  class="days-badge inline-flex items-center justify-center px-1.5 py-0.5 rounded text-[10px] font-bold leading-none shrink-0 w-[56px] tabular-nums"
                  :class="getDaysBadgeState(sub)"
                >
                  {{ getSubDaysLeft(sub) }}{{ t('days_short') }}
                </span>
                <div v-else-if="viewMode === 'default'" class="text-right shrink-0 w-[124px] sm:w-[142px]">
                  <div class="flex items-center gap-1 sm:gap-1.5 justify-end">
                    <p class="text-xs sm:text-sm font-medium tabular-nums" :class="isSubOverdue(sub) ? 'text-red-500' : 'text-text-primary'">
                      <span class="hidden sm:inline tabular-nums whitespace-nowrap">{{ formatDate(sub.nextPayment) }}</span>
                    </p>
                    <span
                      v-if="!sub.inactive"
                      class="days-badge inline-flex items-center justify-center px-1.5 py-0.5 rounded text-[10px] font-bold leading-none w-[56px] tabular-nums"
                      :class="getDaysBadgeState(sub)"
                    >
                      {{ getSubDaysLeft(sub) }}{{ t('days_short') }}
                    </span>
                  </div>
                </div>
              </template>
              <template #value>
                <p v-if="viewMode === 'compact'" class="text-xs font-semibold text-text-primary shrink-0 w-[96px] text-right tabular-nums whitespace-nowrap">{{ fmt(sub.price, sub.currencyId) }}</p>
                <div v-else-if="viewMode === 'default'" class="text-right shrink-0 w-[96px] sm:w-[112px]">
                  <p class="text-xs sm:text-sm font-semibold text-text-primary tabular-nums whitespace-nowrap">{{ fmt(sub.price, sub.currencyId) }}</p>
                </div>
              </template>
              <template #trailing>
                <div v-if="viewMode === 'default'" class="shrink-0 hidden sm:block" :title="getPaymentMethod(sub.paymentMethodId)?.name">
                  <IconDisplay :icon="getPaymentMethod(sub.paymentMethodId)?.icon || '💳'" :size="22" />
                </div>
              </template>
              <template #expanded>
                <div class="flex items-start gap-3 mb-3">
                  <div v-if="selectionMode" class="shrink-0 mt-1" @click.stop="toggleSelected(sub.id)">
                    <div class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors cursor-pointer"
                      :class="selectedIds.has(sub.id) ? 'bg-primary border-primary text-white' : 'border-border hover:border-primary'"
                    ><svg v-if="selectedIds.has(sub.id)" width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 6L5 9L10 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg></div>
                  </div>
                  <div class="w-12 h-12 rounded-lg bg-primary-light border border-border flex items-center justify-center text-lg font-bold text-primary shrink-0 overflow-hidden">
                    <img v-if="sub.logo" :src="sub.logo" class="w-full h-full object-contain" />
                    <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
                  </div>
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <p class="text-sm font-semibold text-text-primary truncate">{{ sub.name }}</p>
                      <Tooltip v-if="!selectionMode" :text="sub.favorite ? t('remove_from_favorites') : t('add_to_favorites')">
                        <button @click.stop="handleToggleFavorite(sub.id)" class="shrink-0 p-0.5 rounded transition-colors" :class="sub.favorite ? 'text-yellow-500' : 'text-border hover:text-yellow-400'">
                          <Star :size="14" :fill="sub.favorite ? 'currentColor' : 'none'" />
                        </button>
                      </Tooltip>
                    </div>
                    <p class="text-xs text-text-muted flex items-center gap-1">
                      <IconDisplay v-if="getCategoryIcon(sub.categoryId)" :icon="getCategoryIcon(sub.categoryId)" :size="12" />
                      {{ getCategoryName(sub.categoryId) }}
                    </p>
                  </div>
                </div>

                <div class="flex items-end justify-between">
                  <div>
                    <p class="text-lg font-bold text-text-primary">{{ fmt(sub.price, sub.currencyId) }}</p>
                    <p class="text-[10px] text-text-muted">{{ billingCycleText(sub.cycle, sub.frequency) }}</p>
                  </div>
                  <div class="text-right w-[136px]">
                    <p class="text-xs font-medium tabular-nums whitespace-nowrap" :class="isSubOverdue(sub) ? 'text-red-500' : 'text-text-primary'">{{ formatDate(sub.nextPayment) }}</p>
                    <span
                      v-if="!sub.inactive"
                      class="days-badge inline-flex items-center justify-center px-1.5 py-0.5 rounded text-[10px] font-bold leading-none mt-0.5 w-[56px] tabular-nums"
                      :class="getDaysBadgeState(sub)"
                    >
                      {{ getSubDaysLeft(sub) }}{{ t('days_short') }}
                    </span>
                  </div>
                </div>

                <!-- Tags in expanded mode -->
                <div v-if="(sub.tags || []).length > 0" class="flex items-center gap-1 mt-2 flex-wrap">
                  <span v-for="tag in sub.tags" :key="tag" class="inline-flex items-center px-1.5 py-0 rounded text-[9px] font-medium bg-surface-hover text-text-muted border border-border">#{{ tag }}</span>
                </div>

                <!-- Payment method -->
                <div class="flex items-center gap-1.5 mt-2">
                  <IconDisplay :icon="getPaymentMethod(sub.paymentMethodId)?.icon || '💳'" :size="16" />
                  <span class="text-[10px] text-text-muted">{{ getPaymentMethod(sub.paymentMethodId)?.name }}</span>
                </div>
              </template>
              <template #after>
                <div v-if="settings?.showSubscriptionProgress && !sub.inactive" class="h-1 bg-surface-hover">
                  <div class="h-full transition-all duration-300"
                    :class="getSubDaysLeft(sub) <= 3 ? 'bg-red-500' : getSubDaysLeft(sub) <= 7 ? 'bg-orange-400' : 'bg-primary'"
                    :style="{ width: ((30 - getSubDaysLeft(sub)) / 30 * 100) + '%' }"
                  />
                </div>
              </template>
            </UniversalListRow>
          </div>
        </div>
      </template>
    </div>

    <!-- Detail Panel -->
    <SubscriptionDetail
      v-if="subscriptionLookupData"
      :show="showDetail"
      :subscription="detailSub"
      :lookupData="subscriptionLookupData"
      @close="showDetail = false"
      @edit="onDetailEdit"
      @clone="onDetailClone"
      @renew="onDetailRenew"
      @delete="onDetailDelete"
      @openUrl="onDetailOpenUrl"
      @toggleFavorite="onDetailToggleFavorite"
      @recordPayment="onDetailRecordPayment"
    />

    <!-- Form -->
    <SubscriptionForm
      v-if="subscriptionFormLookupData"
      :show="showForm"
      :edit-subscription="editingSub"
      :lookupData="subscriptionFormLookupData"
      @close="showForm = false"
      @saved="onSaved"
    />

    <!-- Delete Confirmation Modal -->
    <Teleport to="body">
      <Transition name="app-modal">
        <div v-if="deleteConfirmId" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
          <div class="app-modal-backdrop absolute inset-0 bg-black/50" @click="cancelDelete" />
          <div class="app-modal-panel relative bg-surface w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-sm p-4 sm:p-6">
            <div class="flex items-center gap-3 mb-3 sm:mb-4">
              <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center shrink-0">
                <AlertTriangle :size="18" class="text-red-500" />
              </div>
              <h3 :class="ui.sectionTitle()">{{ t('delete') }}</h3>
            </div>
            <p class="text-xs sm:text-sm text-text-secondary mb-4 sm:mb-6">{{ t('confirm_delete_subscription') }}</p>
            <div class="flex justify-end gap-2 sm:gap-3">
              <button
                @click="cancelDelete"
                class="px-3 sm:px-4 py-2 rounded-lg border border-border text-xs sm:text-sm font-medium text-text-secondary hover:bg-surface-hover"
              >{{ t('cancel') }}</button>
              <button
                @click="confirmDelete"
                class="px-3 sm:px-4 py-2 rounded-lg bg-red-600 text-white text-xs sm:text-sm font-medium hover:bg-red-700"
              >{{ t('delete') }}</button>
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
              <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center shrink-0">
                <AlertTriangle :size="18" class="text-red-500" />
              </div>
              <h3 :class="ui.sectionTitle()">{{ t('batch_delete') }}</h3>
            </div>
            <p class="text-xs sm:text-sm text-text-secondary mb-4 sm:mb-6">{{ t('batch_confirm_delete').replace('{count}', String(batchDeleteConfirmIds.length)) }}</p>
            <div class="flex justify-end gap-2 sm:gap-3">
              <button @click="batchDeleteConfirmIds = []" class="px-3 sm:px-4 py-2 rounded-lg border border-border text-xs sm:text-sm font-medium text-text-secondary hover:bg-surface-hover">{{ t('cancel') }}</button>
              <button @click="confirmBatchDelete" class="px-3 sm:px-4 py-2 rounded-lg bg-red-600 text-white text-xs sm:text-sm font-medium hover:bg-red-700">{{ t('delete') }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Batch Change Category Modal -->
    <Teleport to="body">
      <Transition name="app-modal">
        <div v-if="showBatchCategoryModal" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
          <div class="app-modal-backdrop absolute inset-0 bg-black/50" @click="showBatchCategoryModal = false" />
          <div class="app-modal-panel relative bg-surface w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-sm p-4 sm:p-6">
            <h3 :class="[ui.sectionTitle(), 'mb-3 sm:mb-4']">{{ t('batch_change_category') }}</h3>
            <p class="text-xs sm:text-sm text-text-muted mb-3">{{ selectedIds.size }} {{ t('selected_count') }}</p>
            <AppSelect
              v-model="batchCategoryId"
              :options="categoryFilterOptions.filter((o) => o.value !== '')"
              :label="t('category')"
            />
            <div class="flex justify-end gap-3 mt-5">
              <button @click="showBatchCategoryModal = false" class="px-4 py-2 rounded-lg border border-border text-sm font-medium text-text-secondary hover:bg-surface-hover">{{ t('cancel') }}</button>
              <button @click="confirmBatchCategory" class="px-5 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover">{{ t('save') }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </div>
</template>

<style scoped>
.days-badge {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border-width: 2px;
  border-style: solid;
}

.days-badge--danger {
  border-color: #f87171;
}

.days-badge--warn {
  border-color: #fb923c;
}

.days-badge--normal {
  border-color: var(--color-primary-hover);
}

:global(.dark) .days-badge--danger {
  border-color: #ef4444;
}

:global(.dark) .days-badge--warn {
  border-color: #f97316;
}
</style>
