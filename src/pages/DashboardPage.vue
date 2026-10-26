<script setup lang="ts">
import UiEmptyState from "@/components/ui/UiEmptyState.vue";
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useChartTheme } from "@/composables/useChartTheme";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useHeaderActions } from "@/composables/useHeaderActions";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import {
  type DashboardSummaryDto,
  type DashboardChartsDto,
  type DashboardForecastDto,
  type DashboardTrendsDto,
} from "@/services/dashboardClient";
import { storeToRefs } from "pinia";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { useDashboardStore } from "@/stores/dashboardStore";
import VChart from "vue-echarts";
import type { EChartsCoreOption } from "echarts/core";
import "@/lib/echarts";
import { Wallet, ArrowRight, Settings2, Eye, EyeOff, ChevronUp, ChevronDown, PiggyBank } from "@lucide/vue";
import UiCard from "@/components/ui/UiCard.vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { ui, typo, statValue, button } from "@/lib/tv";
import { moneyFromDecimal, parseMoney, toDisplayNumber, moneyGt, mul, type MoneyString } from "@/lib/money";

import InAppAlerts from "@/components/ui/InAppAlerts.vue";
import { useAlerts } from "@/composables/useAlerts";
import UpcomingPayments from "@/components/dashboard/UpcomingPayments.vue";
import StatsCards from "@/components/dashboard/StatsCards.vue";
import SpendingTrend from "@/components/dashboard/SpendingTrend.vue";
import ForecastCard from "@/components/dashboard/ForecastCard.vue";
import LifetimeCosts from "@/components/dashboard/LifetimeCosts.vue";
import CategoryAverages from "@/components/dashboard/CategoryAverages.vue";
import ExpenseSummary from "@/components/dashboard/ExpenseSummary.vue";
import RateHistoryWidget from "@/components/dashboard/RateHistoryWidget.vue";
import TopExpensesWidget from "@/components/dashboard/TopExpensesWidget.vue";
import AvgCheckWidget from "@/components/dashboard/AvgCheckWidget.vue";
import DayOfWeekWidget from "@/components/dashboard/DayOfWeekWidget.vue";
import MonthCompareWidget from "@/components/dashboard/MonthCompareWidget.vue";
import TagExpensesWidget from "@/components/dashboard/TagExpensesWidget.vue";

const router = useRouter();
const { t } = useI18n();
const chartTheme = useChartTheme();
const { fmt } = useCurrencyFormat();
const { fmtPercent } = useLocaleFormat();
const metaStore = useAppMetaStore();
const dashboardStore = useDashboardStore();
const { settings, categories, currencies } = storeToRefs(metaStore);

function goToSubscriptions() { router.push("/subscriptions"); }


const { setActions, clearActions } = useHeaderActions();
const { alerts, dismiss: dismissAlert, dismissAll: dismissAllAlerts } = useAlerts();
onMounted(async () => {
  setActions([
    {
      id: "configure-widgets",
      icon: Settings2,
      title: t("configure_widgets"),
      onClick: () => (showWidgetConfig.value = !showWidgetConfig.value),
    },
  ]);
  const unlisten = await listen("app:data-changed", () => {
    void dashboardStore.loadPage(true);
    void loadDashboardSubscriptions();
  });
  // Left the page while `listen` was pending: drop it right away instead of leaking it.
  if (unmounted) unlisten();
  else unlistenData = unlisten;
  await metaStore.ensureLoaded();
  await loadDashboardSubscriptions();
});

let unmounted = false;
onUnmounted(() => {
  unmounted = true;
  clearActions();
  void unlistenData?.();
});

const hasSubscriptions = ref(false);
let unlistenData: UnlistenFn | undefined;

// ---- Widget configuration ----
interface WidgetDef { id: string; labelKey: string }

const ALL_WIDGETS: WidgetDef[] = [
  { id: "stats", labelKey: "widget_stats" },
  { id: "overdue", labelKey: "widget_overdue" },
  { id: "upcoming", labelKey: "widget_upcoming" },
  { id: "budget", labelKey: "widget_budget" },
  { id: "savings", labelKey: "widget_savings" },
  { id: "trend", labelKey: "widget_trend" },
  { id: "forecast", labelKey: "widget_forecast" },
  { id: "lifetime", labelKey: "widget_lifetime" },
  { id: "category_avg", labelKey: "widget_category_avg" },
  { id: "charts", labelKey: "widget_charts" },
  { id: "expenses", labelKey: "widget_expenses" },
  { id: "top_expenses", labelKey: "widget_top_expenses" },
  { id: "avg_check", labelKey: "widget_avg_check" },
  { id: "day_of_week", labelKey: "widget_day_of_week" },
  { id: "month_compare", labelKey: "widget_month_compare" },
  { id: "tag_expenses", labelKey: "widget_tag_expenses" },
  { id: "rate_history", labelKey: "widget_rate_history" },
];

const showWidgetConfig = ref(false);

/** Fresh installs start with a focused dashboard; everything else is one toggle away in widget settings. */
const DEFAULT_VISIBLE = new Set(["stats", "overdue", "upcoming", "budget", "trend", "forecast", "expenses", "top_expenses", "charts"]);

/** Widgets that need the full row in the two-column dashboard grid. */
const WIDE_WIDGETS = new Set(["stats", "overdue", "trend", "charts"]);
function widgetSpan(id: string): string {
  return WIDE_WIDGETS.has(id) ? "lg:col-span-2" : "";
}

// Widget order + visibility from settings, with fallback to all visible
const widgetConfig = computed(() => {
  const saved = settings.value?.dashboardWidgets ?? [];
  if (!saved || saved.length === 0) {
    return ALL_WIDGETS.map((w) => ({ id: w.id, visible: DEFAULT_VISIBLE.has(w.id) }));
  }
  // Merge: use saved order, add missing new widgets at end
  const result = saved.map((s) => ({ id: s.id, visible: s.visible }));
  for (const w of ALL_WIDGETS) {
    if (!result.find((r) => r.id === w.id)) {
      result.push({ id: w.id, visible: DEFAULT_VISIBLE.has(w.id) });
    }
  }
  return result.filter((r) => ALL_WIDGETS.some((w) => w.id === r.id));
});

const orderedWidgets = computed(() =>
  widgetConfig.value.map((c) => ({
    ...c,
    def: ALL_WIDGETS.find((w) => w.id === c.id)!,
  })),
);

function isWidgetVisible(id: string): boolean {
  const cfg = widgetConfig.value.find((w) => w.id === id);
  return cfg ? cfg.visible : true;
}

function toggleWidgetVisibility(id: string) {
  const updated = widgetConfig.value.map((w) =>
    w.id === id ? { ...w, visible: !w.visible } : w,
  );
  if (!settings.value) return;
  const next = { ...settings.value, dashboardWidgets: updated };
  settings.value = next;
  void metaStore.updateSettings(next);
}

function moveWidget(id: string, direction: -1 | 1) {
  const arr = [...widgetConfig.value];
  const idx = arr.findIndex((w) => w.id === id);
  if (idx < 0) return;
  const newIdx = idx + direction;
  if (newIdx < 0 || newIdx >= arr.length) return;
  [arr[idx], arr[newIdx]] = [arr[newIdx], arr[idx]];
  if (!settings.value) return;
  const next = { ...settings.value, dashboardWidgets: arr };
  settings.value = next;
  void metaStore.updateSettings(next);
}

// ---- Data computations (loaded from backend DTOs) ----
const summary = computed<DashboardSummaryDto | null>(() => dashboardStore.summary);
const charts = computed<DashboardChartsDto | null>(() => dashboardStore.charts);
const forecastDto = computed<DashboardForecastDto | null>(() => dashboardStore.forecast);
const trends = computed<DashboardTrendsDto | null>(() => dashboardStore.trends);
const upcomingSubscriptions = computed(() => summary.value?.upcomingSubscriptions ?? []);
async function loadDashboardSubscriptions() {
  await dashboardStore.loadPage(true);
  const summaryRes = dashboardStore.summary;
  hasSubscriptions.value = summaryRes?.hasSubscriptions ?? false;
}

const activeCount = computed(() => summary.value?.activeCount ?? 0);
const inactiveCount = computed(() => summary.value?.inactiveCount ?? 0);
const totalMonthly = computed(() => summary.value?.totalMonthly ?? "0");
const totalYearly = computed(() => summary.value?.totalYearly ?? "0");
const avgMonthly = computed(() => summary.value?.avgMonthly ?? "0");
const mostExpensive = computed(() => summary.value?.mostExpensive ?? null);
const amountDueThisMonth = computed(() => summary.value?.amountDueThisMonth ?? "0");
const budget = computed(() => summary.value?.budget ?? "0");

const budgetUsed = computed(() => summary.value?.budgetUsed ?? null);
const budgetLeft = computed(() => summary.value?.budgetLeft ?? null);
const overBudget = computed(() => summary.value?.overBudget ?? null);
const totalSavingsMonthly = computed(() => summary.value?.totalSavingsMonthly ?? "0");

// Charts
const chartColors = ["#3b82f6", "#22c55e", "#f59e0b", "#ef4444", "#8b5cf6", "#06b6d4", "#ec4899", "#f97316", "#14b8a6", "#6366f1"];

const categoryCosts = computed(() => charts.value?.categoryCosts ?? []);
const pmCounts = computed(() => charts.value?.pmCounts ?? []);
const memberCosts = computed(() => charts.value?.memberCosts ?? []);

function splitPieOption(items: { name: string; cost?: MoneyString; count?: number }[]): EChartsCoreOption {
  return {
    animationDuration: 420,
    tooltip: {
      trigger: "item",
      formatter: (params: unknown) => {
        const p = params as { name?: string; value?: number; percent?: number };
        return `${p.name ?? ""}<br/>${p.value ?? ""} (${(p.percent ?? 0).toFixed(1)}%)`;
      },
    },
    legend: {
      bottom: 4,
      left: "center",
      type: "scroll",
      textStyle: { fontSize: 11 },
      itemGap: 10,
    },
    series: [
      {
        type: "pie",
        radius: ["36%", "58%"],
        center: ["50%", "44%"],
        avoidLabelOverlap: true,
        itemStyle: {
          borderRadius: 6,
          borderWidth: 2,
          borderColor: "var(--color-surface, rgba(255,255,255,0.96))",
        },
        label: { show: false },
        emphasis: {
          scale: true,
          itemStyle: { shadowBlur: 14, shadowColor: "rgba(0,0,0,0.14)" },
        },
        data: items.map((c, i) => ({
          name: c.name,
          value: toDisplayNumber(
            parseMoney(
              c.cost ?? (c.count != null ? String(c.count) : "0"),
            ),
          ),
          itemStyle: { color: chartColors[i % chartColors.length] },
        })),
      },
    ],
  };
}

const categoryChartOption = computed(() => splitPieOption(categoryCosts.value));
const pmChartOption = computed(() =>
  splitPieOption(pmCounts.value.map((p) => ({ name: p.name, count: p.count }))),
);
const memberChartOption = computed(() => splitPieOption(memberCosts.value));

const hasCharts = computed(() => categoryCosts.value.length > 1 || pmCounts.value.length > 1 || memberCosts.value.length > 1);

// Analytics
const spendingHistory = computed(() => trends.value?.spendingHistory ?? []);
const forecast = computed(() => forecastDto.value?.forecast ?? null);
const lifetimeCosts = computed(() => trends.value?.lifetimeCosts ?? []);
const categoryAverages = computed(() => trends.value?.categoryAverages ?? []);
const expenseAggregation = computed(
  () =>
    summary.value?.expenseAggregation ?? {
      monthTotal: "0",
      yearTotal: "0",
      recentExpenses: [],
    },
);
const topExpenses = computed(() => trends.value?.topExpenses ?? []);
/** Expense-only users must get the dashboard too, not the "add a subscription" welcome. */
const hasAnyData = computed(() => hasSubscriptions.value || expenseAggregation.value.recentExpenses.length > 0);
const avgExpenseStats = computed(() => trends.value?.avgExpenseStats ?? null);
const dayOfWeekStats = computed(() => trends.value?.dayOfWeekStats ?? []);
const monthComparisonData = computed(() => trends.value?.monthComparisonData ?? null);
const tagExpenseStats = computed(() => trends.value?.tagExpenseStats ?? []);
const hasAnalytics = computed(() => activeCount.value > 0);

const mostExpensiveForCards = computed(() =>
  mostExpensive.value
    ? { name: mostExpensive.value.name, price: mostExpensive.value.price }
    : null,
);

</script>

<template>
  <div class="space-y-4">
    <!-- Empty state -->
    <UiCard v-if="!hasAnyData">
      <UiEmptyState :icon="Wallet" :title="t('welcome_to_subly')" :description="t('get_started_info')">
        <button :class="button({ intent: 'primary' })" @click="goToSubscriptions">
          <ArrowRight :size="15" /> {{ t('go_to_subscriptions') }}
        </button>
      </UiEmptyState>
    </UiCard>

    <template v-if="hasAnyData">
      <!-- Widget config panel -->
      <Transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0 -translate-y-2"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0 -translate-y-2"
      >
        <div v-if="showWidgetConfig" class="bg-surface rounded-xl border border-border shadow-sm p-4">
          <div class="space-y-1">
            <div
              v-for="(w, idx) in orderedWidgets"
              :key="w.id"
              class="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-surface-hover transition-colors"
            >
              <Tooltip :text="t('toggle_visibility')">
                <button
                  @click="toggleWidgetVisibility(w.id)"
                  class="p-1 rounded transition-colors"
                  :class="w.visible ? 'text-primary' : 'text-text-muted opacity-40'"
                >
                  <component :is="w.visible ? Eye : EyeOff" :size="14" />
                </button>
              </Tooltip>
              <span
                class="text-sm flex-1"
                :class="w.visible ? 'text-text-primary' : 'text-text-muted line-through opacity-50'"
              >{{ t(w.def.labelKey) }}</span>
              <Tooltip :text="t('move_up')">
                <button :class="button({ intent: 'ghost', size: 'iconSm' })"
                  @click="moveWidget(w.id, -1)"
                  :disabled="idx === 0"
                ><ChevronUp :size="14" /></button>
              </Tooltip>
              <Tooltip :text="t('move_down')">
                <button :class="button({ intent: 'ghost', size: 'iconSm' })"
                  @click="moveWidget(w.id, 1)"
                  :disabled="idx === orderedWidgets.length - 1"
                ><ChevronDown :size="14" /></button>
              </Tooltip>
            </div>
          </div>
        </div>
      </Transition>

      <!-- Dynamic widgets in configured order: two-column grid, wide widgets span both -->
      <div class="grid grid-cols-1 gap-4 lg:grid-cols-2 items-stretch grid-flow-row-dense [&>*]:h-full">
      <template v-for="w in orderedWidgets" :key="w.id">
        <!-- overdue -->
        <InAppAlerts
          v-if="w.id === 'overdue' && w.visible && alerts.length > 0"
          :class="widgetSpan(w.id)"
          :alerts="alerts"
          @dismiss="dismissAlert"
          @dismiss-all="dismissAllAlerts"
        />

        <!-- upcoming -->
        <UpcomingPayments v-if="w.id === 'upcoming' && w.visible" :class="widgetSpan(w.id)" :subscriptions="upcomingSubscriptions" :fmt="fmt" @navigate="goToSubscriptions" />

        <!-- stats -->
        <StatsCards v-if="w.id === 'stats' && w.visible" :class="widgetSpan(w.id)"
          :activeCount="activeCount"
          :totalMonthly="totalMonthly"
          :totalYearly="totalYearly"
          :amountDue="amountDueThisMonth"
          :avgMonthly="avgMonthly"
          :mostExpensive="mostExpensiveForCards"
          :budgetUsed="budgetUsed"
          :totalSavings="totalSavingsMonthly"
          :fmt="fmt"
        />

        <!-- budget -->
        <UiCard v-if="w.id === 'budget' && w.visible && moneyGt(budget, '0')" :class="widgetSpan(w.id)" :title="t('your_budget')" :icon="Wallet" tone="warning">
          <div class="space-y-3">
            <div class="w-full bg-surface-hover rounded-full h-2">
              <div class="h-full rounded-full transition-all duration-500" :class="moneyGt(budgetUsed ?? '0', '100') ? 'bg-danger' : 'bg-primary'" :style="{ width: Math.min(toDisplayNumber(parseMoney(budgetUsed ?? '0')), 100) + '%' }" />
            </div>
            <div class="grid grid-cols-2 gap-3 sm:grid-cols-4 text-[13px]">
              <div><p class="text-xs text-text-muted">{{ t('budget') }}</p><p class="font-semibold text-text-primary tabular-nums">{{ fmt(budget) }}</p></div>
              <div><p class="text-xs text-text-muted">{{ t('budget_used') }}</p><p class="font-semibold text-text-primary tabular-nums">{{ fmtPercent(toDisplayNumber(parseMoney(budgetUsed ?? '0'))) }}</p></div>
              <div><p class="text-xs text-text-muted">{{ t('budget_remaining') }}</p><p class="font-semibold text-text-primary tabular-nums">{{ fmt(budgetLeft ?? '0') }}</p></div>
              <div v-if="overBudget"><p class="text-xs text-danger">{{ t('over_budget') }}</p><p class="font-semibold text-danger tabular-nums">{{ fmt(overBudget) }}</p></div>
            </div>
          </div>
        </UiCard>

        <!-- savings -->
        <UiCard v-if="w.id === 'savings' && w.visible && inactiveCount > 0" :class="widgetSpan(w.id)" :title="t('your_savings')" :icon="PiggyBank" tone="success">
          <div class="grid grid-cols-3 gap-3">
            <div><p :class="typo.statLabel()">{{ t('inactive_subscriptions') }}</p><p :class="statValue()">{{ inactiveCount }}</p></div>
            <div><p :class="typo.statLabel()">{{ t('monthly_savings') }}</p><p :class="statValue({ tone: 'green' })">{{ fmt(totalSavingsMonthly) }}</p></div>
            <div><p :class="typo.statLabel()">{{ t('yearly_savings') }}</p><p :class="statValue({ tone: 'green' })">{{ fmt(moneyFromDecimal(mul(totalSavingsMonthly, '12'))) }}</p></div>
          </div>
        </UiCard>

        <!-- trend -->
        <SpendingTrend v-if="w.id === 'trend' && w.visible && hasAnalytics" :class="widgetSpan(w.id)" :data="spendingHistory" :fmt="fmt" />

        <!-- forecast -->
        <ForecastCard v-if="w.id === 'forecast' && w.visible && hasAnalytics && forecast" :class="widgetSpan(w.id)" :forecast="forecast" :fmt="fmt" />

        <!-- lifetime -->
        <LifetimeCosts v-if="w.id === 'lifetime' && w.visible && lifetimeCosts.length > 0" :class="widgetSpan(w.id)" :costs="lifetimeCosts" :fmt="fmt" />

        <!-- category_avg -->
        <CategoryAverages v-if="w.id === 'category_avg' && w.visible && categoryAverages.length > 0" :class="widgetSpan(w.id)" :averages="categoryAverages" :fmt="fmt" :categories="categories" />

        <!-- expenses -->
        <ExpenseSummary
          v-if="w.id === 'expenses' && w.visible" :class="widgetSpan(w.id)"
          :categories="categories"
          :currencies="currencies"
          :mainCurrencyId="settings?.mainCurrencyId || 'cur-2'"
          :agg="expenseAggregation"
        />

        <!-- top expenses -->
        <TopExpensesWidget
          v-if="w.id === 'top_expenses' && w.visible" :class="widgetSpan(w.id)"
          :categories="categories"
          :items="topExpenses"
        />

        <!-- avg check -->
        <AvgCheckWidget v-if="w.id === 'avg_check' && w.visible" :class="widgetSpan(w.id)" :stats="avgExpenseStats" />

        <!-- day of week -->
        <DayOfWeekWidget v-if="w.id === 'day_of_week' && w.visible" :class="widgetSpan(w.id)" :raw-stats="dayOfWeekStats" />

        <!-- month compare -->
        <MonthCompareWidget v-if="w.id === 'month_compare' && w.visible" :class="widgetSpan(w.id)" :data="monthComparisonData" />

        <!-- tag expenses -->
        <TagExpensesWidget v-if="w.id === 'tag_expenses' && w.visible" :class="widgetSpan(w.id)" :stats="tagExpenseStats" />

        <!-- rate history -->
        <RateHistoryWidget v-if="w.id === 'rate_history' && w.visible" :class="widgetSpan(w.id)" :settings="settings" :currencies="currencies" />

        <!-- charts -->
        <div
          v-if="w.id === 'charts' && w.visible && hasCharts" :class="widgetSpan(w.id)"
          class="space-y-2"
        >
          <h2 class="px-1 text-xs font-semibold uppercase tracking-wide text-text-muted">{{ t('split_views') }}</h2>
          <div class="grid gap-4 grid-cols-[repeat(auto-fit,minmax(260px,1fr))]">
            <UiCard v-if="categoryCosts.length > 1" :title="t('category_split')">
              <VChart class="h-48 w-full min-h-48" :theme="chartTheme" :option="categoryChartOption" autoresize />
            </UiCard>
            <UiCard v-if="pmCounts.length > 1" :title="t('payment_method_split')">
              <VChart class="h-48 w-full min-h-48" :theme="chartTheme" :option="pmChartOption" autoresize />
            </UiCard>
            <UiCard v-if="memberCosts.length > 1" :title="t('household_split')">
              <VChart class="h-48 w-full min-h-48" :theme="chartTheme" :option="memberChartOption" autoresize />
            </UiCard>
          </div>
        </div>
      </template>
      </div>
    </template>
  </div>
</template>
