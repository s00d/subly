<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useExpensesStore } from "@/stores/expenses";
import { dbGetAvgExpenseStats, type AvgExpenseStats } from "@/services/database";
import { Calculator } from "lucide-vue-next";

const { t } = useI18n();
const { fmt } = useCurrencyFormat();
const expsStore = useExpensesStore();

const stats = ref<AvgExpenseStats | null>(null);

async function loadAvgStats() {
  const d = new Date();
  const prefix = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
  stats.value = await dbGetAvgExpenseStats(prefix);
}

onMounted(loadAvgStats);

watch(
  [() => expsStore.totalCount, () => expsStore.currentPage, () => expsStore.filter],
  () => {
    loadAvgStats();
  },
  { deep: true },
);
</script>

<template>
  <div v-if="stats && stats.count > 0" class="bg-surface rounded-xl border border-border p-3 sm:p-5">
    <div class="flex items-center gap-2 mb-3">
      <Calculator :size="16" class="text-primary" />
      <h2 class="text-sm sm:text-lg font-semibold text-text-primary">{{ t('widget_avg_check') }}</h2>
    </div>
    <div class="grid grid-cols-3 gap-3 text-center">
      <div>
        <p class="text-[10px] sm:text-xs text-text-muted">{{ t('avg_amount') }}</p>
        <p class="text-lg sm:text-xl font-bold text-primary tabular-nums">{{ fmt(stats.avgAmount) }}</p>
      </div>
      <div>
        <p class="text-[10px] sm:text-xs text-text-muted">{{ t('total') }}</p>
        <p class="text-lg sm:text-xl font-bold text-text-primary tabular-nums">{{ fmt(stats.total) }}</p>
      </div>
      <div>
        <p class="text-[10px] sm:text-xs text-text-muted">{{ t('count') }}</p>
        <p class="text-lg sm:text-xl font-bold text-text-primary tabular-nums">{{ stats.count }}</p>
      </div>
    </div>
  </div>
</template>
