<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { dbGetExpenseMonthComparison, type MonthComparisonData } from "@/services/database";
import { ArrowUpRight, ArrowDownRight, Equal } from "lucide-vue-next";

const { t } = useI18n();
const { fmt } = useCurrencyFormat();

const data = ref<MonthComparisonData | null>(null);

onMounted(async () => {
  const now = new Date();
  const curPrefix = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
  const prev = new Date(now.getFullYear(), now.getMonth() - 1, 1);
  const prevPrefix = `${prev.getFullYear()}-${String(prev.getMonth() + 1).padStart(2, "0")}`;
  data.value = await dbGetExpenseMonthComparison(curPrefix, prevPrefix);
});

const diff = computed(() => {
  if (!data.value) return 0;
  return data.value.currentTotal - data.value.previousTotal;
});

const pctChange = computed(() => {
  if (!data.value || data.value.previousTotal === 0) return null;
  return ((diff.value / data.value.previousTotal) * 100);
});

const barMax = computed(() => {
  if (!data.value) return 1;
  return Math.max(data.value.currentTotal, data.value.previousTotal, 1);
});

function monthName(offset: number): string {
  const d = new Date();
  d.setMonth(d.getMonth() + offset);
  return d.toLocaleString(undefined, { month: "long" });
}
</script>

<template>
  <div v-if="data && (data.currentTotal > 0 || data.previousTotal > 0)" class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
    <div class="flex items-center gap-2 mb-3">
      <component
        :is="diff > 0 ? ArrowUpRight : diff < 0 ? ArrowDownRight : Equal"
        :size="16"
        :class="diff > 0 ? 'text-red-500' : diff < 0 ? 'text-green-500' : 'text-[var(--color-text-muted)]'"
      />
      <h2 class="text-sm sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ t('widget_month_compare') }}</h2>
    </div>

    <div class="space-y-3">
      <!-- Current month -->
      <div>
        <div class="flex items-center justify-between mb-1">
          <span class="text-xs text-[var(--color-text-primary)] font-medium capitalize">{{ monthName(0) }}</span>
          <span class="text-sm font-bold text-[var(--color-text-primary)] tabular-nums">{{ fmt(data.currentTotal) }}</span>
        </div>
        <div class="w-full h-3 bg-[var(--color-surface-hover)] rounded-full overflow-hidden">
          <div
            class="h-full rounded-full bg-[var(--color-primary)] transition-all duration-500"
            :style="{ width: (data.currentTotal / barMax * 100) + '%' }"
          />
        </div>
        <span class="text-[10px] text-[var(--color-text-muted)]">{{ data.currentCount }} {{ t('records') }}</span>
      </div>

      <!-- Previous month -->
      <div>
        <div class="flex items-center justify-between mb-1">
          <span class="text-xs text-[var(--color-text-muted)] font-medium capitalize">{{ monthName(-1) }}</span>
          <span class="text-sm font-bold text-[var(--color-text-muted)] tabular-nums">{{ fmt(data.previousTotal) }}</span>
        </div>
        <div class="w-full h-3 bg-[var(--color-surface-hover)] rounded-full overflow-hidden">
          <div
            class="h-full rounded-full bg-gray-400 transition-all duration-500"
            :style="{ width: (data.previousTotal / barMax * 100) + '%' }"
          />
        </div>
        <span class="text-[10px] text-[var(--color-text-muted)]">{{ data.previousCount }} {{ t('records') }}</span>
      </div>

      <!-- Diff -->
      <div class="flex items-center justify-center gap-2 pt-2 border-t border-[var(--color-border)]">
        <span
          class="text-sm font-bold tabular-nums"
          :class="diff > 0 ? 'text-red-500' : diff < 0 ? 'text-green-500' : 'text-[var(--color-text-muted)]'"
        >
          {{ diff > 0 ? '+' : '' }}{{ fmt(diff) }}
        </span>
        <span
          v-if="pctChange !== null"
          class="text-xs font-semibold tabular-nums px-1.5 py-0.5 rounded"
          :class="pctChange > 0 ? 'bg-red-100 text-red-600 dark:bg-red-900/20' : 'bg-green-100 text-green-600 dark:bg-green-900/20'"
        >
          {{ pctChange > 0 ? '+' : '' }}{{ pctChange.toFixed(1) }}%
        </span>
      </div>
    </div>
  </div>
</template>
