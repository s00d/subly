<script setup lang="ts">
import { useI18n } from "@/i18n";
import { useToast } from "@/composables/useToast";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import type { InAppAlert } from "@/services/notifications";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Bell, AlertTriangle, Clock, X, Copy } from "lucide-vue-next";

defineProps<{
  alerts: InAppAlert[];
}>();

const emit = defineEmits<{
  dismiss: [id: string];
  dismissAll: [];
}>();

const { t } = useI18n();
const { fmt } = useCurrencyFormat();
const { toast } = useToast();

function alertIcon(type: InAppAlert["type"]) {
  switch (type) {
    case "due_today": return Bell;
    case "upcoming": return Clock;
    case "overdue": return AlertTriangle;
  }
}

function alertColor(type: InAppAlert["type"]) {
  switch (type) {
    case "due_today": return "text-orange-600 bg-orange-50 border-orange-200 dark:bg-orange-900/20 dark:border-orange-800";
    case "upcoming": return "text-blue-600 bg-blue-50 border-blue-200 dark:bg-blue-900/20 dark:border-blue-800";
    case "overdue": return "text-red-600 bg-red-50 border-red-200 dark:bg-red-900/20 dark:border-red-800";
  }
}

function alertLabel(alert: InAppAlert): string {
  if (alert.type === "due_today") return t("due_today_alert");
  if (alert.type === "upcoming") return t("in_days_alert").replace("{days}", String(alert.daysUntil));
  return t("overdue_by_alert").replace("{days}", String(Math.abs(alert.daysUntil)));
}

function alertText(alert: InAppAlert): string {
  return `${alert.subscriptionName} — ${alertLabel(alert)} — ${fmt(alert.price, alert.currencyId)}`;
}

async function copyAlert(alert: InAppAlert) {
  try {
    await writeText(alertText(alert));
    toast(t("copied_to_clipboard"));
  } catch {
    // fallback
    try {
      await navigator.clipboard.writeText(alertText(alert));
      toast(t("copied_to_clipboard"));
    } catch {
      toast(t("error"), "error");
    }
  }
}
</script>

<template>
  <div v-if="alerts.length > 0" class="space-y-2 mb-4">
    <div class="flex items-center justify-between mb-1">
      <span class="text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider">
        {{ t("notifications") }} ({{ alerts.length }})
      </span>
      <button
        v-if="alerts.length > 1"
        @click="emit('dismissAll')"
        class="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] transition-colors"
      >
        {{ t("dismiss_all") }}
      </button>
    </div>

    <TransitionGroup
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 translate-y-1"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0 -translate-x-4"
      tag="div"
      class="space-y-1.5"
    >
      <div
        v-for="alert in alerts"
        :key="alert.id"
        class="flex items-center gap-3 px-3 py-2 rounded-lg border text-sm"
        :class="alertColor(alert.type)"
      >
        <component :is="alertIcon(alert.type)" :size="16" class="shrink-0" />
        <div class="flex-1 min-w-0">
          <span class="font-medium truncate">{{ alert.subscriptionName }}</span>
          <span class="mx-1.5 opacity-50">·</span>
          <span class="opacity-80">{{ alertLabel(alert) }}</span>
          <span class="mx-1.5 opacity-50">·</span>
          <span class="font-semibold">{{ fmt(alert.price, alert.currencyId) }}</span>
        </div>
        <button
          @click="copyAlert(alert)"
          class="p-1 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors shrink-0"
          :title="t('copy')"
        >
          <Copy :size="13" />
        </button>
        <button
          @click="emit('dismiss', alert.id)"
          class="p-1 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors shrink-0"
          :title="t('dismiss')"
        >
          <X :size="14" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>
