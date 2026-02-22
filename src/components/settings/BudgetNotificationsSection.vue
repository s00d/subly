<script setup lang="ts">
import { ref, computed } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import AppToggle from "@/components/ui/AppToggle.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import { enable as enableAutostart, disable as disableAutostart, isEnabled as isAutostartEnabled } from "@tauri-apps/plugin-autostart";
import { sendTestNotification } from "@/services/notifications";
import { playNotificationSound } from "@/services/sound";
import { Bell, Volume2 } from "lucide-vue-next";
import Tooltip from "@/components/ui/Tooltip.vue";

const settingsStore = useSettingsStore();
const { t } = useI18n();
const { toast } = useToast();

// --- Budget ---
const budgetInput = ref(settingsStore.settings.budget);
function saveBudget() {
  settingsStore.updateSettings({ budget: budgetInput.value });
  toast(t("success"));
}

// --- Notify days ---
const notifyDaysBefore = ref(settingsStore.settings.notifyDaysBefore);
function saveNotifyDays() {
  settingsStore.updateSettings({ notifyDaysBefore: notifyDaysBefore.value });
  toast(t("success"));
}

// --- Recurring notifications ---
const recurringNotifications = ref(settingsStore.settings.recurringNotifications !== false);
function toggleRecurring() {
  recurringNotifications.value = !recurringNotifications.value;
  settingsStore.updateSettings({ recurringNotifications: recurringNotifications.value });
}

// --- Notification sound ---
const notificationSound = ref(settingsStore.settings.notificationSound !== false);
function toggleNotificationSound() {
  notificationSound.value = !notificationSound.value;
  settingsStore.updateSettings({ notificationSound: notificationSound.value });
}
function handleTestSound() {
  playNotificationSound();
}

// --- Notification schedule ---
const notifSchedule = ref(settingsStore.settings.notificationSchedule || "any");
const notifCustomHour = ref(settingsStore.settings.notificationCustomHour ?? 9);

const scheduleOptions = computed<SelectOption[]>(() => [
  { value: "any", label: t("schedule_any") },
  { value: "morning", label: t("schedule_morning") },
  { value: "evening", label: t("schedule_evening") },
  { value: "custom", label: t("schedule_custom") },
]);

function saveSchedule() {
  settingsStore.updateSettings({
    notificationSchedule: notifSchedule.value as "any" | "morning" | "evening" | "custom",
    notificationCustomHour: notifCustomHour.value,
  });
  toast(t("success"));
}

// --- Notification templates ---
const notifTitle = ref(settingsStore.settings.notificationTitle);
const notifBodyToday = ref(settingsStore.settings.notificationBodyDueToday);
const notifBodySoon = ref(settingsStore.settings.notificationBodyDueSoon);
const notifOverdueTitle = ref(settingsStore.settings.notificationOverdueTitle);
const notifOverdueBody = ref(settingsStore.settings.notificationOverdueBody);

function saveNotifTemplates() {
  settingsStore.updateSettings({
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
    const result = await sendTestNotification();
    // Play sound if enabled
    if (settingsStore.settings.notificationSound !== false) {
      playNotificationSound();
    }
    if (result.system) {
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

// --- Autostart ---
const autostartEnabled = ref(false);
async function loadAutostartStatus() {
  try { autostartEnabled.value = await isAutostartEnabled(); }
  catch (e) { console.warn("Autostart check failed:", e); }
}
async function toggleAutostart() {
  try {
    if (autostartEnabled.value) { await disableAutostart(); autostartEnabled.value = false; }
    else { await enableAutostart(); autostartEnabled.value = true; }
    toast(t("success"));
  } catch (e) {
    console.error("Autostart toggle failed:", e);
    toast(t("error"), "error");
  }
}
loadAutostartStatus();
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
    <h2 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)] mb-3 sm:mb-4">{{ t('budget_and_notifications') }}</h2>

    <!-- Monthly budget -->
    <div class="flex gap-2 sm:gap-3 items-end mb-2">
      <div class="flex-1">
        <AppInput v-model="budgetInput" type="number" min="0" step="0.01" :label="t('monthly_budget')" />
      </div>
      <button @click="saveBudget" class="px-3 sm:px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-xs sm:text-sm font-medium hover:bg-[var(--color-primary-hover)] transition-colors shrink-0">{{ t('save') }}</button>
    </div>
    <p class="text-xs text-[var(--color-text-muted)] mb-5">{{ t('budget_info') }}</p>

    <!-- Notify days -->
    <div class="flex gap-2 sm:gap-3 items-end mb-4">
      <div class="flex-1">
        <AppInput v-model="notifyDaysBefore" type="number" :label="t('notify_days_before')" min="0" max="30" />
      </div>
      <button @click="saveNotifyDays" class="px-3 sm:px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-xs sm:text-sm font-medium hover:bg-[var(--color-primary-hover)] transition-colors shrink-0">{{ t('save') }}</button>
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
          class="p-2 rounded-lg border border-[var(--color-border)] text-[var(--color-text-secondary)] hover:border-[var(--color-primary)] hover:text-[var(--color-primary)] transition-colors"
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
      <button @click="saveSchedule" class="px-3 sm:px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-xs sm:text-sm font-medium hover:bg-[var(--color-primary-hover)] transition-colors shrink-0">{{ t('save') }}</button>
    </div>
    <p class="text-xs text-[var(--color-text-muted)] mb-5">{{ t('notification_schedule_info') }}</p>

    <!-- Notification templates -->
    <div class="pt-4 border-t border-[var(--color-border)] mb-4">
      <h3 class="text-sm font-semibold text-[var(--color-text-primary)] mb-2">{{ t('notification_templates') }}</h3>
      <p class="text-xs text-[var(--color-text-muted)] mb-3">{{ t('notification_templates_info') }}</p>
      <div class="space-y-3">
        <AppInput v-model="notifTitle" :label="t('notification_title_template')" size="sm" />
        <AppInput v-model="notifBodyToday" :label="t('notification_body_due_today')" size="sm" />
        <AppInput v-model="notifBodySoon" :label="t('notification_body_due_soon')" size="sm" />
        <AppInput v-model="notifOverdueTitle" :label="t('notification_overdue_title')" size="sm" />
        <AppInput v-model="notifOverdueBody" :label="t('notification_overdue_body')" size="sm" />
        <div class="flex gap-2">
          <button @click="saveNotifTemplates" class="px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:bg-[var(--color-primary-hover)] transition-colors">{{ t('save') }}</button>
          <button
            @click="handleTestNotification"
            :disabled="testingSending"
            class="flex items-center gap-1.5 px-4 py-2 rounded-lg border border-[var(--color-border)] text-sm font-medium text-[var(--color-text-secondary)] hover:border-[var(--color-primary)] hover:text-[var(--color-primary)] transition-colors disabled:opacity-50"
          >
            <Bell :size="14" />
            {{ t('test_notification') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Autostart -->
    <div class="pt-4 border-t border-[var(--color-border)]">
      <AppToggle :modelValue="autostartEnabled" @update:modelValue="toggleAutostart" :label="t('autostart')" :description="t('autostart_info')" />
    </div>
  </section>
</template>
