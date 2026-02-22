<script setup lang="ts">
import { onMounted, onUnmounted, watch, computed } from "vue";
import { useRouter } from "vue-router";
import { useAppStore } from "@/stores/appStore";
import { useSubscriptionsStore } from "@/stores/subscriptions";
import { useSettingsStore } from "@/stores/settings";
import { useCatalogStore } from "@/stores/catalog";
import { checkAndNotify } from "@/services/notifications";
import { shouldUpdateRates, updateCurrencyRates } from "@/services/rates";
import type { RatesProviderType } from "@/services/rates";
import { setupPushNotifications, startPushNotificationListener } from "@/services/pushNotifications";
import { initSync, setSyncCallbacks, syncStatus, pullRemote, dismissPendingUpdate } from "@/services/sync";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { useAlerts } from "@/composables/useAlerts";
import { setupTray, setTraySubscriptionClickHandler } from "@/services/tray";
import AppLayout from "@/components/layout/AppLayout.vue";
import InAppAlerts from "@/components/ui/InAppAlerts.vue";

const router = useRouter();
const appStore = useAppStore();
const subsStore = useSubscriptionsStore();
const settingsStore = useSettingsStore();
const catalogStore = useCatalogStore();
const { alerts, setAlerts, dismiss, dismissAll } = useAlerts();
const { t } = useI18n();
const { toast } = useToast();

async function handlePullRemote() {
  const ok = await pullRemote();
  if (ok) {
    toast(t("sync_pull_success"));
  } else if (syncStatus.error) {
    toast(syncStatus.error, "error");
  }
}

function handleDismissSync() {
  dismissPendingUpdate();
}

async function initTray() {
  try {
    await setupTray(
      subsStore.subscriptions,
      settingsStore.settings,
      catalogStore.currencies,
    );
  } catch (e) {
    console.warn("Tray setup failed:", e);
  }
}

let notificationInterval: ReturnType<typeof setInterval> | null = null;

async function runNotificationCheck() {
  try {
    const result = await checkAndNotify({
      subscriptions: subsStore.subscriptions,
      settings: settingsStore.settings,
      telegram: {
        botToken: settingsStore.telegramBotToken,
        chatId: settingsStore.telegramChatId,
        enabled: settingsStore.telegramEnabled,
      },
      currencies: catalogStore.currencies,
      onNotified: (subId, date) => subsStore.markNotified(subId, date),
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
    const s = settingsStore.settings;
    if (!s.currencyAutoUpdate) return;
    const providerType = settingsStore.ratesProvider as RatesProviderType;
    const apiKey = settingsStore.ratesApiKey;
    if (!apiKey && providerType !== "frankfurter") return;
    if (!shouldUpdateRates(s.lastCurrencyUpdate)) return;

    const result = await updateCurrencyRates(
      providerType,
      apiKey,
      catalogStore.currencies,
      s.mainCurrencyId,
      s.currencyUpdateTargets,
      {
        historyEnabled: s.rateHistoryEnabled,
        historyDays: s.rateHistoryDays,
      },
    );

    if (result.updated > 0) {
      settingsStore.updateSettings({ lastCurrencyUpdate: new Date().toISOString().split("T")[0] });
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
  await appStore.init();
  applyTheme();
  applyColorTheme();

  await runNotificationCheck();
  await runCurrencyUpdate();

  notificationInterval = setInterval(() => {
    runNotificationCheck();
    runCurrencyUpdate();
  }, 30 * 60 * 1000);

  setTraySubscriptionClickHandler((subId: string) => {
    router.push({ path: "/subscriptions", query: { sub: subId } });
  });

  await initTray();

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

  try {
    setSyncCallbacks(
      (data) => appStore.importData(data),
      () => appStore.getExportData(),
    );
    await initSync();
  } catch (e) {
    console.warn("Sync init skipped:", e);
  }

  document.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeyDown);
  if (notificationInterval) clearInterval(notificationInterval);
});

watch(
  () => subsStore.subscriptions,
  () => initTray(),
  { deep: true },
);

function handleKeyDown(e: KeyboardEvent) {
  const isMod = e.metaKey || e.ctrlKey;
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
  const dt = settingsStore.settings.darkTheme;
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
  const theme = settingsStore.settings.colorTheme;
  document.body.className = document.body.className.replace(/theme-\w+/g, "").trim();
  if (theme !== "blue") {
    document.body.classList.add(`theme-${theme}`);
  }
}

watch(() => settingsStore.settings.darkTheme, applyTheme);
watch(() => settingsStore.settings.colorTheme, applyColorTheme);

if (typeof window !== "undefined") {
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
    if (settingsStore.settings.darkTheme === 2) applyTheme();
  });
}
</script>

<template>
  <div v-if="appStore.isLoading" class="h-screen flex items-center justify-center bg-[var(--color-surface-secondary)]">
    <div class="text-center">
      <div class="w-12 h-12 rounded-xl bg-[var(--color-primary)] flex items-center justify-center mx-auto mb-3">
        <span class="text-white font-bold text-xl">W</span>
      </div>
      <p class="text-sm text-[var(--color-text-muted)]">Loading...</p>
    </div>
  </div>
  <AppLayout v-else>
    <!-- Cloud sync update banner -->
    <div
      v-if="syncStatus.pendingUpdate"
      class="mx-3 mt-3 p-3 rounded-xl bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 flex items-start sm:items-center gap-3 flex-col sm:flex-row"
    >
      <div class="flex-1 min-w-0">
        <p class="text-sm font-medium text-blue-800 dark:text-blue-200">{{ t('sync_remote_newer') }}</p>
        <p class="text-xs text-blue-600 dark:text-blue-400 mt-0.5">{{ t('sync_remote_newer_desc') }}</p>
      </div>
      <div class="flex items-center gap-2 shrink-0">
        <button
          @click="handlePullRemote"
          :disabled="syncStatus.syncing"
          class="px-3 py-1.5 rounded-lg bg-blue-600 text-white text-xs font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors"
        >
          {{ syncStatus.syncing ? t('sync_syncing') : t('sync_pull') }}
        </button>
        <button
          @click="handleDismissSync"
          class="px-3 py-1.5 rounded-lg text-blue-600 dark:text-blue-400 text-xs font-medium hover:bg-blue-100 dark:hover:bg-blue-900/40 transition-colors"
        >
          {{ t('sync_dismiss') }}
        </button>
      </div>
    </div>

    <!-- In-app alerts banner (shown on every page) -->
    <InAppAlerts
      :alerts="alerts"
      @dismiss="dismiss"
      @dismissAll="dismissAll"
    />
    <router-view />
  </AppLayout>
</template>
