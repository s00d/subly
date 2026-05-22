import { callCommand } from "@/services/commandClient";

import type { MoneyString } from "@/schemas/appData";

export interface InAppAlert {
  id: string;
  subscriptionId: string;
  subscriptionName: string;
  type: "upcoming" | "due_today" | "overdue";
  daysUntil: number;
  price: MoneyString;
  currencyId: string;
}

export type NotificationsEventKind = "dispatch" | "run_check";

export interface NotificationsEventResponse<T = unknown> {
  ok: boolean;
  event: NotificationsEventKind;
  data: T;
}

export async function notificationsEvent<T = unknown>(
  event: NotificationsEventKind,
  payload?: Record<string, unknown>,
): Promise<NotificationsEventResponse<T>> {
  return callCommand<NotificationsEventResponse<T>>("notifications_event", {
    event,
    payload: payload ?? {},
  });
}
