<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import type { MonthComparisonStyle } from "@/services/dashboardClient";
import { ArrowUpRight, ArrowDownRight, Equal } from "@lucide/vue";

const { t } = useI18n();
const { fmt } = useCurrencyFormat();
const props = defineProps<{
  data?: {
    currentTotal: number;
    currentCount: number;
    previousTotal: number;
    previousCount: number;
    usedFullMonthFallback?: boolean;
    comparisonStyle?: MonthComparisonStyle;
    currentMonthLabel?: string;
    previousMonthLabel?: string;
  } | null;
}>();

const data = computed(() => props.data ?? null);

/** Human-readable month from `YYYY-MM` (backend labels). */
function captionFromYm(ym: string | undefined): string {
  if (!ym || ym.length < 7) return "";
  const y = Number(ym.slice(0, 4));
  const m = Number(ym.slice(5, 7));
  if (!Number.isFinite(y) || !Number.isFinite(m)) return ym;
  return new Date(y, m - 1, 1).toLocaleString(undefined, { month: "long", year: "numeric" });
}

const comparisonHintKey = computed(() => {
  const s = data.value?.comparisonStyle;
  if (s === "completedPair") return "month_comparison_completed_pair_hint";
  if (s === "fullMonths") return "month_comparison_full_month_hint";
  return "month_comparison_mtd_hint";
});

const newerMonthCaption = computed(() => {
  const d = data.value;
  const fromApi = captionFromYm(d?.currentMonthLabel);
  if (fromApi) return fromApi;
  const now = new Date();
  return now.toLocaleString(undefined, { month: "long", year: "numeric" });
});

const olderMonthCaption = computed(() => {
  const d = data.value;
  const fromApi = captionFromYm(d?.previousMonthLabel);
  if (fromApi) return fromApi;
  const x = new Date();
  x.setMonth(x.getMonth() - 1);
  return x.toLocaleString(undefined, { month: "long", year: "numeric" });
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

</script>

<template>
  <div v-if="data && (data.currentTotal > 0 || data.previousTotal > 0)" class="bg-surface rounded-xl border border-border p-2.5 sm:p-4">
    <div class="flex items-center gap-2 mb-2.5">
      <component
        :is="diff > 0 ? ArrowUpRight : diff < 0 ? ArrowDownRight : Equal"
        :size="16"
        :class="diff > 0 ? 'text-red-500' : diff < 0 ? 'text-green-500' : 'text-text-muted'"
      />
      <h2 class="text-sm sm:text-lg font-semibold text-text-primary">{{ t('widget_month_compare') }}</h2>
    </div>
    <p class="text-[10px] text-text-muted mb-2.5 leading-snug">
      {{ t(comparisonHintKey) }}
    </p>

    <div class="space-y-2.5">
      <!-- Newer / current side of comparison -->
      <div>
        <div class="flex items-center justify-between mb-1">
          <span class="text-xs text-text-primary font-medium capitalize">{{ newerMonthCaption }}</span>
          <span class="text-sm font-bold text-text-primary tabular-nums">{{ fmt(data.currentTotal) }}</span>
        </div>
        <div class="w-full h-3 bg-surface-hover rounded-full overflow-hidden">
          <div
            class="h-full rounded-full bg-primary transition-all duration-500"
            :style="{ width: (data.currentTotal / barMax * 100) + '%' }"
          />
        </div>
        <span class="text-[10px] text-text-muted">{{ data.currentCount }} {{ t('records') }}</span>
      </div>

      <!-- Older / previous side of comparison -->
      <div>
        <div class="flex items-center justify-between mb-1">
          <span class="text-xs text-text-muted font-medium capitalize">{{ olderMonthCaption }}</span>
          <span class="text-sm font-bold text-text-muted tabular-nums">{{ fmt(data.previousTotal) }}</span>
        </div>
        <div class="w-full h-3 bg-surface-hover rounded-full overflow-hidden">
          <div
            class="h-full rounded-full bg-gray-400 transition-all duration-500"
            :style="{ width: (data.previousTotal / barMax * 100) + '%' }"
          />
        </div>
        <span class="text-[10px] text-text-muted">{{ data.previousCount }} {{ t('records') }}</span>
      </div>

      <!-- Diff -->
      <div class="flex items-center justify-center gap-2 pt-2 border-t border-border">
        <span
          class="text-sm font-bold tabular-nums"
          :class="diff > 0 ? 'text-red-500' : diff < 0 ? 'text-green-500' : 'text-text-muted'"
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
