/**
 * Telegram Bot API integration — plain fetch, no dependencies.
 */

const TELEGRAM_API = "https://api.telegram.org";

export interface TelegramConfig {
  botToken: string;
  chatId: string;
}

/**
 * Send a message via Telegram Bot API.
 * Returns true if sent successfully.
 */
export async function sendTelegramMessage(
  config: TelegramConfig,
  text: string,
  parseMode: "HTML" | "Markdown" = "HTML",
): Promise<boolean> {
  if (!config.botToken || !config.chatId) return false;

  try {
    const url = `${TELEGRAM_API}/bot${config.botToken}/sendMessage`;
    const res = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        chat_id: config.chatId,
        text,
        parse_mode: parseMode,
      }),
    });

    if (!res.ok) {
      const body = await res.text();
      console.warn("Telegram API error:", res.status, body);
      return false;
    }

    return true;
  } catch (e) {
    console.warn("Telegram send failed:", e);
    return false;
  }
}

/**
 * Send a test message to verify bot token and chat ID.
 */
export async function sendTelegramTestMessage(config: TelegramConfig): Promise<boolean> {
  return sendTelegramMessage(config, "✅ <b>Subly</b> — Telegram notifications are working!");
}

/**
 * Format and send a subscription payment reminder via Telegram.
 */
export async function sendTelegramPaymentReminder(
  config: TelegramConfig,
  subscriptionName: string,
  daysUntil: number,
  price: string,
): Promise<boolean> {
  let text: string;

  if (daysUntil === 0) {
    text = `💰 <b>${escapeHtml(subscriptionName)}</b> — payment is due <b>today</b>!\nAmount: ${escapeHtml(price)}`;
  } else if (daysUntil > 0) {
    text = `🔔 <b>${escapeHtml(subscriptionName)}</b> — payment in <b>${daysUntil}</b> day(s)\nAmount: ${escapeHtml(price)}`;
  } else {
    text = `⚠️ <b>${escapeHtml(subscriptionName)}</b> — <b>overdue</b> by ${Math.abs(daysUntil)} day(s)\nAmount: ${escapeHtml(price)}`;
  }

  return sendTelegramMessage(config, text);
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
