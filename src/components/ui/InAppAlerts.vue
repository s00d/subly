<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import type { InAppAlert } from "@/services/notifications";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Bell, AlertTriangle, Clock, X, Copy } from "lucide-vue-next";
import { tv } from "@/lib/tv";

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
    try {
      await navigator.clipboard.writeText(alertText(alert));
      toast(t("copied_to_clipboard"));
    } catch {
      toast(t("error"), "error");
    }
  }
}

const alertsTv = tv({
  slots: {
    root: "space-y-2 mb-4",
    headerRow: "flex items-center justify-between mb-1",
    headerLabel: "text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider",
    dismissAllBtn: "text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] transition-colors",
    alertRow: "flex items-center gap-2 sm:gap-3 px-2.5 sm:px-3 py-2 rounded-lg border text-xs sm:text-sm",
    actionBtn: "p-1 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors shrink-0",
  },
  variants: {
    alertType: {
      due_today: { alertRow: "text-orange-600 bg-orange-50 border-orange-200 dark:bg-orange-900/20 dark:border-orange-800" },
      upcoming: { alertRow: "text-blue-600 bg-blue-50 border-blue-200 dark:bg-blue-900/20 dark:border-blue-800" },
      overdue: { alertRow: "text-red-600 bg-red-50 border-red-200 dark:bg-red-900/20 dark:border-red-800" },
    },
  },
});

const slots = alertsTv();
</script>

<template>
  <div v-if="alerts.length > 0" :class="slots.root()">
    <div :class="slots.headerRow()">
      <span :class="slots.headerLabel()">
        {{ t("notifications") }} ({{ alerts.length }})
      </span>
      <button
        v-if="alerts.length > 1"
        @click="emit('dismissAll')"
        :class="slots.dismissAllBtn()"
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
        :class="alertsTv({ alertType: alert.type }).alertRow()"
      >
        <component :is="alertIcon(alert.type)" :size="14" class="shrink-0 sm:[&]:w-4 sm:[&]:h-4" />
        <div class="flex-1 min-w-0">
          <span class="font-medium truncate block sm:inline">{{ alert.subscriptionName }}</span>
          <span class="hidden sm:inline mx-1.5 opacity-50">·</span>
          <span class="opacity-80 block sm:inline text-[10px] sm:text-sm">{{ alertLabel(alert) }}</span>
          <span class="mx-1 sm:mx-1.5 opacity-50 hidden sm:inline">·</span>
          <span class="font-semibold sm:inline hidden">{{ fmt(alert.price, alert.currencyId) }}</span>
        </div>
        <span class="font-semibold sm:hidden text-[11px] shrink-0">{{ fmt(alert.price, alert.currencyId) }}</span>
        <button @click="copyAlert(alert)" :class="slots.actionBtn()" :title="t('copy')">
          <Copy :size="13" />
        </button>
        <button @click="emit('dismiss', alert.id)" :class="slots.actionBtn()" :title="t('dismiss')">
          <X :size="14" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>
