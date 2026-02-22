<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { ForecastResult, MonthComparison } from "@/services/analytics";
import { TrendingUp, TrendingDown, Minus, Calendar, ArrowRight } from "lucide-vue-next";

defineProps<{
  forecast: ForecastResult;
  comparison: MonthComparison;
  fmt: (n: number) => string;
}>();

const { t } = useI18n();
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 gap-3 sm:gap-4">
    <!-- Forecast -->
    <div class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
      <h3 class="text-xs sm:text-sm font-semibold text-[var(--color-text-primary)] mb-3 sm:mb-4">{{ t('forecast') }}</h3>
      <div class="space-y-3 sm:space-y-4">
        <div>
          <div class="flex items-center gap-2 mb-1">
            <Calendar :size="14" class="text-[var(--color-text-muted)]" />
            <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)]">{{ forecast.nextMonthLabel }}</span>
          </div>
          <p class="text-xl sm:text-2xl font-bold text-[var(--color-text-primary)]">{{ fmt(forecast.nextMonth) }}</p>
        </div>
        <div class="pt-3 border-t border-[var(--color-border)]">
          <div class="flex items-center gap-2 mb-1">
            <Calendar :size="14" class="text-[var(--color-text-muted)]" />
            <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)]">{{ t('next_quarter') }}</span>
          </div>
          <p class="text-lg sm:text-xl font-bold text-[var(--color-text-primary)]">{{ fmt(forecast.nextQuarter) }}</p>
          <p class="text-[9px] sm:text-[10px] text-[var(--color-text-muted)] mt-0.5">{{ forecast.quarterLabels.join(' → ') }}</p>
        </div>
      </div>
    </div>

    <!-- Month comparison -->
    <div class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
      <h3 class="text-xs sm:text-sm font-semibold text-[var(--color-text-primary)] mb-3 sm:mb-4">{{ t('month_comparison') }}</h3>
      <div class="flex items-center gap-2 sm:gap-3 mb-3 sm:mb-4">
        <div class="flex-1 text-center">
          <p class="text-[10px] sm:text-xs text-[var(--color-text-muted)] mb-1">{{ comparison.previousMonth }}</p>
          <p class="text-base sm:text-lg font-bold text-[var(--color-text-primary)]">{{ fmt(comparison.previous) }}</p>
        </div>
        <ArrowRight :size="14" class="text-[var(--color-text-muted)] shrink-0" />
        <div class="flex-1 text-center">
          <p class="text-[10px] sm:text-xs text-[var(--color-text-muted)] mb-1">{{ comparison.currentMonth }}</p>
          <p class="text-base sm:text-lg font-bold text-[var(--color-text-primary)]">{{ fmt(comparison.current) }}</p>
        </div>
      </div>
      <div
        class="flex items-center justify-center gap-2 py-2 rounded-lg text-sm font-semibold"
        :class="comparison.diff > 0 ? 'bg-red-50 text-red-600 dark:bg-red-900/20 dark:text-red-400' : comparison.diff < 0 ? 'bg-green-50 text-green-600 dark:bg-green-900/20 dark:text-green-400' : 'bg-gray-50 text-gray-500 dark:bg-gray-800 dark:text-gray-400'"
      >
        <TrendingUp v-if="comparison.diff > 0" :size="16" />
        <TrendingDown v-else-if="comparison.diff < 0" :size="16" />
        <Minus v-else :size="16" />
        <span>{{ comparison.diff > 0 ? '+' : '' }}{{ fmt(comparison.diff) }} ({{ comparison.diff > 0 ? '+' : '' }}{{ comparison.diffPercent }}%)</span>
      </div>
    </div>
  </div>
</template>
