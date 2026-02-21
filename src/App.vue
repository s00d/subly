<script setup lang="ts">
import { onMounted, onUnmounted, watch, computed } from "vue";
import { useRouter } from "vue-router";
import { useAppStore } from "@/stores/appStore";
import { checkAndNotify } from "@/services/notifications";
import { shouldUpdateRates, updateCurrencyRates } from "@/services/currencyUpdater";
import { setupPushNotifications, startPushNotificationListener } from "@/services/pushNotifications";
import { useAlerts } from "@/composables/useAlerts";
import { setupTray, setTraySubscriptionClickHandler } from "@/services/tray";
import AppLayout from "@/components/layout/AppLayout.vue";
import InAppAlerts from "@/components/ui/InAppAlerts.vue";

const router = useRouter();
const store = useAppStore();
const { alerts, setAlerts, dismiss, dismissAll } = useAlerts();

async function initTray() {
  try {
    await setupTray(
      store.state.subscriptions,
      store.state.settings,
      store.state.currencies,
    );
  } catch (e) {
    console.warn("Tray setup failed:", e);
  }
}

// Periodic notification check interval
let notificationInterval: ReturnType<typeof setInterval> | null = null;

async function runNotificationCheck() {
  try {
    const result = await checkAndNotify({
      subscriptions: store.state.subscriptions,
      settings: store.state.settings,
      telegram: {
        botToken: store.state.telegramBotToken,
        chatId: store.state.telegramChatId,
        enabled: store.state.telegramEnabled,
      },
      currencies: store.state.currencies,
      onNotified: (subId, date) => store.markNotified(subId, date),
    });
    if (result.alerts.length > 0) {
      setAlerts(result.alerts);
    }
  } catch (e) {
    console.warn("Notification check failed:", e);
  }
}

async function runCurrencyUpdate() {
  try {
    const s = store.state.settings;
    if (!s.currencyAutoUpdate) return;
    if (!store.state.fixerApiKey) return;
    if (!shouldUpdateRates(s.lastCurrencyUpdate)) return;

    const result = await updateCurrencyRates(
      store.state.fixerApiKey,
      store.state.fixerProvider,
      store.state.currencies,
      s.mainCurrencyId,
      s.currencyUpdateTargets,
    );

    if (result.updated > 0) {
      store.updateSettings({ lastCurrencyUpdate: new Date().toISOString().split("T")[0] });
      console.log(`Currency rates updated: ${result.updated} currencies`);
    }
    if (result.error) {
      console.warn("Currency update error:", result.error);
    }
  } catch (e) {
    console.warn("Currency auto-update failed:", e);
  }
}

onMounted(async () => {
  await store.init();
  applyTheme();
  applyColorTheme();

  // Initial notification check
  await runNotificationCheck();

  // Auto-update currency rates
  await runCurrencyUpdate();

  // Check every 30 minutes
  notificationInterval = setInterval(() => {
    runNotificationCheck();
    runCurrencyUpdate();
  }, 30 * 60 * 1000);

  // Setup tray subscription click handler — navigate to subscription detail
  setTraySubscriptionClickHandler((subId: string) => {
    router.push({ path: "/subscriptions", query: { sub: subId } });
  });

  // Setup system tray
  await initTray();

  // Setup push notifications (iOS — safe no-op on desktop)
  try {
    const pushResult = await setupPushNotifications();
    if (pushResult.registered && pushResult.token) {
      console.log("Push notifications registered, token:", pushResult.token);
    }
    await startPushNotificationListener((event) => {
      console.log("Push notification event:", event.type, event.payload);
      if (event.type === "FOREGROUND_DELIVERY" || event.type === "BACKGROUND_TAP") {
        runNotificationCheck();
      }
    });
  } catch (e) {
    console.warn("Push notification init skipped:", e);
  }

  // Global keyboard shortcuts
  document.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeyDown);
  if (notificationInterval) clearInterval(notificationInterval);
});

// Update tray when subscriptions change
watch(
  () => store.state.subscriptions,
  () => initTray(),
  { deep: true },
);

function handleKeyDown(e: KeyboardEvent) {
  const isMod = e.metaKey || e.ctrlKey;

  // Ctrl/Cmd+1..4 for navigation
  if (isMod && !e.shiftKey) {
    switch (e.key) {
      case "1": e.preventDefault(); router.push("/"); break;
      case "2": e.preventDefault(); router.push("/subscriptions"); break;
      case "3": e.preventDefault(); router.push("/calendar"); break;
      case "4": e.preventDefault(); router.push("/settings"); break;
    }
  }
}

function applyTheme() {
  const dt = store.state.settings.darkTheme;
  if (dt === 1) {
    document.documentElement.classList.add("dark");
  } else if (dt === 0) {
    document.documentElement.classList.remove("dark");
  } else {
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    document.documentElement.classList.toggle("dark", prefersDark);
  }
}

function applyColorTheme() {
  const theme = store.state.settings.colorTheme;
  document.body.className = document.body.className.replace(/theme-\w+/g, "").trim();
  if (theme !== "blue") {
    document.body.classList.add(`theme-${theme}`);
  }
}

// Watch for theme changes
watch(() => store.state.settings.darkTheme, applyTheme);
watch(() => store.state.settings.colorTheme, applyColorTheme);

// Listen for system theme changes
if (typeof window !== "undefined") {
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
    if (store.state.settings.darkTheme === 2) applyTheme();
  });
}
</script>

<template>
  <div v-if="store.isLoading.value" class="h-screen flex items-center justify-center bg-[var(--color-surface-secondary)]">
    <div class="text-center">
      <div class="w-12 h-12 rounded-xl bg-[var(--color-primary)] flex items-center justify-center mx-auto mb-3">
        <span class="text-white font-bold text-xl">W</span>
      </div>
      <p class="text-sm text-[var(--color-text-muted)]">Loading...</p>
    </div>
  </div>
  <AppLayout v-else>
    <!-- In-app alerts banner (shown on every page) -->
    <InAppAlerts
      :alerts="alerts"
      @dismiss="dismiss"
      @dismissAll="dismissAll"
    />
    <router-view />
  </AppLayout>
</template>
