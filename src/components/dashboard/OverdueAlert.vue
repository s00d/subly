<script setup lang="ts">
import type { Subscription } from "@/schemas/appData";
import { AlertTriangle } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";

defineProps<{
  subscriptions: Subscription[];
  fmt: (amount: number, currencyId?: string) => string;
}>();

const { t } = useI18n();
const { fmtDateShort: formatDate } = useLocaleFormat();
</script>

<template>
  <div v-if="subscriptions.length > 0" class="bg-surface rounded-xl border border-border p-2.5 sm:p-4">
    <div class="flex items-center gap-2 mb-2.5 sm:mb-3">
      <AlertTriangle :size="16" class="text-red-500 shrink-0" />
      <h2 class="text-sm sm:text-lg font-semibold text-text-primary">{{ t('overdue_renewals') }}</h2>
      <span class="ml-auto inline-flex items-center rounded-full bg-red-100 px-2 py-0.5 text-[10px] sm:text-xs font-medium text-red-700 dark:bg-red-900/30 dark:text-red-300">
        {{ subscriptions.length }}
      </span>
    </div>
    <div class="rounded-lg border border-border bg-surface-secondary overflow-hidden divide-y divide-border">
      <div
        v-for="sub in subscriptions"
        :key="sub.id"
        class="flex items-center gap-2 sm:gap-3 px-3 py-2.5 sm:py-3 transition-colors hover:bg-surface dark:hover:bg-white/6"
      >
        <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-lg bg-red-100 dark:bg-red-900/30 flex items-center justify-center text-xs sm:text-sm font-bold text-red-600 dark:text-red-300 shrink-0 overflow-hidden">
          <img v-if="sub.logo" :src="sub.logo" class="w-6 h-6 sm:w-7 sm:h-7 object-contain rounded" />
          <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-xs sm:text-sm font-medium text-text-primary truncate">{{ sub.name }}</p>
          <p class="text-[10px] sm:text-[11px] text-red-500 dark:text-red-400">{{ formatDate(sub.nextPayment) }}</p>
        </div>
        <span class="text-xs sm:text-sm font-semibold text-text-primary tabular-nums whitespace-nowrap shrink-0">
          {{ fmt(sub.price, sub.currencyId) }}
        </span>
      </div>
    </div>
  </div>
</template>
