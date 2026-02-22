<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { dbGetAvgExpenseStats, type AvgExpenseStats } from "@/services/database";
import { Calculator } from "lucide-vue-next";

const { t } = useI18n();
const { fmt } = useCurrencyFormat();

const stats = ref<AvgExpenseStats | null>(null);

onMounted(async () => {
  const d = new Date();
  const prefix = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
  stats.value = await dbGetAvgExpenseStats(prefix);
});
</script>

<template>
  <div v-if="stats && stats.count > 0" class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
    <div class="flex items-center gap-2 mb-3">
      <Calculator :size="16" class="text-[var(--color-primary)]" />
      <h2 class="text-sm sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ t('widget_avg_check') }}</h2>
    </div>
    <div class="grid grid-cols-3 gap-3 text-center">
      <div>
        <p class="text-[10px] sm:text-xs text-[var(--color-text-muted)]">{{ t('avg_amount') }}</p>
        <p class="text-lg sm:text-xl font-bold text-[var(--color-primary)] tabular-nums">{{ fmt(stats.avgAmount) }}</p>
      </div>
      <div>
        <p class="text-[10px] sm:text-xs text-[var(--color-text-muted)]">{{ t('total') }}</p>
        <p class="text-lg sm:text-xl font-bold text-[var(--color-text-primary)] tabular-nums">{{ fmt(stats.total) }}</p>
      </div>
      <div>
        <p class="text-[10px] sm:text-xs text-[var(--color-text-muted)]">{{ t('count') }}</p>
        <p class="text-lg sm:text-xl font-bold text-[var(--color-text-primary)] tabular-nums">{{ stats.count }}</p>
      </div>
    </div>
  </div>
</template>
