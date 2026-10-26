<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import type { Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import AppInput from "@/components/ui/AppInput.vue";
import AppToggle from "@/components/ui/AppToggle.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import { platform } from "@tauri-apps/plugin-os";
import { enable as enableAutostart, disable as disableAutostart, isEnabled as isAutostartEnabled } from "@tauri-apps/plugin-autostart";
import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { notificationsEvent, notificationsRescheduleAll } from "@/services/notificationsClient";
import { Bell, Volume2 } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { button } from "@/lib/tv";
import UiCard from "@/components/ui/UiCard.vue";
import SavedIndicator, { useAutoSave } from "@/components/settings/SavedIndicator.vue";
import { formatErrorForToast } from "@/utils/formatError";
import {
  NOTIFICATION_TAGS_TOOLTIP,
  NOTIFICATION_TEMPLATE_DEFAULTS,
} from "@/constants/notificationTemplateDefaults";
import { moneyFromNumber, parseMoney, toDisplayNumber } from "@/lib/money";

const props = defineProps<{ lookupSettings: Settings | null }>();
const { t } = useI18n();
const { toast } = useToast();
const metaStore = useAppMetaStore();
const settings = ref<Settings | null>(null);
const { savedAt, run, debounce } = useAutoSave();

function isMobilePlatform(): boolean {
  try {
    const p = platform();
    return p === "ios" || p === "android";
  } catch {
    return false;
  }
}

async function ensureNotificationPermission(): Promise<boolean> {
  if (!isMobilePlatform()) return true;
  try {
    let granted = await isPermissionGranted();
    if (!granted) {
      const permission = await requestPermission();
      granted = permission === "granted";
    }
    if (!granted) {
      toast(t("notification_permission_denied"), "error");
    }
    return granted;
  } catch (e) {
    console.warn("Notification permission check failed:", e);
    return true;
  }
}

/** Returns false when an error toast was shown, so callers don't overwrite it with "success". */
async function rescheduleAfterSettingsChange(): Promise<boolean> {
  try {
    const granted = await ensureNotificationPermission();
    await notificationsRescheduleAll();
    return granted;
  } catch (e) {
    console.error("Notification reschedule failed:", e);
    toast(formatErrorForToast(e, t), "error");
    return false;
  }
}

// --- Budget ---
const budgetInput = ref<number | "">(0);

// --- Notify days ---
const notifyDaysBefore = ref<number | "">(1);

// --- Recurring notifications ---
const recurringNotifications = ref(true);

// --- Notification sound ---
const notificationSound = ref(true);

// --- Notification schedule ---
const notifSchedule = ref("any");
const notifCustomHour = ref<number | "">(9);

// --- Notification templates ---
const notifTitle = ref("");
const notifBodyToday = ref("");
const notifBodySoon = ref("");
const notifOverdueTitle = ref("");
const notifOverdueBody = ref("");

/**
 * Empty stored template = built-in default. On first load show the default text; afterwards keep a
 * field the user just cleared empty (autosave writes "" back, which must not refill it mid-edit).
 */
let templatesLoaded = false;
function templateFieldOrDefault(local: string, stored: string, fallback: string): string {
  if (stored.trim().length > 0) return stored;
  return templatesLoaded && !local.trim() ? local : fallback;
}

watch(
  () => props.lookupSettings,
  (s) => {
    if (!s) return;
    settings.value = s;
    budgetInput.value = toDisplayNumber(parseMoney(s.budget));
    notifyDaysBefore.value = s.notifyDaysBefore;
    recurringNotifications.value = s.recurringNotifications !== false;
    notificationSound.value = s.notificationSound !== false;
    notifSchedule.value = s.notificationSchedule || "any";
    notifCustomHour.value = s.notificationCustomHour ?? 9;
    notifTitle.value = templateFieldOrDefault(
      notifTitle.value,
      s.notificationTitle,
      NOTIFICATION_TEMPLATE_DEFAULTS.notificationTitle,
    );
    notifBodyToday.value = templateFieldOrDefault(
      notifBodyToday.value,
      s.notificationBodyDueToday,
      NOTIFICATION_TEMPLATE_DEFAULTS.notificationBodyDueToday,
    );
    notifBodySoon.value = templateFieldOrDefault(
      notifBodySoon.value,
      s.notificationBodyDueSoon,
      NOTIFICATION_TEMPLATE_DEFAULTS.notificationBodyDueSoon,
    );
    notifOverdueTitle.value = templateFieldOrDefault(
      notifOverdueTitle.value,
      s.notificationOverdueTitle,
      NOTIFICATION_TEMPLATE_DEFAULTS.notificationOverdueTitle,
    );
    notifOverdueBody.value = templateFieldOrDefault(
      notifOverdueBody.value,
      s.notificationOverdueBody,
      NOTIFICATION_TEMPLATE_DEFAULTS.notificationOverdueBody,
    );
    templatesLoaded = true;
  },
  { immediate: true, deep: true },
);

async function updateSettings(updates: Partial<Settings>) {
  if (!settings.value) return;
  const next = { ...settings.value, ...updates };
  settings.value = next;
  await metaStore.updateSettings(next);
}

/** `AppInput type="number"` emits "" while the field is empty — keep it, but never save it. */
function asNumberInput(v: string | number): number | "" {
  return v === "" ? "" : Number(v);
}

// Text/number fields save ~400ms after typing stops; values are captured at edit time so a
// settings refresh in between can't make the debounced save write a stale value.
function onBudgetInput(v: string | number) {
  const val = asNumberInput(v);
  budgetInput.value = val;
  if (val === "" || !Number.isFinite(val) || val < 0) return;
  debounce("budget", () => updateSettings({ budget: moneyFromNumber(val) }));
}

function onNotifyDaysInput(v: string | number) {
  const val = asNumberInput(v);
  notifyDaysBefore.value = val;
  if (val === "" || !Number.isInteger(val) || val < 0 || val > 30) return;
  debounce("notifyDays", async () => {
    await updateSettings({ notifyDaysBefore: val });
    return rescheduleAfterSettingsChange();
  });
}

function toggleRecurring() {
  recurringNotifications.value = !recurringNotifications.value;
  const val = recurringNotifications.value;
  void run(() => updateSettings({ recurringNotifications: val }));
}

function toggleNotificationSound() {
  notificationSound.value = !notificationSound.value;
  const val = notificationSound.value;
  void run(() => updateSettings({ notificationSound: val }));
}
async function handleTestSound() {
  try {
    await notificationsEvent("dispatch", {
      showSystem: false,
      playSound: true,
      forceSound: true,
    });
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

const scheduleOptions = computed<SelectOption[]>(() => [
  { value: "any", label: t("schedule_any") },
  { value: "morning", label: t("schedule_morning") },
  { value: "evening", label: t("schedule_evening") },
  { value: "custom", label: t("schedule_custom") },
]);

type NotificationSchedule = "any" | "morning" | "evening" | "custom";

function saveSchedule(schedule: NotificationSchedule, hour: number) {
  return async () => {
    await updateSettings({ notificationSchedule: schedule, notificationCustomHour: hour });
    return rescheduleAfterSettingsChange();
  };
}

function savedCustomHour(): number {
  const h = notifCustomHour.value;
  return h !== "" && Number.isInteger(h) && h >= 0 && h <= 23 ? h : settings.value?.notificationCustomHour ?? 9;
}

function onScheduleChange(v: string | number) {
  notifSchedule.value = String(v);
  void run(saveSchedule(notifSchedule.value as NotificationSchedule, savedCustomHour()));
}

function onCustomHourInput(v: string | number) {
  const val = asNumberInput(v);
  notifCustomHour.value = val;
  if (val === "" || !Number.isInteger(val) || val < 0 || val > 23) return;
  debounce("schedule", saveSchedule(notifSchedule.value as NotificationSchedule, val));
}

type TemplateField = "notifTitle" | "notifBodyToday" | "notifBodySoon" | "notifOverdueTitle" | "notifOverdueBody";
const templateRefs = { notifTitle, notifBodyToday, notifBodySoon, notifOverdueTitle, notifOverdueBody };

function onTemplateInput(field: TemplateField, v: string | number) {
  templateRefs[field].value = String(v);
  const patch: Partial<Settings> = {
    notificationTitle: notifTitle.value,
    notificationBodyDueToday: notifBodyToday.value,
    notificationBodyDueSoon: notifBodySoon.value,
    notificationOverdueTitle: notifOverdueTitle.value,
    notificationOverdueBody: notifOverdueBody.value,
  };
  debounce("templates", async () => {
    await updateSettings(patch);
    return rescheduleAfterSettingsChange();
  });
}

// --- Test notification ---
const testingSending = ref(false);
async function handleTestNotification() {
  testingSending.value = true;
  try {
    if (!(await ensureNotificationPermission())) return;
    const result = await notificationsEvent<{ system: boolean; sound: boolean; telegram: boolean }>("dispatch", {
      title: "Subly - Test Notification",
      body: "Notifications are working correctly!",
      showSystem: true,
      playSound: true,
      forceSound: false,
    });
    if (result.data.system) {
      toast(t("test_notification_sent"));
    } else {
      toast(t("test_notification_sent_inapp"));
    }
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    testingSending.value = false;
  }
}

// --- Autostart (Rust registers tauri-plugin-autostart only on desktop; iOS/Android → plugin missing) ---
function desktopAutostartAvailable(): boolean {
  try {
    const p = platform();
    return p !== "ios" && p !== "android";
  } catch {
    return false;
  }
}
const autostartSupported = desktopAutostartAvailable();
const autostartEnabled = ref(false);
async function loadAutostartStatus() {
  if (!autostartSupported) return;
  try {
    autostartEnabled.value = await isAutostartEnabled();
  } catch (e) {
    console.error("Autostart check failed:", e);
    toast(formatErrorForToast(e, t), "error");
  }
}
async function toggleAutostart() {
  if (!autostartSupported) return;
  try {
    if (autostartEnabled.value) {
      await disableAutostart();
      autostartEnabled.value = false;
    } else {
      await enableAutostart();
      autostartEnabled.value = true;
    }
    savedAt.value = Date.now();
  } catch (e) {
    console.error("Autostart toggle failed:", e);
    toast(formatErrorForToast(e, t), "error");
  }
}
loadAutostartStatus();
</script>

<template>
  <UiCard :title="t('budget_and_notifications')">
    <template #actions>
      <SavedIndicator :at="savedAt" />
    </template>

    <!-- Monthly budget -->
    <div class="mb-2">
      <AppInput :modelValue="budgetInput" @update:modelValue="onBudgetInput" type="number" min="0" step="0.01" :label="t('monthly_budget')" />
    </div>
    <p class="text-xs text-text-muted mb-5">{{ t('budget_info') }}</p>

    <!-- Notify days -->
    <div class="mb-4">
      <AppInput :modelValue="notifyDaysBefore" @update:modelValue="onNotifyDaysInput" type="number" :label="t('notify_days_before')" min="0" max="30" />
    </div>

    <!-- Recurring notifications -->
    <div class="mb-4">
      <AppToggle
        :modelValue="recurringNotifications"
        @update:modelValue="toggleRecurring"
        :label="t('recurring_notifications')"
        :description="t('recurring_notifications_info')"
      />
    </div>

    <!-- Notification sound -->
    <div class="mb-4 flex items-center gap-3">
      <div class="flex-1">
        <AppToggle
          :modelValue="notificationSound"
          @update:modelValue="toggleNotificationSound"
          :label="t('notification_sound')"
          :description="t('notification_sound_info')"
        />
      </div>
      <Tooltip :text="t('test_sound')">
        <button
          type="button"
          :aria-label="t('test_sound')"
          @click="handleTestSound"
          :class="button({ intent: 'secondary', size: 'icon' })"
        >
          <Volume2 :size="16" />
        </button>
      </Tooltip>
    </div>

    <!-- Notification schedule -->
    <div class="flex flex-wrap gap-2 sm:gap-3 items-end mb-4">
      <div class="flex-1 min-w-[140px]">
        <AppSelect :modelValue="notifSchedule" @update:modelValue="onScheduleChange" :options="scheduleOptions" :label="t('notification_schedule')" />
      </div>
      <div v-if="notifSchedule === 'custom'" class="w-20 sm:w-24">
        <AppInput :modelValue="notifCustomHour" @update:modelValue="onCustomHourInput" type="number" :label="t('hour')" min="0" max="23" />
      </div>
    </div>
    <p class="text-xs text-text-muted mb-5">{{ t('notification_schedule_info') }}</p>

    <!-- Notification templates -->
    <div class="pt-4 border-t border-border mb-4">
      <h3 class="text-sm font-semibold text-text-primary mb-2">{{ t('notification_templates') }}</h3>
      <p class="text-xs text-text-muted mb-2">{{ t('notification_templates_info') }}</p>
      <p class="text-xs text-text-muted mb-3">{{ t('notification_templates_scheduled_hint') }}</p>
      <div class="space-y-3">
        <AppInput
          :modelValue="notifTitle"
          @update:modelValue="(v: string | number) => onTemplateInput('notifTitle', v)"
          :label="t('notification_title_template')"
          :placeholder="NOTIFICATION_TEMPLATE_DEFAULTS.notificationTitle"
          :tooltip="NOTIFICATION_TAGS_TOOLTIP"
          size="sm"
        />
        <AppInput
          :modelValue="notifBodyToday"
          @update:modelValue="(v: string | number) => onTemplateInput('notifBodyToday', v)"
          :label="t('notification_body_due_today')"
          :placeholder="NOTIFICATION_TEMPLATE_DEFAULTS.notificationBodyDueToday"
          :tooltip="NOTIFICATION_TAGS_TOOLTIP"
          size="sm"
        />
        <AppInput
          :modelValue="notifBodySoon"
          @update:modelValue="(v: string | number) => onTemplateInput('notifBodySoon', v)"
          :label="t('notification_body_due_soon')"
          :placeholder="NOTIFICATION_TEMPLATE_DEFAULTS.notificationBodyDueSoon"
          :tooltip="NOTIFICATION_TAGS_TOOLTIP"
          size="sm"
        />
        <AppInput
          :modelValue="notifOverdueTitle"
          @update:modelValue="(v: string | number) => onTemplateInput('notifOverdueTitle', v)"
          :label="t('notification_overdue_title')"
          :placeholder="NOTIFICATION_TEMPLATE_DEFAULTS.notificationOverdueTitle"
          :tooltip="NOTIFICATION_TAGS_TOOLTIP"
          size="sm"
        />
        <AppInput
          :modelValue="notifOverdueBody"
          @update:modelValue="(v: string | number) => onTemplateInput('notifOverdueBody', v)"
          :label="t('notification_overdue_body')"
          :placeholder="NOTIFICATION_TEMPLATE_DEFAULTS.notificationOverdueBody"
          :tooltip="NOTIFICATION_TAGS_TOOLTIP"
          size="sm"
        />
        <div class="flex gap-2">
          <button
            type="button"
            @click="handleTestNotification"
            :disabled="testingSending"
            :class="button({ intent: 'secondary' })"
          >
            <Bell :size="14" />
            {{ t('test_notification') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Autostart (desktop only; plugin not loaded on mobile) -->
    <div v-if="autostartSupported" class="pt-4 border-t border-border">
      <AppToggle :modelValue="autostartEnabled" @update:modelValue="toggleAutostart" :label="t('autostart')" :description="t('autostart_info')" />
    </div>
  </UiCard>
</template>
