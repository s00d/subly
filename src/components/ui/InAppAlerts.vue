<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useClipboard } from "@/composables/useClipboard";
import type { InAppAlert } from "@/services/notificationsClient";
import { Bell, AlertTriangle, Clock, X, Copy } from "@lucide/vue";
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
const { copyToClipboard } = useClipboard();

function alertIcon(type: InAppAlert["type"]) {
  switch (type) {
    case "due_today": return Bell;
    case "upcoming": return Clock;
    case "overdue": return AlertTriangle;
  }
}

function alertLabel(alert: InAppAlert): string {
  if (alert.type === "due_today") return t("due_today_alert");
  if (alert.type === "upcoming") return t("in_days_alert", { days: alert.daysUntil }, alert.daysUntil);
  return t("overdue_by_alert", { days: Math.abs(alert.daysUntil) }, Math.abs(alert.daysUntil));
}

function alertText(alert: InAppAlert): string {
  return `${alert.subscriptionName} — ${alertLabel(alert)} — ${fmt(alert.price, alert.currencyId)}`;
}

async function copyAlert(alert: InAppAlert) {
  const copied = await copyToClipboard(alertText(alert));
  if (copied) {
    toast(t("copied_to_clipboard"));
  } else {
    toast(t("clipboard_copy_failed"), "error");
  }
}

const alertsTv = tv({
  slots: {
    root: "rounded-xl border border-border bg-surface shadow-sm overflow-hidden",
    headerRow: "flex items-center justify-between px-4 pt-3 pb-1.5",
    headerLabel: "text-xs font-semibold text-text-secondary",
    dismissAllBtn: "text-xs font-medium text-text-muted hover:text-text-primary transition-colors",
    alertRow:
      "group flex items-center gap-3 px-4 py-2 text-[13px] text-text-primary border-t border-border first:border-t-0",
    alertIcon: "flex h-6 w-6 shrink-0 items-center justify-center rounded-md",
    actionBtn:
      "p-1 rounded-md text-text-muted opacity-0 group-hover:opacity-100 hover:text-text-primary hover:bg-surface-hover transition shrink-0",
  },
  variants: {
    alertType: {
      due_today: { alertIcon: "bg-warning/14 text-warning" },
      upcoming: { alertIcon: "bg-primary/12 text-primary" },
      overdue: { alertIcon: "bg-danger/12 text-danger" },
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
    >
      <div
        v-for="alert in alerts"
        :key="alert.id"
        :class="alertsTv({ alertType: alert.type }).alertRow()"
      >
        <span :class="alertsTv({ alertType: alert.type }).alertIcon()">
          <component :is="alertIcon(alert.type)" :size="13" />
        </span>
        <div class="flex-1 min-w-0 sm:flex sm:items-baseline sm:gap-2">
          <p class="font-medium truncate">{{ alert.subscriptionName }}</p>
          <p class="text-xs sm:text-[13px] text-text-muted truncate">{{ alertLabel(alert) }}</p>
        </div>
        <span class="font-semibold tabular-nums shrink-0">{{ fmt(alert.price, alert.currencyId) }}</span>
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
