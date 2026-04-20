<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useCatalogStore } from "@/stores/catalog";
import { useExpensesStore } from "@/stores/expenses";
import { dbGetTopExpenses, type TopExpense } from "@/services/database";
import { TrendingDown } from "lucide-vue-next";

const { t } = useI18n();
const { fmt } = useCurrencyFormat();
const catalogStore = useCatalogStore();
const expsStore = useExpensesStore();

const items = ref<TopExpense[]>([]);

async function loadTopExpenses() {
  const d = new Date();
  const prefix = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
  items.value = await dbGetTopExpenses(prefix, 5);
}

onMounted(loadTopExpenses);

watch(
  [() => expsStore.totalCount, () => expsStore.currentPage, () => expsStore.filter],
  () => {
    loadTopExpenses();
  },
  { deep: true },
);

function catName(id: string): string {
  return catalogStore.categories.find((c) => c.id === id)?.name ?? "";
}
</script>

<template>
  <div v-if="items.length > 0" class="bg-surface rounded-xl border border-border p-3 sm:p-5">
    <div class="flex items-center gap-2 mb-3">
      <TrendingDown :size="16" class="text-red-500" />
      <h2 class="text-sm sm:text-lg font-semibold text-text-primary">{{ t('widget_top_expenses') }}</h2>
    </div>
    <div class="space-y-2">
      <div
        v-for="(exp, i) in items"
        :key="exp.id"
        class="flex items-center gap-3 rounded-lg px-2 py-1.5"
      >
        <span class="text-xs font-bold text-text-muted w-5 text-center tabular-nums">{{ i + 1 }}</span>
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium text-text-primary truncate">{{ exp.name }}</p>
          <p class="text-[10px] text-text-muted">{{ catName(exp.categoryId) }} · {{ exp.date }}</p>
        </div>
        <span class="text-sm font-bold text-text-primary tabular-nums shrink-0">{{ fmt(exp.amount, exp.currencyId) }}</span>
      </div>
    </div>
  </div>
</template>
