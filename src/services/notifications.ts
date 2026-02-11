import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import type { Subscription, Settings } from "@/schemas/appData";
import { sendTelegramPaymentReminder, type TelegramConfig } from "@/services/telegram";
import { formatCurrency } from "@/services/calculations";

// =============================================
// Types
// =============================================

export interface InAppAlert {
  id: string;
  subscriptionId: string;
  subscriptionName: string;
  type: "upcoming" | "due_today" | "overdue";
  daysUntil: number;
  price: number;
  currencyId: string;
}

export interface NotifyContext {
  subscriptions: Subscription[];
  settings: Settings;
  telegram?: TelegramConfig & { enabled: boolean };
  currencies?: { id: string; code: string; symbol: string }[];
  /** Callback to persist lastNotifiedDate on a subscription */
  onNotified?: (subId: string, date: string) => void;
}

// =============================================
// Permission
// =============================================

export async function checkNotificationPermission(): Promise<boolean> {
  try {
    let granted = await isPermissionGranted();
    if (!granted) {
      const result = await requestPermission();
      granted = result === "granted";
    }
    return granted;
  } catch {
    return false;
  }
}

// =============================================
// Schedule helpers
// =============================================

/**
 * Check if current time is within the allowed notification window.
 *
 * Schedule modes:
 * - "any"     → always allowed
 * - "morning" → 7:00–11:59
 * - "evening" → 17:00–21:59
 * - "custom"  → exact hour (± 30 min window for interval tolerance)
 */
export function isWithinSchedule(settings: Settings): boolean {
  const schedule = settings.notificationSchedule || "any";
  if (schedule === "any") return true;

  const now = new Date();
  const hour = now.getHours();

  switch (schedule) {
    case "morning":
      return hour >= 7 && hour < 12;
    case "evening":
      return hour >= 17 && hour < 22;
    case "custom": {
      const target = settings.notificationCustomHour ?? 9;
      // Allow ±1 hour window to account for check interval
      return hour >= target && hour <= target + 1;
    }
    default:
      return true;
  }
}

/**
 * Check if a notification was already sent today for this subscription.
 */
function alreadyNotifiedToday(lastNotifiedDate: string): boolean {
  if (!lastNotifiedDate) return false;
  const today = new Date().toISOString().split("T")[0];
  return lastNotifiedDate === today;
}

// =============================================
// Test notification
// =============================================

export async function sendTestNotification(): Promise<{ system: boolean }> {
  try {
    // Always try to request permission first
    let granted = await isPermissionGranted();
    if (!granted) {
      const permission = await requestPermission();
      granted = permission === "granted";
    }

    // Send regardless — wrapped in try/catch
    sendNotification({
      title: "Subly — Test Notification",
      body: "Notifications are working correctly!",
    });
    return { system: true };
  } catch (e) {
    console.warn("Test notification failed:", e);
    return { system: false };
  }
}

// =============================================
// Template helpers
// =============================================

function applyTemplate(template: string, vars: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (_, key) => String(vars[key] ?? `{${key}}`));
}

// =============================================
// Main notification check
// =============================================

export async function checkAndNotify(
  ctx: NotifyContext,
): Promise<{ sentCount: number; alerts: InAppAlert[] }> {
  const { subscriptions, settings, telegram, currencies, onNotified } = ctx;

  // Check schedule — if not in the right time window, only generate alerts (no push/telegram)
  const withinSchedule = isWithinSchedule(settings);
  const recurring = settings.recurringNotifications !== false;

  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const todayStr = today.toISOString().split("T")[0];

  let sentCount = 0;
  const alerts: InAppAlert[] = [];

  const telegramEnabled = telegram?.enabled && telegram.botToken && telegram.chatId;

  for (const sub of subscriptions) {
    if (sub.inactive) continue;
    if (!sub.notify) continue;

    const daysBefore =
      sub.notifyDaysBefore >= 0
        ? sub.notifyDaysBefore
        : settings.notifyDaysBefore;

    const nextPayment = new Date(sub.nextPayment);
    nextPayment.setHours(0, 0, 0, 0);

    const diffMs = nextPayment.getTime() - today.getTime();
    const diffDays = Math.round(diffMs / (1000 * 60 * 60 * 24));

    // Format price for telegram
    const priceStr = currencies
      ? (() => {
          const c = currencies.find((cur) => cur.id === sub.currencyId);
          return formatCurrency(sub.price, c?.code || "USD", c?.symbol);
        })()
      : `${sub.price}`;

    // Determine if we should send a push/telegram (not just an alert)
    // Recurring: send every day within the window if not already sent today
    // Non-recurring: only send on the exact daysBefore match
    const shouldSendPush = withinSchedule && !alreadyNotifiedToday(sub.lastNotifiedDate || "");

    // Upcoming / due today
    if (diffDays >= 0 && diffDays <= daysBefore) {
      const alertType: InAppAlert["type"] = diffDays === 0 ? "due_today" : "upcoming";

      alerts.push({
        id: `${sub.id}-${alertType}`,
        subscriptionId: sub.id,
        subscriptionName: sub.name,
        type: alertType,
        daysUntil: diffDays,
        price: sub.price,
        currencyId: sub.currencyId,
      });

      // Decide whether to send push
      const shouldSendForThisSub = recurring
        ? shouldSendPush                      // recurring: every day (once per day)
        : (shouldSendPush && diffDays === daysBefore); // non-recurring: only on exact day

      if (shouldSendForThisSub) {
        // System notification — always try, ignore errors
        const vars = { name: sub.name, days: diffDays, price: sub.price };
        const title = settings.notificationTitle || "Subly — Payment Reminder";
        const body = diffDays === 0
          ? applyTemplate(settings.notificationBodyDueToday || 'Payment for "{name}" is due today!', vars)
          : applyTemplate(settings.notificationBodyDueSoon || 'Payment for "{name}" is due in {days} day(s).', vars);

        try {
          sendNotification({ title, body });
          sentCount++;
        } catch (e) {
          console.warn("Failed to send notification for:", sub.name, e);
        }

        // Telegram
        if (telegramEnabled) {
          sendTelegramPaymentReminder(
            { botToken: telegram!.botToken, chatId: telegram!.chatId },
            sub.name,
            diffDays,
            priceStr,
          ).catch((e) => console.warn("Telegram notify failed:", e));
        }

        // Mark as notified today
        if (onNotified) onNotified(sub.id, todayStr);
      }
    }

    // Overdue (manual renewal only)
    if (!sub.autoRenew && diffDays < 0) {
      alerts.push({
        id: `${sub.id}-overdue`,
        subscriptionId: sub.id,
        subscriptionName: sub.name,
        type: "overdue",
        daysUntil: diffDays,
        price: sub.price,
        currencyId: sub.currencyId,
      });

      const shouldSendOverdue = recurring
        ? shouldSendPush
        : (shouldSendPush && !alreadyNotifiedToday(sub.lastNotifiedDate || ""));

      if (shouldSendOverdue) {
        const vars = { name: sub.name, days: Math.abs(diffDays), price: sub.price };
        const title = settings.notificationOverdueTitle || "Subly — Overdue Payment";
        const body = applyTemplate(
          settings.notificationOverdueBody || '"{name}" is overdue by {days} day(s).',
          vars,
        );
        try {
          sendNotification({ title, body });
          sentCount++;
        } catch (e) {
          console.warn("Failed to send overdue notification for:", sub.name, e);
        }

        if (telegramEnabled) {
          sendTelegramPaymentReminder(
            { botToken: telegram!.botToken, chatId: telegram!.chatId },
            sub.name,
            diffDays,
            priceStr,
          ).catch((e) => console.warn("Telegram overdue notify failed:", e));
        }

        if (onNotified) onNotified(sub.id, todayStr);
      }
    }
  }

  return { sentCount, alerts };
}
