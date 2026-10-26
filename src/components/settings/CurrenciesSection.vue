<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { upsertCurrency, deleteCurrency as deleteCurrencyApi, maxSortOrder, updateCurrencyRates } from "@/services/catalogClient";
import { div, parseMoney } from "@/lib/money";
import type { Currency, Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { useCatalogsUsageStore } from "@/stores/catalogsUsageStore";
import { formatErrorForToast } from "@/utils/formatError";
import AppInput from "@/components/ui/AppInput.vue";
import UiCard from "@/components/ui/UiCard.vue";
import SavedIndicator, { useAutoSave } from "@/components/settings/SavedIndicator.vue";
import SettingsSearch from "@/components/settings/SettingsSearch.vue";
import ReorderButtons from "@/components/settings/ReorderButtons.vue";
import { Trash2, Plus } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { button } from "@/lib/tv";

const props = defineProps<{
  lookupData: {
    currencies: Currency[];
    settings: Settings;
    currencyUsage: Record<string, number>;
  } | null;
}>();
const { t } = useI18n();
const { toast } = useToast();
const metaStore = useAppMetaStore();
const catalogsUsageStore = useCatalogsUsageStore();
const { savedAt, run, debounce } = useAutoSave();
const iconBtn = button({ intent: "ghost", size: "icon", class: "h-7 w-7" });
const currencies = ref<Currency[]>([]);
const settings = ref<Settings | null>(null);
const currencyUsage = ref<Record<string, number>>({});
watch(
  () => props.lookupData,
  (lookup) => {
    currencies.value = lookup?.currencies ?? [];
    settings.value = lookup?.settings ?? null;
    currencyUsage.value = lookup?.currencyUsage ?? {};
  },
  { immediate: true, deep: true },
);

const curSearch = ref("");
const isCurSearching = computed(() => curSearch.value.length > 0);

/** The main currency is pinned on top by the sort: it can't move and nothing moves above it. */
const pinnedCount = computed(() =>
  currencies.value.some((c) => c.id === settings.value?.mainCurrencyId) ? 1 : 0,
);
const sortedCurrencies = computed(() => {
  const mainId = settings.value?.mainCurrencyId;
  return [...currencies.value].sort((a, b) => {
    if (a.id === mainId && b.id !== mainId) return -1;
    if (b.id === mainId && a.id !== mainId) return 1;
    return a.sortOrder - b.sortOrder;
  });
});

const filteredCurrencies = computed(() => {
  if (!curSearch.value) return sortedCurrencies.value;
  const q = curSearch.value.toLowerCase();
  return sortedCurrencies.value.filter((c) =>
    c.name.toLowerCase().includes(q) || c.code.toLowerCase().includes(q) || c.symbol.includes(q)
  );
});

const isUsedCurrency = (id: string) => (currencyUsage.value[id] ?? 0) > 0;
const isDefault = (c: { i18nKey?: string }) => !!c.i18nKey;

async function updateCurrency(id: string, updates: Partial<Currency>) {
  const cur = currencies.value.find((c) => c.id === id);
  if (!cur) return;
  const next = { ...cur, ...updates };
  await upsertCurrency(next);
  metaStore.refreshSoon();
  Object.assign(cur, next);
}
function setMainCurrency(id: string) {
  if (!settings.value) return;
  const validIds = new Set(currencies.value.map((c) => c.id));
  const nextTargets = (settings.value.currencyUpdateTargets ?? []).filter(
    (targetId) => targetId !== id && validIds.has(targetId),
  );
  const next = { ...settings.value, mainCurrencyId: id, currencyUpdateTargets: nextTargets };
  settings.value = next;
  // Rates are "units per 1 main": rebase so the new main is exactly 1, otherwise every
  // conversion (amount / rate) keeps using the old base.
  const newMain = currencies.value.find((c) => c.id === id);
  const base = newMain ? parseMoney(newMain.rate) : null;
  const rebased = base && base.gt(0)
    ? currencies.value.map((c) => ({ id: c.id, rate: c.id === id ? "1" : div(c.rate, base).toFixed() }))
    : [];
  void run(async () => {
    if (rebased.length > 0) await updateCurrencyRates(rebased);
    await metaStore.updateSettings(next);
    await metaStore.refresh();
  });
}
async function reorderCurrencies(ids: string[]) {
  for (let i = 0; i < ids.length; i += 1) {
    const cur = currencies.value.find((c) => c.id === ids[i]);
    if (!cur) continue;
    const next = { ...cur, sortOrder: i };
    await upsertCurrency(next);
    metaStore.refreshSoon();
    Object.assign(cur, next);
  }
}
function addCur() {
  void run(async () => {
    const order = await maxSortOrder("currencies");
    const cur: Currency = { id: crypto.randomUUID(), name: "Currency", symbol: "$", code: "CODE", rate: "1", sortOrder: order + 1, i18nKey: "" };
    await upsertCurrency(cur);
    metaStore.refreshSoon();
    currencies.value.push(cur);
  });
}
/** Field edits: update the row now, persist (with normalized code) once typing pauses. */
function saveCur(id: string, field: "name" | "symbol" | "code", value: string) {
  const cur = currencies.value.find((c) => c.id === id);
  if (!cur) return;
  cur[field] = value;
  debounce(`cur:${id}`, () =>
    updateCurrency(id, { name: cur.name, symbol: cur.symbol, code: cur.code.trim().toUpperCase() }),
  );
}
async function removeCur(id: string) {
  if (id === settings.value?.mainCurrencyId) {
    toast(t("currency_cannot_delete"), "error");
    return;
  }
  try {
    await deleteCurrencyApi(id);
    metaStore.refreshSoon();
    currencies.value = currencies.value.filter((c) => c.id !== id);
    await catalogsUsageStore.refresh();
    toast(t("success"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}
function moveCurUp(id: string) {
  const ids = sortedCurrencies.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx <= 0) return;
  [ids[idx - 1], ids[idx]] = [ids[idx], ids[idx - 1]];
  void run(() => reorderCurrencies(ids));
}
function moveCurDown(id: string) {
  const ids = sortedCurrencies.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx < 0 || idx >= ids.length - 1) return;
  [ids[idx], ids[idx + 1]] = [ids[idx + 1], ids[idx]];
  void run(() => reorderCurrencies(ids));
}
</script>

<template>
  <UiCard :title="t('currencies')">
    <template #actions>
      <SavedIndicator :at="savedAt" />
      <SettingsSearch v-model="curSearch" class="w-32 sm:w-44" />
    </template>
    <div class="space-y-1 max-h-72 overflow-y-auto overflow-x-hidden">
      <div v-for="(c, idx) in filteredCurrencies" :key="c.id" class="flex gap-1.5 items-center rounded-lg px-1.5 py-1" :class="c.id === settings?.mainCurrencyId ? 'bg-primary-light/50' : ''">
        <ReorderButtons
          v-if="!isCurSearching && idx >= pinnedCount"
          :up-disabled="idx <= pinnedCount"
          :down-disabled="idx === sortedCurrencies.length - 1"
          @up="moveCurUp(c.id)"
          @down="moveCurDown(c.id)"
        />
        <div v-else-if="!isCurSearching" class="w-14 shrink-0" />
        <!-- Main currency (radio) -->
        <Tooltip :text="t('set_as_primary')">
          <button
            type="button"
            role="radio"
            :aria-checked="c.id === settings?.mainCurrencyId"
            :aria-label="t('set_as_primary')"
            :class="iconBtn"
            @click="setMainCurrency(c.id)"
          >
            <span class="flex h-3.5 w-3.5 items-center justify-center rounded-full border-2" :class="c.id === settings?.mainCurrencyId ? 'border-primary' : 'border-border-strong'">
              <span v-if="c.id === settings?.mainCurrencyId" class="h-1.5 w-1.5 rounded-full bg-primary" />
            </span>
          </button>
        </Tooltip>
        <!-- Default item: read-only -->
        <template v-if="isDefault(c)">
          <div class="w-10 shrink-0 text-sm text-text-primary text-center">{{ c.symbol }}</div>
          <div class="flex-1 min-w-0 text-sm text-text-primary truncate">{{ c.name }}</div>
          <div class="w-14 shrink-0 text-sm text-text-muted text-center">{{ c.code }}</div>
        </template>
        <!-- User item: editable -->
        <template v-else>
          <div class="w-10 shrink-0"><AppInput :modelValue="c.symbol" @update:modelValue="(v: string | number) => saveCur(c.id, 'symbol', String(v))" size="sm" /></div>
          <div class="flex-1 min-w-0"><AppInput :modelValue="c.name" @update:modelValue="(v: string | number) => saveCur(c.id, 'name', String(v))" size="sm" /></div>
          <div class="w-14 shrink-0"><AppInput :modelValue="c.code" @update:modelValue="(v: string | number) => saveCur(c.id, 'code', String(v))" size="sm" :disabled="c.id === settings?.mainCurrencyId || isUsedCurrency(c.id)" /></div>
        </template>
        <span v-if="c.id === settings?.mainCurrencyId" class="shrink-0 rounded-full bg-primary/12 px-1.5 py-0.5 text-[10px] font-medium text-primary">{{ t('catalog_default') }}</span>
        <Tooltip :text="t('delete')">
          <button
            type="button"
            :aria-label="t('delete')"
            @click="removeCur(c.id)"
            :disabled="c.id === settings?.mainCurrencyId"
            :class="[iconBtn, 'text-danger hover:bg-danger/12 hover:text-danger disabled:text-text-muted']"
          ><Trash2 :size="14" /></button>
        </Tooltip>
      </div>
    </div>
    <button type="button" @click="addCur" :class="button({ intent: 'secondary', size: 'sm', class: 'mt-3' })">
      <Plus :size="14" /> {{ t('add') }}
    </button>
  </UiCard>
</template>
