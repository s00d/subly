import type { Expense, Subscription } from "@/schemas/appData";

/** IPC DTOs for dashboard commands — neutral module so `commandClient` avoids importing `dashboardClient` (cycle). */

export interface MonthlySpending {
  label: string;
  year: number;
  month: number;
  amount: number;
}

export type MonthComparisonStyle = "mtd" | "completedPair" | "fullMonths";

export interface MonthComparison {
  currentMonth: string;
  previousMonth: string;
  current: number;
  previous: number;
  diff: number;
  diffPercent: number;
  usedFullMonthFallback?: boolean;
  comparisonStyle?: MonthComparisonStyle;
}

export interface ForecastResult {
  nextMonth: number;
  nextQuarter: number;
  nextMonthLabel: string;
  quarterLabels: string[];
}

export interface LifetimeCost {
  subscriptionId: string;
  name: string;
  logo: string;
  startDate: string;
  monthsActive: number;
  totalPaid: number;
  monthlyEquivalent: number;
}

export interface CategoryAverage {
  categoryId: string;
  categoryName: string;
  totalMonthly: number;
  subscriptionCount: number;
  averageMonthly: number;
}

export interface DashboardSummaryDto {
  hasSubscriptions: boolean;
  activeCount: number;
  inactiveCount: number;
  totalMonthly: number;
  totalYearly: number;
  avgMonthly: number;
  amountDueThisMonth: number;
  mostExpensive: { name: string; price: number } | null;
  budget: number;
  budgetUsed: number | null;
  budgetLeft: number | null;
  overBudget: number | null;
  totalSavingsMonthly: number;
  monthlyExpensesTotal: number;
  overdueSubscriptions: Subscription[];
  upcomingSubscriptions: Subscription[];
  expenseAggregation: {
    monthTotal: number;
    yearTotal: number;
    recentExpenses: Expense[];
  };
}

export interface DashboardChartsDto {
  categoryCosts: Array<{ id: string; name: string; cost: number }>;
  pmCounts: Array<{ id: string; name: string; count: number }>;
  memberCosts: Array<{ id: string; name: string; cost: number }>;
}

export interface DashboardForecastDto {
  forecast: ForecastResult;
  monthComparison: MonthComparison;
}

export interface DashboardTrendsDto {
  spendingHistory?: MonthlySpending[];
  lifetimeCosts?: LifetimeCost[];
  categoryAverages?: CategoryAverage[];
  topExpenses: Expense[];
  avgExpenseStats: { avgAmount: number; count: number; total: number };
  dayOfWeekStats: Array<{ dayOfWeek: number; total: number; count: number }>;
  monthComparisonData: {
    currentTotal: number;
    currentCount: number;
    previousTotal: number;
    previousCount: number;
    usedFullMonthFallback?: boolean;
    comparisonStyle?: MonthComparisonStyle;
    currentMonthLabel?: string;
    previousMonthLabel?: string;
  };
  tagExpenseStats: Array<{ tag: string; total: number }>;
}

export interface RateHistoryPoint {
  rate: number;
  recordedAt: string;
}
