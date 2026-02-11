<script setup lang="ts">
import { ref, computed } from "vue";
import { useRouter } from "vue-router";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { getPricePerMonth, isOverdue, isUpcoming } from "@/services/calculations";
import { getMonthlySpendingHistory, getForecast, getMonthComparison, getLifetimeCosts, getCategoryAverages } from "@/services/analytics";
import { Doughnut } from "vue-chartjs";
import { Chart as ChartJS, ArcElement, Tooltip, Legend } from "chart.js";
import { Wallet, ArrowRight, Settings2, Eye, EyeOff, ChevronUp, ChevronDown } from "lucide-vue-next";

import OverdueAlert from "@/components/dashboard/OverdueAlert.vue";
import UpcomingPayments from "@/components/dashboard/UpcomingPayments.vue";
import StatsCards from "@/components/dashboard/StatsCards.vue";
import SpendingTrend from "@/components/dashboard/SpendingTrend.vue";
import ForecastCard from "@/components/dashboard/ForecastCard.vue";
import LifetimeCosts from "@/components/dashboard/LifetimeCosts.vue";
import CategoryAverages from "@/components/dashboard/CategoryAverages.vue";
import ExpenseSummary from "@/components/dashboard/ExpenseSummary.vue";

ChartJS.register(ArcElement, Tooltip, Legend);

const router = useRouter();
const store = useAppStore();
const { t } = useI18n();
const { fmt, toMainCurrency } = useCurrencyFormat();

function goToSubscriptions() { router.push("/subscriptions"); }

const hasSubscriptions = computed(() => store.state.subscriptions.length > 0);

// ---- Widget configuration ----
interface WidgetDef { id: string; labelKey: string }

const ALL_WIDGETS: WidgetDef[] = [
  { id: "overdue", labelKey: "widget_overdue" },
  { id: "upcoming", labelKey: "widget_upcoming" },
  { id: "stats", labelKey: "widget_stats" },
  { id: "budget", labelKey: "widget_budget" },
  { id: "savings", labelKey: "widget_savings" },
  { id: "trend", labelKey: "widget_trend" },
  { id: "forecast", labelKey: "widget_forecast" },
  { id: "lifetime", labelKey: "widget_lifetime" },
  { id: "category_avg", labelKey: "widget_category_avg" },
  { id: "charts", labelKey: "widget_charts" },
  { id: "expenses", labelKey: "widget_expenses" },
];

const showWidgetConfig = ref(false);

// Widget order + visibility from settings, with fallback to all visible
const widgetConfig = computed(() => {
  const saved = store.state.settings.dashboardWidgets;
  if (!saved || saved.length === 0) {
    return ALL_WIDGETS.map((w) => ({ id: w.id, visible: true }));
  }
  // Merge: use saved order, add missing new widgets at end
  const result = saved.map((s) => ({ id: s.id, visible: s.visible }));
  for (const w of ALL_WIDGETS) {
    if (!result.find((r) => r.id === w.id)) {
      result.push({ id: w.id, visible: true });
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
  store.updateSettings({ dashboardWidgets: updated });
}

function moveWidget(id: string, direction: -1 | 1) {
  const arr = [...widgetConfig.value];
  const idx = arr.findIndex((w) => w.id === id);
  if (idx < 0) return;
  const newIdx = idx + direction;
  if (newIdx < 0 || newIdx >= arr.length) return;
  [arr[idx], arr[newIdx]] = [arr[newIdx], arr[idx]];
  store.updateSettings({ dashboardWidgets: arr });
}

// ---- Data computations ----
const overdueSubscriptions = computed(() =>
  store.state.subscriptions.filter((s) => isOverdue(s))
    .sort((a, b) => new Date(a.nextPayment).getTime() - new Date(b.nextPayment).getTime())
);

const upcomingSubscriptions = computed(() =>
  store.state.subscriptions.filter((s) => isUpcoming(s, 30))
    .sort((a, b) => new Date(a.nextPayment).getTime() - new Date(b.nextPayment).getTime())
    .slice(0, 5)
);

const activeSubs = computed(() => store.activeSubscriptions.value);
const inactiveSubs = computed(() => store.inactiveSubscriptions.value);

const totalMonthly = computed(() =>
  activeSubs.value.reduce((sum, s) => sum + getPricePerMonth(s.cycle, s.frequency, toMainCurrency(s.price, s.currencyId)), 0)
);
const totalYearly = computed(() => totalMonthly.value * 12);
const avgMonthly = computed(() => activeSubs.value.length > 0 ? totalMonthly.value / activeSubs.value.length : 0);

const mostExpensive = computed(() => {
  if (activeSubs.value.length === 0) return null;
  let max = { name: "", price: 0 };
  for (const s of activeSubs.value) {
    const monthly = getPricePerMonth(s.cycle, s.frequency, toMainCurrency(s.price, s.currencyId));
    if (monthly > max.price) max = { name: s.name, price: monthly };
  }
  return max;
});

const amountDueThisMonth = computed(() => {
  const now = new Date();
  const endOfMonth = new Date(now.getFullYear(), now.getMonth() + 1, 0);
  return activeSubs.value.reduce((sum, s) => {
    const next = new Date(s.nextPayment);
    return (next >= now && next <= endOfMonth) ? sum + toMainCurrency(s.price, s.currencyId) : sum;
  }, 0);
});

const budget = computed(() => store.state.settings.budget);

// Monthly expenses (one-time)
const now2 = new Date();
const currentMonthStr = `${now2.getFullYear()}-${String(now2.getMonth() + 1).padStart(2, "0")}`;
const monthlyExpensesTotal = computed(() =>
  store.state.expenses
    .filter((e) => e.date.startsWith(currentMonthStr))
    .reduce((s, e) => s + toMainCurrency(e.amount, e.currencyId), 0)
);

const totalSpending = computed(() => totalMonthly.value + monthlyExpensesTotal.value);
const budgetUsed = computed(() => budget.value > 0 ? Math.min(100, (totalSpending.value / budget.value) * 100) : null);
const budgetLeft = computed(() => budget.value > 0 ? Math.max(0, budget.value - totalSpending.value) : null);
const overBudget = computed(() => budget.value > 0 && totalSpending.value > budget.value ? totalSpending.value - budget.value : null);

const totalSavingsMonthly = computed(() =>
  inactiveSubs.value.reduce((sum, s) => sum + getPricePerMonth(s.cycle, s.frequency, toMainCurrency(s.price, s.currencyId)), 0)
);

// Charts
const chartColors = ["#3b82f6", "#22c55e", "#f59e0b", "#ef4444", "#8b5cf6", "#06b6d4", "#ec4899", "#f97316", "#14b8a6", "#6366f1"];
const chartOptions = { responsive: true, plugins: { legend: { position: "bottom" as const } } };

function buildCostMap(keyFn: (s: typeof activeSubs.value[0]) => { id: string; name: string }) {
  const map: Record<string, { name: string; cost: number }> = {};
  for (const s of activeSubs.value) {
    const { id, name } = keyFn(s);
    if (!map[id]) map[id] = { name, cost: 0 };
    map[id].cost += getPricePerMonth(s.cycle, s.frequency, toMainCurrency(s.price, s.currencyId));
  }
  return Object.values(map).filter((x) => x.cost > 0);
}

const categoryCosts = computed(() => buildCostMap((s) => {
  const cat = store.state.categories.find((c) => c.id === s.categoryId);
  return { id: s.categoryId, name: cat?.name || "Other" };
}));

const pmCounts = computed(() => {
  const map: Record<string, { name: string; count: number }> = {};
  for (const s of activeSubs.value) {
    const pm = store.state.paymentMethods.find((p) => p.id === s.paymentMethodId);
    if (!map[s.paymentMethodId]) map[s.paymentMethodId] = { name: pm?.name || "Other", count: 0 };
    map[s.paymentMethodId].count++;
  }
  return Object.values(map).filter((x) => x.count > 0);
});

const memberCosts = computed(() => buildCostMap((s) => {
  const m = store.state.household.find((h) => h.id === s.payerUserId);
  return { id: s.payerUserId, name: m?.name || "Other" };
}));

function toChartData(items: { name: string; cost?: number; count?: number }[]) {
  return {
    labels: items.map((c) => c.name),
    datasets: [{ data: items.map((c) => Math.round((c.cost ?? c.count ?? 0) * 100) / 100), backgroundColor: chartColors.slice(0, items.length) }],
  };
}

const categoryChartData = computed(() => toChartData(categoryCosts.value));
const pmChartData = computed(() => toChartData(pmCounts.value.map((p) => ({ name: p.name, cost: p.count }))));
const memberChartData = computed(() => toChartData(memberCosts.value));

const hasCharts = computed(() => categoryCosts.value.length > 1 || pmCounts.value.length > 1 || memberCosts.value.length > 1);

// Analytics
const spendingHistory = computed(() => getMonthlySpendingHistory(store.state.subscriptions, toMainCurrency, 12, store.state.expenses));
const forecast = computed(() => getForecast(store.state.subscriptions, toMainCurrency));
const monthComparison = computed(() => getMonthComparison(store.state.subscriptions, toMainCurrency));
const lifetimeCosts = computed(() => getLifetimeCosts(store.state.subscriptions, toMainCurrency));
const categoryAverages = computed(() => getCategoryAverages(store.state.subscriptions, store.state.categories, toMainCurrency, store.state.expenses));
const hasAnalytics = computed(() => activeSubs.value.length > 0);
</script>

<template>
  <div class="space-y-4 sm:space-y-6 max-w-5xl mx-auto">
    <!-- Empty state -->
    <div v-if="!hasSubscriptions" class="text-center py-16">
      <div class="w-20 h-20 mx-auto mb-4 rounded-full bg-[var(--color-primary-light)] flex items-center justify-center">
        <Wallet :size="36" class="text-[var(--color-primary)]" />
      </div>
      <h2 class="text-xl font-bold text-[var(--color-text-primary)] mb-2">{{ t('welcome_to_subly') }}</h2>
      <p class="text-sm text-[var(--color-text-muted)] mb-6 max-w-md mx-auto">{{ t('get_started_info') }}</p>
      <button @click="goToSubscriptions" class="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg bg-[var(--color-primary)] text-white font-medium hover:bg-[var(--color-primary-hover)] shadow-sm">
        <ArrowRight :size="18" /> {{ t('go_to_subscriptions') }}
      </button>
    </div>

    <template v-if="hasSubscriptions">
      <!-- Widget config button -->
      <div class="flex justify-end">
        <button
          @click="showWidgetConfig = !showWidgetConfig"
          class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border text-xs font-medium transition-colors"
          :class="showWidgetConfig ? 'border-[var(--color-primary)] text-[var(--color-primary)] bg-[var(--color-primary-light)]' : 'border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]'"
        >
          <Settings2 :size="14" />
          {{ t('configure_widgets') }}
        </button>
      </div>

      <!-- Widget config panel -->
      <Transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0 -translate-y-2"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0 -translate-y-2"
      >
        <div v-if="showWidgetConfig" class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-4">
          <div class="space-y-1">
            <div
              v-for="(w, idx) in orderedWidgets"
              :key="w.id"
              class="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-[var(--color-surface-hover)] transition-colors"
            >
              <button
                @click="toggleWidgetVisibility(w.id)"
                class="p-1 rounded transition-colors"
                :class="w.visible ? 'text-[var(--color-primary)]' : 'text-[var(--color-text-muted)] opacity-40'"
              >
                <component :is="w.visible ? Eye : EyeOff" :size="14" />
              </button>
              <span
                class="text-sm flex-1"
                :class="w.visible ? 'text-[var(--color-text-primary)]' : 'text-[var(--color-text-muted)] line-through opacity-50'"
              >{{ t(w.def.labelKey) }}</span>
              <button
                @click="moveWidget(w.id, -1)"
                :disabled="idx === 0"
                class="p-1 rounded text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] disabled:opacity-20 transition-colors"
              ><ChevronUp :size="14" /></button>
              <button
                @click="moveWidget(w.id, 1)"
                :disabled="idx === orderedWidgets.length - 1"
                class="p-1 rounded text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] disabled:opacity-20 transition-colors"
              ><ChevronDown :size="14" /></button>
            </div>
          </div>
        </div>
      </Transition>

      <!-- Dynamic widgets in configured order -->
      <template v-for="w in orderedWidgets" :key="w.id">
        <!-- overdue -->
        <OverdueAlert v-if="w.id === 'overdue' && w.visible" :subscriptions="overdueSubscriptions" :fmt="fmt" />

        <!-- upcoming -->
        <UpcomingPayments v-if="w.id === 'upcoming' && w.visible" :subscriptions="upcomingSubscriptions" :fmt="fmt" @navigate="goToSubscriptions" />

        <!-- stats -->
        <StatsCards v-if="w.id === 'stats' && w.visible"
          :activeCount="activeSubs.length" :totalMonthly="totalMonthly" :totalYearly="totalYearly"
          :amountDue="amountDueThisMonth" :avgMonthly="avgMonthly" :mostExpensive="mostExpensive"
          :budgetUsed="budgetUsed" :totalSavings="totalSavingsMonthly" :fmt="fmt"
        />

        <!-- budget -->
        <div v-if="w.id === 'budget' && w.visible && budget > 0" class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
          <h2 class="text-lg font-semibold text-[var(--color-text-primary)] mb-4">{{ t('your_budget') }}</h2>
          <div class="space-y-3">
            <div class="w-full bg-[var(--color-surface-hover)] rounded-full h-3">
              <div class="h-3 rounded-full transition-all duration-500" :class="(budgetUsed || 0) > 100 ? 'bg-red-500' : 'bg-[var(--color-primary)]'" :style="{ width: Math.min(budgetUsed || 0, 100) + '%' }" />
            </div>
            <div class="grid grid-cols-2 gap-3 sm:grid-cols-4 sm:gap-4 text-sm">
              <div><p class="text-[var(--color-text-muted)]">{{ t('budget') }}</p><p class="font-semibold text-[var(--color-text-primary)]">{{ fmt(budget) }}</p></div>
              <div><p class="text-[var(--color-text-muted)]">{{ t('budget_used') }}</p><p class="font-semibold text-[var(--color-text-primary)]">{{ (budgetUsed || 0).toFixed(1) }}%</p></div>
              <div><p class="text-[var(--color-text-muted)]">{{ t('budget_remaining') }}</p><p class="font-semibold text-[var(--color-text-primary)]">{{ fmt(budgetLeft || 0) }}</p></div>
              <div v-if="overBudget"><p class="text-red-500">{{ t('over_budget') }}</p><p class="font-semibold text-red-500">{{ fmt(overBudget) }}</p></div>
            </div>
          </div>
        </div>

        <!-- savings -->
        <div v-if="w.id === 'savings' && w.visible && inactiveSubs.length > 0" class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
          <h2 class="text-lg font-semibold text-[var(--color-text-primary)] mb-3">{{ t('your_savings') }}</h2>
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div><p class="text-sm text-[var(--color-text-muted)]">{{ t('inactive_subscriptions') }}</p><p class="text-xl font-bold text-[var(--color-text-primary)]">{{ inactiveSubs.length }}</p></div>
            <div><p class="text-sm text-[var(--color-text-muted)]">{{ t('monthly_savings') }}</p><p class="text-xl font-bold text-green-600">{{ fmt(totalSavingsMonthly) }}</p></div>
            <div><p class="text-sm text-[var(--color-text-muted)]">{{ t('yearly_savings') }}</p><p class="text-xl font-bold text-green-600">{{ fmt(totalSavingsMonthly * 12) }}</p></div>
          </div>
        </div>

        <!-- trend -->
        <SpendingTrend v-if="w.id === 'trend' && w.visible && hasAnalytics" :data="spendingHistory" :fmt="fmt" />

        <!-- forecast -->
        <ForecastCard v-if="w.id === 'forecast' && w.visible && hasAnalytics" :forecast="forecast" :comparison="monthComparison" :fmt="fmt" />

        <!-- lifetime -->
        <LifetimeCosts v-if="w.id === 'lifetime' && w.visible && lifetimeCosts.length > 0" :costs="lifetimeCosts" :fmt="fmt" />

        <!-- category_avg -->
        <CategoryAverages v-if="w.id === 'category_avg' && w.visible && categoryAverages.length > 0" :averages="categoryAverages" :fmt="fmt" />

        <!-- expenses -->
        <ExpenseSummary v-if="w.id === 'expenses' && w.visible" />

        <!-- charts -->
        <div v-if="w.id === 'charts' && w.visible && hasCharts" class="space-y-4">
          <h2 class="text-lg font-semibold text-[var(--color-text-primary)]">{{ t('split_views') }}</h2>
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-5">
            <div v-if="categoryCosts.length > 1" class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
              <h3 class="text-sm font-semibold text-[var(--color-text-primary)] mb-4">{{ t('category_split') }}</h3>
              <Doughnut :data="categoryChartData" :options="chartOptions" />
            </div>
            <div v-if="pmCounts.length > 1" class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
              <h3 class="text-sm font-semibold text-[var(--color-text-primary)] mb-4">{{ t('payment_method_split') }}</h3>
              <Doughnut :data="pmChartData" :options="chartOptions" />
            </div>
            <div v-if="memberCosts.length > 1" class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
              <h3 class="text-sm font-semibold text-[var(--color-text-primary)] mb-4">{{ t('household_split') }}</h3>
              <Doughnut :data="memberChartData" :options="chartOptions" />
            </div>
          </div>
        </div>
      </template>
    </template>
  </div>
</template>
