import type { CycleType, Subscription, Currency } from "@/schemas/appData";

/**
 * Convert price to monthly equivalent
 */
export function getPricePerMonth(cycle: CycleType, frequency: number, price: number): number {
  switch (cycle) {
    case 1: return price * (30 / frequency);       // days
    case 2: return price * (4.35 / frequency);      // weeks
    case 3: return price * (1 / frequency);          // months
    case 4: return price / (12 * frequency);         // years
    default: return price;
  }
}

/**
 * Convert price from one currency using its rate (relative to base)
 */
export function convertPrice(price: number, currencyRate: number): number {
  if (currencyRate <= 0) return price;
  return price / currencyRate;
}

/**
 * Days remaining until next payment (clamped 0..30)
 */
export function getDaysUntilPayment(nextPayment: string): number {
  const now = new Date();
  now.setHours(0, 0, 0, 0);
  const next = new Date(nextPayment);
  next.setHours(0, 0, 0, 0);
  const diff = Math.ceil((next.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
  return Math.max(0, Math.min(30, diff));
}

/**
 * Get subscription progress percentage (0-100)
 */
export function getSubscriptionProgress(cycle: CycleType, frequency: number, nextPayment: string): number {
  const nextDate = new Date(nextPayment);
  const now = new Date();

  let cycleDays = 30;
  if (cycle === 1) cycleDays = 1 * frequency;
  else if (cycle === 2) cycleDays = 7 * frequency;
  else if (cycle === 3) cycleDays = 30 * frequency;
  else if (cycle === 4) cycleDays = 365 * frequency;

  const lastPaymentDate = new Date(nextDate);
  lastPaymentDate.setDate(lastPaymentDate.getDate() - cycleDays);

  const totalMs = nextDate.getTime() - lastPaymentDate.getTime();
  const elapsedMs = now.getTime() - lastPaymentDate.getTime();

  if (totalMs <= 0) return 100;
  const progress = (elapsedMs / totalMs) * 100;
  return Math.min(100, Math.max(0, Math.floor(progress)));
}

/**
 * Get billing cycle display text
 */
export function getBillingCycleText(cycle: CycleType, frequency: number, t: (key: string) => string): string {
  switch (cycle) {
    case 1: return frequency === 1 ? t("daily") : `${frequency} ${t("days")}`;
    case 2: return frequency === 1 ? t("weekly") : `${frequency} ${t("weeks")}`;
    case 3: return frequency === 1 ? t("monthly") : `${frequency} ${t("months")}`;
    case 4: return frequency === 1 ? t("yearly") : `${frequency} ${t("years")}`;
    default: return "";
  }
}

/**
 * Format currency (locale-aware)
 */
export { formatCurrencyLocale as formatCurrency } from "@/composables/useLocaleFormat";

/**
 * Calculate next payment dates for a subscription within a month
 */
export function getPaymentDatesInMonth(
  sub: Subscription,
  year: number,
  month: number
): number[] {
  const days: number[] = [];
  const nextPayment = new Date(sub.nextPayment);
  const startOfMonth = new Date(year, month, 1);
  const endOfMonth = new Date(year, month + 1, 0);

  const incrementMs = getIncrementMs(sub.cycle, sub.frequency);

  // Walk backwards to find start before month
  let current = new Date(nextPayment);
  while (current > startOfMonth) {
    current = addCycleIncrement(current, sub.cycle, sub.frequency, -1);
  }

  // Walk forward through the month
  const safeEnd = new Date(endOfMonth);
  safeEnd.setFullYear(safeEnd.getFullYear() + 2);

  while (current <= safeEnd) {
    if (current >= startOfMonth && current <= endOfMonth) {
      days.push(current.getDate());
    }
    if (current > endOfMonth) break;
    current = addCycleIncrement(current, sub.cycle, sub.frequency, 1);
  }

  return days;
}

function addCycleIncrement(date: Date, cycle: CycleType, frequency: number, direction: 1 | -1): Date {
  const d = new Date(date);
  const mult = direction;
  switch (cycle) {
    case 1: d.setDate(d.getDate() + frequency * mult); break;
    case 2: d.setDate(d.getDate() + 7 * frequency * mult); break;
    case 3: d.setMonth(d.getMonth() + frequency * mult); break;
    case 4: d.setFullYear(d.getFullYear() + frequency * mult); break;
  }
  return d;
}

function getIncrementMs(cycle: CycleType, frequency: number): number {
  const day = 86400000;
  switch (cycle) {
    case 1: return day * frequency;
    case 2: return day * 7 * frequency;
    case 3: return day * 30 * frequency;
    case 4: return day * 365 * frequency;
    default: return day * 30;
  }
}

/**
 * Check if a subscription payment is overdue (manual renewal only)
 */
export function isOverdue(sub: Subscription): boolean {
  if (sub.inactive || sub.autoRenew) return false;
  const next = new Date(sub.nextPayment);
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  return next < today;
}

/**
 * Check if a subscription payment is upcoming (within N days)
 */
export function isUpcoming(sub: Subscription, withinDays: number = 30): boolean {
  if (sub.inactive) return false;
  const next = new Date(sub.nextPayment);
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const future = new Date(today);
  future.setDate(future.getDate() + withinDays);
  return next >= today && next <= future;
}
