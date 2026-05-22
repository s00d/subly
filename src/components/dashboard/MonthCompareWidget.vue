<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { moneyFromDecimal, moneyGt, parseMoney, sub, toDisplayNumber, type MoneyString } from "@/lib/money";
import type { MonthComparisonStyle } from "@/services/dashboardClient";
import { ArrowUpRight, ArrowDownRight, Equal } from "@lucide/vue";

const { t } = useI18n();
const { fmt } = useCurrencyFormat();
const props = defineProps<{
  data?: {
    currentTotal: MoneyString;
    currentCount: number;
    previousTotal: MoneyString;
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
  if (!data.value) return moneyFromDecimal(parseMoney("0"));
  return moneyFromDecimal(sub(data.value.currentTotal, data.value.previousTotal));
});

const pctChange = computed(() => {
  if (!data.value) return null;
  const prev = parseMoney(data.value.previousTotal);
  if (prev.isZero()) return null;
  const d = sub(data.value.currentTotal, data.value.previousTotal);
  return toDisplayNumber(d.dividedBy(prev).times(100), 1);
});

const barMax = computed(() => {
  if (!data.value) return 1;
  const cur = toDisplayNumber(parseMoney(data.value.currentTotal));
  const prev = toDisplayNumber(parseMoney(data.value.previousTotal));
  return Math.max(cur, prev, 1);
});

const hasData = computed(() => {
  const d = data.value;
  if (!d) return false;
  return moneyGt(d.currentTotal, "0") || moneyGt(d.previousTotal, "0");
});

</script>

<template>
  <div v-if="data && hasData" class="bg-surface rounded-xl border border-border p-2.5 sm:p-4">
    <div class="flex items-center gap-2 mb-2.5">
      <component
        :is="moneyGt(diff, '0') ? ArrowUpRight : moneyGt('0', diff) ? ArrowDownRight : Equal"
        :size="16"
        :class="moneyGt(diff, '0') ? 'text-red-500' : moneyGt('0', diff) ? 'text-green-500' : 'text-text-muted'"
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
            :style="{ width: (toDisplayNumber(parseMoney(data.currentTotal)) / barMax * 100) + '%' }"
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
            :style="{ width: (toDisplayNumber(parseMoney(data.previousTotal)) / barMax * 100) + '%' }"
          />
        </div>
        <span class="text-[10px] text-text-muted">{{ data.previousCount }} {{ t('records') }}</span>
      </div>

      <!-- Diff -->
      <div class="flex items-center justify-center gap-2 pt-2 border-t border-border">
        <span
          class="text-sm font-bold tabular-nums"
          :class="moneyGt(diff, '0') ? 'text-red-500' : moneyGt('0', diff) ? 'text-green-500' : 'text-text-muted'"
        >
          {{ moneyGt(diff, '0') ? '+' : '' }}{{ fmt(diff) }}
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
