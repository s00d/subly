<script setup lang="ts">
import UiCard from "@/components/ui/UiCard.vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useChartTheme } from "@/composables/useChartTheme";
import VChart from "@/components/ui/LazyVChart.vue";
import type { EChartsCoreOption } from "echarts/core";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { moneyFromNumber, moneyGt, parseMoney, toDisplayNumber, type MoneyString } from "@/lib/money";
import { CalendarDays } from "@lucide/vue";

const { t } = useI18n();
const { fmt } = useCurrencyFormat();
const chartTheme = useChartTheme();
const props = defineProps<{
  rawStats?: Array<{ dayOfWeek: number; total: MoneyString; count: number }>;
}>();
const rawStats = computed(() => props.rawStats ?? []);

const dayLabels = computed(() => [
  t("day_sun"), t("day_mon"), t("day_tue"), t("day_wed"),
  t("day_thu"), t("day_fri"), t("day_sat"),
]);

const chartOption = computed((): EChartsCoreOption => {
  const totals = Array(7).fill(0);
  for (const s of rawStats.value) {
    totals[s.dayOfWeek] = toDisplayNumber(parseMoney(s.total));
  }
  const maxVal = Math.max(...totals);
  return {
    animationDuration: 400,
    grid: { left: 8, right: 8, top: 10, bottom: 6, containLabel: true },
    tooltip: {
      trigger: "axis",
      axisPointer: { type: "shadow" },
      formatter: (params: unknown) => {
        const arr = Array.isArray(params) ? params : [params];
        const p = arr[0] as { axisValue?: string; value?: number | number[] };
        const v = typeof p.value === "number" ? p.value : Number((p.value as number[])?.[0] ?? 0);
        return `${p.axisValue ?? ""}<br/><span style="font-weight:600">${fmt(moneyFromNumber(v))}</span>`;
      },
    },
    xAxis: {
      type: "category",
      data: dayLabels.value,
      axisTick: { show: false },
      axisLine: { lineStyle: { color: "rgba(128,128,128,0.22)" } },
      axisLabel: { color: "rgba(127,127,127,0.92)", fontSize: 10 },
    },
    yAxis: {
      type: "value",
      splitLine: { lineStyle: { color: "rgba(128,128,128,0.1)" } },
      axisLabel: {
        color: "rgba(127,127,127,0.85)",
        fontSize: 10,
        formatter: (val: number) => fmt(moneyFromNumber(val)),
      },
    },
    series: [
      {
        type: "bar",
        barMaxWidth: 36,
        itemStyle: {
          borderRadius: [6, 6, 0, 0],
          color: (item: { dataIndex: number }) => {
            const v = totals[item.dataIndex];
            return v === maxVal && maxVal > 0 ? "#ef4444" : "#3b82f6";
          },
        },
        data: totals,
      },
    ],
  };
});

const hasData = computed(() => rawStats.value.length > 0);
const peakDay = computed(() => {
  if (!hasData.value) return "";
  const max = rawStats.value.reduce((a, b) =>
    moneyGt(b.total, a.total) ? b : a,
  rawStats.value[0]);
  return `${dayLabels.value[max.dayOfWeek]} — ${fmt(max.total)}`;
});
</script>

<template>
  <UiCard v-if="hasData" :title="t('widget_day_of_week')" :icon="CalendarDays" tone="warning">
    <template #actions>
    <span v-if="peakDay" class="text-[11px] text-text-muted">{{ t('peak') }}: {{ peakDay }}</span>
    </template>
    <div class="h-48 sm:h-52">
      <VChart class="h-full w-full" :theme="chartTheme" :option="chartOption" autoresize />
    </div>
  </UiCard>
</template>
