<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { useCatalogStore } from "@/stores/catalog";
import { useI18n } from "vue-i18n";
import { dbGetRateHistoryBatch } from "@/services/database";
import type { RateHistoryPoint } from "@/services/database";
import { currencyFlag } from "@/services/currencyFlags";
import { TrendingUp } from "lucide-vue-next";
import { useRouter } from "vue-router";
import { Line } from "vue-chartjs";
import {
  Chart as ChartJS,
  LineElement,
  PointElement,
  LinearScale,
  CategoryScale,
  Tooltip as ChartTooltip,
  Legend,
  Filler,
} from "chart.js";

ChartJS.register(LineElement, PointElement, LinearScale, CategoryScale, ChartTooltip, Legend, Filler);

const LINE_COLORS = [
  "#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6",
  "#06b6d4", "#ec4899", "#f97316", "#14b8a6", "#6366f1",
  "#a855f7", "#84cc16", "#e11d48", "#0ea5e9", "#d946ef",
];

const settingsStore = useSettingsStore();
const catalogStore = useCatalogStore();
const { t } = useI18n();
const router = useRouter();

const history = ref<Record<string, RateHistoryPoint[]>>({});

const targetIds = computed(() => settingsStore.settings.currencyUpdateTargets);
const mainCurrency = computed(() => catalogStore.mainCurrency);

const currencies = computed(() =>
  targetIds.value
    .map((id) => catalogStore.currencies.find((c) => c.id === id))
    .filter((c): c is NonNullable<typeof c> => !!c && c.rate > 0),
);

const hasHistory = computed(() => Object.values(history.value).some((h) => h.length >= 1));

onMounted(async () => {
  if (!settingsStore.settings.rateHistoryEnabled || targetIds.value.length === 0) return;
  history.value = await dbGetRateHistoryBatch(targetIds.value, settingsStore.settings.rateHistoryDays);
});

function sparkPath(data: number[], w: number, h: number): string {
  if (data.length === 0) return "";
  if (data.length === 1) {
    const cx = w / 2;
    const cy = h / 2;
    return `M${cx - 0.5},${cy} L${cx + 0.5},${cy}`;
  }
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;
  const pad = 1;
  const iw = w - pad * 2;
  const ih = h - pad * 2;
  const step = iw / (data.length - 1);
  return data
    .map((v, i) => {
      const x = pad + i * step;
      const y = pad + ih - ((v - min) / range) * ih;
      return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
}

function trendDir(data: number[]): number {
  if (data.length < 2) return 0;
  return data[data.length - 1] - data[0];
}

function pctChange(data: number[]): string {
  if (data.length < 2) return "";
  const first = data[0];
  if (first === 0) return "";
  const pct = ((data[data.length - 1] - first) / first) * 100;
  const sign = pct >= 0 ? "+" : "";
  return `${sign}${pct.toFixed(2)}%`;
}

// Line chart data
const allDates = computed(() => {
  const dateSet = new Set<string>();
  for (const points of Object.values(history.value)) {
    for (const p of points) dateSet.add(p.recordedAt);
  }
  return [...dateSet].sort();
});

const chartData = computed(() => {
  const labels = allDates.value.map((d) => {
    const parts = d.split("-");
    return `${parts[2]}.${parts[1]}`;
  });

  const datasets = currencies.value
    .filter((cur) => (history.value[cur.id] ?? []).length >= 1)
    .map((cur, i) => {
      const points = history.value[cur.id] ?? [];
      const rateMap = new Map(points.map((p) => [p.recordedAt, p.rate]));
      const data = allDates.value.map((d) => rateMap.get(d) ?? null);
      const color = LINE_COLORS[i % LINE_COLORS.length];
      const flag = currencyFlag(cur.code);
      return {
        label: `${flag ? flag + " " : ""}${cur.code}`,
        data,
        borderColor: color,
        backgroundColor: color + "20",
        fill: false,
        tension: 0.3,
        pointRadius: allDates.value.length > 30 ? 0 : 3,
        pointHoverRadius: 5,
        borderWidth: 2,
        spanGaps: true,
      };
    });

  return { labels, datasets };
});

const chartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  interaction: {
    mode: "index" as const,
    intersect: false,
  },
  plugins: {
    legend: {
      position: "bottom" as const,
      labels: {
        usePointStyle: true,
        pointStyle: "line" as const,
        padding: 12,
        font: { size: 11 },
      },
    },
    tooltip: {
      callbacks: {
        label: (ctx: { dataset: { label?: string }; parsed: { y: number | null } }) => {
          const val = ctx.parsed.y;
          if (val === null) return "";
          return `${ctx.dataset.label}: ${val.toFixed(4)}`;
        },
      },
    },
  },
  scales: {
    x: {
      grid: { display: false },
      ticks: { font: { size: 10 }, maxRotation: 45 },
    },
    y: {
      grid: { color: "rgba(128,128,128,0.1)" },
      ticks: { font: { size: 10 } },
    },
  },
}));
</script>

<template>
  <div
    v-if="settingsStore.settings.rateHistoryEnabled && currencies.length > 0 && hasHistory"
    class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5"
  >
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <TrendingUp :size="16" class="text-[var(--color-primary)]" />
        <h2 class="text-sm sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ t('rate_history') }}</h2>
      </div>
      <button
        @click="router.push('/currencies')"
        class="text-xs text-[var(--color-primary)] hover:underline"
      >{{ t('exchange_rates') }} →</button>
    </div>

    <!-- Mini cards -->
    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2 sm:gap-3 mb-4">
      <div
        v-for="cur in currencies"
        :key="cur.id"
        class="rounded-lg border border-[var(--color-border)] p-2.5 sm:p-3 bg-[var(--color-surface-secondary)]"
      >
        <div class="flex items-center gap-1.5 mb-1.5">
          <span v-if="currencyFlag(cur.code)" class="text-sm">{{ currencyFlag(cur.code) }}</span>
          <span class="text-xs font-bold text-[var(--color-text-primary)]">{{ cur.code }}</span>
          <span
            v-if="pctChange((history[cur.id] ?? []).map(p => p.rate))"
            class="ml-auto text-[10px] font-semibold tabular-nums"
            :class="trendDir((history[cur.id] ?? []).map(p => p.rate)) >= 0 ? 'text-green-500' : 'text-red-500'"
          >{{ pctChange((history[cur.id] ?? []).map(p => p.rate)) }}</span>
        </div>

        <div class="flex items-end justify-between gap-1">
          <svg
            v-if="(history[cur.id] ?? []).length >= 1"
            :width="80"
            :height="28"
            :viewBox="`0 0 80 28`"
            class="flex-1"
          >
            <path
              :d="sparkPath((history[cur.id] ?? []).map(p => p.rate), 80, 28)"
              fill="none"
              :stroke="trendDir((history[cur.id] ?? []).map(p => p.rate)) >= 0 ? '#22c55e' : '#ef4444'"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          <span v-else class="text-[10px] text-[var(--color-text-muted)] flex-1">—</span>
          <span class="text-[11px] font-mono font-semibold text-[var(--color-text-primary)] tabular-nums shrink-0">
            {{ cur.rate.toFixed(2) }}
          </span>
        </div>

        <div class="text-[10px] text-[var(--color-text-muted)] mt-1 truncate">
          1 {{ mainCurrency?.code }} = {{ cur.rate.toFixed(4).replace(/\.?0+$/, '') }} {{ cur.code }}
        </div>
      </div>
    </div>

    <!-- Line chart -->
    <div v-if="hasHistory && chartData.datasets.length > 0" class="h-64 sm:h-80">
      <Line :data="chartData" :options="chartOptions" />
    </div>
  </div>
</template>
