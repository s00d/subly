import { TrayIcon } from "@tauri-apps/api/tray";
import { Menu, MenuItem, PredefinedMenuItem } from "@tauri-apps/api/menu";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Subscription, Settings, Currency } from "@/schemas/appData";
import { isUpcoming, isOverdue, formatCurrency, getPricePerMonth } from "@/services/calculations";

const TRAY_ID = "main-tray";

// Callback that gets invoked when a subscription is clicked in the tray.
// The host (App.vue) sets this up to handle navigation.
let onSubscriptionClick: ((subId: string) => void) | null = null;

export function setTraySubscriptionClickHandler(handler: (subId: string) => void) {
  onSubscriptionClick = handler;
}

async function showAndFocus() {
  const win = getCurrentWindow();
  await win.show();
  await win.setFocus();
}

/**
 * Update the system tray menu with current subscription data.
 * The tray icon itself is created from Rust side for reliable icon display.
 */
export async function setupTray(
  subscriptions: Subscription[],
  settings: Settings,
  currencies: Currency[],
): Promise<void> {
  try {
    // Get the tray icon created by Rust
    const tray = await TrayIcon.getById(TRAY_ID);
    if (!tray) {
      console.warn("Tray icon not found by id:", TRAY_ID);
      return;
    }

    const mainCurrency = currencies.find((c) => c.id === settings.mainCurrencyId) || currencies[0];

    // Build upcoming payments list (next 7 days)
    const upcoming = subscriptions
      .filter((s) => !s.inactive && isUpcoming(s, 7))
      .sort((a, b) => new Date(a.nextPayment).getTime() - new Date(b.nextPayment).getTime())
      .slice(0, 8);

    // Build overdue list
    const overdue = subscriptions.filter((s) => isOverdue(s));

    // Calculate monthly cost
    const activeSubs = subscriptions.filter((s) => !s.inactive);
    const monthlyCost = activeSubs.reduce((sum, s) => {
      const rate = currencies.find((c) => c.id === s.currencyId)?.rate || 1;
      const converted = rate > 0 ? s.price / rate : s.price;
      return sum + getPricePerMonth(s.cycle, s.frequency, converted);
    }, 0);

    const fmt = (amount: number) =>
      formatCurrency(amount, mainCurrency?.code || "USD", mainCurrency?.symbol);

    // Build menu items
    const menuItems: any[] = [];

    // Header: Monthly cost
    menuItems.push(
      await MenuItem.new({
        id: "tray-header",
        text: `💰 Monthly: ${fmt(monthlyCost)}`,
        enabled: false,
      }),
    );

    menuItems.push(await PredefinedMenuItem.new({ item: "Separator" }));

    // Overdue section
    if (overdue.length > 0) {
      menuItems.push(
        await MenuItem.new({
          id: "tray-overdue-header",
          text: `⚠️ Overdue (${overdue.length})`,
          enabled: false,
        }),
      );
      for (const sub of overdue.slice(0, 5)) {
        const cur = currencies.find((c) => c.id === sub.currencyId);
        const price = formatCurrency(sub.price, cur?.code || "USD", cur?.symbol);
        const subId = sub.id;
        menuItems.push(
          await MenuItem.new({
            id: `tray-overdue-${sub.id}`,
            text: `  ❗ ${sub.name} — ${price}`,
            enabled: true,
            action: async () => {
              await showAndFocus();
              onSubscriptionClick?.(subId);
            },
          }),
        );
      }
      menuItems.push(await PredefinedMenuItem.new({ item: "Separator" }));
    }

    // Upcoming section
    if (upcoming.length > 0) {
      menuItems.push(
        await MenuItem.new({
          id: "tray-upcoming-header",
          text: `📅 Upcoming (7 days)`,
          enabled: false,
        }),
      );
      for (const sub of upcoming) {
        const cur = currencies.find((c) => c.id === sub.currencyId);
        const price = formatCurrency(sub.price, cur?.code || "USD", cur?.symbol);
        const date = new Date(sub.nextPayment);
        const dateStr = date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
        const subId = sub.id;
        menuItems.push(
          await MenuItem.new({
            id: `tray-upcoming-${sub.id}`,
            text: `  ${sub.name} — ${price} (${dateStr})`,
            enabled: true,
            action: async () => {
              await showAndFocus();
              onSubscriptionClick?.(subId);
            },
          }),
        );
      }
    } else {
      menuItems.push(
        await MenuItem.new({
          id: "tray-no-upcoming",
          text: "  No upcoming payments",
          enabled: false,
        }),
      );
    }

    menuItems.push(await PredefinedMenuItem.new({ item: "Separator" }));

    // Active subscriptions count
    menuItems.push(
      await MenuItem.new({
        id: "tray-active-count",
        text: `📊 Active: ${activeSubs.length}`,
        enabled: false,
      }),
    );

    menuItems.push(await PredefinedMenuItem.new({ item: "Separator" }));

    // Actions
    menuItems.push(
      await MenuItem.new({
        id: "tray-show",
        text: "Open Subly",
        action: async () => {
          await showAndFocus();
        },
      }),
    );

    menuItems.push(
      await MenuItem.new({
        id: "tray-quit",
        text: "Quit",
        action: async () => {
          // destroy() force-closes the window bypassing CloseRequested,
          // which causes the app to exit since it's the last window
          const win = getCurrentWindow();
          await win.destroy();
        },
      }),
    );

    // Build menu and update tray
    const menu = await Menu.new({ items: menuItems });
    await tray.setMenu(menu);
    await tray.setTooltip(`Subly — ${fmt(monthlyCost)}/mo`);
  } catch (e) {
    console.warn("Failed to update tray menu:", e);
  }
}
