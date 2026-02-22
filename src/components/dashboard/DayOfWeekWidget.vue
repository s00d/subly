<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { dbGetExpensesByDayOfWeek, type DayOfWeekStat } from "@/services/database";
import { Bar } from "vue-chartjs";
import {
  Chart as ChartJS,
  BarElement,
  CategoryScale,
  LinearScale,
  Tooltip as ChartTooltip,
} from "chart.js";
import { CalendarDays } from "lucide-vue-next";

ChartJS.register(BarElement, CategoryScale, LinearScale, ChartTooltip);

const { t } = useI18n();
const { fmt } = useCurrencyFormat();

const rawStats = ref<DayOfWeekStat[]>([]);

onMounted(async () => {
  const d = new Date();
  const prefix = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
  rawStats.value = await dbGetExpensesByDayOfWeek(prefix);
});

const dayLabels = computed(() => [
  t("day_sun"), t("day_mon"), t("day_tue"), t("day_wed"),
  t("day_thu"), t("day_fri"), t("day_sat"),
]);

const chartData = computed(() => {
  const totals = Array(7).fill(0);
  for (const s of rawStats.value) {
    totals[s.dayOfWeek] = s.total;
  }
  const maxVal = Math.max(...totals);
  const colors = totals.map((v) =>
    v === maxVal && maxVal > 0 ? "#ef4444" : "#3b82f6",
  );
  return {
    labels: dayLabels.value,
    datasets: [{
      data: totals,
      backgroundColor: colors,
      borderRadius: 4,
      barPercentage: 0.7,
    }],
  };
});

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: { legend: { display: false }, tooltip: { enabled: true } },
  scales: {
    x: { grid: { display: false }, ticks: { font: { size: 10 } } },
    y: { grid: { color: "rgba(128,128,128,0.1)" }, ticks: { font: { size: 10 } }, beginAtZero: true },
  },
};

const hasData = computed(() => rawStats.value.length > 0);
const peakDay = computed(() => {
  if (!hasData.value) return "";
  const max = rawStats.value.reduce((a, b) => (b.total > a.total ? b : a), rawStats.value[0]);
  return `${dayLabels.value[max.dayOfWeek]} — ${fmt(max.total)}`;
});
</script>

<template>
  <div v-if="hasData" class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <CalendarDays :size="16" class="text-[var(--color-primary)]" />
        <h2 class="text-sm sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ t('widget_day_of_week') }}</h2>
      </div>
      <span v-if="peakDay" class="text-[10px] text-[var(--color-text-muted)]">{{ t('peak') }}: {{ peakDay }}</span>
    </div>
    <div class="h-48 sm:h-56">
      <Bar :data="chartData" :options="chartOptions" />
    </div>
  </div>
</template>
