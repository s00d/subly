<script setup lang="ts">
import type { Subscription } from "@/schemas/appData";
import { AlertTriangle } from "lucide-vue-next";
import { useI18n } from "@/i18n";

defineProps<{
  subscriptions: Subscription[];
  fmt: (amount: number, currencyId?: string) => string;
}>();

const { t } = useI18n();

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString(undefined, { month: "short", day: "numeric" });
}
</script>

<template>
  <div v-if="subscriptions.length > 0" class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-5">
    <div class="flex items-center gap-2 mb-3">
      <AlertTriangle :size="20" class="text-red-500" />
      <h2 class="text-lg font-semibold text-red-700 dark:text-red-300">{{ t('overdue_renewals') }}</h2>
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
      <div v-for="sub in subscriptions" :key="sub.id" class="flex items-center gap-3 bg-white dark:bg-[var(--color-surface)] rounded-lg p-3">
        <div class="w-10 h-10 rounded-lg bg-[var(--color-surface-hover)] flex items-center justify-center text-lg shrink-0 overflow-hidden">
          <img v-if="sub.logo" :src="sub.logo" class="w-8 h-8 object-contain rounded" />
          <span v-else>{{ sub.name.charAt(0) }}</span>
        </div>
        <div class="min-w-0">
          <p class="text-sm font-medium text-[var(--color-text-primary)] truncate">{{ sub.name }}</p>
          <p class="text-xs text-red-500">{{ formatDate(sub.nextPayment) }}</p>
        </div>
        <span class="ml-auto text-sm font-semibold text-[var(--color-text-primary)] whitespace-nowrap">{{ fmt(sub.price, sub.currencyId) }}</span>
      </div>
    </div>
  </div>
</template>
