import type { Subscription, Expense, Category, CycleType } from "@/schemas/appData";
import { getPricePerMonth, getPaymentDatesInMonth } from "@/services/calculations";

// =============================================
// Types
// =============================================

export interface MonthlySpending {
  label: string;        // "Jan 2026"
  year: number;
  month: number;        // 0-11
  amount: number;       // in main currency
}

export interface MonthComparison {
  currentMonth: string;
  previousMonth: string;
  current: number;
  previous: number;
  diff: number;
  diffPercent: number;
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

// =============================================
// Monthly spending history (past 12 months)
// =============================================

const shortMonthNames = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/**
 * Calculate monthly spending for the past N months.
 * Uses actual payment dates per subscription in each month.
 */
export function getMonthlySpendingHistory(
  subscriptions: Subscription[],
  convertToMain: (price: number, currencyId: string) => number,
  months: number = 12,
  expenses: Expense[] = [],
): MonthlySpending[] {
  const now = new Date();
  const result: MonthlySpending[] = [];

  for (let i = months - 1; i >= 0; i--) {
    const d = new Date(now.getFullYear(), now.getMonth() - i, 1);
    const year = d.getFullYear();
    const month = d.getMonth();

    let amount = 0;
    const activeSubs = subscriptions.filter((s) => {
      if (s.inactive) {
        // Was it active during this month? Check if start date is before end of this month
        // and if cancellation is after start of this month (or no cancellation)
        const start = new Date(s.startDate);
        const endOfMonth = new Date(year, month + 1, 0);
        if (start > endOfMonth) return false;
        if (s.cancellationDate) {
          const cancel = new Date(s.cancellationDate);
          const startOfMonth = new Date(year, month, 1);
          if (cancel < startOfMonth) return false;
        }
        return true;
      }
      // Active — check if subscription existed in this month
      const start = new Date(s.startDate);
      const endOfMonth = new Date(year, month + 1, 0);
      return start <= endOfMonth;
    });

    for (const sub of activeSubs) {
      const dates = getPaymentDatesInMonth(sub, year, month);
      if (dates.length > 0) {
        amount += dates.length * convertToMain(sub.price, sub.currencyId);
      }
    }

    // Add one-time expenses for this month
    const monthStr = `${year}-${String(month + 1).padStart(2, "0")}`;
    for (const exp of expenses) {
      if (exp.date.startsWith(monthStr)) {
        amount += convertToMain(exp.amount, exp.currencyId);
      }
    }

    result.push({
      label: `${shortMonthNames[month]} ${year}`,
      year,
      month,
      amount: Math.round(amount * 100) / 100,
    });
  }

  return result;
}

// =============================================
// Forecast
// =============================================

/**
 * Forecast spending for next month and next quarter based on active subscriptions.
 */
export function getForecast(
  subscriptions: Subscription[],
  convertToMain: (price: number, currencyId: string) => number,
): ForecastResult {
  const now = new Date();
  const activeSubs = subscriptions.filter((s) => !s.inactive);

  let nextMonth = 0;
  let nextQuarter = 0;
  const quarterLabels: string[] = [];

  for (let offset = 1; offset <= 3; offset++) {
    const d = new Date(now.getFullYear(), now.getMonth() + offset, 1);
    const year = d.getFullYear();
    const month = d.getMonth();
    quarterLabels.push(`${shortMonthNames[month]} ${year}`);

    let monthTotal = 0;
    for (const sub of activeSubs) {
      const dates = getPaymentDatesInMonth(sub, year, month);
      monthTotal += dates.length * convertToMain(sub.price, sub.currencyId);
    }

    if (offset === 1) nextMonth = monthTotal;
    nextQuarter += monthTotal;
  }

  const nextMonthDate = new Date(now.getFullYear(), now.getMonth() + 1, 1);

  return {
    nextMonth: Math.round(nextMonth * 100) / 100,
    nextQuarter: Math.round(nextQuarter * 100) / 100,
    nextMonthLabel: `${shortMonthNames[nextMonthDate.getMonth()]} ${nextMonthDate.getFullYear()}`,
    quarterLabels,
  };
}

// =============================================
// Month-to-month comparison
// =============================================

/**
 * Compare current month spending to previous month.
 */
export function getMonthComparison(
  subscriptions: Subscription[],
  convertToMain: (price: number, currencyId: string) => number,
): MonthComparison {
  const history = getMonthlySpendingHistory(subscriptions, convertToMain, 2);

  const prev = history[0];
  const curr = history[1];

  const diff = curr.amount - prev.amount;
  const diffPercent = prev.amount > 0 ? (diff / prev.amount) * 100 : (curr.amount > 0 ? 100 : 0);

  return {
    currentMonth: curr.label,
    previousMonth: prev.label,
    current: curr.amount,
    previous: prev.amount,
    diff: Math.round(diff * 100) / 100,
    diffPercent: Math.round(diffPercent * 10) / 10,
  };
}

// =============================================
// Lifetime cost per subscription
// =============================================

/**
 * Calculate total estimated cost since subscription start date.
 */
export function getLifetimeCosts(
  subscriptions: Subscription[],
  convertToMain: (price: number, currencyId: string) => number,
): LifetimeCost[] {
  const now = new Date();
  now.setHours(0, 0, 0, 0);

  return subscriptions
    .map((sub) => {
      const start = new Date(sub.startDate);
      start.setHours(0, 0, 0, 0);
      const end = sub.cancellationDate ? new Date(sub.cancellationDate) : now;

      const diffMs = Math.max(0, end.getTime() - start.getTime());
      const diffDays = diffMs / (1000 * 60 * 60 * 24);

      // Calculate approximate number of payments
      let cycleDays: number;
      switch (sub.cycle as CycleType) {
        case 1: cycleDays = sub.frequency; break;
        case 2: cycleDays = 7 * sub.frequency; break;
        case 3: cycleDays = 30.44 * sub.frequency; break;
        case 4: cycleDays = 365.25 * sub.frequency; break;
        default: cycleDays = 30.44;
      }

      const payments = Math.floor(diffDays / cycleDays) + 1;
      const totalPaid = payments * convertToMain(sub.price, sub.currencyId);
      const monthsActive = Math.max(1, Math.round(diffDays / 30.44));
      const monthlyEquivalent = getPricePerMonth(sub.cycle, sub.frequency, convertToMain(sub.price, sub.currencyId));

      return {
        subscriptionId: sub.id,
        name: sub.name,
        logo: sub.logo,
        startDate: sub.startDate,
        monthsActive,
        totalPaid: Math.round(totalPaid * 100) / 100,
        monthlyEquivalent: Math.round(monthlyEquivalent * 100) / 100,
      };
    })
    .sort((a, b) => b.totalPaid - a.totalPaid);
}

// =============================================
// Average cost by category
// =============================================

/**
 * Calculate average monthly subscription cost per category.
 */
export function getCategoryAverages(
  subscriptions: Subscription[],
  categories: Category[],
  convertToMain: (price: number, currencyId: string) => number,
  expenses: Expense[] = [],
): CategoryAverage[] {
  const activeSubs = subscriptions.filter((s) => !s.inactive);
  const map: Record<string, { name: string; total: number; count: number }> = {};

  for (const sub of activeSubs) {
    const cat = categories.find((c) => c.id === sub.categoryId);
    const catName = cat?.name || "Other";
    if (!map[sub.categoryId]) map[sub.categoryId] = { name: catName, total: 0, count: 0 };
    map[sub.categoryId].total += getPricePerMonth(sub.cycle, sub.frequency, convertToMain(sub.price, sub.currencyId));
    map[sub.categoryId].count++;
  }

  // Include expenses (average over last 3 months for monthly equivalent)
  if (expenses.length > 0) {
    const now = new Date();
    const threeMonthsAgo = new Date(now.getFullYear(), now.getMonth() - 3, 1).toISOString().split("T")[0];
    const recentExpenses = expenses.filter((e) => e.date >= threeMonthsAgo);
    for (const exp of recentExpenses) {
      const cat = categories.find((c) => c.id === exp.categoryId);
      const catName = cat?.name || "Other";
      if (!map[exp.categoryId]) map[exp.categoryId] = { name: catName, total: 0, count: 0 };
      map[exp.categoryId].total += convertToMain(exp.amount, exp.currencyId) / 3; // monthly equivalent
      map[exp.categoryId].count++;
    }
  }

  return Object.entries(map)
    .map(([id, data]) => ({
      categoryId: id,
      categoryName: data.name,
      totalMonthly: Math.round(data.total * 100) / 100,
      subscriptionCount: data.count,
      averageMonthly: Math.round((data.total / data.count) * 100) / 100,
    }))
    .sort((a, b) => b.totalMonthly - a.totalMonthly);
}
