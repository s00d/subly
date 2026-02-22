<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { dbGetExpensesByTags, type TagExpenseStat } from "@/services/database";
import { Doughnut } from "vue-chartjs";
import { Chart as ChartJS, ArcElement, Tooltip as ChartTooltip, Legend } from "chart.js";
import { Hash } from "lucide-vue-next";

ChartJS.register(ArcElement, ChartTooltip, Legend);

const COLORS = [
  "#3b82f6", "#22c55e", "#f59e0b", "#ef4444", "#8b5cf6",
  "#06b6d4", "#ec4899", "#f97316", "#14b8a6", "#6366f1",
];

const { t } = useI18n();
const { fmt } = useCurrencyFormat();

const stats = ref<TagExpenseStat[]>([]);

onMounted(async () => {
  const d = new Date();
  const prefix = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
  stats.value = await dbGetExpensesByTags(prefix);
});

const chartData = computed(() => ({
  labels: stats.value.map((s) => s.tag),
  datasets: [{
    data: stats.value.map((s) => Math.round(s.total * 100) / 100),
    backgroundColor: COLORS.slice(0, stats.value.length),
  }],
}));

const chartOptions = {
  responsive: true,
  plugins: {
    legend: { position: "bottom" as const, labels: { padding: 10, font: { size: 11 } } },
  },
};
</script>

<template>
  <div v-if="stats.length > 0" class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
    <div class="flex items-center gap-2 mb-3">
      <Hash :size="16" class="text-[var(--color-primary)]" />
      <h2 class="text-sm sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ t('widget_tag_expenses') }}</h2>
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
      <div class="max-w-[220px] mx-auto">
        <Doughnut :data="chartData" :options="chartOptions" />
      </div>
      <div class="space-y-1.5 overflow-y-auto max-h-52">
        <div
          v-for="(s, i) in stats"
          :key="s.tag"
          class="flex items-center gap-2"
        >
          <span class="w-2.5 h-2.5 rounded-full shrink-0" :style="{ background: COLORS[i % COLORS.length] }" />
          <span class="text-sm text-[var(--color-text-primary)] flex-1 truncate">#{{ s.tag }}</span>
          <span class="text-sm font-semibold text-[var(--color-text-primary)] tabular-nums shrink-0">{{ fmt(s.total) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
