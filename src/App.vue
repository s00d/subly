<script setup lang="ts">
import { onMounted, onUnmounted, watch, ref } from "vue";
import { useRouter } from "vue-router";
import {
  notificationsEvent,
  type InAppAlert,
} from "@/services/notificationsClient";
import {
  initSync,
  syncStatus,
  pullRemote,
  dismissPendingUpdate,
  flushSyncBeforeExit,
  finishOAuth,
  checkRemote,
} from "@/services/syncClient";
import { setLanguage } from "@/i18n";
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { useAlerts } from "@/composables/useAlerts";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { getCurrent as getCurrentDeepLinks, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { useDashboardStore } from "@/stores/dashboardStore";
import { useExpensesStore } from "@/stores/expensesStore";
import { useSubscriptionsStore } from "@/stores/subscriptionsStore";
import { useCalendarStore } from "@/stores/calendarStore";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { useCatalogsUsageStore } from "@/stores/catalogsUsageStore";
import { useNowStore } from "@/stores/nowStore";
import AppLayout from "@/components/layout/AppLayout.vue";
import InAppAlerts from "@/components/ui/InAppAlerts.vue";
import { formatErrorForToast } from "@/utils/formatError";

const router = useRouter();
const isLoading = ref(true);
const { alerts, setAlerts, dismiss, dismissAll } = useAlerts();
const { t } = useI18n();
const { toast } = useToast();
let audioCtx: AudioContext | null = null;

function playNotificationSoundFrontend() {
  try {
    const Ctx = window.AudioContext || (window as Window & { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!Ctx) return;
    if (!audioCtx) audioCtx = new Ctx();
    const osc = audioCtx.createOscillator();
    const gain = audioCtx.createGain();
    osc.type = "sine";
    osc.frequency.value = 880;
    gain.gain.value = 0.06;
    osc.connect(gain);
    gain.connect(audioCtx.destination);
    const now = audioCtx.currentTime;
    osc.start(now);
    osc.stop(now + 0.16);
  } catch (e) {
    console.warn("Notification sound playback failed:", e);
  }
}

async function handlePullRemote() {
  const result = await pullRemote();
  if (result.ok) {
    await metaStore.refresh();
    await Promise.all([
      dashboardStore.loadPage(true),
      subscriptionsStore.loadBrief(subscriptionsStore.currentFilter),
      expensesStore.loadPage(expensesStore.page, expensesStore.currentFilter),
      expensesStore.refreshSummary(expensesStore.currentFilter),
      calendarStore.reloadCurrentMonth(),
      catalogsUsageStore.refreshIfLoaded(),
    ]);
    toast(t("sync_pull_success"));
  } else {
    const msg =
      (result.messageKey && t(result.messageKey)) ||
      syncStatus.error ||
      t("sync_operation_failed");
    toast(msg, "error");
  }
}

function handleDismissSync() {
  dismissPendingUpdate();
}

let notificationInterval: ReturnType<typeof setInterval> | null = null;
let unlistenCloseRequested: (() => void) | null = null;
let closingWithSync = false;
let mediaQueryList: MediaQueryList | null = null;
let mediaQueryListener: ((e: MediaQueryListEvent) => void) | null = null;
let unlistenDeepLinks: (() => void) | null = null;
let unlistenLocalNotifications: (() => void) | null = null;
let unlistenSingleInstanceDeepLink: (() => void) | null = null;
let unlistenDataChanged: (() => void) | null = null;
let wakeHandler: (() => void) | null = null;
let visibilityHandler: (() => void) | null = null;
let refreshDataTimer: ReturnType<typeof setTimeout> | null = null;
const dashboardStore = useDashboardStore();
const expensesStore = useExpensesStore();
const subscriptionsStore = useSubscriptionsStore();
const calendarStore = useCalendarStore();
const metaStore = useAppMetaStore();
const catalogsUsageStore = useCatalogsUsageStore();
const nowStore = useNowStore();
const { settings } = storeToRefs(metaStore);

interface DataChangedEventPayload {
  entity?: string;
  action?: string;
}

function scheduleStoresRefresh() {
  if (refreshDataTimer) clearTimeout(refreshDataTimer);
  refreshDataTimer = setTimeout(() => {
    void dashboardStore.loadPage(true);
    void subscriptionsStore.loadBrief(subscriptionsStore.currentFilter);
    void expensesStore.loadPage(expensesStore.page, expensesStore.currentFilter);
    void expensesStore.refreshSummary(expensesStore.currentFilter);
    void calendarStore.reloadCurrentMonth();
    void catalogsUsageStore.refreshIfLoaded();
  }, 120);
}

function logUnhandledRejection(event: PromiseRejectionEvent) {
  console.error("[Global] Unhandled promise rejection", {
    reason: event.reason,
  });
}

function logWindowError(event: ErrorEvent) {
  console.error("[Global] Unhandled runtime error", {
    message: event.message,
    source: event.filename,
    line: event.lineno,
    column: event.colno,
    error: event.error,
  });
}

async function runNotificationCheck() {
  try {
    const result = await notificationsEvent<{ sentCount: number; alerts: InAppAlert[] }>("run_check");
    const data = result.data;
    if (data.alerts.length > 0) {
      setAlerts(data.alerts);
    }
  } catch (e) {
    console.error("Notification check failed:", e);
  }
}

function parseOAuthUrl(url: string): { code: string; provider: "gdrive" | "dropbox" | "onedrive" | undefined } | null {
  try {
    const parsed = new URL(url);
    const code = parsed.searchParams.get("code");
    if (!code) return null;
    const state = parsed.searchParams.get("state");
    const provider = state === "gdrive" || state === "dropbox" || state === "onedrive" ? state : undefined;
    return { code, provider };
  } catch {
    return null;
  }
}

async function handleOAuthUrls(urls: string[]) {
  for (const url of urls) {
    if (!url.startsWith("subly://oauth/callback")) continue;
    const parsed = parseOAuthUrl(url);
    if (!parsed) continue;
    const ok = await finishOAuth(parsed.code, parsed.provider);
    if (ok) {
      toast(t("sync_connected"));
    } else {
      toast(syncStatus.error?.trim() || t("sync_oauth_failed"), "error");
    }
  }
}

onMounted(async () => {
  isLoading.value = true;
  nowStore.ensureStarted();
  await metaStore.ensureLoaded();
  if (!settings.value) {
    isLoading.value = false;
    return;
  }
  applyTheme();
  applyColorTheme();
  isLoading.value = false;

  await runNotificationCheck();

  notificationInterval = setInterval(() => {
    runNotificationCheck();
  }, 30 * 60 * 1000);

  unlistenLocalNotifications = await listen<{ title: string; body: string }>("notifications:local", (event) => {
    const payload = event.payload;
    if (!payload?.body) return;
    playNotificationSoundFrontend();
    toast(`${payload.title}: ${payload.body}`);
  });

  unlistenDataChanged = await listen<DataChangedEventPayload>("app:data-changed", (event) => {
    const payload = event.payload;
    if (payload?.entity === "appData" || payload?.entity === "settings") {
      void metaStore.refresh();
    }
    scheduleStoresRefresh();
  });

  try {
    await initSync();
    const startUrls = await getCurrentDeepLinks();
    if (startUrls && startUrls.length > 0) {
      await handleOAuthUrls(startUrls);
    }
    unlistenDeepLinks = await onOpenUrl(handleOAuthUrls);
    unlistenSingleInstanceDeepLink = await listen<string[]>("deep-link:single-instance", (event) => {
      if (event.payload?.length) {
        void handleOAuthUrls(event.payload);
      }
    });
  } catch (e) {
    const message = formatErrorForToast(e, t);
    syncStatus.error = message;
    console.error("Sync init failed:", e);
    toast(message, "error");
  }

  wakeHandler = () => {
    runNotificationCheck();
    checkRemote().catch((e) => {
      const message = formatErrorForToast(e, t);
      syncStatus.error = message;
      console.error("Sync checkRemote failed:", e);
    });
  };
  visibilityHandler = () => {
    if (!document.hidden) wakeHandler?.();
  };
  window.addEventListener("focus", wakeHandler);
  document.addEventListener("visibilitychange", visibilityHandler);

  document.addEventListener("keydown", handleKeyDown);
  window.addEventListener("unhandledrejection", logUnhandledRejection);
  window.addEventListener("error", logWindowError);

  try {
    const appWindow = getCurrentWindow();
    unlistenCloseRequested = await appWindow.onCloseRequested(async (event) => {
      if (closingWithSync || !syncStatus.enabled) return;
      event.preventDefault();
      closingWithSync = true;
      await flushSyncBeforeExit();
      await appWindow.close();
    });
  } catch (e) {
    console.warn("Close sync hook skipped:", e);
  }
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeyDown);
  window.removeEventListener("unhandledrejection", logUnhandledRejection);
  window.removeEventListener("error", logWindowError);
  if (notificationInterval) clearInterval(notificationInterval);
  if (unlistenCloseRequested) unlistenCloseRequested();
  if (mediaQueryList && mediaQueryListener) {
    mediaQueryList.removeEventListener("change", mediaQueryListener);
  }
  if (unlistenDeepLinks) unlistenDeepLinks();
  if (unlistenSingleInstanceDeepLink) unlistenSingleInstanceDeepLink();
  if (unlistenLocalNotifications) unlistenLocalNotifications();
  if (unlistenDataChanged) unlistenDataChanged();
  if (refreshDataTimer) clearTimeout(refreshDataTimer);
  if (wakeHandler) window.removeEventListener("focus", wakeHandler);
  if (visibilityHandler) document.removeEventListener("visibilitychange", visibilityHandler);
});

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
  const dt = settings.value?.darkTheme;
  if (dt === undefined) return;
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
  const theme = settings.value?.colorTheme || "blue";
  document.body.className = document.body.className.replace(/theme-\w+/g, "").trim();
  if (theme !== "blue") {
    document.body.classList.add(`theme-${theme}`);
  }
}

watch(
  () => settings.value?.language,
  async (lang) => {
    if (!lang) return;
    await setLanguage(lang);
  },
);
watch(() => settings.value?.darkTheme, applyTheme);
watch(() => settings.value?.colorTheme, applyColorTheme);

if (typeof window !== "undefined") {
  mediaQueryList = window.matchMedia("(prefers-color-scheme: dark)");
  mediaQueryListener = () => {
    if (settings.value?.darkTheme === 2) applyTheme();
  };
  mediaQueryList.addEventListener("change", mediaQueryListener);
}
</script>

<template>
  <div v-if="isLoading" class="h-screen flex items-center justify-center bg-surface-secondary">
    <div class="text-center">
      <div class="w-12 h-12 rounded-xl bg-primary flex items-center justify-center mx-auto mb-3">
        <span class="text-white font-bold text-xl">W</span>
      </div>
      <p class="text-sm text-text-muted">Loading...</p>
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
