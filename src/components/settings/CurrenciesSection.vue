<script setup lang="ts">
import { ref, computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import { Trash2, Plus, ChevronUp, ChevronDown, Star, Search } from "lucide-vue-next";

const store = useAppStore();
const { t } = useI18n();
const { toast } = useToast();

const curSearch = ref("");
const isCurSearching = computed(() => curSearch.value.length > 0);

/** Sorted: primary first, then by order */
const sortedCurrencies = computed(() => {
  const mainId = store.state.settings.mainCurrencyId;
  return [...store.state.currencies].sort((a, b) => {
    if (a.id === mainId && b.id !== mainId) return -1;
    if (b.id === mainId && a.id !== mainId) return 1;
    return a.order - b.order;
  });
});

const filteredCurrencies = computed(() => {
  if (!curSearch.value) return sortedCurrencies.value;
  const q = curSearch.value.toLowerCase();
  return sortedCurrencies.value.filter((c) =>
    c.name.toLowerCase().includes(q) || c.code.toLowerCase().includes(q) || c.symbol.includes(q)
  );
});

const isUsedCurrency = (id: string) => store.state.subscriptions.some((s) => s.currencyId === id);
const isDefault = (c: { i18nKey?: string }) => !!c.i18nKey;

function addCur() { store.addCurrency("Currency", "$", "CODE"); }
function saveCur(id: string, name: string, symbol: string, code: string) {
  store.updateCurrency(id, { name, symbol, code });
  toast(t("success"));
}
function removeCur(id: string) {
  if (!store.deleteCurrency(id)) toast(t("error"), "error");
  else toast(t("success"));
}
function setMainCurrency(id: string) { store.updateSettings({ mainCurrencyId: id }); }

function moveCurUp(id: string) {
  const ids = sortedCurrencies.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx <= 0) return;
  [ids[idx - 1], ids[idx]] = [ids[idx], ids[idx - 1]];
  store.reorderCurrencies(ids);
}

function moveCurDown(id: string) {
  const ids = sortedCurrencies.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx < 0 || idx >= ids.length - 1) return;
  [ids[idx], ids[idx + 1]] = [ids[idx + 1], ids[idx]];
  store.reorderCurrencies(ids);
}
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
    <div class="flex items-center justify-between mb-3">
      <h2 class="text-lg font-semibold text-[var(--color-text-primary)]">{{ t('currencies') }}</h2>
      <div class="relative w-44">
        <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
        <input v-model="curSearch" type="text" :placeholder="t('search')" class="w-full pl-8 pr-3 py-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] text-xs text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] transition-shadow" />
      </div>
    </div>
    <div class="space-y-1.5 max-h-72 overflow-y-auto overflow-x-hidden">
      <div v-for="(c, idx) in filteredCurrencies" :key="c.id" class="flex gap-2 items-center rounded-lg px-2 py-1" :class="c.id === store.state.settings.mainCurrencyId ? 'bg-[var(--color-primary-light)]/50' : ''">
        <div v-if="!isCurSearching" class="flex flex-col shrink-0">
          <button @click="moveCurUp(c.id)" :disabled="idx === 0" class="p-0.5 rounded text-[var(--color-text-muted)] hover:text-[var(--color-primary)] disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronUp :size="14" /></button>
          <button @click="moveCurDown(c.id)" :disabled="idx === sortedCurrencies.length - 1" class="p-0.5 rounded text-[var(--color-text-muted)] hover:text-[var(--color-primary)] disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronDown :size="14" /></button>
        </div>
        <button @click="setMainCurrency(c.id)" class="p-1 rounded-lg transition-colors shrink-0" :class="c.id === store.state.settings.mainCurrencyId ? 'text-yellow-500' : 'text-[var(--color-text-muted)] hover:text-yellow-500'" :title="t('set_as_primary')">
          <Star :size="14" :fill="c.id === store.state.settings.mainCurrencyId ? 'currentColor' : 'none'" />
        </button>
        <!-- Default item: read-only -->
        <template v-if="isDefault(c)">
          <div class="w-10 shrink-0 text-sm text-[var(--color-text-primary)] text-center">{{ c.symbol }}</div>
          <div class="flex-1 min-w-0 text-sm text-[var(--color-text-primary)] truncate">{{ c.name }}</div>
          <div class="w-14 shrink-0 text-sm text-[var(--color-text-muted)] text-center">{{ c.code }}</div>
        </template>
        <!-- User item: editable -->
        <template v-else>
          <div class="w-10 shrink-0"><AppInput :modelValue="c.symbol" @update:modelValue="(v: any) => saveCur(c.id, c.name, String(v), c.code)" size="sm" /></div>
          <div class="flex-1 min-w-0"><AppInput :modelValue="c.name" @update:modelValue="(v: any) => saveCur(c.id, String(v), c.symbol, c.code)" size="sm" /></div>
          <div class="w-14 shrink-0"><AppInput :modelValue="c.code" @update:modelValue="(v: any) => saveCur(c.id, c.name, c.symbol, String(v))" size="sm" :disabled="c.id === store.state.settings.mainCurrencyId || isUsedCurrency(c.id)" /></div>
        </template>
        <button @click="removeCur(c.id)" :disabled="c.id === store.state.settings.mainCurrencyId || isUsedCurrency(c.id)" class="p-1.5 rounded-lg transition-colors shrink-0" :class="(c.id === store.state.settings.mainCurrencyId || isUsedCurrency(c.id)) ? 'text-[var(--color-text-muted)] cursor-not-allowed' : 'text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20'"><Trash2 :size="14" /></button>
      </div>
    </div>
    <button @click="addCur" class="mt-3 px-3 py-1.5 rounded-lg bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)] transition-colors">
      <Plus :size="14" class="inline mr-1" /> {{ t('add') }}
    </button>
  </section>
</template>
