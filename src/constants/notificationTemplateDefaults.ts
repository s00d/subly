/**
 * Должны совпадать с дефолтами в `src-tauri/src/commands/notifications.rs`
 * (`default_due_title` / `default_overdue_title` / `FALLBACK_BODY_*`).
 */
export const NOTIFICATION_TEMPLATE_DEFAULTS = {
  notificationTitle: "Subly - {name}",
  notificationBodyDueToday: 'Payment for "{name}" is due today.',
  notificationBodyDueSoon: 'Payment for "{name}" is due in {days} day(s).',
  notificationOverdueTitle: "Subly - Overdue: {name}",
  notificationOverdueBody: '"{name}" is overdue by {days} day(s).',
} as const;

/** Плейсхолдеры: placeholder и title у полей ввода */
export const NOTIFICATION_TAGS_TOOLTIP =
  "{name}, {days}, {due_date}, {price}, {currency}";
