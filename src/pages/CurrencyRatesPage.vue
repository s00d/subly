<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores/settings";
import { useCatalogStore } from "@/stores/catalog";
import { useToast } from "@/composables/useToast";
import { useHeaderActions } from "@/composables/useHeaderActions";
import { updateCurrencyRates } from "@/services/rates";
import type { RatesProviderType } from "@/services/rates";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import Toast from "@/components/ui/Toast.vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import AppSlider from "@/components/ui/AppSlider.vue";
import ExpenseForm from "@/components/expenses/ExpenseForm.vue";
import {
  RefreshCw, ClipboardCopy, ArrowRightLeft, TrendingUp,
  Search, Star, Check, Plus, RotateCcw, ChevronUp, ChevronDown, ArrowUpDown,
} from "lucide-vue-next";
import { useConverterStore } from "@/stores/converter";
import { currencyFlag } from "@/services/currencyFlags";

const { t } = useI18n();
const settingsStore = useSettingsStore();
const catalogStore = useCatalogStore();
const converterStore = useConverterStore();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();
const { clearActions } = useHeaderActions();

onMounted(() => {
  clearActions();
});

const searchQuery = ref("");
const isUpdating = ref(false);
const copiedId = ref<string | null>(null);

type RateSortKey = "code" | "name" | "rate" | "enabled";
const rateSortBy = ref<RateSortKey>("enabled");
const rateSortOptions: { key: RateSortKey; labelKey: string }[] = [
  { key: "enabled", labelKey: "sort_enabled_first" },
  { key: "code", labelKey: "sort_by_code" },
  { key: "name", labelKey: "sort_by_name" },
  { key: "rate", labelKey: "sort_by_rate" },
];

const showExpenseForm = ref(false);
const expensePrefill = ref<{ amount: number; currencyId: string; name: string } | null>(null);

const baseAmount = computed({
  get: () => converterStore.baseAmount,
  set: (v: number) => { converterStore.baseAmount = v; },
});
const editingId = ref<string | null>(null);

const mainCurrency = computed(() => catalogStore.mainCurrency);
const otherCurrencies = computed(() =>
  catalogStore.currencies.filter((c) => c.id !== settingsStore.settings.mainCurrencyId),
);
const selectedTargetIds = computed(() => settingsStore.settings.currencyUpdateTargets);

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
  return [base.id, ...selectedTargetIds.value.filter((id) => {
    const c = catalogStore.currencies.find((cur) => cur.id === id);
    return c && c.rate > 0;
  })];
});

watch(allConverterIds, (ids) => {
  const current = converterStore.order;
  if (current.length === 0 || ids.length !== current.length || !ids.every((id) => current.includes(id))) {
    converterStore.setOrder(ids);
  }
}, { immediate: true });

const converterCurrencies = computed<ConverterCurrency[]>(() => {
  const base = mainCurrency.value;
  if (!base) return [];
  const ordered = converterStore.order.length ? converterStore.order : allConverterIds.value;
  return ordered
    .map((id) => {
      const c = catalogStore.currencies.find((cur) => cur.id === id);
      if (!c) return null;
      return {
        id: c.id, code: c.code, name: c.name, symbol: c.symbol,
        rate: c.rate, isBase: c.id === base.id,
      };
    })
    .filter((x): x is ConverterCurrency => x !== null);
});

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

const lastUpdate = computed(() => settingsStore.settings.lastCurrencyUpdate || t("never"));

function toggleTarget(id: string) {
  const current = [...selectedTargetIds.value];
  const idx = current.indexOf(id);
  if (idx >= 0) current.splice(idx, 1);
  else current.push(id);
  settingsStore.updateSettings({ currencyUpdateTargets: current });
}

function selectAllTargets() {
  const ids = otherCurrencies.value.map((c) => c.id);
  settingsStore.updateSettings({ currencyUpdateTargets: ids });
}

function deselectAllTargets() {
  settingsStore.updateSettings({ currencyUpdateTargets: [] });
}

async function copyRow(text: string, id: string) {
  try {
    await writeText(text);
  } catch {
    try { await navigator.clipboard.writeText(text); } catch { toast(t("error"), "error"); return; }
  }
  copiedId.value = id;
  setTimeout(() => { if (copiedId.value === id) copiedId.value = null; }, 1200);
  toast(t("copied_to_clipboard"));
}

async function handleUpdate() {
  const providerType = settingsStore.ratesProvider as RatesProviderType;
  const apiKey = settingsStore.ratesApiKey;
  if (!apiKey && providerType !== "frankfurter") {
    toast(t("rates_key_required"), "error");
    return;
  }
  isUpdating.value = true;
  try {
    const result = await updateCurrencyRates(
      providerType, apiKey, catalogStore.currencies,
      settingsStore.settings.mainCurrencyId, [],
      {
        historyEnabled: settingsStore.settings.rateHistoryEnabled,
        historyDays: settingsStore.settings.rateHistoryDays,
      },
    );
    if (result.error) toast(result.error, "error");
    else if (result.updated > 0) {
      settingsStore.updateSettings({ lastCurrencyUpdate: new Date().toISOString().split("T")[0] });
      toast(t("rates_updated").replace("{count}", String(result.updated)));
    } else toast(t("no_rates_updated"));
  } catch (e) { toast(String(e), "error"); }
  finally {
    isUpdating.value = false;
  }
}

function fmtNum(n: number): string {
  return n.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 4 });
}

async function copyAllConversion() {
  const lines = converterCurrencies.value.map((cur) => {
    const flag = currencyFlag(cur.code);
    const val = fmtNum(getRawValue(cur));
    return `${flag ? flag + " " : ""}${cur.code}: ${val} ${cur.symbol}`;
  });
  const text = lines.join("\n");
  try {
    await writeText(text);
  } catch {
    try { await navigator.clipboard.writeText(text); } catch { toast(t("error"), "error"); return; }
  }
  toast(t("copied_to_clipboard"));
}
</script>

<template>
  <div class="space-y-4">

    <!-- Header -->
    <div class="flex items-center justify-between gap-3">
      <div class="min-w-0">
        <h1 class="text-xl font-bold text-text-primary">{{ t('exchange_rates') }}</h1>
        <p class="text-xs text-text-muted mt-0.5">{{ t('last_update') }}: {{ lastUpdate }}</p>
      </div>
      <button
        @click="handleUpdate"
        :disabled="isUpdating"
        class="flex items-center gap-1.5 px-3.5 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover disabled:opacity-50 transition-colors shrink-0"
      >
        <RefreshCw :size="14" :class="{ 'animate-spin': isUpdating }" />
        <span class="hidden sm:inline">{{ t('update_now') }}</span>
      </button>
    </div>

    <!-- Converter empty state -->
    <div v-if="converterCurrencies.length <= 1" class="bg-surface rounded-xl border border-dashed border-border p-5 text-center">
      <ArrowRightLeft :size="28" class="mx-auto mb-2 text-text-muted opacity-40" />
      <p class="text-sm font-semibold text-text-secondary mb-1">{{ t('converter_empty') }}</p>
      <p class="text-xs text-text-muted">{{ t('converter_empty_hint') }}</p>
    </div>

    <!-- Converter -->
    <div v-else class="bg-surface rounded-xl border border-border p-4">
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center gap-2 text-sm font-semibold text-text-primary">
          <ArrowRightLeft :size="15" class="text-primary" />
          {{ t('currency_converter') }}
        </div>
        <div class="flex items-center gap-1">
          <Tooltip :text="t('copy')">
            <button
              @click="copyAllConversion"
              class="p-1.5 rounded-lg text-text-muted hover:text-primary hover:bg-primary-light transition-colors"
            >
              <ClipboardCopy :size="14" />
            </button>
          </Tooltip>
          <Tooltip :text="t('reset')">
            <button
              @click="converterStore.reset()"
              class="p-1.5 rounded-lg text-text-muted hover:text-primary hover:bg-primary-light transition-colors"
            >
              <RotateCcw :size="14" />
            </button>
          </Tooltip>
        </div>
      </div>

      <!-- Quick presets -->
      <div class="flex flex-wrap gap-1.5 mb-3">
        <button
          v-for="preset in settingsStore.settings.converterPresets"
          :key="preset"
          @click="baseAmount = preset"
          :class="[
            'px-3 py-1 text-xs font-semibold rounded-lg border transition-colors',
            baseAmount === preset
              ? 'bg-primary text-white border-primary'
              : 'bg-surface-secondary text-text-secondary border-border hover:border-primary hover:text-primary',
          ]"
        >
          {{ fmtNum(preset) }}
        </button>
      </div>

      <div class="space-y-2">
        <div
          v-for="(cur, idx) in converterCurrencies"
          :key="cur.id"
          class="flex items-center gap-1.5 sm:gap-2"
        >
          <!-- Reorder buttons -->
          <div class="flex flex-row sm:flex-col shrink-0">
            <Tooltip :text="t('move_up')">
              <button
                @click="converterStore.moveUp(cur.id)"
                :disabled="idx === 0"
                class="p-0.5 rounded text-text-muted hover:text-primary disabled:opacity-20 disabled:cursor-not-allowed transition-colors"
              ><ChevronUp :size="14" /></button>
            </Tooltip>
            <Tooltip :text="t('move_down')">
              <button
                @click="converterStore.moveDown(cur.id)"
                :disabled="idx === converterCurrencies.length - 1"
                class="p-0.5 rounded text-text-muted hover:text-primary disabled:opacity-20 disabled:cursor-not-allowed transition-colors"
              ><ChevronDown :size="14" /></button>
            </Tooltip>
          </div>

          <!-- Flag + Code + name label -->
          <div class="shrink-0 w-16 sm:w-20">
            <span class="text-sm font-bold text-text-primary block leading-tight">
              <span v-if="currencyFlag(cur.code)" class="mr-0.5">{{ currencyFlag(cur.code) }}</span>{{ cur.code }}
            </span>
            <span class="text-[10px] text-text-muted leading-tight truncate block">{{ cur.name }}</span>
          </div>

          <!-- Input + symbol + slider -->
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

          <!-- Actions row -->
          <div class="flex flex-row shrink-0 gap-0.5">
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
                  : 'text-text-muted hover:text-primary hover:bg-primary-light'"
              >
                <component :is="copiedId === `conv-${cur.id}` ? Check : ClipboardCopy" :size="14" />
              </button>
            </Tooltip>
          </div>
        </div>
      </div>
    </div>

    <!-- All rates -->
    <div class="bg-surface rounded-xl border border-border p-4">
      <div class="flex items-center justify-between mb-3">
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
            class="px-2 py-0.5 text-[11px] rounded border border-border text-text-secondary hover:border-primary hover:text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          >{{ t('select_all') }}</button>
          <button
            @click="deselectAllTargets"
            :disabled="selectedTargetIds.length === 0"
            class="px-2 py-0.5 text-[11px] rounded border border-border text-text-secondary hover:border-primary hover:text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          >{{ t('deselect_all') }}</button>
          <span class="text-xs text-text-muted tabular-nums ml-1">
            1 {{ mainCurrency?.code }} =
          </span>
        </div>
      </div>

      <!-- Search + Sort -->
      <div class="flex items-center gap-2 mb-3">
        <div class="relative flex-1">
          <Search :size="14" class="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('search') + '...'"
            class="w-full pl-9 pr-3 py-2 rounded-lg border border-border bg-surface text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-primary placeholder:text-text-muted"
          />
        </div>
        <div class="flex items-center gap-0.5 shrink-0">
          <ArrowUpDown :size="13" class="text-text-muted mr-0.5" />
          <button
            v-for="opt in rateSortOptions"
            :key="opt.key"
            @click="rateSortBy = opt.key"
            :class="[
              'px-2 py-1 text-[11px] rounded border transition-colors',
              rateSortBy === opt.key
                ? 'bg-primary text-white border-primary'
                : 'bg-surface-secondary text-text-secondary border-border hover:border-primary hover:text-primary',
            ]"
          >{{ t(opt.labelKey) }}</button>
        </div>
      </div>

      <!-- List -->
      <div class="max-h-[32rem] overflow-y-auto -mx-1 px-1">
        <div
          v-for="rate in allRates"
          :key="rate.id"
          class="flex items-center gap-2 px-2 py-2 rounded-lg hover:bg-surface-hover transition-colors group border-b border-border/50 last:border-b-0"
        >
          <!-- Star toggle -->
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

          <!-- Flag + Code -->
          <span class="text-sm font-bold text-text-primary w-16 shrink-0">
            <span v-if="currencyFlag(rate.code)" class="mr-0.5">{{ currencyFlag(rate.code) }}</span>{{ rate.code }}
          </span>

          <!-- Name -->
          <span class="text-sm text-text-muted flex-1 min-w-0 truncate">{{ rate.name }}</span>

          <!-- Rate -->
          <span class="text-sm font-mono font-semibold text-text-primary tabular-nums">{{ rate.rateFormatted }}</span>

          <!-- Inverse -->
          <span class="text-xs font-mono text-text-muted tabular-nums w-18 text-right hidden sm:block">1/{{ rate.inverse }}</span>

          <!-- Copy -->
          <Tooltip :text="t('copy')">
            <button
              @click="copyRow(`1 ${mainCurrency?.code} = ${rate.rateFormatted} ${rate.code}`, `rate-${rate.id}`)"
              class="p-1 rounded-lg transition-all shrink-0"
              :class="copiedId === `rate-${rate.id}`
                ? 'text-green-500'
                : 'text-text-muted hover:text-primary'"
            >
              <component :is="copiedId === `rate-${rate.id}` ? Check : ClipboardCopy" :size="13" />
            </button>
          </Tooltip>
        </div>

        <p v-if="allRates.length === 0" class="text-center text-sm text-text-muted py-6">
          {{ t('no_results') }}
        </p>
      </div>
    </div>

    <ExpenseForm
      :show="showExpenseForm"
      :prefill="expensePrefill"
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
