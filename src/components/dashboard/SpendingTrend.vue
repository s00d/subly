<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Bar } from "vue-chartjs";
import { Chart as ChartJS, CategoryScale, LinearScale, BarElement, PointElement, LineElement, Tooltip, Legend, Filler } from "chart.js";
import type { MonthlySpending } from "@/services/analytics";

ChartJS.register(CategoryScale, LinearScale, BarElement, PointElement, LineElement, Tooltip, Legend, Filler);

const props = defineProps<{
  data: MonthlySpending[];
  fmt: (n: number) => string;
}>();

const { t } = useI18n();

const chartData = computed(() => ({
  labels: props.data.map((d) => d.label),
  datasets: [
    {
      label: t("monthly_cost"),
      data: props.data.map((d) => d.amount),
      backgroundColor: props.data.map((_, i) =>
        i === props.data.length - 1 ? "rgba(99, 102, 241, 0.8)" : "rgba(99, 102, 241, 0.35)",
      ),
      borderColor: "rgb(99, 102, 241)",
      borderWidth: 1,
      borderRadius: 4,
    },
  ],
}));

const chartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { display: false },
    tooltip: {
      callbacks: {
        label: (ctx: { raw: unknown }) => props.fmt(ctx.raw as number),
      },
    },
  },
  scales: {
    y: {
      beginAtZero: true,
      ticks: {
        callback: (val: string | number) => props.fmt(Number(val)),
        maxTicksLimit: 5,
      },
      grid: { color: "rgba(128,128,128,0.1)" },
    },
    x: {
      grid: { display: false },
    },
  },
}));

const avg = computed(() => {
  if (props.data.length === 0) return 0;
  return props.data.reduce((s, d) => s + d.amount, 0) / props.data.length;
});
</script>

<template>
  <div class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
    <div class="flex items-center justify-between mb-3 sm:mb-4">
      <h3 class="text-xs sm:text-sm font-semibold text-[var(--color-text-primary)]">{{ t('spending_trend') }}</h3>
      <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)]">{{ t('avg') }}: {{ fmt(avg) }}</span>
    </div>
    <div class="h-40 sm:h-56">
      <Bar :data="chartData" :options="chartOptions" />
    </div>
  </div>
</template>
