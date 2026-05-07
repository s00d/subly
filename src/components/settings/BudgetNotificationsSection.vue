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
import { notificationsEvent } from "@/services/notificationsClient";
import { Bell, Volume2 } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { ui } from "@/lib/tv";
import { formatErrorForToast } from "@/utils/formatError";
import {
  NOTIFICATION_TAGS_TOOLTIP,
  NOTIFICATION_TEMPLATE_DEFAULTS,
} from "@/constants/notificationTemplateDefaults";

const props = defineProps<{ lookupSettings: Settings | null }>();
const { t } = useI18n();
const { toast } = useToast();
const metaStore = useAppMetaStore();
const settings = ref<Settings | null>(null);

// --- Budget ---
const budgetInput = ref(0);

// --- Notify days ---
const notifyDaysBefore = ref(1);

// --- Recurring notifications ---
const recurringNotifications = ref(true);

// --- Notification sound ---
const notificationSound = ref(true);

// --- Notification schedule ---
const notifSchedule = ref("any");
const notifCustomHour = ref(9);

// --- Notification templates ---
const notifTitle = ref("");
const notifBodyToday = ref("");
const notifBodySoon = ref("");
const notifOverdueTitle = ref("");
const notifOverdueBody = ref("");

function templateFieldOrDefault(stored: string, fallback: string): string {
  return stored.trim().length > 0 ? stored : fallback;
}

watch(
  () => props.lookupSettings,
  (s) => {
    if (!s) return;
    settings.value = s;
    budgetInput.value = s.budget;
    notifyDaysBefore.value = s.notifyDaysBefore;
    recurringNotifications.value = s.recurringNotifications !== false;
    notificationSound.value = s.notificationSound !== false;
    notifSchedule.value = s.notificationSchedule || "any";
    notifCustomHour.value = s.notificationCustomHour ?? 9;
    notifTitle.value = templateFieldOrDefault(
      s.notificationTitle,
      NOTIFICATION_TEMPLATE_DEFAULTS.notificationTitle,
    );
    notifBodyToday.value = templateFieldOrDefault(
      s.notificationBodyDueToday,
      NOTIFICATION_TEMPLATE_DEFAULTS.notificationBodyDueToday,
    );
    notifBodySoon.value = templateFieldOrDefault(
      s.notificationBodyDueSoon,
      NOTIFICATION_TEMPLATE_DEFAULTS.notificationBodyDueSoon,
    );
    notifOverdueTitle.value = templateFieldOrDefault(
      s.notificationOverdueTitle,
      NOTIFICATION_TEMPLATE_DEFAULTS.notificationOverdueTitle,
    );
    notifOverdueBody.value = templateFieldOrDefault(
      s.notificationOverdueBody,
      NOTIFICATION_TEMPLATE_DEFAULTS.notificationOverdueBody,
    );
  },
  { immediate: true, deep: true },
);

async function updateSettings(updates: Partial<Settings>) {
  if (!settings.value) return;
  const next = { ...settings.value, ...updates };
  settings.value = next;
  await metaStore.updateSettings(next);
}

function saveBudget() {
  updateSettings({ budget: budgetInput.value });
  toast(t("success"));
}

function saveNotifyDays() {
  updateSettings({ notifyDaysBefore: notifyDaysBefore.value });
  toast(t("success"));
}

function toggleRecurring() {
  recurringNotifications.value = !recurringNotifications.value;
  updateSettings({ recurringNotifications: recurringNotifications.value });
}

function toggleNotificationSound() {
  notificationSound.value = !notificationSound.value;
  updateSettings({ notificationSound: notificationSound.value });
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

function saveSchedule() {
  updateSettings({
    notificationSchedule: notifSchedule.value as "any" | "morning" | "evening" | "custom",
    notificationCustomHour: notifCustomHour.value,
  });
  toast(t("success"));
}

function saveNotifTemplates() {
  updateSettings({
    notificationTitle: notifTitle.value,
    notificationBodyDueToday: notifBodyToday.value,
    notificationBodyDueSoon: notifBodySoon.value,
    notificationOverdueTitle: notifOverdueTitle.value,
    notificationOverdueBody: notifOverdueBody.value,
  });
  toast(t("success"));
}

// --- Test notification ---
const testingSending = ref(false);
async function handleTestNotification() {
  testingSending.value = true;
  try {
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
  } catch {
    toast(t("test_notification_failed"), "error");
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
    toast(t("success"));
  } catch (e) {
    console.error("Autostart toggle failed:", e);
    toast(formatErrorForToast(e, t), "error");
  }
}
loadAutostartStatus();
</script>

<template>
  <section class="bg-surface rounded-xl border border-border p-3 sm:p-5">
    <h2 :class="[ui.sectionTitle(), 'mb-3 sm:mb-4']">{{ t('budget_and_notifications') }}</h2>

    <!-- Monthly budget -->
    <div class="flex gap-2 sm:gap-3 items-end mb-2">
      <div class="flex-1">
        <AppInput v-model="budgetInput" type="number" min="0" step="0.01" :label="t('monthly_budget')" />
      </div>
      <button @click="saveBudget" class="px-3 sm:px-4 py-2 rounded-lg bg-primary text-white text-xs sm:text-sm font-medium hover:bg-primary-hover transition-colors shrink-0">{{ t('save') }}</button>
    </div>
    <p class="text-xs text-text-muted mb-5">{{ t('budget_info') }}</p>

    <!-- Notify days -->
    <div class="flex gap-2 sm:gap-3 items-end mb-4">
      <div class="flex-1">
        <AppInput v-model="notifyDaysBefore" type="number" :label="t('notify_days_before')" min="0" max="30" />
      </div>
      <button @click="saveNotifyDays" class="px-3 sm:px-4 py-2 rounded-lg bg-primary text-white text-xs sm:text-sm font-medium hover:bg-primary-hover transition-colors shrink-0">{{ t('save') }}</button>
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
          @click="handleTestSound"
          class="p-2 rounded-lg border border-border text-text-secondary hover:border-primary hover:text-primary transition-colors"
        >
          <Volume2 :size="16" />
        </button>
      </Tooltip>
    </div>

    <!-- Notification schedule -->
    <div class="flex flex-wrap gap-2 sm:gap-3 items-end mb-4">
      <div class="flex-1 min-w-[140px]">
        <AppSelect v-model="notifSchedule" :options="scheduleOptions" :label="t('notification_schedule')" />
      </div>
      <div v-if="notifSchedule === 'custom'" class="w-20 sm:w-24">
        <AppInput v-model="notifCustomHour" type="number" :label="t('hour')" min="0" max="23" />
      </div>
      <button @click="saveSchedule" class="px-3 sm:px-4 py-2 rounded-lg bg-primary text-white text-xs sm:text-sm font-medium hover:bg-primary-hover transition-colors shrink-0">{{ t('save') }}</button>
    </div>
    <p class="text-xs text-text-muted mb-5">{{ t('notification_schedule_info') }}</p>

    <!-- Notification templates -->
    <div class="pt-4 border-t border-border mb-4">
      <h3 class="text-sm font-semibold text-text-primary mb-2">{{ t('notification_templates') }}</h3>
      <p class="text-xs text-text-muted mb-2">{{ t('notification_templates_info') }}</p>
      <p class="text-xs text-text-muted mb-3">{{ t('notification_templates_scheduled_hint') }}</p>
      <div class="space-y-3">
        <AppInput
          v-model="notifTitle"
          :label="t('notification_title_template')"
          :placeholder="NOTIFICATION_TEMPLATE_DEFAULTS.notificationTitle"
          :tooltip="NOTIFICATION_TAGS_TOOLTIP"
          size="sm"
        />
        <AppInput
          v-model="notifBodyToday"
          :label="t('notification_body_due_today')"
          :placeholder="NOTIFICATION_TEMPLATE_DEFAULTS.notificationBodyDueToday"
          :tooltip="NOTIFICATION_TAGS_TOOLTIP"
          size="sm"
        />
        <AppInput
          v-model="notifBodySoon"
          :label="t('notification_body_due_soon')"
          :placeholder="NOTIFICATION_TEMPLATE_DEFAULTS.notificationBodyDueSoon"
          :tooltip="NOTIFICATION_TAGS_TOOLTIP"
          size="sm"
        />
        <AppInput
          v-model="notifOverdueTitle"
          :label="t('notification_overdue_title')"
          :placeholder="NOTIFICATION_TEMPLATE_DEFAULTS.notificationOverdueTitle"
          :tooltip="NOTIFICATION_TAGS_TOOLTIP"
          size="sm"
        />
        <AppInput
          v-model="notifOverdueBody"
          :label="t('notification_overdue_body')"
          :placeholder="NOTIFICATION_TEMPLATE_DEFAULTS.notificationOverdueBody"
          :tooltip="NOTIFICATION_TAGS_TOOLTIP"
          size="sm"
        />
        <div class="flex gap-2">
          <button @click="saveNotifTemplates" class="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors">{{ t('save') }}</button>
          <button
            @click="handleTestNotification"
            :disabled="testingSending"
            class="flex items-center gap-1.5 px-4 py-2 rounded-lg border border-border text-sm font-medium text-text-secondary hover:border-primary hover:text-primary transition-colors disabled:opacity-50"
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
  </section>
</template>
