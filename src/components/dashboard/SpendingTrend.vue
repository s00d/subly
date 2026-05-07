<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import VChart from "vue-echarts";
import type { EChartsCoreOption } from "echarts/core";
import type { MonthlySpending } from "@/services/dashboardClient";

const props = defineProps<{
  data: MonthlySpending[];
  fmt: (n: number) => string;
}>();

const { t } = useI18n();

const chartOption = computed((): EChartsCoreOption => {
  const labels = props.data.map((d) => d.label);
  const values = props.data.map((d) => d.amount);
  const n = values.length;
  return {
    animationDuration: 420,
    grid: { left: 8, right: 8, top: 10, bottom: 6, containLabel: true },
    tooltip: {
      trigger: "axis",
      axisPointer: { type: "shadow", shadowStyle: { color: "rgba(99, 102, 241, 0.08)" } },
      formatter: (params: unknown) => {
        const arr = Array.isArray(params) ? params : [params];
        const p = arr[0] as { axisValue?: string; value?: number | number[] };
        const v = typeof p.value === "number" ? p.value : Number((p.value as number[])?.[0] ?? 0);
        return `${p.axisValue ?? ""}<br/><span style="font-weight:600">${props.fmt(v)}</span>`;
      },
    },
    xAxis: {
      type: "category",
      data: labels,
      axisTick: { show: false },
      axisLine: { lineStyle: { color: "rgba(128,128,128,0.22)" } },
      axisLabel: { color: "rgba(127,127,127,0.92)", fontSize: 11 },
    },
    yAxis: {
      type: "value",
      splitLine: { lineStyle: { color: "rgba(128,128,128,0.1)" } },
      axisLabel: {
        color: "rgba(127,127,127,0.85)",
        fontSize: 10,
        formatter: (val: number) => props.fmt(val),
      },
    },
    series: [
      {
        name: t("monthly_cost"),
        type: "bar",
        barMaxWidth: 40,
        itemStyle: {
          borderRadius: [6, 6, 0, 0],
          color: (item: { dataIndex: number }) =>
            item.dataIndex === n - 1 ? "rgba(99, 102, 241, 0.92)" : "rgba(99, 102, 241, 0.38)",
        },
        emphasis: {
          itemStyle: {
            color: (item: { dataIndex: number }) =>
              item.dataIndex === n - 1 ? "rgba(79, 70, 229, 0.98)" : "rgba(99, 102, 241, 0.52)",
          },
        },
        data: values,
      },
    ],
  };
});

const avg = computed(() => {
  if (props.data.length === 0) return 0;
  return props.data.reduce((s, d) => s + d.amount, 0) / props.data.length;
});
</script>

<template>
  <div class="bg-surface rounded-xl border border-border p-2.5 sm:p-4">
    <div class="flex items-center justify-between mb-2.5 sm:mb-3">
      <h3 class="text-xs sm:text-sm font-semibold text-text-primary">{{ t('spending_trend') }}</h3>
      <span class="text-[10px] sm:text-xs text-text-muted">{{ t('avg') }}: {{ fmt(avg) }}</span>
    </div>
    <div class="h-36 sm:h-48">
      <VChart class="h-full w-full min-h-[10rem]" :option="chartOption" autoresize />
    </div>
  </div>
</template>
