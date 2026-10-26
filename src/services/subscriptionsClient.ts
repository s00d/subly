import type { Subscription, SubscriptionListItem, SubscriptionUpsertPayload } from "@/schemas/appData";
import { callCommand } from "@/services/commandClient";

export interface SubscriptionFilter {
  search?: string;
  state?: "active" | "inactive";
  categoryId?: string;
  paymentMethodId?: string;
  tag?: string;
  sortBy?: "next_payment" | "name" | "price";
  disabledToBottom?: boolean;
  hideDisabled?: boolean;
}

export async function listSubscriptionsPage(): Promise<SubscriptionListItem[]> {
  return callCommand("list_subscriptions_page", { request: {} });
}

export async function listSubscriptionsFiltered(filter: SubscriptionFilter): Promise<SubscriptionListItem[]> {
  return callCommand("list_subscriptions_page", { request: filter });
}

export async function getNextCycleDate(date: string, cycle: number, frequency: number): Promise<string> {
  return callCommand("subscriptions_next_cycle_date", { date, cycle, frequency });
}

export async function getPaymentDatesInMonth(year: number, month: number): Promise<Record<string, number[]>> {
  return callCommand("subscriptions_payment_dates_in_month", { year, month });
}

export interface CalendarMonthStats {
  activeSubscriptions: number;
  paymentsCount: number;
  totalCost: string;
  amountDue: string;
}

/** Subscription payments in a month, converted to the main currency (due = today or later). */
export async function getCalendarMonthStats(year: number, month: number): Promise<CalendarMonthStats> {
  return callCommand("calendar_month_stats", { year, month });
}

export async function listOverdueSubscriptions(): Promise<SubscriptionListItem[]> {
  return callCommand("get_overdue_subscriptions");
}

export async function listUpcomingSubscriptions(days: number, limit: number): Promise<SubscriptionListItem[]> {
  return callCommand("get_upcoming_subscriptions", { days, limit });
}

export async function insertSubscription(subscription: Subscription): Promise<void> {
  return callCommand("subscriptions_insert", { subscription });
}

export async function upsertSubscription(subscription: SubscriptionUpsertPayload): Promise<void> {
  return callCommand("subscriptions_upsert", { subscription });
}

export async function updateSubscription(subscription: Subscription): Promise<void> {
  return callCommand("subscriptions_update", { subscription });
}

export async function deleteSubscription(id: string): Promise<void> {
  return callCommand("subscriptions_delete", { id });
}

export async function deleteSubscriptionsBatch(ids: string[]): Promise<void> {
  return callCommand("subscriptions_delete_batch", { ids });
}

export async function insertPaymentRecord(subId: string, paymentRecord: import("@/schemas/appData").PaymentRecord): Promise<void> {
  return callCommand("subscriptions_insert_payment_record", { subId, paymentRecord });
}

/**
 * One atomic backend step: payment record + linked expense + next payment advanced one cycle.
 * Omitted `date` = today (local), `amount` = subscription price.
 */
export async function recordSubscriptionPayment(
  subId: string,
  opts: { date?: string; amount?: string; note?: string } = {},
): Promise<Subscription> {
  return callCommand("subscriptions_record_payment", {
    subId,
    date: opts.date ?? null,
    amount: opts.amount ?? null,
    note: opts.note ?? null,
  });
}

export async function deletePaymentRecord(id: string): Promise<void> {
  return callCommand("subscriptions_delete_payment_record", { id });
}

