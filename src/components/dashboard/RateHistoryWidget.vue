<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { getRateHistoryWidget, type RateHistoryPoint } from "@/services/dashboardClient";
import { getCurrencyFlags } from "@/services/ratesClient";
import type { Settings, Currency } from "@/schemas/appData";
import { TrendingUp } from "@lucide/vue";
import { useRouter } from "vue-router";
import VChart from "vue-echarts";
import type { EChartsCoreOption } from "echarts/core";

const LINE_COLORS = [
  "#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6",
  "#06b6d4", "#ec4899", "#f97316", "#14b8a6", "#6366f1",
  "#a855f7", "#84cc16", "#e11d48", "#0ea5e9", "#d946ef",
];
const props = defineProps<{
  settings?: Settings | null;
  currencies?: Currency[];
}>();

const settings = ref<Settings | null>(null);
const allCurrencies = ref<Currency[]>([]);
const { t } = useI18n();
const router = useRouter();

const history = ref<Record<string, RateHistoryPoint[]>>({});
const currencyFlags = ref<Record<string, string>>({});

const targetIds = computed(() => settings.value?.currencyUpdateTargets ?? []);
const mainCurrency = computed(() =>
  allCurrencies.value.find((c) => c.id === settings.value?.mainCurrencyId) ?? null,
);

const currencies = computed(() =>
  targetIds.value
    .map((id) => allCurrencies.value.find((c) => c.id === id))
    .filter((c): c is NonNullable<typeof c> => !!c && c.rate > 0),
);

const hasHistory = computed(() => Object.values(history.value).some((h) => h.length >= 1));

async function loadHistory() {
  if (!settings.value?.rateHistoryEnabled || targetIds.value.length === 0) return;
  history.value = await getRateHistoryWidget(targetIds.value, settings.value.rateHistoryDays);
}

async function loadFlags() {
  const codes = [...new Set(currencies.value.map((c) => c.code).filter(Boolean))];
  currencyFlags.value = codes.length ? await getCurrencyFlags(codes) : {};
}

function flagFor(code: string): string {
  return currencyFlags.value[code.toUpperCase()] || "";
}

onMounted(async () => {
  settings.value = props.settings ?? null;
  allCurrencies.value = props.currencies ?? [];
  await loadHistory();
  await loadFlags();
});

watch(
  [targetIds, () => settings.value?.rateHistoryEnabled, () => settings.value?.rateHistoryDays],
  () => {
    loadHistory();
    loadFlags();
  },
  { deep: true },
);

function sparkPath(data: number[], w: number, h: number): string {
  if (data.length === 0) return "";
  if (data.length === 1) {
    const cx = w / 2;
    const cy = h / 2;
    return `M${cx - 0.5},${cy} L${cx + 0.5},${cy}`;
  }
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min;
  const pad = 1;
  const iw = w - pad * 2;
  const ih = h - pad * 2;
  const step = iw / (data.length - 1);
  return data
    .map((v, i) => {
      const x = pad + i * step;
      const y = range === 0 ? h / 2 : pad + ih - ((v - min) / range) * ih;
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

const allDates = computed(() => {
  const dateSet = new Set<string>();
  for (const points of Object.values(history.value)) {
    for (const p of points) dateSet.add(p.recordedAt);
  }
  return [...dateSet].sort();
});

const rateLineSeriesCount = computed(
  () => currencies.value.filter((cur) => (history.value[cur.id] ?? []).length >= 1).length,
);

const rateChartOption = computed((): EChartsCoreOption => {
  const labels = allDates.value.map((d) => {
    const parts = d.split("-");
    return `${parts[2]}.${parts[1]}`;
  });

  const series = currencies.value
    .filter((cur) => (history.value[cur.id] ?? []).length >= 1)
    .map((cur, i) => {
      const points = history.value[cur.id] ?? [];
      const rateMap = new Map(points.map((p) => [p.recordedAt, p.rate]));
      const data = allDates.value.map((d) => rateMap.get(d) ?? null);
      const color = LINE_COLORS[i % LINE_COLORS.length];
      const flag = flagFor(cur.code);
      return {
        name: `${flag ? flag + " " : ""}${cur.code}`,
        type: "line" as const,
        smooth: 0.35,
        connectNulls: true,
        showSymbol: allDates.value.length <= 30,
        symbolSize: allDates.value.length > 30 ? 0 : 6,
        lineStyle: { width: 2.5, color },
        itemStyle: { color },
        data,
      };
    });

  return {
    animationDuration: 480,
    color: LINE_COLORS,
    tooltip: {
      trigger: "axis",
      axisPointer: { type: "line" },
      formatter: (params: unknown) => {
        const arr = Array.isArray(params) ? params : [params];
        if (arr.length === 0) return "";
        const first = arr[0] as { axisValue?: string };
        const lines = arr
          .map((raw: unknown) => {
            const x = raw as { seriesName?: string; value?: number | null };
            if (x.value === null || x.value === undefined) return "";
            return `${x.seriesName}: ${Number(x.value).toFixed(4)}`;
          })
          .filter(Boolean);
        return lines.length ? `${first.axisValue ?? ""}<br/>${lines.join("<br/>")}` : "";
      },
    },
    legend: {
      bottom: 0,
      type: "scroll",
      textStyle: { fontSize: 11 },
      itemGap: 12,
    },
    grid: {
      left: 10,
      right: 12,
      top: 20,
      bottom: series.length > 3 ? 56 : 48,
      containLabel: true,
    },
    xAxis: {
      type: "category",
      boundaryGap: false,
      data: labels,
      axisLine: { lineStyle: { color: "rgba(128,128,128,0.2)" } },
      axisLabel: { fontSize: 10, rotate: labels.length > 14 ? 36 : 0 },
    },
    yAxis: {
      type: "value",
      splitLine: { lineStyle: { color: "rgba(128,128,128,0.1)" } },
      axisLabel: { fontSize: 10 },
    },
    series,
  };
});
</script>

<template>
  <div
    v-if="settings?.rateHistoryEnabled && currencies.length > 0 && hasHistory"
    class="bg-surface rounded-xl border border-border p-2.5 sm:p-4"
  >
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <TrendingUp :size="16" class="text-primary" />
        <h2 class="text-sm sm:text-lg font-semibold text-text-primary">{{ t('rate_history') }}</h2>
      </div>
      <button
        @click="router.push('/currencies')"
        class="text-xs text-primary hover:underline"
      >{{ t('exchange_rates') }} →</button>
    </div>

    <!-- Mini cards -->
    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2 sm:gap-2.5 mb-3">
      <div
        v-for="cur in currencies"
        :key="cur.id"
        class="rounded-lg border border-border p-2.5 sm:p-3 bg-surface-secondary"
      >
        <div class="flex items-center gap-1.5 mb-1.5">
          <span v-if="flagFor(cur.code)" class="text-sm">{{ flagFor(cur.code) }}</span>
          <span class="text-xs font-bold text-text-primary">{{ cur.code }}</span>
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
          <span v-else class="text-[10px] text-text-muted flex-1">—</span>
          <span class="text-[11px] font-mono font-semibold text-text-primary tabular-nums shrink-0">
            {{ cur.rate.toFixed(2) }}
          </span>
        </div>

        <div class="text-[10px] text-text-muted mt-1 truncate">
          1 {{ mainCurrency?.code }} = {{ cur.rate.toFixed(4).replace(/\.?0+$/, '') }} {{ cur.code }}
        </div>
      </div>
    </div>

    <!-- Line chart -->
    <div v-if="hasHistory && rateLineSeriesCount > 0" class="h-56 sm:h-72 min-h-[14rem]">
      <VChart class="h-full w-full" :option="rateChartOption" autoresize />
    </div>
  </div>
</template>
