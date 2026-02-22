<script setup lang="ts">
import { useI18n } from "vue-i18n";

defineProps<{
  count: number;
  totalCost: string;
  totalDue: string;
}>();

const { t } = useI18n();
</script>

<template>
  <div class="mt-4 sm:mt-6 bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
    <h3 class="text-xs sm:text-sm font-semibold text-[var(--color-text-primary)] mb-2 sm:mb-3">{{ t('stats') }}</h3>
    <div class="grid grid-cols-3 gap-2 sm:gap-4">
      <div>
        <p class="text-[10px] sm:text-sm text-[var(--color-text-muted)]">{{ t('active_subscriptions') }}</p>
        <p class="text-lg sm:text-xl font-bold text-[var(--color-text-primary)]">{{ count }}</p>
      </div>
      <div>
        <p class="text-[10px] sm:text-sm text-[var(--color-text-muted)]">{{ t('total_cost') }}</p>
        <p class="text-lg sm:text-xl font-bold text-[var(--color-text-primary)]">{{ totalCost }}</p>
      </div>
      <div>
        <p class="text-[10px] sm:text-sm text-[var(--color-text-muted)]">{{ t('amount_due') }}</p>
        <p class="text-lg sm:text-xl font-bold text-[var(--color-text-primary)]">{{ totalDue }}</p>
      </div>
    </div>
  </div>
</template>
