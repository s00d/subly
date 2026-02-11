<script setup lang="ts">
import { ref, computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import AppToggle from "@/components/ui/AppToggle.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import { enable as enableAutostart, disable as disableAutostart, isEnabled as isAutostartEnabled } from "@tauri-apps/plugin-autostart";
import { sendTestNotification } from "@/services/notifications";
import { Bell } from "lucide-vue-next";

const store = useAppStore();
const { t } = useI18n();
const { toast } = useToast();

// --- Budget ---
const budgetInput = ref(store.state.settings.budget);
function saveBudget() {
  store.updateSettings({ budget: budgetInput.value });
  toast(t("success"));
}

// --- Notify days ---
const notifyDaysBefore = ref(store.state.settings.notifyDaysBefore);
function saveNotifyDays() {
  store.updateSettings({ notifyDaysBefore: notifyDaysBefore.value });
  toast(t("success"));
}

// --- Recurring notifications ---
const recurringNotifications = ref(store.state.settings.recurringNotifications !== false);
function toggleRecurring() {
  recurringNotifications.value = !recurringNotifications.value;
  store.updateSettings({ recurringNotifications: recurringNotifications.value });
}

// --- Notification schedule ---
const notifSchedule = ref(store.state.settings.notificationSchedule || "any");
const notifCustomHour = ref(store.state.settings.notificationCustomHour ?? 9);

const scheduleOptions = computed<SelectOption[]>(() => [
  { value: "any", label: t("schedule_any") },
  { value: "morning", label: t("schedule_morning") },
  { value: "evening", label: t("schedule_evening") },
  { value: "custom", label: t("schedule_custom") },
]);

function saveSchedule() {
  store.updateSettings({
    notificationSchedule: notifSchedule.value as "any" | "morning" | "evening" | "custom",
    notificationCustomHour: notifCustomHour.value,
  });
  toast(t("success"));
}

// --- Notification templates ---
const notifTitle = ref(store.state.settings.notificationTitle);
const notifBodyToday = ref(store.state.settings.notificationBodyDueToday);
const notifBodySoon = ref(store.state.settings.notificationBodyDueSoon);
const notifOverdueTitle = ref(store.state.settings.notificationOverdueTitle);
const notifOverdueBody = ref(store.state.settings.notificationOverdueBody);

function saveNotifTemplates() {
  store.updateSettings({
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
    if (result.system) {
      toast(t("test_notification_sent"));
    } else {
      // System notification unavailable — show in-app only
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
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
    <h2 class="text-lg font-semibold text-[var(--color-text-primary)] mb-4">{{ t('budget_and_notifications') }}</h2>

    <!-- Monthly budget -->
    <div class="flex gap-3 items-end mb-2">
      <div class="flex-1">
        <AppInput v-model="budgetInput" type="number" min="0" step="0.01" :label="t('monthly_budget')" />
      </div>
      <button @click="saveBudget" class="px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:bg-[var(--color-primary-hover)] transition-colors">{{ t('save') }}</button>
    </div>
    <p class="text-xs text-[var(--color-text-muted)] mb-5">{{ t('budget_info') }}</p>

    <!-- Notify days -->
    <div class="flex gap-3 items-end mb-4">
      <div class="flex-1">
        <AppInput v-model="notifyDaysBefore" type="number" :label="t('notify_days_before')" min="0" max="30" />
      </div>
      <button @click="saveNotifyDays" class="px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:bg-[var(--color-primary-hover)] transition-colors">{{ t('save') }}</button>
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

    <!-- Notification schedule -->
    <div class="flex gap-3 items-end mb-4">
      <div class="flex-1">
        <AppSelect v-model="notifSchedule" :options="scheduleOptions" :label="t('notification_schedule')" />
      </div>
      <div v-if="notifSchedule === 'custom'" class="w-24">
        <AppInput v-model="notifCustomHour" type="number" :label="t('hour')" min="0" max="23" />
      </div>
      <button @click="saveSchedule" class="px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:bg-[var(--color-primary-hover)] transition-colors">{{ t('save') }}</button>
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
