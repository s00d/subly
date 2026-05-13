import { invokeCommand } from "@/services/commandClient";
export type {
  MonthlySpending,
  MonthComparisonStyle,
  MonthComparison,
  ForecastResult,
  LifetimeCost,
  CategoryAverage,
  DashboardSummaryDto,
  DashboardChartsDto,
  DashboardForecastDto,
  DashboardTrendsDto,
  RateHistoryPoint,
} from "@/types/dashboardIpc";

export async function getDashboardSummary() {
  return invokeCommand("get_dashboard_summary");
}

export async function getDashboardCharts() {
  return invokeCommand("get_dashboard_charts");
}

export async function getDashboardForecast() {
  return invokeCommand("get_dashboard_forecast");
}

export async function getDashboardTrends() {
  return invokeCommand("get_dashboard_trends");
}

export async function getRateHistoryWidget(
  targetIds: string[],
  days: number,
) {
  return invokeCommand("get_rate_history_widget", { targetIds, days });
}
