import type { Expense, MoneyString, Subscription } from "@/schemas/appData";

/** IPC DTOs for dashboard commands — neutral module so `commandClient` avoids importing `dashboardClient` (cycle). */

export interface MonthlySpending {
  label: string;
  year: number;
  month: number;
  amount: MoneyString;
}

export type MonthComparisonStyle = "mtd" | "completedPair" | "fullMonths";

export interface MonthComparison {
  currentMonth: string;
  previousMonth: string;
  current: MoneyString;
  previous: MoneyString;
  diff: MoneyString;
  diffPercent: MoneyString;
  usedFullMonthFallback?: boolean;
  comparisonStyle?: MonthComparisonStyle;
}

export interface ForecastResult {
  nextMonth: MoneyString;
  nextQuarter: MoneyString;
  nextMonthLabel: string;
  quarterLabels: string[];
}

export interface LifetimeCost {
  subscriptionId: string;
  name: string;
  logo: string;
  startDate: string;
  monthsActive: number;
  totalPaid: MoneyString;
  monthlyEquivalent: MoneyString;
}

export interface CategoryAverage {
  categoryId: string;
  categoryName: string;
  totalMonthly: MoneyString;
  subscriptionCount: number;
  averageMonthly: MoneyString;
}

export interface DashboardSummaryDto {
  hasSubscriptions: boolean;
  activeCount: number;
  inactiveCount: number;
  totalMonthly: MoneyString;
  totalYearly: MoneyString;
  avgMonthly: MoneyString;
  amountDueThisMonth: MoneyString;
  mostExpensive: { name: string; price: MoneyString } | null;
  budget: MoneyString;
  budgetUsed: MoneyString | null;
  budgetLeft: MoneyString | null;
  overBudget: MoneyString | null;
  totalSavingsMonthly: MoneyString;
  monthlyExpensesTotal: MoneyString;
  overdueSubscriptions: Subscription[];
  upcomingSubscriptions: Subscription[];
  expenseAggregation: {
    monthTotal: MoneyString;
    yearTotal: MoneyString;
    recentExpenses: Expense[];
  };
}

export interface DashboardChartsDto {
  categoryCosts: Array<{ id: string; name: string; cost: MoneyString }>;
  pmCounts: Array<{ id: string; name: string; count: number }>;
  memberCosts: Array<{ id: string; name: string; cost: MoneyString }>;
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
  avgExpenseStats: { avgAmount: MoneyString; count: number; total: MoneyString };
  dayOfWeekStats: Array<{ dayOfWeek: number; total: MoneyString; count: number }>;
  monthComparisonData: {
    currentTotal: MoneyString;
    currentCount: number;
    previousTotal: MoneyString;
    previousCount: number;
    usedFullMonthFallback?: boolean;
    comparisonStyle?: MonthComparisonStyle;
    currentMonthLabel?: string;
    previousMonthLabel?: string;
  };
  tagExpenseStats: Array<{ tag: string; total: MoneyString }>;
}

export interface RateHistoryPoint {
  rate: MoneyString;
  recordedAt: string;
}
