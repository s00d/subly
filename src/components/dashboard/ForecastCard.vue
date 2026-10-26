<script setup lang="ts">
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import UiCard from "@/components/ui/UiCard.vue";
import { useI18n } from "vue-i18n";
import type { ForecastResult } from "@/services/dashboardClient";
import type { MoneyString } from "@/lib/money";
import { Calendar } from "@lucide/vue";
import { typo, iconSize, statValue } from "@/lib/tv";

defineProps<{
  forecast: ForecastResult;
  fmt: (n: MoneyString) => string;
}>();

const { t } = useI18n();
const { fmtMonthYear } = useLocaleFormat();
/** "2026-11" → "November 2026" in the UI locale; anything else passes through. */
function monthLabel(ym: string): string {
  const m = /^(\d{4})-(\d{2})$/.exec(ym);
  return m ? fmtMonthYear(new Date(Number(m[1]), Number(m[2]) - 1, 1)) : ym;
}
</script>

<template>
  <UiCard :title="t('forecast')" :icon="Calendar" tone="success">
    <div class="space-y-2.5 sm:space-y-3">
      <div>
        <div class="flex items-center gap-2 mb-1">
          <Calendar :size="iconSize.xs" class="text-text-muted" />
          <span class="text-xs text-text-muted">{{ forecast.nextMonthLabel ? monthLabel(forecast.nextMonthLabel) : t('next_month') }}</span>
        </div>
        <p :class="statValue()">{{ fmt(forecast.nextMonth) }}</p>
      </div>
      <div class="pt-2.5 border-t border-border">
        <div class="flex items-center gap-2 mb-1">
          <Calendar :size="iconSize.xs" class="text-text-muted" />
          <span class="text-[10px] sm:text-xs text-text-muted">{{ t('next_quarter') }}</span>
        </div>
        <p :class="typo.statValueMd()">{{ fmt(forecast.nextQuarter) }}</p>
        <p v-if="forecast.quarterLabels.length" class="text-[11px] text-text-muted mt-0.5">{{ forecast.quarterLabels.map(monthLabel).join(' → ') }}</p>
      </div>
    </div>
  </UiCard>
</template>
