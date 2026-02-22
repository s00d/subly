<script setup lang="ts">
import type { Subscription } from "@/schemas/appData";
import { AlertTriangle } from "lucide-vue-next";
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
  <div v-if="subscriptions.length > 0" class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-3 sm:p-5">
    <div class="flex items-center gap-2 mb-2 sm:mb-3">
      <AlertTriangle :size="18" class="text-red-500 shrink-0" />
      <h2 class="text-sm sm:text-lg font-semibold text-red-700 dark:text-red-300">{{ t('overdue_renewals') }}</h2>
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2 sm:gap-3">
      <div v-for="sub in subscriptions" :key="sub.id" class="flex items-center gap-2.5 sm:gap-3 bg-white dark:bg-[var(--color-surface)] rounded-lg p-2.5 sm:p-3">
        <div class="w-8 h-8 sm:w-10 sm:h-10 rounded-lg bg-[var(--color-surface-hover)] flex items-center justify-center text-sm sm:text-lg shrink-0 overflow-hidden">
          <img v-if="sub.logo" :src="sub.logo" class="w-6 h-6 sm:w-8 sm:h-8 object-contain rounded" />
          <span v-else>{{ sub.name.charAt(0) }}</span>
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-xs sm:text-sm font-medium text-[var(--color-text-primary)] truncate">{{ sub.name }}</p>
          <p class="text-[10px] sm:text-xs text-red-500">{{ formatDate(sub.nextPayment) }}</p>
        </div>
        <span class="text-xs sm:text-sm font-semibold text-[var(--color-text-primary)] whitespace-nowrap shrink-0">{{ fmt(sub.price, sub.currencyId) }}</span>
      </div>
    </div>
  </div>
</template>
