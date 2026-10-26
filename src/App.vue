<script setup lang="ts">
import { button } from "@/lib/tv";
import { onMounted, onUnmounted, watch, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  notificationsEvent,
  type InAppAlert,
  type LocalNotificationPayload,
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
import { callCommand, invokeCommand } from "@/services/commandClient";
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
import Modal from "@/components/ui/Modal.vue";
import { importFromSubly } from "@/services/exportClient";
import { formatErrorForToast } from "@/utils/formatError";

const router = useRouter();
const route = useRoute();
const isLoading = ref(true);
const showBackupRestoreConfirm = ref(false);
const pendingBackupPath = ref<string | null>(null);
const { setAlerts } = useAlerts();
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

async function refreshAllData() {
  await metaStore.refresh();
  await Promise.all([
    dashboardStore.loadPage(true),
    subscriptionsStore.loadBrief(),
    expensesStore.loadPage(expensesStore.page, expensesStore.currentFilter),
    expensesStore.refreshSummary(expensesStore.currentFilter),
    calendarStore.reloadCurrentMonth(),
    catalogsUsageStore.refreshIfLoaded(),
  ]);
}

async function handlePullRemote() {
  const result = await pullRemote();
  if (result.ok) {
    await refreshAllData();
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
let unlistenNotificationAlerts: (() => void) | null = null;
let unlistenBackupOpened: (() => void) | null = null;
let visibilityHandler: (() => void) | null = null;
let refreshDataTimer: ReturnType<typeof setTimeout> | null = null;
let lastNotificationCheckAt = 0;
let notificationCheckInFlight = false;
const dashboardStore = useDashboardStore();
const expensesStore = useExpensesStore();
const subscriptionsStore = useSubscriptionsStore();
const calendarStore = useCalendarStore();
const metaStore = useAppMetaStore();
const catalogsUsageStore = useCatalogsUsageStore();
const nowStore = useNowStore();
const { settings } = storeToRefs(metaStore);
const { todayId } = storeToRefs(nowStore);

interface DataChangedEventPayload {
  entity?: string;
  action?: string;
}

function scheduleStoresRefresh() {
  if (refreshDataTimer) clearTimeout(refreshDataTimer);
  refreshDataTimer = setTimeout(() => {
    void refreshAllData();
  }, 120);
}

watch(todayId, (newId, oldId) => {
  if (oldId === undefined || oldId === newId) return;
  console.log("[App] Day changed, refreshing data...");
  void refreshAllData();
});

function logUnhandledRejection(event: PromiseRejectionEvent) {
  console.error("[Global] Unhandled promise rejection", {
    reason: event.reason,
  });
}

function logWindowError(event: ErrorEvent) {
  // Benign browser notice (layout settled in a later frame), not an app error.
  if (event.message?.includes("ResizeObserver loop")) return;
  console.error("[Global] Unhandled runtime error", {
    message: event.message,
    source: event.filename,
    line: event.lineno,
    column: event.colno,
    error: event.error,
  });
}

async function runNotificationCheck() {
  if (notificationCheckInFlight) return;
  const now = Date.now();
  if (now - lastNotificationCheckAt < 5000) return;
  notificationCheckInFlight = true;
  lastNotificationCheckAt = now;
  try {
    const result = await notificationsEvent<{ sentCount: number; alerts: InAppAlert[] }>("run_check");
    const data = result.data;
    setAlerts(data.alerts ?? []);
  } catch (e) {
    console.error("Notification check failed:", e);
  } finally {
    notificationCheckInFlight = false;
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

async function finishBoot() {
  isLoading.value = false;
  try {
    await invokeCommand("app_ready");
  } catch (e) {
    console.warn("app_ready failed:", e);
  }
  const el = document.getElementById("bootSplash");
  if (!el) return;
  let removed = false;
  const finalize = () => {
    if (removed) return;
    removed = true;
    el.remove();
    // iOS WKWebView caches visualViewport metrics while the fixed overlay is on top,
    // which leaves MobileTabBar with a stale bottom. Trigger one reflow tick.
    window.dispatchEvent(new Event("resize"));
  };
  el.addEventListener("transitionend", finalize, { once: true });
  el.classList.add("is-ready");
  setTimeout(finalize, 350);
}

async function maybeRestoreBackup(path: string) {
  pendingBackupPath.value = path;
  showBackupRestoreConfirm.value = true;
}

async function confirmRestoreBackup() {
  const path = pendingBackupPath.value;
  showBackupRestoreConfirm.value = false;
  pendingBackupPath.value = null;
  if (!path) return;
  try {
    const res = await importFromSubly(path);
    if (!res.ok) {
      toast(t("import_error"), "error");
      return;
    }
    await refreshAllData();
    toast(t("import_success"));
  } catch (e) {
    console.error("Backup restore failed:", e);
    toast(formatErrorForToast(e, t), "error");
  }
}

function cancelRestoreBackup() {
  showBackupRestoreConfirm.value = false;
  pendingBackupPath.value = null;
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
    await finishBoot();
    return;
  }
  applyTheme();
  applyColorTheme();

  // Listeners first: the startup check below emits `notifications:local` and marks the cycle as
  // notified, so an event emitted before the listener exists is lost for the day.
  unlistenNotificationAlerts = await listen<InAppAlert[]>("notifications:alerts", (event) => {
    setAlerts(event.payload ?? []);
  });
  unlistenLocalNotifications = await listen<LocalNotificationPayload>("notifications:local", (event) => {
    const payload = event.payload;
    const items = payload?.items ?? [];
    const body = (payload?.body ?? "").trim();
    if (!body && items.length === 0) return;
    if (settings.value?.notificationSound !== false) {
      playNotificationSoundFrontend();
    }
    const itemCount = items.length > 0 ? items.length : 1;
    if (itemCount > 1) {
      toast(t("notification_digest_many", { count: itemCount }), "info");
    } else if (body) {
      toast(`${payload.title}: ${body}`, "info");
    }
  });

  await runNotificationCheck();

  notificationInterval = setInterval(() => {
    runNotificationCheck();
  }, 30 * 60 * 1000);

  unlistenDataChanged = await listen<DataChangedEventPayload>("app:data-changed", (event) => {
    const payload = event.payload;
    if (payload?.entity === "appData" || payload?.entity === "settings") {
      void metaStore.refresh();
    }
    scheduleStoresRefresh();
  });
  unlistenBackupOpened = await listen<string[]>("backup:opened", (event) => {
    const firstPath = event.payload?.[0];
    if (firstPath) {
      void maybeRestoreBackup(firstPath);
    }
  });

  try {
    // Avoid hanging the splash if initSync is slow (e.g. flaky network) — cap at 3s.
    await Promise.race([
      initSync(),
      new Promise<void>((resolve) => setTimeout(resolve, 3000)),
    ]);
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
    const initialBackupPaths = await callCommand<string[]>("opened_backup_paths");
    const firstBackupPath = initialBackupPaths?.[0];
    if (firstBackupPath) {
      void maybeRestoreBackup(firstBackupPath);
    }
  } catch (e) {
    const message = formatErrorForToast(e, t);
    syncStatus.error = message;
    console.error("Sync init failed:", e);
    toast(message, "error");
  }

  await finishBoot();

  visibilityHandler = () => {
    if (document.hidden) return;
    nowStore.refresh();
    void runNotificationCheck();
    checkRemote().catch((e) => {
      const message = formatErrorForToast(e, t);
      syncStatus.error = message;
      console.error("Sync checkRemote failed:", e);
    });
  };
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
  if (unlistenNotificationAlerts) unlistenNotificationAlerts();
  if (unlistenBackupOpened) unlistenBackupOpened();
  if (refreshDataTimer) clearTimeout(refreshDataTimer);
  if (visibilityHandler) document.removeEventListener("visibilitychange", visibilityHandler);
});

function handleKeyDown(e: KeyboardEvent) {
  const isMod = e.metaKey || e.ctrlKey;
  if (isMod && !e.shiftKey) {
    switch (e.key) {
      // Cmd/Ctrl+1…6 in sidebar order
      case "1": e.preventDefault(); router.push("/"); break;
      case "2": e.preventDefault(); router.push("/subscriptions"); break;
      case "3": e.preventDefault(); router.push("/expenses"); break;
      case "4": e.preventDefault(); router.push("/calendar"); break;
      case "5": e.preventDefault(); router.push("/currencies"); break;
      case "6": e.preventDefault(); router.push("/settings"); break;
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
  <AppLayout v-if="!isLoading">
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
        <button :class="button({ intent: 'primary', size: 'sm' })"
          @click="handlePullRemote"
          :disabled="syncStatus.syncing"
        >
          {{ syncStatus.syncing ? t('sync_syncing') : t('sync_pull') }}
        </button>
        <button :class="button({ intent: 'ghost', size: 'sm' })"
          @click="handleDismissSync"
        >
          {{ t('sync_dismiss') }}
        </button>
      </div>
    </div>

    <router-view />
  </AppLayout>
  <Modal
    :show="showBackupRestoreConfirm"
    :title="t('import_subly')"
    @close="cancelRestoreBackup"
  >
    <p class="text-sm text-text-muted">
      {{ t('backup_restore_prompt') }}
    </p>
    <template #footer>
      <button :class="button()" @click="cancelRestoreBackup">
        {{ t('cancel') }}
      </button>
      <button :class="button({ intent: 'primary' })" @click="confirmRestoreBackup">
        {{ t('confirm') }}
      </button>
    </template>
  </Modal>
</template>
