import { defineStore } from "pinia";
import { ref } from "vue";
import type {
  DashboardChartsDto,
  DashboardForecastDto,
  DashboardSummaryDto,
  DashboardTrendsDto,
} from "@/services/dashboardClient";
import {
  getDashboardCharts,
  getDashboardForecast,
  getDashboardSummary,
  getDashboardTrends,
} from "@/services/dashboardClient";

const DASHBOARD_CACHE_TTL_MS = 30_000;

export const useDashboardStore = defineStore("dashboard", () => {
  const summary = ref<DashboardSummaryDto | null>(null);
  const charts = ref<DashboardChartsDto | null>(null);
  const forecast = ref<DashboardForecastDto | null>(null);
  const trends = ref<DashboardTrendsDto | null>(null);
  const loading = ref(false);
  const lastLoadedAt = ref(0);

  async function loadPage(force = false) {
    if (!force && Date.now() - lastLoadedAt.value < DASHBOARD_CACHE_TTL_MS) return;
    loading.value = true;
    try {
      const [s, c, f, t] = await Promise.all([
        getDashboardSummary(),
        getDashboardCharts(),
        getDashboardForecast(),
        getDashboardTrends(),
      ]);
      summary.value = s;
      charts.value = c;
      forecast.value = f;
      trends.value = t;
      lastLoadedAt.value = Date.now();
    } finally {
      loading.value = false;
    }
  }

  return { summary, charts, forecast, trends, loading, loadPage };
});

