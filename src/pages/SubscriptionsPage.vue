<script setup lang="ts">
import AppCheckbox from "@/components/ui/AppCheckbox.vue";
import UiViewSwitch from "@/components/ui/UiViewSwitch.vue";
import Modal from "@/components/ui/Modal.vue";
import ConfirmDialog from "@/components/ui/ConfirmDialog.vue";
import UiEmptyState from "@/components/ui/UiEmptyState.vue";
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useHeaderActions, type HeaderAction } from "@/composables/useHeaderActions";
import {
  type SubscriptionFilter,
} from "@/services/subscriptionsClient";
import { storeToRefs } from "pinia";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { moneyFromDecimal, sumMoney } from "@/lib/money";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { useToast } from "@/composables/useToast";
import { useScrollLock } from "@/composables/useScrollLock";
import { useClipboard } from "@/composables/useClipboard";
import { type Subscription, type Settings, type SubscriptionListItem } from "@/schemas/appData";
import SubscriptionForm from "@/components/subscriptions/SubscriptionForm.vue";
import SubscriptionDetail from "@/components/subscriptions/SubscriptionDetail.vue";
import SubscriptionsListSkeleton from "@/components/subscriptions/SubscriptionsListSkeleton.vue";
import AiSmartDialog from "@/components/ai/AiSmartDialog.vue";
import Toast from "@/components/ui/Toast.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import UniversalListRow from "@/components/ui/UniversalListRow.vue";
import ContextActionMenu, {
  type ContextMenuRow,
  type ContextMenuExcludeRect,
} from "@/components/ui/ContextActionMenu.vue";
import { Plus, Search, Pencil, Trash2, RefreshCw, ExternalLink, CreditCard, Star, StarOff, CopyPlus, ClipboardCopy, CheckSquare, Square, CircleDollarSign, FolderOpen, Filter, Sparkles } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { useSubscriptionsStore } from "@/stores/subscriptionsStore";
import { ui, avatarTone, button } from "@/lib/tv";
import { formatErrorForToast } from "@/utils/formatError";
import { useAiConfigStore } from "@/stores/aiConfigStore";

const route = useRoute();
const vueRouter = useRouter();
const { t } = useI18n();
const { setActions, clearActions } = useHeaderActions();
const { fmt } = useCurrencyFormat();
const { fmtDateMedium } = useLocaleFormat();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();
const { copyToClipboard } = useClipboard();
const showFilters = ref(false);
const metaStore = useAppMetaStore();
const subscriptionsStore = useSubscriptionsStore();
const { initialListLoaded } = storeToRefs(subscriptionsStore);
const metaRefs = storeToRefs(metaStore);
const settings = computed(() => metaRefs.settings.value);
const categories = computed(() => metaRefs.categories.value ?? []);
const paymentMethods = computed(() => metaRefs.paymentMethods.value ?? []);
const tags = computed(() => metaRefs.tags.value ?? []);
const currencies = computed(() => metaRefs.currencies.value ?? []);
const household = computed(() => metaRefs.household.value ?? []);
const subscriptions = computed(() => subscriptionsStore.items);

// AI smart-input state (must be declared before updateHeaderActions so the
// header-action builder can read the availability flag without forward refs).
const showAiSmart = ref(false);
const aiConfigStore = useAiConfigStore();
const {
  enabled: aiEnabled,
  features: aiFeatures,
  hasApiKey: aiHasApiKey,
  activeProvider: aiActiveProvider,
  loaded: aiLoaded,
} = storeToRefs(aiConfigStore);
/** Sparkles in the header when config is loaded and the surface feature is on. Keyring is read only on click. */
const showAiHeaderButton = computed(() => {
  if (!aiLoaded.value) return false;
  return !!aiFeatures.value.subscriptionInput;
});

function openAiSmart() {
  showAiSmart.value = true;
}

function openAiSettings() {
  vueRouter.push("/settings?section=ai");
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

function onAiImported(_count: number) {
  fetchFilteredSubs().catch((e) => {
    console.warn("[SubscriptionsPage] reload after AI import failed", e);
  });
}

// Register header action
function updateHeaderActions() {
  const actions: HeaderAction[] = [];
  if (showAiHeaderButton.value) {
    actions.push({
      id: "ai-smart-sub",
      icon: Sparkles,
      title: t("ai_smart_open"),
      onClick: () => {
        void onAiAssistantClick();
      },
      style: "accent",
    });
  }
  actions.push({ id: "add-sub", icon: Plus, title: t("new_subscription"), label: t("add"), onClick: openAdd, style: "primary" });
  setActions(actions);
}

onMounted(() => {
  loadInitial().then(() => {
    updateHeaderActions();
    handleSubQueryParam();
    fetchFilteredSubs();
  });
  aiConfigStore.load().then(() => updateHeaderActions());
});

watch(showAiHeaderButton, () => updateHeaderActions());
onUnmounted(() => {
  clearActions();
});

// Watch for route query changes (e.g. from tray navigation)
watch(() => route.query.sub, () => handleSubQueryParam());

async function handleSubQueryParam() {
  const subId = route.query.sub as string | undefined;
  if (!subId) return;
  // Clear the query to avoid re-opening on route re-render
  vueRouter.replace({ query: {} });
  let sub = subscriptions.value.find((s) => s.id === subId);
  if (!sub && activeFilterCount.value > 0) {
    // Hidden by the current filter: drop it and load the full list first.
    resetFilters();
    await fetchFilteredSubs();
    sub = subscriptions.value.find((s) => s.id === subId);
  }
  if (sub) openDetail(sub);
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

/** Non-default filters in effect (drives the badge on the Filter button and "no results" state). */
const activeFilterCount = computed(() => (filterCategory.value ? 1 : 0) + (filterPayment.value ? 1 : 0) + (filterState.value ? 1 : 0) + (filterTag.value ? 1 : 0) + (searchQuery.value.trim() ? 1 : 0));

function resetFilters() {
  searchQuery.value = "";
  filterCategory.value = "";
  filterPayment.value = "";
  filterState.value = "";
  filterTag.value = "";
}

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

/** Per-group monthly sum in the main currency (active only), shown in the group header. */
function groupMonthlyTotal(subs: SubscriptionListItem[]) {
  return moneyFromDecimal(sumMoney(subs.filter((s) => !s.inactive).map((s) => s.monthlyPriceMain)));
}

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

/** Mutations from list actions: surface backend errors as a toast and resync the list. */
async function guarded(fn: () => Promise<void>) {
  try {
    await fn();
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
    fetchFilteredSubs();
  }
}

function batchDeleteSelected() {
  batchDeleteConfirmIds.value = [...selectedIds.value];
}

async function confirmBatchDelete() {
  const ids = batchDeleteConfirmIds.value;
  batchDeleteConfirmIds.value = [];
  if (ids.length === 0) return;
  await guarded(async () => {
    await subscriptionsStore.batchDelete(ids);
    toast(t("batch_deleted", { count: ids.length }));
    selectedIds.value = new Set();
    selectionMode.value = false;
    fetchFilteredSubs();
  });
}

async function batchSetInactive(inactive: boolean) {
  const ids = [...selectedIds.value];
  await guarded(async () => {
    await subscriptionsStore.batchSetInactive(ids, inactive);
    toast(t("batch_updated", { count: ids.length }));
    selectedIds.value = new Set();
    fetchFilteredSubs();
  });
}

const batchDeactivate = () => batchSetInactive(true);
const batchActivate = () => batchSetInactive(false);

function openBatchCategoryModal() {
  batchCategoryId.value = settings.value?.defaultCategoryId || "cat-1";
  showBatchCategoryModal.value = true;
}

async function confirmBatchCategory() {
  const ids = [...selectedIds.value];
  showBatchCategoryModal.value = false;
  await guarded(async () => {
    await subscriptionsStore.batchSetCategory(ids, batchCategoryId.value);
    toast(t("batch_updated", { count: ids.length }));
    selectedIds.value = new Set();
    fetchFilteredSubs();
  });
}

const contextMenuOpen = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const contextMenuRows = ref<ContextMenuRow[]>([]);
const contextMenuExcludeRect = ref<ContextMenuExcludeRect | null>(null);

watch(contextMenuOpen, (open) => {
  if (!open) contextMenuExcludeRect.value = null;
});

function showContextMenu(sub: Subscription, event: MouseEvent, anchorRect?: DOMRect | null) {
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
      label: sub.favorite ? t("remove_from_favorites") : t("add_to_favorites"),
      icon: sub.favorite ? StarOff : Star,
      run: async () => {
        await handleToggleFavorite(sub.id);
      },
    },
  ];
  if (!sub.inactive) {
    rows.push({
      kind: "button",
      label: t("record_payment"),
      icon: CreditCard,
      run: () => {
        handleRecordPayment(sub.id);
      },
    });
  }
  rows.push(
    {
      kind: "button",
      label: t("edit_subscription"),
      icon: Pencil,
      run: () => {
        openEdit(sub);
      },
    },
    {
      kind: "button",
      label: t("clone"),
      icon: CopyPlus,
      run: () => {
        handleClone(sub.id);
      },
    },
    {
      kind: "button",
      label: t("renew"),
      icon: RefreshCw,
      run: () => {
        handleRenew(sub.id);
      },
    },
  );
  if (sub.url) {
    rows.push({
      kind: "button",
      label: t("url"),
      icon: ExternalLink,
      run: () => {
        handleOpenUrl(sub.url);
      },
    });
  }
  rows.push(
    {
      kind: "button",
      label: t("copy"),
      icon: ClipboardCopy,
      run: () => {
        handleCopyName(sub);
      },
    },
    { kind: "separator" },
    {
      kind: "button",
      label: t("delete"),
      danger: true,
      icon: Trash2,
      run: () => {
        requestDelete(sub.id);
      },
    },
  );

  contextMenuRows.value = rows;
  contextMenuOpen.value = true;
}

// Filtered subscriptions from backend DTO query
const filteredSubscriptions = computed<SubscriptionListItem[]>(
  () => subscriptionsStore.items,
);
/** Monthly total in the main currency (backend converts each row; summing raw prices mixed currencies). */
const totalSubscriptionsAmount = computed(() =>
  moneyFromDecimal(sumMoney(filteredSubscriptions.value.filter((s) => !s.inactive).map((s) => s.monthlyPriceMain))),
);
const subscriptionsSummaryCurrencyId = computed(() => settings.value?.mainCurrencyId || "cur-1");

/** Row price honouring display settings: convert to main currency, original price, monthly equivalent. */
function priceView(sub: SubscriptionListItem) {
  const main = subscriptionsSummaryCurrencyId.value;
  const converted = Boolean(settings.value?.convertCurrency) && sub.currencyId !== main;
  const primary = converted ? fmt(sub.priceMain, main) : fmt(sub.price, sub.currencyId);
  const extra: string[] = [];
  if (converted && settings.value?.showOriginalPrice) extra.push(fmt(sub.price, sub.currencyId));
  const isMonthly = sub.cycle === 3 && sub.frequency === 1;
  if (settings.value?.monthlyPrice && !isMonthly) {
    extra.push(`≈ ${converted ? fmt(sub.monthlyPriceMain, main) : fmt(sub.monthlyPrice, sub.currencyId)}/${t("mo")}`);
  }
  return { primary, secondary: extra.join(" · ") };
}

/** "in 2 days" / "due today" / "overdue by 3 days" — readable instead of a bare "2d". */
function dueLabel(sub: SubscriptionListItem): string {
  const d = sub.daysLeft;
  if (d < 0) return t("overdue_by_alert", { days: -d }, -d);
  if (d === 0) return t("due_today_alert");
  return t("in_days_alert", { days: d }, d);
}

function dueTone(sub: SubscriptionListItem): string {
  if (sub.urgency === "overdue" || sub.urgency === "danger") return "text-danger";
  if (sub.urgency === "warn") return "text-warning";
  return "text-text-muted";
}

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

watch([searchQuery, filterCategory, filterPayment, filterTag, filterState, sortBy], () => {
  // Hidden rows must not stay selected: batch actions would hit subscriptions the user can't see.
  selectedIds.value = new Set();
  fetchFilteredSubsDebounced();
});

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
  if (sub.urgency === "overdue" || sub.urgency === "danger") return "days-badge--danger";
  if (sub.urgency === "warn") return "days-badge--warn";
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
  const id = deleteConfirmId.value;
  deleteConfirmId.value = null;
  if (!id) return;
  await guarded(async () => {
    await subscriptionsStore.deleteById(id);
    toast(t("subscription_deleted"));
    fetchFilteredSubs();
  });
}

function cancelDelete() { deleteConfirmId.value = null; }

async function handleClone(id: string) {
  showDetail.value = false;
  await guarded(async () => {
    await subscriptionsStore.cloneById(id);
    toast(t("subscription_added"));
  });
}

async function handleRenew(id: string) {
  showDetail.value = false;
  await handleRecordPayment(id);
}

async function handleRecordPayment(id: string) {
  try {
    await subscriptionsStore.recordPayment(id);
    toast(t("payment_recorded"));
    await fetchFilteredSubs();
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
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
  try {
    await subscriptionsStore.toggleFavorite(id);
    await fetchFilteredSubs();
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
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
  { value: "", label: t("all_states") },
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
  await guarded(async () => {
    await subscriptionsStore.toggleFavorite(id);
    fetchFilteredSubs();
  });
}

async function updateSettings(updates: Partial<Settings>) {
  if (!settings.value) return;
  await metaStore.updateSettings({ ...settings.value, ...updates });
}

async function loadInitial() {
  await metaStore.ensureLoaded();
  await subscriptionsStore.loadBrief(buildSubFilter());
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

    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 -translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-2"
    >
      <div v-if="showFilters" class="mb-3 p-3 rounded-xl border border-border bg-surface shadow-sm space-y-2">
        <div v-if="activeFilterCount > 0" class="flex justify-end -mb-1">
          <button type="button" class="text-xs font-medium text-primary hover:text-primary-hover" @click="resetFilters">{{ t('reset_filters') }}</button>
        </div>
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-2">
          <div class="min-w-0">
            <AppSelect v-model="sortBy" :options="sortOptions" />
          </div>
          <div class="min-w-0">
            <AppSelect v-model="filterCategory" :options="categoryFilterOptions" />
          </div>
          <div class="min-w-0">
            <AppSelect v-model="filterPayment" :options="paymentFilterOptions" />
          </div>
          <div class="min-w-0">
            <AppSelect v-model="filterState" :options="stateFilterOptions" />
          </div>
          <div v-if="tags.length > 0" class="min-w-0">
            <AppSelect v-model="filterTag" :options="tagFilterOptions" />
          </div>
          <div class="min-w-0">
            <AppSelect :modelValue="groupBy" @update:modelValue="(v: string | number) => setGroupBy(String(v) as 'none' | 'category' | 'payment_method')" :options="groupByOptions" />
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
      <div v-if="selectionMode" class="sticky top-2 z-10 flex items-center gap-1.5 mb-3 px-3 py-2 rounded-xl border border-border-strong bg-surface shadow-md overflow-x-auto">
        <span class="text-xs font-semibold text-text-primary tabular-nums whitespace-nowrap">{{ selectedIds.size }} {{ t('selected_count') }}</span>
        <button :class="button({ intent: 'ghost', size: 'iconSm' })" @click="selectAll" :title="t('select_all')">
          <CheckSquare :size="13" />
        </button>
        <button :class="button({ intent: 'ghost', size: 'iconSm' })" @click="deselectAll" :title="t('deselect_all')">
          <Square :size="13" />
        </button>
        <div class="flex-1" />
        <button
          @click="batchActivate"
          :disabled="selectedIds.size === 0"
          :class="button({ size: 'sm' })"
        >{{ t('batch_activate') }}</button>
        <button
          @click="batchDeactivate"
          :disabled="selectedIds.size === 0"
          :class="button({ size: 'sm' })"
        >{{ t('batch_deactivate') }}</button>
        <button
          @click="openBatchCategoryModal"
          :disabled="selectedIds.size === 0"
          :class="button({ size: 'sm' })"
        >{{ t('batch_change_category') }}</button>
        <button
          @click="batchDeleteSelected"
          :disabled="selectedIds.size === 0"
          :class="button({ intent: 'ghost', size: 'sm' })" class="text-danger hover:text-danger"
        >{{ t('batch_delete') }}</button>
      </div>
    </Transition>

    <SubscriptionsListSkeleton v-if="!initialListLoaded" />

    <!-- Empty state -->
    <div v-else-if="filteredSubscriptions.length === 0" :class="ui.panel()">
      <UiEmptyState v-if="activeFilterCount > 0" :icon="Search" :title="t('nothing_found')">
        <button :class="button()" @click="resetFilters">{{ t('reset_filters') }}</button>
      </UiEmptyState>
      <UiEmptyState v-else :icon="CreditCard" :title="t('no_subscriptions_yet')">
        <button :class="button({ intent: 'primary' })" @click="openAdd">
          <Plus :size="15" /> {{ t('add_first_subscription') }}
        </button>
      </UiEmptyState>
    </div>

    <!-- Subscription list -->
    <div v-else>
      <!-- Summary bar -->
      <div class="flex items-end justify-between px-1 pb-3">
        <div class="flex items-center gap-1.5 text-xs text-text-muted">
          <CircleDollarSign :size="14" />
          <span>{{ filteredSubscriptions.length }} {{ t('subscriptions').toLowerCase() }}</span>
        </div>
        <div class="text-xl font-semibold text-text-primary tabular-nums tracking-[-0.02em]">
          {{ fmt(totalSubscriptionsAmount, subscriptionsSummaryCurrencyId) }}
        </div>
      </div>

      <template v-for="group in groupedSubscriptions" :key="group.key">
        <!-- Group header -->
        <div v-if="groupBy !== 'none' && group.label" class="flex items-center gap-2 mb-2 mt-4 first:mt-0">
          <IconDisplay v-if="groupBy === 'category' && getCategoryIcon(group.key)" :icon="getCategoryIcon(group.key)" :size="16" />
          <FolderOpen v-else :size="14" class="text-primary" />
          <span class="text-xs font-semibold text-text-primary uppercase tracking-wide">{{ group.label }}</span>
          <span class="text-[11px] text-text-muted tabular-nums">({{ group.subs.length }})</span>
          <div class="flex-1 h-px bg-border" />
          <span class="text-xs font-semibold text-text-secondary tabular-nums">
            {{ fmt(groupMonthlyTotal(group.subs), subscriptionsSummaryCurrencyId) }}<span class="font-normal text-text-muted">/{{ t('mo') }}</span>
          </span>
        </div>

        <!-- Grid for expanded mode -->
        <div
          :class="
 viewMode === 'expanded'
              ? 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 mb-4'
              : viewMode === 'default'
                ? 'flex flex-col gap-1.5 mb-4'
                : 'flex flex-col gap-1 mb-4'
          "
        >
          <div
            v-for="sub in group.subs"
            :key="sub.id"
            class="relative min-w-0 overflow-hidden"
            :class="[
 'bg-surface border shadow-sm transition-[border-color,box-shadow] hover:border-border-strong',
              viewMode === 'compact' ? 'rounded-lg' : 'rounded-xl',
              selectedIds.has(sub.id) ? 'border-primary ring-1 ring-primary/30' : 'border-border',
              sub.inactive ? 'opacity-50' : '',
            ]"
          >
            <UniversalListRow
              :mode="viewMode as 'compact' | 'default' | 'expanded'"
              @click="selectionMode ? toggleSelected(sub.id) : openDetail(sub)"
              @contextmenu="(e, rect) => showContextMenu(sub, e, rect)"
            >
              <template #selection>
                <div
                  v-if="selectionMode && viewMode !== 'expanded'"
                  class="shrink-0"
                  @click.stop
                  >
                    <AppCheckbox :model-value="selectedIds.has(sub.id)" :aria-label="t('select')" @update:model-value="toggleSelected(sub.id)" />
                </div>
              </template>
              <template #leading>
                <template v-if="viewMode === 'compact'">
                  <Tooltip v-if="!selectionMode" :text="sub.favorite ? t('remove_from_favorites') : t('add_to_favorites')">
                    <button @click.stop="handleToggleFavorite(sub.id)" class="shrink-0 p-0.5 rounded transition-colors" :class="sub.favorite ? 'text-warning' : 'text-border-strong hover:text-warning'">
                      <Star :size="12" :fill="sub.favorite ? 'currentColor' : 'none'" />
                    </button>
                  </Tooltip>
                  <div class="w-6 h-6 rounded-md flex items-center justify-center text-[11px] font-semibold shrink-0 overflow-hidden" :class="sub.logo ? 'bg-surface ring-1 ring-border' : avatarTone(sub.name)">
                    <img v-if="sub.logo" :src="sub.logo" class="w-full h-full object-contain" />
                    <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
                  </div>
                </template>
                <template v-else-if="viewMode === 'default'">
                  <Tooltip v-if="!selectionMode" :text="sub.favorite ? t('remove_from_favorites') : t('add_to_favorites')">
                    <button @click.stop="handleToggleFavorite(sub.id)" class="shrink-0 p-0.5 rounded transition-colors" :class="sub.favorite ? 'text-warning' : 'text-border-strong hover:text-warning'">
                      <Star :size="16" :fill="sub.favorite ? 'currentColor' : 'none'" />
                    </button>
                  </Tooltip>
                  <div class="w-9 h-9 rounded-[10px] flex items-center justify-center text-sm font-semibold shrink-0 overflow-hidden" :class="sub.logo ? 'bg-surface ring-1 ring-border' : avatarTone(sub.name)">
                    <img v-if="sub.logo" :src="sub.logo" class="w-full h-full object-contain" />
                    <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
                  </div>
                </template>
              </template>
              <template #main>
                <div
                  v-if="viewMode === 'compact'"
                  class="flex min-w-0 flex-1 items-center gap-x-1 overflow-hidden text-xs"
                >
                  <span class="min-w-0 shrink truncate font-medium text-text-primary">{{ sub.name }}</span>

                  <template v-if="sub.inactive">
                    <span class="hidden shrink-0 select-none text-border sm:inline" aria-hidden="true">·</span>
                    <span class="hidden shrink-0 text-[10px] font-normal text-text-muted sm:inline">{{ t('inactive') }}</span>
                  </template>
                  <template v-else>
                    <span class="hidden shrink-0 select-none text-border sm:inline" aria-hidden="true">·</span>
                    <span
                      class="hidden max-w-[5rem] shrink-0 truncate text-[10px] text-text-muted sm:inline sm:max-w-[8rem] md:max-w-none"
                    >{{ billingCycleText(sub.cycle, sub.frequency) }}</span>

                    <span class="hidden shrink-0 select-none text-border md:inline" aria-hidden="true">·</span>
                    <span
                      class="hidden shrink-0 tabular-nums text-[10px] whitespace-nowrap md:inline"
                      :class="isSubOverdue(sub) ? 'font-medium text-danger' : 'text-text-muted'"
                    >{{ formatDate(sub.nextPayment) }}</span>

                    <span
                      v-if="!sub.autoRenew"
                      class="hidden shrink-0 rounded bg-warning/15 px-1 py-px text-[9px] font-semibold text-warning lg:inline dark:text-warning"
                    >{{ t('manual_renewal_short') }}</span>

                    <span
                      v-if="getCategoryName(sub.categoryId) && groupBy !== 'category'"
                      class="hidden max-w-[5rem] shrink-0 truncate text-[10px] text-text-muted lg:inline lg:max-w-[7rem]"
                    >{{ getCategoryName(sub.categoryId) }}</span>
                  </template>
                </div>
                <div v-else-if="viewMode === 'default'" class="min-w-0">
                  <p class="text-sm font-semibold text-text-primary truncate">{{ sub.name }}</p>
                  <div class="mt-0.5 flex items-center gap-1.5 overflow-hidden whitespace-nowrap text-xs text-text-muted sm:flex-wrap">
                    <span class="shrink-0">{{ billingCycleText(sub.cycle, sub.frequency) }}</span>
                    <!-- Phones: the date column is hidden, so the due hint moves into this line -->
                    <template v-if="!sub.inactive">
                      <span class="text-border-strong sm:hidden" aria-hidden="true">·</span>
                      <span class="truncate font-medium sm:hidden" :class="dueTone(sub)">{{ dueLabel(sub) }}</span>
                    </template>
                    <template v-if="getCategoryName(sub.categoryId) && groupBy !== 'category'">
                      <span class="hidden text-border-strong sm:inline" aria-hidden="true">·</span>
                      <span class="hidden items-center gap-1 max-w-[10rem] truncate sm:inline-flex">
                        <IconDisplay v-if="getCategoryIcon(sub.categoryId)" :icon="getCategoryIcon(sub.categoryId)" :size="12" />
                        {{ getCategoryName(sub.categoryId) }}
                      </span>
                    </template>
                    <span v-if="!sub.autoRenew" class="hidden items-center rounded px-1.5 py-px text-[10px] font-semibold bg-warning/15 text-warning sm:inline-flex">{{ t('manual_renewal') }}</span>
                    <span v-for="tag in (sub.tags || []).slice(0, 3)" :key="tag" class="hidden sm:inline-flex items-center px-1.5 py-0 rounded text-[10px] font-medium bg-surface-hover text-text-muted border border-border">#{{ tag }}</span>
                    <span v-if="(sub.tags || []).length > 3" class="hidden sm:inline text-[10px] text-text-muted">+{{ sub.tags.length - 3 }}</span>
                  </div>
                </div>
              </template>
              <template #meta>
                <span
                  v-if="viewMode === 'compact' && !sub.inactive"
                  class="days-badge inline-flex items-center justify-center px-1.5 py-0.5 rounded text-[11px] font-semibold leading-none shrink-0 w-[56px] tabular-nums"
                  :class="getDaysBadgeState(sub)"
                >
                  {{ getSubDaysLeft(sub) }}{{ t('days_short') }}
                </span>
                <div v-else-if="viewMode === 'default'" class="hidden text-right shrink-0 sm:block sm:w-[150px]">
                  <p class="text-[13px] font-medium text-text-primary tabular-nums whitespace-nowrap">{{ formatDate(sub.nextPayment) }}</p>
                  <p v-if="!sub.inactive" class="text-[11px] font-medium whitespace-nowrap truncate" :class="dueTone(sub)">{{ dueLabel(sub) }}</p>
                  <p v-else class="text-[11px] text-text-muted">{{ t('inactive') }}</p>
                </div>
              </template>
              <template #value>
                <p v-if="viewMode === 'compact'" class="text-xs font-semibold text-text-primary shrink-0 w-[96px] text-right tabular-nums whitespace-nowrap">{{ priceView(sub).primary }}</p>
                <div v-else-if="viewMode === 'default'" class="text-right shrink-0 w-[88px] sm:w-[124px]">
                  <p class="text-[15px] font-semibold text-text-primary tabular-nums tracking-[-0.01em] whitespace-nowrap">{{ priceView(sub).primary }}</p>
                  <p v-if="priceView(sub).secondary" class="text-[11px] text-text-muted tabular-nums whitespace-nowrap truncate">{{ priceView(sub).secondary }}</p>
                </div>
              </template>
              <template #trailing>
                <Tooltip
                  v-if="viewMode === 'compact'"
                  :text="getPaymentMethod(sub.paymentMethodId)?.name || t('payment_method')"
                >
                  <span class="inline-flex shrink-0 opacity-90">
                    <IconDisplay :icon="getPaymentMethod(sub.paymentMethodId)?.icon || '💳'" :size="18" />
                  </span>
                </Tooltip>
                <div v-else-if="viewMode === 'default'" class="shrink-0 hidden sm:block" :title="getPaymentMethod(sub.paymentMethodId)?.name">
                  <IconDisplay :icon="getPaymentMethod(sub.paymentMethodId)?.icon || '💳'" :size="22" />
                </div>
              </template>
              <template #expanded>
                <div class="flex items-start gap-3 mb-3">
                  <div v-if="selectionMode" class="shrink-0 mt-1" @click.stop>
                    <AppCheckbox :model-value="selectedIds.has(sub.id)" :aria-label="t('select')" @update:model-value="toggleSelected(sub.id)" />
                  </div>
                  <div class="w-11 h-11 rounded-xl flex items-center justify-center text-base font-semibold shrink-0 overflow-hidden" :class="sub.logo ? 'bg-surface ring-1 ring-border' : avatarTone(sub.name)">
                    <img v-if="sub.logo" :src="sub.logo" class="w-full h-full object-contain" />
                    <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
                  </div>
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <p class="text-sm font-semibold text-text-primary truncate">{{ sub.name }}</p>
                      <Tooltip v-if="!selectionMode" :text="sub.favorite ? t('remove_from_favorites') : t('add_to_favorites')">
                        <button @click.stop="handleToggleFavorite(sub.id)" class="shrink-0 p-0.5 rounded transition-colors" :class="sub.favorite ? 'text-warning' : 'text-border-strong hover:text-warning'">
                          <Star :size="14" :fill="sub.favorite ? 'currentColor' : 'none'" />
                        </button>
                      </Tooltip>
                    </div>
                    <p v-if="groupBy !== 'category'" class="text-xs text-text-muted flex items-center gap-1">
                      <IconDisplay v-if="getCategoryIcon(sub.categoryId)" :icon="getCategoryIcon(sub.categoryId)" :size="12" />
                      {{ getCategoryName(sub.categoryId) }}
                    </p>
                  </div>
                </div>

                <div class="flex items-end justify-between">
                  <div>
                    <p class="text-lg font-semibold text-text-primary tabular-nums tracking-[-0.01em]">{{ priceView(sub).primary }}</p>
                    <p v-if="priceView(sub).secondary" class="text-[11px] text-text-muted tabular-nums">{{ priceView(sub).secondary }}</p>
                    <p class="text-[11px] text-text-muted">{{ billingCycleText(sub.cycle, sub.frequency) }}</p>
                  </div>
                  <div class="text-right w-[136px]">
                    <p class="text-xs font-medium tabular-nums whitespace-nowrap" :class="isSubOverdue(sub) ? 'text-danger' : 'text-text-primary'">{{ formatDate(sub.nextPayment) }}</p>
                    <span
                      v-if="!sub.inactive"
                      class="days-badge inline-flex items-center justify-center px-1.5 py-0.5 rounded text-[11px] font-semibold leading-none mt-0.5 w-[56px] tabular-nums"
                      :class="getDaysBadgeState(sub)"
                    >
                      {{ getSubDaysLeft(sub) }}{{ t('days_short') }}
                    </span>
                  </div>
                </div>

                <!-- Tags in expanded mode -->
                <div v-if="(sub.tags || []).length > 0" class="flex items-center gap-1 mt-2 flex-wrap">
                  <span v-for="tag in sub.tags" :key="tag" class="inline-flex items-center px-1.5 py-0 rounded text-[10px] font-medium bg-surface-hover text-text-muted border border-border">#{{ tag }}</span>
                </div>

                <!-- Payment method -->
                <div class="flex items-center gap-1.5 mt-2">
                  <IconDisplay :icon="getPaymentMethod(sub.paymentMethodId)?.icon || '💳'" :size="16" />
                  <span class="text-[11px] text-text-muted">{{ getPaymentMethod(sub.paymentMethodId)?.name }}</span>
                </div>
              </template>
              <template #after>
                <!-- Cycle progress: a real bar with a track, inset to the text column in list modes -->
                <div
                  v-if="settings?.showSubscriptionProgress && !sub.inactive"
                  class="absolute bottom-0 overflow-hidden"
                  :class="viewMode === 'compact' ? 'inset-x-0 h-[2px] bg-surface-hover' : 'inset-x-0 h-[3px] bg-surface-hover'"
                >
                  <div class="h-full rounded-full transition-all duration-300"
                    :class="sub.urgency === 'overdue' || sub.urgency === 'danger' ? 'bg-danger' : sub.urgency === 'warn' ? 'bg-warning' : 'bg-primary'"
                    :style="{ width: sub.cycleProgress * 100 + '%' }"
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

    <AiSmartDialog
      :show="showAiSmart"
      surface="subscription"
      @close="showAiSmart = false"
      @imported="onAiImported"
    />

    <ConfirmDialog
      :show="Boolean(deleteConfirmId)"
      :title="t('delete')"
      :message="t('confirm_delete_subscription')"
      @confirm="confirmDelete"
      @cancel="cancelDelete"
    />

    <ConfirmDialog
      :show="batchDeleteConfirmIds.length > 0"
      :title="t('batch_delete')"
      :message="t('batch_confirm_delete', { count: batchDeleteConfirmIds.length })"
      @confirm="confirmBatchDelete"
      @cancel="batchDeleteConfirmIds = []"
    />

    <!-- Batch Change Category Modal -->
    <Modal :show="showBatchCategoryModal" :title="t('batch_change_category')" max-width="24rem" @close="showBatchCategoryModal = false">
      <p class="text-xs text-text-muted mb-3">{{ selectedIds.size }} {{ t('selected_count') }}</p>
      <AppSelect
        v-model="batchCategoryId"
        :options="categoryFilterOptions.filter((o) => o.value !== '')"
        :label="t('category')"
      />
      <template #footer>
        <button :class="button()" @click="showBatchCategoryModal = false">{{ t('cancel') }}</button>
        <button :class="button({ intent: 'primary' })" @click="confirmBatchCategory">{{ t('save') }}</button>
      </template>
    </Modal>

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

<style scoped>
.days-badge {
  border-radius: 9999px;
  font-weight: 600;
}
.days-badge--danger {
  background: color-mix(in srgb, var(--color-danger) 14%, transparent);
  color: var(--color-danger);
}
.days-badge--warn {
  background: color-mix(in srgb, var(--color-warning) 16%, transparent);
  color: var(--color-warning);
}
.days-badge--normal {
  background: var(--color-surface-hover);
  color: var(--color-text-secondary);
}
</style>
