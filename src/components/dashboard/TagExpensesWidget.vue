<script setup lang="ts">
import UiCard from "@/components/ui/UiCard.vue";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useChartTheme } from "@/composables/useChartTheme";
import VChart from "@/components/ui/LazyVChart.vue";
import type { EChartsCoreOption } from "echarts/core";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { moneyFromNumber, parseMoney, toDisplayNumber, type MoneyString } from "@/lib/money";
import { Hash } from "@lucide/vue";

/**
 * Pie slice separators are painted by ECharts as a literal color string, so
 * we can't pass `var(--color-surface)` directly — it would be treated as an
 * invalid color and prevent the chart from rendering at all. Instead we
 * resolve the variable once on mount and keep it in sync if the theme flips.
 */
const sliceBorderColor = ref("#ffffff");
function syncBorderColor() {
  if (typeof window === "undefined") return;
  const v = getComputedStyle(document.documentElement)
    .getPropertyValue("--color-surface")
    .trim();
  if (v) sliceBorderColor.value = v;
}
let themeObserver: MutationObserver | null = null;
onMounted(() => {
  syncBorderColor();
  themeObserver = new MutationObserver(syncBorderColor);
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["class"],
  });
});
onUnmounted(() => {
  themeObserver?.disconnect();
  themeObserver = null;
});

const COLORS = [
  "#3b82f6", "#22c55e", "#f59e0b", "#ef4444", "#8b5cf6",
  "#06b6d4", "#ec4899", "#f97316", "#14b8a6", "#6366f1",
];

const { t } = useI18n();
const { fmt } = useCurrencyFormat();
const chartTheme = useChartTheme();
const props = defineProps<{
  stats?: Array<{ tag: string; total: MoneyString }>;
}>();
const stats = computed(() => props.stats ?? []);

const chartOption = computed((): EChartsCoreOption => ({
  animationDuration: 450,
  tooltip: {
    trigger: "item",
    formatter: (params: unknown) => {
      const p = params as { name?: string; value?: number; percent?: number };
      const name = p.name ?? "";
      const val = typeof p.value === "number" ? p.value : 0;
      const pct = typeof p.percent === "number" ? p.percent.toFixed(1) : "";
      return `#${name}<br/>${fmt(moneyFromNumber(val))} (${pct}%)`;
    },
  },
  legend: {
    show: false,
  },
  series: [
    {
      type: "pie",
      radius: ["42%", "68%"],
      center: ["50%", "50%"],
      avoidLabelOverlap: true,
      itemStyle: {
        borderRadius: 6,
        borderColor: sliceBorderColor.value,
        borderWidth: 2,
      },
      emphasis: {
        scale: true,
        itemStyle: {
          shadowBlur: 14,
          shadowOffsetY: 2,
          shadowColor: "rgba(0,0,0,0.12)",
        },
      },
      label: { show: false },
      data: stats.value.map((s, i) => ({
        name: s.tag,
        value: toDisplayNumber(parseMoney(s.total)),
        itemStyle: { color: COLORS[i % COLORS.length] },
      })),
    },
  ],
}));
</script>

<template>
  <UiCard v-if="stats.length > 0" :title="t('widget_tag_expenses')" :icon="Hash" tone="violet">
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 items-center">
      <div class="w-full max-w-[220px] mx-auto h-44 sm:h-48 aspect-square">
        <VChart class="h-full w-full" :theme="chartTheme" :option="chartOption" autoresize />
      </div>
      <div class="space-y-1.5 overflow-y-auto max-h-44 sm:max-h-48">
        <div
          v-for="(s, i) in stats"
          :key="s.tag"
          class="flex items-center gap-2"
        >
          <span class="w-2.5 h-2.5 rounded-full shrink-0" :style="{ background: COLORS[i % COLORS.length] }" />
          <span class="text-sm text-text-primary flex-1 truncate">#{{ s.tag }}</span>
          <span class="text-sm font-semibold text-text-primary tabular-nums shrink-0">{{ fmt(s.total) }}</span>
        </div>
      </div>
    </div>
  </UiCard>
</template>
