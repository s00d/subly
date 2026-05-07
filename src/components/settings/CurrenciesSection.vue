<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { upsertCurrency, deleteCurrency as deleteCurrencyApi, maxSortOrder } from "@/services/catalogClient";
import type { Currency, Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import AppInput from "@/components/ui/AppInput.vue";
import { Trash2, Plus, ChevronUp, ChevronDown, Star, Search } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { ui } from "@/lib/tv";

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
  Object.assign(cur, next);
}
async function setMainCurrency(id: string) {
  if (!settings.value) return;
  const validIds = new Set(currencies.value.map((c) => c.id));
  const nextTargets = (settings.value.currencyUpdateTargets ?? []).filter(
    (targetId) => targetId !== id && validIds.has(targetId),
  );
  const next = { ...settings.value, mainCurrencyId: id, currencyUpdateTargets: nextTargets };
  settings.value = next;
  await metaStore.updateSettings(next);
}
async function reorderCurrencies(ids: string[]) {
  for (let i = 0; i < ids.length; i += 1) {
    const cur = currencies.value.find((c) => c.id === ids[i]);
    if (!cur) continue;
    const next = { ...cur, sortOrder: i };
    await upsertCurrency(next);
    Object.assign(cur, next);
  }
}
async function addCur() {
  const order = await maxSortOrder("currencies");
  const cur: Currency = { id: crypto.randomUUID(), name: "Currency", symbol: "$", code: "CODE", rate: 1, sortOrder: order + 1, i18nKey: "" };
  await upsertCurrency(cur);
  currencies.value.push(cur);
}
async function saveCur(id: string, name: string, symbol: string, code: string) {
  const normalizedCode = code.trim().toUpperCase();
  await updateCurrency(id, { name, symbol, code: normalizedCode });
  toast(t("success"));
}
async function removeCur(id: string) {
  if (id === settings.value?.mainCurrencyId || isUsedCurrency(id)) {
    toast(t("currency_cannot_delete"), "error");
    return;
  }
  await deleteCurrencyApi(id);
  currencies.value = currencies.value.filter((c) => c.id !== id);
  toast(t("success"));
}
function moveCurUp(id: string) {
  const ids = sortedCurrencies.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx <= 0) return;
  [ids[idx - 1], ids[idx]] = [ids[idx], ids[idx - 1]];
  void reorderCurrencies(ids);
}
function moveCurDown(id: string) {
  const ids = sortedCurrencies.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx < 0 || idx >= ids.length - 1) return;
  [ids[idx], ids[idx + 1]] = [ids[idx + 1], ids[idx]];
  void reorderCurrencies(ids);
}
</script>

<template>
  <section class="bg-surface rounded-xl border border-border p-3 sm:p-5">
    <div class="flex items-center justify-between gap-2 mb-3">
      <h2 :class="[ui.sectionTitle(), 'shrink-0']">{{ t('currencies') }}</h2>
      <div class="relative w-32 sm:w-44">
        <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
        <input v-model="curSearch" type="text" :placeholder="t('search')" class="w-full pl-8 pr-3 py-1.5 rounded-lg border border-border bg-surface text-xs text-text-primary placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-primary transition-shadow" />
      </div>
    </div>
    <div class="space-y-1.5 max-h-72 overflow-y-auto overflow-x-hidden">
      <div v-for="(c, idx) in filteredCurrencies" :key="c.id" class="flex gap-2 items-center rounded-lg px-2 py-1" :class="c.id === settings?.mainCurrencyId ? 'bg-primary-light/50' : ''">
        <div v-if="!isCurSearching" class="flex flex-row sm:flex-col shrink-0">
          <Tooltip :text="t('move_up')"><button @click="moveCurUp(c.id)" :disabled="idx === 0" class="p-0.5 rounded text-text-muted hover:text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronUp :size="14" /></button></Tooltip>
          <Tooltip :text="t('move_down')"><button @click="moveCurDown(c.id)" :disabled="idx === sortedCurrencies.length - 1" class="p-0.5 rounded text-text-muted hover:text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronDown :size="14" /></button></Tooltip>
        </div>
        <Tooltip :text="t('set_as_primary')">
          <button @click="setMainCurrency(c.id)" class="p-1 rounded-lg transition-colors shrink-0" :class="c.id === settings?.mainCurrencyId ? 'text-yellow-500' : 'text-text-muted hover:text-yellow-500'">
            <Star :size="14" :fill="c.id === settings?.mainCurrencyId ? 'currentColor' : 'none'" />
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
          <div class="w-10 shrink-0"><AppInput :modelValue="c.symbol" @update:modelValue="(v: string | number) => saveCur(c.id, c.name, String(v), c.code)" size="sm" /></div>
          <div class="flex-1 min-w-0"><AppInput :modelValue="c.name" @update:modelValue="(v: string | number) => saveCur(c.id, String(v), c.symbol, c.code)" size="sm" /></div>
          <div class="w-14 shrink-0"><AppInput :modelValue="c.code" @update:modelValue="(v: string | number) => saveCur(c.id, c.name, c.symbol, String(v))" size="sm" :disabled="c.id === settings?.mainCurrencyId || isUsedCurrency(c.id)" /></div>
        </template>
        <Tooltip :text="t('delete')">
          <button @click="removeCur(c.id)" :disabled="c.id === settings?.mainCurrencyId || isUsedCurrency(c.id)" class="p-1.5 rounded-lg transition-colors shrink-0" :class="(c.id === settings?.mainCurrencyId || isUsedCurrency(c.id)) ? 'text-text-muted cursor-not-allowed' : 'text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20'"><Trash2 :size="14" /></button>
        </Tooltip>
      </div>
    </div>
    <button @click="addCur" class="mt-3 px-3 py-1.5 rounded-lg bg-primary text-white text-sm hover:bg-primary-hover transition-colors">
      <Plus :size="14" class="inline mr-1" /> {{ t('add') }}
    </button>
  </section>
</template>
