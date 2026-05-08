<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { useClipboard } from "@/composables/useClipboard";
import { useHeaderActions } from "@/composables/useHeaderActions";
import { getCurrencyFlags, ratesRunBackendUpdate } from "@/services/ratesClient";
import { updateCurrencyRates } from "@/services/catalogClient";
import { storeToRefs } from "pinia";
import type { Currency, Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import Toast from "@/components/ui/Toast.vue";
import { typo } from "@/lib/tv";
import Tooltip from "@/components/ui/Tooltip.vue";
import { formatErrorForToast } from "@/utils/formatError";
import AppSlider from "@/components/ui/AppSlider.vue";
import ExpenseForm from "@/components/expenses/ExpenseForm.vue";
import {
  RefreshCw, Copy, ArrowRightLeft, TrendingUp,
  Search, Star, Check, Plus, RotateCcw, ChevronUp, ChevronDown, ArrowUpDown, LayoutList, LayoutGrid, Rows3, Hash, Type, CircleDollarSign, CheckSquare, Square, Pencil, X,
} from "@lucide/vue";

const { t } = useI18n();
const metaStore = useAppMetaStore();
const { settings, currencies, paymentMethods, household, categories, tags } = storeToRefs(metaStore);
const converterBaseAmount = ref(1);
const converterOrder = ref<string[]>([]);
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();
const { copyToClipboard } = useClipboard();
const { setActions, clearActions } = useHeaderActions();

onMounted(() => {
  updateHeaderActions();
});
onUnmounted(() => {
  clearActions();
});

const searchQuery = ref("");
const isUpdating = ref(false);
const copiedId = ref<string | null>(null);
const editingRateId = ref<string | null>(null);
const editingRateValue = ref("");
const currencyFlags = ref<Record<string, string>>({});

type RateSortKey = "code" | "name" | "rate" | "enabled";
const rateSortBy = ref<RateSortKey>("enabled");
const rateSortOptions: { key: RateSortKey; labelKey: string }[] = [
  { key: "enabled", labelKey: "sort_enabled_first" },
  { key: "code", labelKey: "sort_by_code" },
  { key: "name", labelKey: "sort_by_name" },
  { key: "rate", labelKey: "sort_by_rate" },
];

function getSortIcon(key: RateSortKey) {
  if (key === "enabled") return Star;
  if (key === "code") return Hash;
  if (key === "name") return Type;
  return CircleDollarSign;
}

const showExpenseForm = ref(false);
const expensePrefill = ref<{ amount: number; currencyId: string; name: string } | null>(null);
const viewMode = computed(() => settings.value?.currencyViewMode || "default");
const isCompactView = computed(() => viewMode.value === "compact");

async function updateSettings(updates: Partial<Settings>) {
  if (!settings.value) return;
  await metaStore.updateSettings({ ...settings.value, ...updates });
}

function setViewMode(mode: "default" | "compact" | "expanded") {
  updateSettings({ currencyViewMode: mode });
}

function updateHeaderActions() {
  const viewIcon = viewMode.value === "compact" ? Rows3 : viewMode.value === "expanded" ? LayoutGrid : LayoutList;
  const nextViewMode = viewMode.value === "compact" ? "default" : viewMode.value === "default" ? "expanded" : "compact";
  const currentViewTitle = viewMode.value === "compact" ? t("view_compact") : viewMode.value === "expanded" ? t("view_expanded") : t("view_default");
  const nextViewTitle = nextViewMode === "compact" ? t("view_compact") : nextViewMode === "expanded" ? t("view_expanded") : t("view_default");

  setActions([
    { id: "currency-update", icon: RefreshCw, title: t("update_now"), onClick: handleUpdate, style: "primary" },
    { id: "cycle-currency-view", icon: viewIcon, title: `${currentViewTitle} → ${nextViewTitle}`, onClick: () => setViewMode(nextViewMode), style: "accent" },
  ]);
}

watch(viewMode, updateHeaderActions);
watch(isUpdating, updateHeaderActions);

async function refreshCurrencyFlags() {
  const codes = [...new Set(currencies.value.map((c) => c.code).filter(Boolean))];
  currencyFlags.value = codes.length ? await getCurrencyFlags(codes) : {};
}

function flagFor(code: string): string {
  return currencyFlags.value[code.toUpperCase()] || "";
}

const baseAmount = computed({
  get: () => converterBaseAmount.value,
  set: (v: number) => { converterBaseAmount.value = v; },
});
const editingId = ref<string | null>(null);

const mainCurrency = computed(() =>
  currencies.value.find((c) => c.id === settings.value?.mainCurrencyId) ?? null,
);
function normalizedCode(code: string | null | undefined): string {
  return String(code ?? "").trim().toUpperCase();
}

const otherCurrencies = computed(() => {
  const mainCode = normalizedCode(mainCurrency.value?.code);
  const seenCodes = new Set<string>();
  const out: Currency[] = [];

  for (const c of currencies.value) {
    if (c.id === settings.value?.mainCurrencyId) continue;
    const code = normalizedCode(c.code);
    if (!code) continue;
    if (mainCode && code === mainCode) continue;
    if (seenCodes.has(code)) continue;
    seenCodes.add(code);
    out.push(c);
  }
  return out;
});
const selectedTargetIds = computed(() => settings.value?.currencyUpdateTargets ?? []);
const expenseFormLookupData = computed(() => {
  if (!settings.value) return null;
  return {
    settings: settings.value,
    currencies: currencies.value,
    paymentMethods: paymentMethods.value,
    household: household.value,
    categories: categories.value,
    tags: tags.value,
    expensesCount: 0,
  };
});

interface ConverterCurrency {
  id: string;
  code: string;
  name: string;
  symbol: string;
  rate: number;
  isBase: boolean;
}

const allConverterIds = computed(() => {
  const base = mainCurrency.value;
  if (!base) return [];
  const baseCode = normalizedCode(base.code);
  const seenCodes = new Set<string>([baseCode]);
  const orderedTargets: string[] = [];

  for (const id of selectedTargetIds.value) {
    const c = currencies.value.find((cur) => cur.id === id);
    if (!c || c.rate <= 0) continue;
    const code = normalizedCode(c.code);
    if (!code || seenCodes.has(code)) continue;
    seenCodes.add(code);
    orderedTargets.push(c.id);
  }

  return [base.id, ...orderedTargets];
});

watch(allConverterIds, (ids) => {
  const current = converterOrder.value;
  if (current.length === 0 || ids.length !== current.length || !ids.every((id) => current.includes(id))) {
    converterOrder.value = [...ids];
  }
}, { immediate: true });

const converterCurrencies = computed<ConverterCurrency[]>(() => {
  const base = mainCurrency.value;
  if (!base) return [];
  const ordered = converterOrder.value.length ? converterOrder.value : allConverterIds.value;
  return ordered
    .map((id) => {
      const c = currencies.value.find((cur) => cur.id === id);
      if (!c) return null;
      return {
        id: c.id, code: c.code, name: c.name, symbol: c.symbol,
        rate: c.rate, isBase: c.id === base.id,
      };
    })
    .filter((x): x is ConverterCurrency => x !== null);
});

function moveConverterUp(id: string) {
  const idx = converterOrder.value.indexOf(id);
  if (idx > 0) {
    const arr = [...converterOrder.value];
    [arr[idx - 1], arr[idx]] = [arr[idx], arr[idx - 1]];
    converterOrder.value = arr;
  }
}

function moveConverterDown(id: string) {
  const idx = converterOrder.value.indexOf(id);
  if (idx >= 0 && idx < converterOrder.value.length - 1) {
    const arr = [...converterOrder.value];
    [arr[idx], arr[idx + 1]] = [arr[idx + 1], arr[idx]];
    converterOrder.value = arr;
  }
}

function resetConverter() {
  converterBaseAmount.value = 1;
}

function getRawValue(cur: ConverterCurrency): number {
  return cur.isBase ? baseAmount.value : baseAmount.value * cur.rate;
}

function openExpenseForm(cur: ConverterCurrency) {
  expensePrefill.value = {
    amount: Math.round(getRawValue(cur) * 100) / 100,
    currencyId: cur.id,
    name: "",
  };
  showExpenseForm.value = true;
}

function getDisplayValue(cur: ConverterCurrency): string {
  if (editingId.value === cur.id) {
    const raw = getRawValue(cur);
    const s = raw.toFixed(4);
    return s.replace(/\.?0+$/, "") || "0";
  }
  return fmtNum(getRawValue(cur));
}

function onInput(cur: ConverterCurrency, raw: string) {
  editingId.value = cur.id;
  const val = parseFloat(raw.replace(/[^\d.\-]/g, ""));
  if (isNaN(val) || val < 0) return;
  if (cur.isBase) {
    baseAmount.value = val;
  } else {
    baseAmount.value = cur.rate > 0 ? val / cur.rate : 0;
  }
}

function onFocus(cur: ConverterCurrency, el: HTMLInputElement) {
  editingId.value = cur.id;
  const raw = getRawValue(cur);
  const s = raw.toFixed(4).replace(/\.?0+$/, "") || "0";
  el.value = s;
  requestAnimationFrame(() => el.select());
}

function sliderParams(cur: ConverterCurrency): { max: number; step: number } {
  const raw = getRawValue(cur);
  if (raw <= 0) return { max: 100, step: 0.1 };
  const exp = Math.floor(Math.log10(raw));
  const mag = Math.pow(10, exp);
  const max = Math.ceil((raw * 3) / mag) * mag;
  const step = mag / 100;
  return { max, step: Math.max(step, 1e-10) };
}

function onSlider(cur: ConverterCurrency, val: number) {
  if (cur.isBase) {
    baseAmount.value = val;
  } else {
    baseAmount.value = cur.rate > 0 ? val / cur.rate : 0;
  }
}

function onBlur() {
  editingId.value = null;
}

// Rates table
const filteredCurrencies = computed(() => {
  if (!searchQuery.value) return otherCurrencies.value;
  const q = searchQuery.value.toLowerCase();
  return otherCurrencies.value.filter(
    (c) => c.name.toLowerCase().includes(q) || c.code.toLowerCase().includes(q),
  );
});

const allRates = computed(() => {
  const items = filteredCurrencies.value.map((c) => ({
    ...c,
    rateFormatted: c.rate?.toFixed(4) ?? "—",
    inverse: c.rate ? (1 / c.rate).toFixed(4) : "—",
    enabled: selectedTargetIds.value.includes(c.id),
  }));
  const key = rateSortBy.value;
  return items.sort((a, b) => {
    if (key === "enabled") {
      if (a.enabled !== b.enabled) return a.enabled ? -1 : 1;
      return a.code.localeCompare(b.code);
    }
    if (key === "code") return a.code.localeCompare(b.code);
    if (key === "name") return a.name.localeCompare(b.name);
    return (a.rate ?? 0) - (b.rate ?? 0);
  });
});

const lastUpdate = computed(() => settings.value?.lastCurrencyUpdate || t("never"));

function toggleTarget(id: string) {
  const current = [...selectedTargetIds.value];
  const idx = current.indexOf(id);
  if (idx >= 0) current.splice(idx, 1);
  else current.push(id);
  updateSettings({ currencyUpdateTargets: current });
}

function selectAllTargets() {
  const ids = otherCurrencies.value.map((c) => c.id);
  updateSettings({ currencyUpdateTargets: ids });
}

function deselectAllTargets() {
  updateSettings({ currencyUpdateTargets: [] });
}

async function copyRow(text: string, id: string) {
  const copied = await copyToClipboard(text);
  if (!copied) {
    toast(t("clipboard_copy_failed"), "error");
    return;
  }
  copiedId.value = id;
  setTimeout(() => { if (copiedId.value === id) copiedId.value = null; }, 1200);
  toast(t("copied_to_clipboard"));
}

async function handleUpdate() {
  isUpdating.value = true;
  try {
    const result = await ratesRunBackendUpdate();
    if (result.error) toast(result.error, "error");
    else if (result.updated > 0) toast(t("rates_updated").replace("{count}", String(result.updated)));
    else toast(t("no_rates_updated"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
  finally {
    isUpdating.value = false;
  }
}

function startRateEdit(rate: { id: string; rate: number }) {
  editingRateId.value = rate.id;
  editingRateValue.value = Number.isFinite(rate.rate) ? String(rate.rate) : "";
}

function cancelRateEdit() {
  editingRateId.value = null;
  editingRateValue.value = "";
}

async function saveRateEdit(rate: { id: string; code: string }) {
  const normalized = editingRateValue.value.replace(",", ".").trim();
  const nextRate = Number(normalized);
  if (!Number.isFinite(nextRate) || nextRate <= 0) {
    toast("Rate must be a positive number", "error");
    return;
  }

  try {
    await updateCurrencyRates([{ id: rate.id, rate: nextRate }]);
    await metaStore.refresh();
    await refreshCurrencyFlags();
    toast(`${rate.code} ${t("save").toLowerCase()}`);
    cancelRateEdit();
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

function fmtNum(n: number): string {
  return n.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 4 });
}

async function copyAllConversion() {
  const lines = converterCurrencies.value.map((cur) => {
    const flag = flagFor(cur.code);
    const val = fmtNum(getRawValue(cur));
    return `${flag ? flag + " " : ""}${cur.code}: ${val} ${cur.symbol}`;
  });
  const text = lines.join("\n");
  const copied = await copyToClipboard(text);
  if (!copied) {
    toast(t("clipboard_copy_failed"), "error");
    return;
  }
  toast(t("copied_to_clipboard"));
}

onMounted(async () => {
  await metaStore.ensureLoaded();
  await refreshCurrencyFlags();
});
watch(
  () => currencies.value.map((c) => c.code).join(","),
  () => {
    refreshCurrencyFlags();
  },
);
</script>

<template>
  <div class="space-y-4">

    <!-- Header -->
    <div class="flex items-center justify-between gap-3">
      <div class="min-w-0">
        <h1 :class="typo.screenTitle()">{{ t('exchange_rates') }}</h1>
        <p class="text-xs text-text-muted mt-0.5">{{ t('last_update') }}: {{ lastUpdate }}</p>
      </div>
    </div>

    <!-- Converter empty state -->
    <div v-if="currencies.length === 0" class="bg-surface rounded-xl border border-dashed border-border p-5 text-center">
      <ArrowRightLeft :size="28" class="mx-auto mb-2 text-text-muted opacity-40" />
      <p class="text-sm font-semibold text-text-secondary mb-1">{{ t('no_data_yet') }}</p>
      <p class="text-xs text-text-muted">{{ t('exchange_rates_desc') }}</p>
    </div>
    <div v-else-if="converterCurrencies.length <= 1" class="bg-surface rounded-xl border border-dashed border-border p-5 text-center">
      <ArrowRightLeft :size="28" class="mx-auto mb-2 text-text-muted opacity-40" />
      <p class="text-sm font-semibold text-text-secondary mb-1">{{ t('converter_empty') }}</p>
      <p class="text-xs text-text-muted">{{ t('converter_empty_hint') }}</p>
    </div>

    <!-- Converter -->
    <div
      v-else
      class="bg-surface rounded-xl border border-border"
      :class="isCompactView ? 'p-3' : viewMode === 'expanded' ? 'p-3 sm:p-5' : 'p-3 sm:p-4'"
    >
      <div class="flex items-center justify-between" :class="isCompactView ? 'mb-2' : 'mb-3'">
        <div class="flex items-center gap-2 text-sm font-semibold text-text-primary">
          <ArrowRightLeft :size="15" class="text-primary" />
          {{ t('currency_converter') }}
        </div>
        <div v-if="!isCompactView" class="flex items-center gap-1">
          <Tooltip :text="t('copy')">
            <button
              @click="copyAllConversion"
              class="p-1.5 rounded-lg text-text-muted hover:text-primary hover:bg-surface-secondary transition-colors"
            >
              <Copy :size="14" />
            </button>
          </Tooltip>
          <Tooltip :text="t('reset')">
            <button
              @click="resetConverter()"
              class="p-1.5 rounded-lg text-text-muted hover:text-primary hover:bg-surface-secondary transition-colors"
            >
              <RotateCcw :size="14" />
            </button>
          </Tooltip>
        </div>
      </div>

      <!-- Quick presets -->
      <div class="flex flex-wrap gap-1.5" :class="isCompactView ? 'mb-2' : 'mb-3'">
        <button
          v-for="preset in (settings?.converterPresets || [])"
          :key="preset"
          @click="baseAmount = preset"
          :class="[
            'px-3 py-1 text-xs font-semibold rounded-lg border transition-colors',
            baseAmount === preset
              ? 'bg-primary text-white border-primary'
              : 'bg-surface-secondary text-text-secondary border-border hover:bg-surface hover:border-text-muted',
          ]"
        >
          {{ fmtNum(preset) }}
        </button>
      </div>

      <div v-if="viewMode === 'expanded'" class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
        <div
          v-for="(cur, idx) in converterCurrencies"
          :key="cur.id"
          class="rounded-xl border border-border bg-surface-secondary p-3"
        >
          <div class="flex items-start justify-between gap-2 mb-2">
            <div class="min-w-0">
              <div class="text-sm font-bold text-text-primary leading-tight">
                <span v-if="flagFor(cur.code)" class="mr-0.5">{{ flagFor(cur.code) }}</span>{{ cur.code }}
              </div>
              <div class="text-xs text-text-muted truncate">{{ cur.name }}</div>
            </div>
            <div class="flex items-center gap-0.5 shrink-0">
              <Tooltip :text="t('move_up')">
                <button
                  @click="moveConverterUp(cur.id)"
                  :disabled="idx === 0"
                  class="p-1 rounded text-text-muted hover:text-primary disabled:opacity-20 disabled:cursor-not-allowed transition-colors"
                ><ChevronUp :size="14" /></button>
              </Tooltip>
              <Tooltip :text="t('move_down')">
                <button
                  @click="moveConverterDown(cur.id)"
                  :disabled="idx === converterCurrencies.length - 1"
                  class="p-1 rounded text-text-muted hover:text-primary disabled:opacity-20 disabled:cursor-not-allowed transition-colors"
                ><ChevronDown :size="14" /></button>
              </Tooltip>
            </div>
          </div>

          <div class="relative">
            <input
              type="text"
              inputmode="decimal"
              pattern="[0-9]*[.,]?[0-9]*"
              :value="getDisplayValue(cur)"
              @input="onInput(cur, ($event.target as HTMLInputElement).value)"
              @focus="onFocus(cur, $event.target as HTMLInputElement)"
              @blur="onBlur"
              class="w-full pl-3 pr-12 py-2.5 text-lg font-bold rounded-lg bg-surface border border-border text-text-primary text-right tabular-nums focus:outline-none focus:ring-2 focus:ring-primary focus:border-primary transition-shadow"
            />
            <span class="absolute right-3 top-1/2 -translate-y-1/2 text-base font-semibold text-primary pointer-events-none select-none">
              {{ cur.symbol }}
            </span>
          </div>
          <AppSlider
            :modelValue="getRawValue(cur)"
            @update:modelValue="onSlider(cur, $event)"
            :min="0"
            :max="sliderParams(cur).max"
            :step="sliderParams(cur).step"
          />
          <div v-if="!cur.isBase" class="text-[10px] text-text-muted tabular-nums mt-1">
            1 {{ mainCurrency?.code }} = {{ cur.rate.toFixed(4).replace(/\.?0+$/, '') }} {{ cur.code }}
            <span class="opacity-60 mx-1">·</span>
            1 {{ cur.code }} = {{ (1 / cur.rate).toFixed(4).replace(/\.?0+$/, '') }} {{ mainCurrency?.code }}
          </div>

          <div class="flex items-center justify-end gap-1 mt-2">
            <Tooltip :text="t('add_expense')">
              <button
                @click="openExpenseForm(cur)"
                class="p-1.5 rounded-lg transition-all text-text-muted hover:text-emerald-600 hover:bg-emerald-50 dark:hover:bg-emerald-900/20"
              >
                <Plus :size="14" />
              </button>
            </Tooltip>
            <Tooltip :text="t('copy')">
              <button
                @click="copyRow(`${getDisplayValue(cur)} ${cur.code}`, `conv-${cur.id}`)"
                class="p-1.5 rounded-lg transition-all"
                :class="copiedId === `conv-${cur.id}`
                  ? 'text-green-500'
                  : 'text-text-muted hover:text-primary hover:bg-surface'"
              >
                <component :is="copiedId === `conv-${cur.id}` ? Check : Copy" :size="14" />
              </button>
            </Tooltip>
          </div>
        </div>
      </div>
      <div v-else :class="isCompactView ? 'space-y-1.5' : 'space-y-2'">
        <div
          v-for="(cur, idx) in converterCurrencies"
          :key="cur.id"
          class="flex items-center"
          :class="isCompactView ? 'gap-1' : 'gap-1.5 sm:gap-2'"
        >
          <div v-if="!isCompactView" class="flex flex-row sm:flex-col shrink-0">
            <Tooltip :text="t('move_up')">
              <button
                @click="moveConverterUp(cur.id)"
                :disabled="idx === 0"
                class="p-0.5 rounded text-text-muted hover:text-primary disabled:opacity-20 disabled:cursor-not-allowed transition-colors"
              ><ChevronUp :size="14" /></button>
            </Tooltip>
            <Tooltip :text="t('move_down')">
              <button
                @click="moveConverterDown(cur.id)"
                :disabled="idx === converterCurrencies.length - 1"
                class="p-0.5 rounded text-text-muted hover:text-primary disabled:opacity-20 disabled:cursor-not-allowed transition-colors"
              ><ChevronDown :size="14" /></button>
            </Tooltip>
          </div>
          <div class="shrink-0" :class="isCompactView ? 'w-14 sm:w-16' : 'w-14 sm:w-20'">
            <span class="text-sm font-bold text-text-primary block leading-tight">
              <span v-if="flagFor(cur.code)" class="mr-0.5">{{ flagFor(cur.code) }}</span>{{ cur.code }}
            </span>
            <span class="text-[10px] text-text-muted leading-tight truncate block max-w-14 sm:max-w-none">{{ cur.name }}</span>
          </div>
          <div class="flex-1 min-w-0">
            <div class="relative">
              <input
                type="text"
                inputmode="decimal"
                pattern="[0-9]*[.,]?[0-9]*"
                :value="getDisplayValue(cur)"
                @input="onInput(cur, ($event.target as HTMLInputElement).value)"
                @focus="onFocus(cur, $event.target as HTMLInputElement)"
                @blur="onBlur"
                class="w-full pl-3 pr-12 py-2.5 text-lg font-bold rounded-lg bg-surface-secondary border border-border text-text-primary text-right tabular-nums focus:outline-none focus:ring-2 focus:ring-primary focus:border-primary transition-shadow"
              />
              <span class="absolute right-3 top-1/2 -translate-y-1/2 text-base font-semibold text-primary pointer-events-none select-none">
                {{ cur.symbol }}
              </span>
            </div>
            <AppSlider
              :modelValue="getRawValue(cur)"
              @update:modelValue="onSlider(cur, $event)"
              :min="0"
              :max="sliderParams(cur).max"
              :step="sliderParams(cur).step"
            />
            <div v-if="!cur.isBase" class="text-[10px] text-text-muted tabular-nums mt-0.5 pl-1">
              1 {{ mainCurrency?.code }} = {{ cur.rate.toFixed(4).replace(/\.?0+$/, '') }} {{ cur.code }}
              <span class="opacity-60 mx-1">·</span>
              1 {{ cur.code }} = {{ (1 / cur.rate).toFixed(4).replace(/\.?0+$/, '') }} {{ mainCurrency?.code }}
            </div>
          </div>
          <div v-if="!isCompactView" class="flex flex-row shrink-0 gap-0.5">
            <Tooltip :text="t('add_expense')">
              <button
                @click="openExpenseForm(cur)"
                class="p-1 rounded-lg sm:p-1.5 transition-all text-text-muted hover:text-emerald-600 hover:bg-emerald-50 dark:hover:bg-emerald-900/20"
              >
                <Plus :size="14" />
              </button>
            </Tooltip>
            <Tooltip :text="t('copy')">
              <button
                @click="copyRow(`${getDisplayValue(cur)} ${cur.code}`, `conv-${cur.id}`)"
                class="p-1 rounded-lg sm:p-1.5 transition-all"
                :class="copiedId === `conv-${cur.id}`
                  ? 'text-green-500'
                  : 'text-text-muted hover:text-primary hover:bg-surface'"
              >
                <component :is="copiedId === `conv-${cur.id}` ? Check : Copy" :size="14" />
              </button>
            </Tooltip>
          </div>
        </div>
      </div>
    </div>

    <!-- All rates -->
    <div
      class="bg-surface rounded-xl border border-border"
      :class="isCompactView ? 'p-3' : viewMode === 'expanded' ? 'p-3 sm:p-5' : 'p-3 sm:p-4'"
    >
      <div class="flex items-center justify-between" :class="isCompactView ? 'mb-2' : 'mb-3'">
        <div class="flex items-center gap-2 text-sm font-semibold text-text-primary">
          <TrendingUp :size="15" class="text-primary" />
          {{ t('all_rates') }}
          <span class="text-xs font-normal text-text-muted">
            ({{ selectedTargetIds.length }}/{{ otherCurrencies.length }})
          </span>
        </div>
        <div class="flex items-center gap-1.5">
          <button
            @click="selectAllTargets"
            :disabled="selectedTargetIds.length === otherCurrencies.length"
            class="px-2 py-0.5 text-[11px] rounded border border-border text-text-secondary hover:bg-surface hover:border-text-muted disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
            :title="t('select_all')"
          >
            <CheckSquare :size="13" class="sm:hidden" />
            <span class="hidden sm:inline">{{ t('select_all') }}</span>
          </button>
          <button
            @click="deselectAllTargets"
            :disabled="selectedTargetIds.length === 0"
            class="px-2 py-0.5 text-[11px] rounded border border-border text-text-secondary hover:bg-surface hover:border-text-muted disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
            :title="t('deselect_all')"
          >
            <Square :size="13" class="sm:hidden" />
            <span class="hidden sm:inline">{{ t('deselect_all') }}</span>
          </button>
          <span class="text-xs text-text-muted tabular-nums ml-1">
            1 {{ mainCurrency?.code }} =
          </span>
        </div>
      </div>

      <!-- Search + Sort -->
      <div class="flex items-center gap-1.5 sm:gap-2" :class="isCompactView ? 'mb-2' : 'mb-3'">
        <div class="relative flex-1">
          <Search :size="14" class="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('search') + '...'"
            class="w-full pl-8 sm:pl-9 pr-2.5 sm:pr-3 py-1.5 sm:py-2 rounded-lg border border-border bg-surface text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary placeholder:text-text-muted"
          />
        </div>
        <div class="flex items-center gap-0.5 shrink-0">
          <ArrowUpDown :size="12" class="text-text-muted mr-0.5" />
          <button
            v-for="opt in rateSortOptions"
            :key="opt.key"
            @click="rateSortBy = opt.key"
            :title="t(opt.labelKey)"
            :class="[
              'px-1.5 sm:px-2 py-1 text-[10px] sm:text-[11px] rounded border transition-colors inline-flex items-center justify-center gap-1',
              rateSortBy === opt.key
                ? 'bg-primary text-white border-primary'
                : 'bg-surface-secondary text-text-secondary border-border hover:bg-surface hover:border-text-muted',
            ]"
          >
            <component :is="getSortIcon(opt.key)" :size="12" class="sm:hidden" />
            <span class="hidden sm:inline">{{ t(opt.labelKey) }}</span>
          </button>
        </div>
      </div>

      <!-- List -->
      <div v-if="viewMode === 'expanded'" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2.5">
        <div
          v-for="rate in allRates"
          :key="rate.id"
          class="rounded-xl border border-border bg-surface-secondary p-3"
        >
          <div class="flex items-start justify-between gap-2">
            <div class="min-w-0">
              <div class="text-sm font-bold text-text-primary">
                <span v-if="flagFor(rate.code)" class="mr-0.5">{{ flagFor(rate.code) }}</span>{{ rate.code }}
              </div>
              <div class="text-xs text-text-muted truncate">{{ rate.name }}</div>
            </div>
            <Tooltip :text="rate.enabled ? t('disable') : t('enable')">
              <button
                @click="toggleTarget(rate.id)"
                class="p-1 rounded transition-colors shrink-0"
                :class="rate.enabled
                  ? 'text-yellow-500 hover:text-yellow-600'
                  : 'text-text-muted opacity-40 hover:opacity-70'"
              >
                <Star :size="14" :fill="rate.enabled ? 'currentColor' : 'none'" />
              </button>
            </Tooltip>
          </div>
          <div class="mt-2 text-lg font-mono font-semibold text-text-primary tabular-nums">
            <input
              v-if="editingRateId === rate.id"
              v-model="editingRateValue"
              type="text"
              inputmode="decimal"
              @keydown.enter.prevent="saveRateEdit(rate)"
              class="w-full px-2 py-1 rounded-md border border-border bg-surface text-base font-mono text-text-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
            <template v-else>{{ rate.rateFormatted }}</template>
          </div>
          <div class="text-xs font-mono text-text-muted tabular-nums mt-0.5">
            1/{{ rate.inverse }}
          </div>
          <div class="flex justify-end mt-2">
            <Tooltip v-if="editingRateId !== rate.id" :text="t('save')">
              <button
                @click="startRateEdit(rate)"
                class="p-1.5 rounded-lg transition-all shrink-0 text-text-muted hover:text-primary"
              >
                <Pencil :size="13" />
              </button>
            </Tooltip>
            <template v-else>
              <Tooltip :text="t('save')">
                <button
                  @click="saveRateEdit(rate)"
                  class="p-1.5 rounded-lg transition-all shrink-0 text-green-600 hover:text-green-700"
                >
                  <Check :size="13" />
                </button>
              </Tooltip>
              <Tooltip :text="t('cancel')">
                <button
                  @click="cancelRateEdit"
                  class="p-1.5 rounded-lg transition-all shrink-0 text-text-muted hover:text-primary"
                >
                  <X :size="13" />
                </button>
              </Tooltip>
            </template>
            <Tooltip :text="t('copy')">
              <button
                @click="copyRow(`1 ${mainCurrency?.code} = ${rate.rateFormatted} ${rate.code}`, `rate-${rate.id}`)"
                class="p-1.5 rounded-lg transition-all shrink-0"
                :class="copiedId === `rate-${rate.id}`
                  ? 'text-green-500'
                  : 'text-text-muted hover:text-primary'"
              >
                <component :is="copiedId === `rate-${rate.id}` ? Check : Copy" :size="13" />
              </button>
            </Tooltip>
          </div>
        </div>
        <p v-if="allRates.length === 0" class="col-span-full text-center text-sm text-text-muted py-6">
          {{ t('no_results') }}
        </p>
      </div>
      <div v-else class="max-h-128 overflow-y-auto -mx-1 px-0.5 sm:px-1">
        <div
          v-for="rate in allRates"
          :key="rate.id"
          class="flex items-center gap-1.5 sm:gap-2 px-1.5 sm:px-2 py-1.5 sm:py-2 rounded-lg hover:bg-surface-hover transition-colors group border-b border-border/50 last:border-b-0"
        >
          <Tooltip :text="rate.enabled ? t('disable') : t('enable')">
            <button
              @click="toggleTarget(rate.id)"
              class="p-0.5 rounded transition-colors shrink-0"
              :class="rate.enabled
                ? 'text-yellow-500 hover:text-yellow-600'
                : 'text-text-muted opacity-25 hover:opacity-60'"
            >
              <Star :size="14" :fill="rate.enabled ? 'currentColor' : 'none'" />
            </button>
          </Tooltip>
          <span class="text-sm font-bold text-text-primary w-14 sm:w-16 shrink-0">
            <span v-if="flagFor(rate.code)" class="mr-0.5">{{ flagFor(rate.code) }}</span>{{ rate.code }}
          </span>
          <span class="text-xs sm:text-sm text-text-muted flex-1 min-w-0 truncate">{{ rate.name }}</span>
          <input
            v-if="editingRateId === rate.id"
            v-model="editingRateValue"
            type="text"
            inputmode="decimal"
            @keydown.enter.prevent="saveRateEdit(rate)"
            class="w-24 px-2 py-1 rounded-md border border-border bg-surface text-xs sm:text-sm font-mono font-semibold text-text-primary tabular-nums focus:outline-none focus:ring-1 focus:ring-primary"
          />
          <span v-else class="text-xs sm:text-sm font-mono font-semibold text-text-primary tabular-nums">{{ rate.rateFormatted }}</span>
          <span class="text-xs font-mono text-text-muted tabular-nums w-18 text-right hidden sm:block">1/{{ rate.inverse }}</span>
          <Tooltip :text="editingRateId === rate.id ? t('save') : t('save')">
            <button
              v-if="editingRateId !== rate.id"
              @click="startRateEdit(rate)"
              class="p-1 rounded-lg transition-all shrink-0 text-text-muted hover:text-primary"
            >
              <Pencil :size="13" />
            </button>
            <button
              v-else
              @click="saveRateEdit(rate)"
              class="p-1 rounded-lg transition-all shrink-0 text-green-600 hover:text-green-700"
            >
              <Check :size="13" />
            </button>
          </Tooltip>
          <Tooltip v-if="editingRateId === rate.id" :text="t('cancel')">
            <button
              @click="cancelRateEdit"
              class="p-1 rounded-lg transition-all shrink-0 text-text-muted hover:text-primary"
            >
              <X :size="13" />
            </button>
          </Tooltip>
          <Tooltip v-if="!isCompactView" :text="t('copy')">
            <button
              @click="copyRow(`1 ${mainCurrency?.code} = ${rate.rateFormatted} ${rate.code}`, `rate-${rate.id}`)"
              class="p-1 rounded-lg transition-all shrink-0"
              :class="copiedId === `rate-${rate.id}`
                ? 'text-green-500'
                : 'text-text-muted hover:text-primary'"
            >
              <component :is="copiedId === `rate-${rate.id}` ? Check : Copy" :size="13" />
            </button>
          </Tooltip>
        </div>
        <p v-if="allRates.length === 0" class="text-center text-sm text-text-muted py-6">
          {{ t('no_results') }}
        </p>
      </div>
    </div>

    <ExpenseForm
      v-if="expenseFormLookupData"
      :show="showExpenseForm"
      :prefill="expensePrefill"
      :lookupData="expenseFormLookupData"
      @close="showExpenseForm = false"
      @saved="showExpenseForm = false"
    />

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </div>
</template>

<style scoped>
input[type="number"]::-webkit-inner-spin-button,
input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
input[type="number"] {
  -moz-appearance: textfield;
}
</style>
