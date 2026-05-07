<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { ForecastResult, MonthComparison } from "@/services/dashboardClient";
import { TrendingUp, TrendingDown, Minus, Calendar, ArrowRight } from "@lucide/vue";
import { ui, typo, iconSize, statValue } from "@/lib/tv";

const props = defineProps<{
  forecast: ForecastResult;
  comparison: MonthComparison;
  fmt: (n: number) => string;
}>();

const { t } = useI18n();

const comparisonHintKey = computed(() => {
  const s = props.comparison.comparisonStyle;
  if (s === "completedPair") return "month_comparison_completed_pair_hint";
  if (s === "fullMonths") return "month_comparison_full_month_hint";
  return "month_comparison_mtd_hint";
});
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 gap-2.5 sm:gap-3">
    <!-- Forecast -->
    <div class="bg-surface rounded-xl border border-border p-2.5 sm:p-4">
      <h3 :class="[ui.sectionTitle(), 'mb-2.5 sm:mb-3']">{{ t('forecast') }}</h3>
      <div class="space-y-2.5 sm:space-y-3">
        <div>
          <div class="flex items-center gap-2 mb-1">
            <Calendar :size="iconSize.xs" class="text-text-muted" />
            <span class="text-[10px] sm:text-xs text-text-muted">{{ forecast.nextMonthLabel }}</span>
          </div>
          <p :class="statValue()">{{ fmt(forecast.nextMonth) }}</p>
        </div>
        <div class="pt-2.5 border-t border-border">
          <div class="flex items-center gap-2 mb-1">
            <Calendar :size="iconSize.xs" class="text-text-muted" />
            <span class="text-[10px] sm:text-xs text-text-muted">{{ t('next_quarter') }}</span>
          </div>
          <p :class="typo.statValueMd()">{{ fmt(forecast.nextQuarter) }}</p>
          <p class="text-[9px] sm:text-[10px] text-text-muted mt-0.5">{{ forecast.quarterLabels.join(' → ') }}</p>
        </div>
      </div>
    </div>

    <!-- Month comparison -->
    <div class="bg-surface rounded-xl border border-border p-2.5 sm:p-4">
      <h3 :class="[ui.sectionTitle(), 'mb-1']">{{ t('month_comparison') }}</h3>
      <p class="text-[10px] text-text-muted mb-2.5 sm:mb-3 leading-snug">
        {{ t(comparisonHintKey) }}
      </p>
      <div class="flex items-center gap-2 sm:gap-2.5 mb-2.5 sm:mb-3">
        <div class="flex-1 text-center">
          <p class="text-[10px] sm:text-xs text-text-muted mb-1">{{ comparison.previousMonth }}</p>
          <p :class="typo.statValueMd()">{{ fmt(comparison.previous) }}</p>
        </div>
        <ArrowRight :size="iconSize.xs" class="text-text-muted shrink-0" />
        <div class="flex-1 text-center">
          <p class="text-[10px] sm:text-xs text-text-muted mb-1">{{ comparison.currentMonth }}</p>
          <p :class="typo.statValueMd()">{{ fmt(comparison.current) }}</p>
        </div>
      </div>
      <div
        class="flex items-center justify-center gap-2 py-2 rounded-lg text-sm font-semibold"
        :class="comparison.diff > 0 ? 'bg-red-50 text-red-600 dark:bg-red-900/20 dark:text-red-400' : comparison.diff < 0 ? 'bg-green-50 text-green-600 dark:bg-green-900/20 dark:text-green-400' : 'bg-gray-50 text-gray-500 dark:bg-gray-800 dark:text-gray-400'"
      >
        <TrendingUp v-if="comparison.diff > 0" :size="iconSize.sm" />
        <TrendingDown v-else-if="comparison.diff < 0" :size="iconSize.sm" />
        <Minus v-else :size="iconSize.sm" />
        <span>{{ comparison.diff > 0 ? '+' : '' }}{{ fmt(comparison.diff) }} ({{ comparison.diff > 0 ? '+' : '' }}{{ comparison.diffPercent.toFixed(1) }}%)</span>
      </div>
    </div>
  </div>
</template>
