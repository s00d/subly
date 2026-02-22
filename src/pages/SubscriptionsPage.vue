<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useSubscriptionsStore } from "@/stores/subscriptions";
import { useSettingsStore } from "@/stores/settings";
import { useCatalogStore } from "@/stores/catalog";
import { useI18n } from "vue-i18n";
import { useHeaderActions } from "@/composables/useHeaderActions";
import { getPricePerMonth, getDaysUntilPayment, getBillingCycleText, isOverdue } from "@/services/calculations";
import { dbLoadSubscriptionsFiltered, type SubscriptionFilter } from "@/services/database";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { useToast } from "@/composables/useToast";
import type { Subscription } from "@/schemas/appData";
import SubscriptionForm from "@/components/subscriptions/SubscriptionForm.vue";
import SubscriptionDetail from "@/components/subscriptions/SubscriptionDetail.vue";
import Toast from "@/components/ui/Toast.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import { Plus, Search, Pencil, Trash2, Copy, RefreshCw, ExternalLink, CreditCard, AlertTriangle, Star, CheckSquare, Hash, CircleDollarSign, LayoutList, LayoutGrid, Rows3, FolderOpen } from "lucide-vue-next";
import Tooltip from "@/components/ui/Tooltip.vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Menu } from "@tauri-apps/api/menu";
import type { MenuItemOptions, PredefinedMenuItemOptions } from "@tauri-apps/api/menu";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

const subsStore = useSubscriptionsStore();
const settingsStore = useSettingsStore();
const catalogStore = useCatalogStore();
const route = useRoute();
const vueRouter = useRouter();
const { t } = useI18n();
const { setActions, clearActions } = useHeaderActions();
const { fmt } = useCurrencyFormat();
const { fmtDateMedium } = useLocaleFormat();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();

const now = ref(Date.now());
let nowInterval: ReturnType<typeof setInterval> | null = null;

// Register header action
onMounted(() => {
  setActions([{ id: "add-sub", icon: Plus, title: t("new_subscription"), onClick: openAdd }]);
  handleSubQueryParam();
  fetchFilteredSubs();
  nowInterval = setInterval(() => { now.value = Date.now(); }, 60_000);
});
onUnmounted(() => {
  clearActions();
  if (nowInterval) clearInterval(nowInterval);
});

// Watch for route query changes (e.g. from tray navigation)
watch(() => route.query.sub, () => handleSubQueryParam());

function handleSubQueryParam() {
  const subId = route.query.sub as string | undefined;
  if (subId) {
    const sub = subsStore.subscriptions.find((s) => s.id === subId);
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
    ?? subsStore.subscriptions.find((s) => s.id === detailSubId.value)
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
const viewMode = computed(() => settingsStore.settings.subscriptionViewMode || "default");
const groupBy = computed(() => settingsStore.settings.subscriptionGroupBy || "none");

function setViewMode(mode: "default" | "compact" | "expanded") {
  settingsStore.updateSettings({ subscriptionViewMode: mode });
}

function setGroupBy(g: "none" | "category" | "payment_method") {
  settingsStore.updateSettings({ subscriptionGroupBy: g });
}

// Grouped subscriptions
interface SubGroup {
  key: string;
  label: string;
  subs: Subscription[];
}

const groupedSubscriptions = computed<SubGroup[]>(() => {
  const subs = filteredSubscriptions.value;
  if (groupBy.value === "none") return [{ key: "__all", label: "", subs }];

  const map = new Map<string, { label: string; subs: Subscription[] }>();

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
    await subsStore.batchDelete(batchDeleteConfirmIds.value);
    toast(t("batch_deleted").replace("{count}", String(batchDeleteConfirmIds.value.length)));
    selectedIds.value = new Set();
    selectionMode.value = false;
    fetchFilteredSubs();
  }
  batchDeleteConfirmIds.value = [];
}

async function batchDeactivate() {
  const ids = [...selectedIds.value];
  await subsStore.batchSetInactive(ids, true);
  toast(t("batch_updated").replace("{count}", String(ids.length)));
  selectedIds.value = new Set();
  fetchFilteredSubs();
}

async function batchActivate() {
  const ids = [...selectedIds.value];
  await subsStore.batchSetInactive(ids, false);
  toast(t("batch_updated").replace("{count}", String(ids.length)));
  selectedIds.value = new Set();
  fetchFilteredSubs();
}

function openBatchCategoryModal() {
  batchCategoryId.value = settingsStore.settings.defaultCategoryId || "cat-1";
  showBatchCategoryModal.value = true;
}

async function confirmBatchCategory() {
  const ids = [...selectedIds.value];
  await subsStore.batchSetCategory(ids, batchCategoryId.value);
  toast(t("batch_updated").replace("{count}", String(ids.length)));
  showBatchCategoryModal.value = false;
  selectedIds.value = new Set();
  fetchFilteredSubs();
}

const batchDeleteConfirmIds = ref<string[]>([]);

// Native context menu
async function showContextMenu(sub: Subscription, event: MouseEvent) {
  if (selectionMode.value) return;

  const items: (MenuItemOptions | PredefinedMenuItemOptions)[] = [
    { id: "favorite", text: sub.favorite ? t("remove_from_favorites") : t("add_to_favorites"), action: async () => { await subsStore.toggleFavorite(sub.id); fetchFilteredSubs(); } },
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
    console.warn("Context menu failed:", e);
  }
}

// Filtered subscriptions from SQL
const filteredSubscriptions = ref<Subscription[]>([]);
const subsLoading = ref(false);

function buildSubFilter(): SubscriptionFilter {
  const f: SubscriptionFilter = {
    sortBy: (sortBy.value || "next_payment") as SubscriptionFilter["sortBy"],
    disabledToBottom: settingsStore.settings.disabledToBottom,
    hideDisabled: settingsStore.settings.hideDisabled,
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
  subsLoading.value = true;
  try {
    filteredSubscriptions.value = await dbLoadSubscriptionsFiltered(buildSubFilter());
  } finally {
    subsLoading.value = false;
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
  ...catalogStore.tags.map((tag) => ({ value: tag.name, label: tag.name })),
]);

function getCategoryName(id: string): string {
  return catalogStore.categories.find((c) => c.id === id)?.name || "";
}

function getCategoryIcon(id: string): string {
  return catalogStore.categories.find((c) => c.id === id)?.icon || "";
}

function getPaymentMethod(id: string) {
  return catalogStore.paymentMethods.find((p) => p.id === id);
}

const formatDate = fmtDateMedium;

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

// Delete confirmation
const deleteConfirmId = ref<string | null>(null);

function requestDelete(id: string) {
  showDetail.value = false;
  detailSubId.value = null;
  deleteConfirmId.value = id;
}

async function confirmDelete() {
  if (deleteConfirmId.value) {
    await subsStore.deleteSubscription(deleteConfirmId.value);
    toast(t("subscription_deleted"));
    fetchFilteredSubs();
  }
  deleteConfirmId.value = null;
}

function cancelDelete() { deleteConfirmId.value = null; }

async function handleClone(id: string) {
  showDetail.value = false;
  await subsStore.cloneSubscription(id);
  toast(t("subscription_added"));
  fetchFilteredSubs();
}

async function handleRenew(id: string) {
  showDetail.value = false;
  await subsStore.renewSubscription(id);
  toast(t("payment_recorded"));
  fetchFilteredSubs();
}

async function handleRecordPayment(id: string) {
  await subsStore.recordPayment(id);
  toast(t("payment_recorded"));
  fetchFilteredSubs();
}

async function handleOpenUrl(url: string) {
  const fullUrl = url.startsWith("http") ? url : `https://${url}`;
  try { await openUrl(fullUrl); } catch (e) { console.error("Failed to open URL:", e); }
}

async function handleCopyName(sub: Subscription) {
  try { await writeText(sub.name); toast(t("copied_to_clipboard")); }
  catch (e) { console.error("Failed to copy:", e); }
}

function onSaved() {
  toast(editingSub.value ? t("subscription_updated") : t("subscription_added"));
  fetchFilteredSubs();
}

// Filter options
const sortOptions = computed<SelectOption[]>(() => [
  { value: "next_payment", label: t("next_payment") },
  { value: "name", label: t("subscription_name") },
  { value: "price", label: t("price") },
]);

const categoryFilterOptions = computed<SelectOption[]>(() => [
  { value: "", label: t("category") },
  ...catalogStore.sortedCategories.map((c) => ({ value: c.id, label: c.name, icon: c.icon || undefined })),
]);

const paymentFilterOptions = computed<SelectOption[]>(() => [
  { value: "", label: t("payment_method") },
  ...catalogStore.enabledPaymentMethods.map((pm) => ({ value: pm.id, label: pm.name, icon: pm.icon })),
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
  await subsStore.toggleFavorite(id);
  fetchFilteredSubs();
}
</script>

<template>
  <div class="max-w-5xl mx-auto">
    <!-- Search row (always visible) -->
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
      <!-- View mode + batch toggle always visible -->
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

    <!-- Filters row (scrollable on mobile) -->
    <div class="flex items-center gap-2 mb-3 overflow-x-auto pb-1 -mx-1 px-1 scrollbar-none">
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
      <div v-if="catalogStore.tags.length > 0" class="w-28 shrink-0" >
        <AppSelect v-model="filterTag" :options="tagFilterOptions" size="sm" />
      </div>
      <div class="w-32 shrink-0">
        <AppSelect :modelValue="groupBy" @update:modelValue="(v: string | number) => setGroupBy(String(v) as 'none' | 'category' | 'payment_method')" :options="groupByOptions" size="sm" />
      </div>
    </div>

    <!-- Batch toolbar -->
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 -translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-2"
    >
      <div v-if="selectionMode" class="flex items-center gap-1.5 sm:gap-2 mb-3 px-2 sm:px-3 py-2 rounded-lg bg-[var(--color-primary-light)] border border-[var(--color-primary)]/20 overflow-x-auto">
        <span class="text-xs font-medium text-[var(--color-primary)]">{{ selectedIds.size }} {{ t('selected_count') }}</span>
        <button @click="selectAll" class="text-[10px] font-medium text-[var(--color-primary)] hover:underline">{{ t('select_all') }}</button>
        <button @click="deselectAll" class="text-[10px] font-medium text-[var(--color-text-muted)] hover:underline">{{ t('deselect_all') }}</button>
        <div class="flex-1" />
        <button
          @click="batchActivate"
          :disabled="selectedIds.size === 0"
          class="px-2.5 py-1 rounded-md text-[11px] font-medium bg-green-100 text-green-700 hover:bg-green-200 dark:bg-green-900/30 dark:text-green-400 dark:hover:bg-green-900/50 disabled:opacity-30 transition-colors"
        >{{ t('batch_activate') }}</button>
        <button
          @click="batchDeactivate"
          :disabled="selectedIds.size === 0"
          class="px-2.5 py-1 rounded-md text-[11px] font-medium bg-orange-100 text-orange-700 hover:bg-orange-200 dark:bg-orange-900/30 dark:text-orange-400 dark:hover:bg-orange-900/50 disabled:opacity-30 transition-colors"
        >{{ t('batch_deactivate') }}</button>
        <button
          @click="openBatchCategoryModal"
          :disabled="selectedIds.size === 0"
          class="px-2.5 py-1 rounded-md text-[11px] font-medium bg-blue-100 text-blue-700 hover:bg-blue-200 dark:bg-blue-900/30 dark:text-blue-400 dark:hover:bg-blue-900/50 disabled:opacity-30 transition-colors"
        >{{ t('batch_change_category') }}</button>
        <button
          @click="batchDeleteSelected"
          :disabled="selectedIds.size === 0"
          class="px-2.5 py-1 rounded-md text-[11px] font-medium bg-red-100 text-red-700 hover:bg-red-200 dark:bg-red-900/30 dark:text-red-400 dark:hover:bg-red-900/50 disabled:opacity-30 transition-colors"
        >{{ t('batch_delete') }}</button>
      </div>
    </Transition>

    <!-- Empty state -->
    <div v-if="filteredSubscriptions.length === 0" class="text-center py-16">
      <div class="w-20 h-20 mx-auto mb-4 rounded-full bg-[var(--color-surface-hover)] flex items-center justify-center">
        <CreditCard :size="36" class="text-[var(--color-text-muted)]" />
      </div>
      <p class="text-[var(--color-text-muted)] mb-4">{{ t('no_subscriptions_yet') }}</p>
      <button
        @click="openAdd"
        class="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:bg-[var(--color-primary-hover)]"
      >
        <Plus :size="18" />
        {{ t('add_first_subscription') }}
      </button>
    </div>

    <!-- Subscription list -->
    <div v-else>
      <template v-for="group in groupedSubscriptions" :key="group.key">
        <!-- Group header -->
        <div v-if="groupBy !== 'none' && group.label" class="flex items-center gap-2 mb-2 mt-4 first:mt-0">
          <IconDisplay v-if="groupBy === 'category' && getCategoryIcon(group.key)" :icon="getCategoryIcon(group.key)" :size="16" />
          <FolderOpen v-else :size="14" class="text-[var(--color-primary)]" />
          <span class="text-xs font-semibold text-[var(--color-text-primary)] uppercase tracking-wide">{{ group.label }}</span>
          <span class="text-[10px] text-[var(--color-text-muted)]">({{ group.subs.length }})</span>
          <div class="flex-1 h-px bg-[var(--color-border)]" />
        </div>

        <!-- Grid for expanded mode -->
        <div :class="viewMode === 'expanded' ? 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2 sm:gap-3 mb-3' : 'space-y-1.5 sm:space-y-2 mb-3'">
          <div
            v-for="sub in group.subs"
            :key="sub.id"
            class="bg-[var(--color-surface)] rounded-xl border overflow-hidden transition-colors"
            :class="[
              sub.inactive ? 'opacity-50' : '',
              selectedIds.has(sub.id) ? 'border-[var(--color-primary)] ring-1 ring-[var(--color-primary)]/30' : 'border-[var(--color-border)]',
            ]"
          >
            <!-- COMPACT VIEW -->
            <div v-if="viewMode === 'compact'" class="flex items-center gap-2 px-3 py-2 cursor-pointer" @click="selectionMode ? toggleSelected(sub.id) : openDetail(sub)" @contextmenu.prevent="showContextMenu(sub, $event)">
              <div v-if="selectionMode" class="shrink-0" @click.stop="toggleSelected(sub.id)">
                <div class="w-4 h-4 rounded border-2 flex items-center justify-center transition-colors cursor-pointer"
                  :class="selectedIds.has(sub.id) ? 'bg-[var(--color-primary)] border-[var(--color-primary)] text-white' : 'border-[var(--color-border)] hover:border-[var(--color-primary)]'"
                ><svg v-if="selectedIds.has(sub.id)" width="10" height="10" viewBox="0 0 12 12" fill="none"><path d="M2 6L5 9L10 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg></div>
              </div>
              <Tooltip v-if="!selectionMode" :text="sub.favorite ? t('remove_from_favorites') : t('add_to_favorites')">
                <button @click.stop="subsStore.toggleFavorite(sub.id)" class="shrink-0 p-0.5 rounded transition-colors" :class="sub.favorite ? 'text-yellow-500' : 'text-[var(--color-border)] hover:text-yellow-400'">
                  <Star :size="12" :fill="sub.favorite ? 'currentColor' : 'none'" />
                </button>
              </Tooltip>
              <div class="w-6 h-6 rounded bg-[var(--color-primary-light)] flex items-center justify-center text-[10px] font-bold text-[var(--color-primary)] shrink-0 overflow-hidden">
                <img v-if="sub.logo" :src="sub.logo" class="w-full h-full object-contain" />
                <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
              </div>
              <p class="text-xs font-medium text-[var(--color-text-primary)] truncate min-w-0 flex-1">{{ sub.name }}</p>
              <span v-if="!sub.inactive" class="text-[10px] font-bold shrink-0"
                :class="getDaysUntilPayment(sub.nextPayment) <= 3 ? 'text-red-500' : getDaysUntilPayment(sub.nextPayment) <= 7 ? 'text-orange-500' : 'text-[var(--color-text-muted)]'"
              >{{ getDaysUntilPayment(sub.nextPayment) }}{{ t('days_short') }}</span>
              <p class="text-xs font-semibold text-[var(--color-text-primary)] shrink-0">{{ fmt(sub.price, sub.currencyId) }}</p>
            </div>

            <!-- EXPANDED VIEW (card) -->
            <template v-else-if="viewMode === 'expanded'">
              <div class="p-4 cursor-pointer" @click="selectionMode ? toggleSelected(sub.id) : openDetail(sub)" @contextmenu.prevent="showContextMenu(sub, $event)">
                <div class="flex items-start gap-3 mb-3">
                  <div v-if="selectionMode" class="shrink-0 mt-1" @click.stop="toggleSelected(sub.id)">
                    <div class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors cursor-pointer"
                      :class="selectedIds.has(sub.id) ? 'bg-[var(--color-primary)] border-[var(--color-primary)] text-white' : 'border-[var(--color-border)] hover:border-[var(--color-primary)]'"
                    ><svg v-if="selectedIds.has(sub.id)" width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 6L5 9L10 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg></div>
                  </div>
                  <div class="w-12 h-12 rounded-lg bg-[var(--color-primary-light)] flex items-center justify-center text-lg font-bold text-[var(--color-primary)] shrink-0 overflow-hidden">
                    <img v-if="sub.logo" :src="sub.logo" class="w-full h-full object-contain" />
                    <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
                  </div>
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <p class="text-sm font-semibold text-[var(--color-text-primary)] truncate">{{ sub.name }}</p>
                      <Tooltip v-if="!selectionMode" :text="sub.favorite ? t('remove_from_favorites') : t('add_to_favorites')">
                        <button @click.stop="subsStore.toggleFavorite(sub.id)" class="shrink-0 p-0.5 rounded transition-colors" :class="sub.favorite ? 'text-yellow-500' : 'text-[var(--color-border)] hover:text-yellow-400'">
                          <Star :size="14" :fill="sub.favorite ? 'currentColor' : 'none'" />
                        </button>
                      </Tooltip>
                    </div>
                    <p class="text-xs text-[var(--color-text-muted)] flex items-center gap-1">
                      <IconDisplay v-if="getCategoryIcon(sub.categoryId)" :icon="getCategoryIcon(sub.categoryId)" :size="12" />
                      {{ getCategoryName(sub.categoryId) }}
                    </p>
                  </div>
                </div>

                <div class="flex items-end justify-between">
                  <div>
                    <p class="text-lg font-bold text-[var(--color-text-primary)]">{{ fmt(sub.price, sub.currencyId) }}</p>
                    <p class="text-[10px] text-[var(--color-text-muted)]">{{ getBillingCycleText(sub.cycle, sub.frequency, t) }}</p>
                  </div>
                  <div class="text-right">
                    <p class="text-xs font-medium" :class="isOverdue(sub) ? 'text-red-500' : 'text-[var(--color-text-primary)]'">{{ formatDate(sub.nextPayment) }}</p>
                    <span v-if="!sub.inactive" class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-bold leading-none mt-0.5"
                      :class="getDaysUntilPayment(sub.nextPayment) <= 3 ? 'bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-400' : getDaysUntilPayment(sub.nextPayment) <= 7 ? 'bg-orange-100 text-orange-600 dark:bg-orange-900/30 dark:text-orange-400' : 'bg-[var(--color-primary-light)] text-[var(--color-primary)]'"
                    >{{ getDaysUntilPayment(sub.nextPayment) }}{{ t('days_short') }}</span>
                  </div>
                </div>

                <!-- Tags in expanded mode -->
                <div v-if="(sub.tags || []).length > 0" class="flex items-center gap-1 mt-2 flex-wrap">
                  <span v-for="tag in sub.tags" :key="tag" class="inline-flex items-center px-1.5 py-0 rounded text-[9px] font-medium bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] border border-[var(--color-border)]">#{{ tag }}</span>
                </div>

                <!-- Payment method -->
                <div class="flex items-center gap-1.5 mt-2">
                  <IconDisplay :icon="getPaymentMethod(sub.paymentMethodId)?.icon || '💳'" :size="16" />
                  <span class="text-[10px] text-[var(--color-text-muted)]">{{ getPaymentMethod(sub.paymentMethodId)?.name }}</span>
                </div>
              </div>
              <div v-if="settingsStore.settings.showSubscriptionProgress && !sub.inactive" class="h-1 bg-[var(--color-surface-hover)]">
                <div class="h-full transition-all duration-300"
                  :class="getDaysUntilPayment(sub.nextPayment) <= 3 ? 'bg-red-500' : getDaysUntilPayment(sub.nextPayment) <= 7 ? 'bg-orange-400' : 'bg-[var(--color-primary)]'"
                  :style="{ width: ((30 - getDaysUntilPayment(sub.nextPayment)) / 30 * 100) + '%' }"
                />
              </div>
            </template>

            <!-- DEFAULT VIEW -->
            <template v-else>
              <div class="flex items-center gap-2 sm:gap-3 p-3 sm:p-4 cursor-pointer" @click="selectionMode ? toggleSelected(sub.id) : openDetail(sub)" @contextmenu.prevent="showContextMenu(sub, $event)">
                <div v-if="selectionMode" class="shrink-0" @click.stop="toggleSelected(sub.id)">
                  <div class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors cursor-pointer"
                    :class="selectedIds.has(sub.id) ? 'bg-[var(--color-primary)] border-[var(--color-primary)] text-white' : 'border-[var(--color-border)] hover:border-[var(--color-primary)]'"
                  ><svg v-if="selectedIds.has(sub.id)" width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 6L5 9L10 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg></div>
                </div>
                <Tooltip v-if="!selectionMode" :text="sub.favorite ? t('remove_from_favorites') : t('add_to_favorites')">
                  <button @click.stop="subsStore.toggleFavorite(sub.id)" class="shrink-0 p-0.5 rounded transition-colors hidden sm:block" :class="sub.favorite ? 'text-yellow-500' : 'text-[var(--color-border)] hover:text-yellow-400'">
                    <Star :size="16" :fill="sub.favorite ? 'currentColor' : 'none'" />
                  </button>
                </Tooltip>
                <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-lg bg-[var(--color-primary-light)] flex items-center justify-center text-xs sm:text-sm font-bold text-[var(--color-primary)] shrink-0 overflow-hidden">
                  <img v-if="sub.logo" :src="sub.logo" class="w-full h-full object-contain" />
                  <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
                </div>
                <div class="min-w-0 flex-1">
                  <p class="text-xs sm:text-sm font-medium text-[var(--color-text-primary)] truncate">{{ sub.name }}</p>
                  <div class="flex items-center gap-1.5 flex-wrap">
                    <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)]">
                      {{ getBillingCycleText(sub.cycle, sub.frequency, t) }}
                      <span v-if="!sub.autoRenew" class="ml-1 text-orange-500">({{ t('manual_renewal') }})</span>
                    </span>
                    <span v-for="tag in (sub.tags || []).slice(0, 3)" :key="tag" class="hidden sm:inline-flex items-center px-1.5 py-0 rounded text-[9px] font-medium bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] border border-[var(--color-border)]">#{{ tag }}</span>
                    <span v-if="(sub.tags || []).length > 3" class="hidden sm:inline text-[9px] text-[var(--color-text-muted)]">+{{ sub.tags.length - 3 }}</span>
                  </div>
                </div>
                <div class="text-right shrink-0">
                  <div class="flex items-center gap-1 sm:gap-1.5 justify-end">
                    <p class="text-xs sm:text-sm font-medium" :class="isOverdue(sub) ? 'text-red-500' : 'text-[var(--color-text-primary)]'">
                      <span class="hidden sm:inline">{{ formatDate(sub.nextPayment) }}</span>
                    </p>
                    <span v-if="!sub.inactive" class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-bold leading-none"
                      :class="getDaysUntilPayment(sub.nextPayment) <= 3 ? 'bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-400' : getDaysUntilPayment(sub.nextPayment) <= 7 ? 'bg-orange-100 text-orange-600 dark:bg-orange-900/30 dark:text-orange-400' : 'bg-[var(--color-primary-light)] text-[var(--color-primary)]'"
                    >{{ getDaysUntilPayment(sub.nextPayment) }}{{ t('days_short') }}</span>
                  </div>
                </div>
                <div class="text-right shrink-0">
                  <p class="text-xs sm:text-sm font-semibold text-[var(--color-text-primary)]">{{ fmt(sub.price, sub.currencyId) }}</p>
                </div>
                <div class="shrink-0 hidden sm:block" :title="getPaymentMethod(sub.paymentMethodId)?.name">
                  <IconDisplay :icon="getPaymentMethod(sub.paymentMethodId)?.icon || '💳'" :size="22" />
                </div>
              </div>
              <div v-if="settingsStore.settings.showSubscriptionProgress && !sub.inactive" class="h-1 bg-[var(--color-surface-hover)]">
                <div class="h-full transition-all duration-300"
                  :class="getDaysUntilPayment(sub.nextPayment) <= 3 ? 'bg-red-500' : getDaysUntilPayment(sub.nextPayment) <= 7 ? 'bg-orange-400' : 'bg-[var(--color-primary)]'"
                  :style="{ width: ((30 - getDaysUntilPayment(sub.nextPayment)) / 30 * 100) + '%' }"
                  :title="getDaysUntilPayment(sub.nextPayment) + ' ' + t('days')"
                />
              </div>
            </template>
          </div>
        </div>
      </template>
    </div>

    <!-- Detail Panel -->
    <SubscriptionDetail
      :show="showDetail"
      :subscription="detailSub"
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
      :show="showForm"
      :edit-subscription="editingSub"
      @close="showForm = false"
      @saved="onSaved"
    />

    <!-- Delete Confirmation Modal -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div v-if="deleteConfirmId" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
          <div class="absolute inset-0 bg-black/50" @click="cancelDelete" />
          <div class="relative bg-[var(--color-surface)] w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-sm p-4 sm:p-6">
            <div class="flex items-center gap-3 mb-3 sm:mb-4">
              <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center shrink-0">
                <AlertTriangle :size="18" class="text-red-500" />
              </div>
              <h3 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ t('delete') }}</h3>
            </div>
            <p class="text-xs sm:text-sm text-[var(--color-text-secondary)] mb-4 sm:mb-6">{{ t('confirm_delete_subscription') }}</p>
            <div class="flex justify-end gap-2 sm:gap-3">
              <button
                @click="cancelDelete"
                class="px-3 sm:px-4 py-2 rounded-lg border border-[var(--color-border)] text-xs sm:text-sm font-medium text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]"
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
      <Transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div v-if="batchDeleteConfirmIds.length > 0" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
          <div class="absolute inset-0 bg-black/50" @click="batchDeleteConfirmIds = []" />
          <div class="relative bg-[var(--color-surface)] w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-sm p-4 sm:p-6">
            <div class="flex items-center gap-3 mb-3 sm:mb-4">
              <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center shrink-0">
                <AlertTriangle :size="18" class="text-red-500" />
              </div>
              <h3 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ t('batch_delete') }}</h3>
            </div>
            <p class="text-xs sm:text-sm text-[var(--color-text-secondary)] mb-4 sm:mb-6">{{ t('batch_confirm_delete').replace('{count}', String(batchDeleteConfirmIds.length)) }}</p>
            <div class="flex justify-end gap-2 sm:gap-3">
              <button @click="batchDeleteConfirmIds = []" class="px-3 sm:px-4 py-2 rounded-lg border border-[var(--color-border)] text-xs sm:text-sm font-medium text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]">{{ t('cancel') }}</button>
              <button @click="confirmBatchDelete" class="px-3 sm:px-4 py-2 rounded-lg bg-red-600 text-white text-xs sm:text-sm font-medium hover:bg-red-700">{{ t('delete') }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Batch Change Category Modal -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div v-if="showBatchCategoryModal" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
          <div class="absolute inset-0 bg-black/50" @click="showBatchCategoryModal = false" />
          <div class="relative bg-[var(--color-surface)] w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-sm p-4 sm:p-6">
            <h3 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)] mb-3 sm:mb-4">{{ t('batch_change_category') }}</h3>
            <p class="text-xs sm:text-sm text-[var(--color-text-muted)] mb-3">{{ selectedIds.size }} {{ t('selected_count') }}</p>
            <AppSelect
              v-model="batchCategoryId"
              :options="categoryFilterOptions.filter((o) => o.value !== '')"
              :label="t('category')"
            />
            <div class="flex justify-end gap-3 mt-5">
              <button @click="showBatchCategoryModal = false" class="px-4 py-2 rounded-lg border border-[var(--color-border)] text-sm font-medium text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]">{{ t('cancel') }}</button>
              <button @click="confirmBatchCategory" class="px-5 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:bg-[var(--color-primary-hover)]">{{ t('save') }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </div>
</template>
