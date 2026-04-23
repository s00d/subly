<script setup lang="ts">
import { useI18n } from "vue-i18n";

defineProps<{
  count: number;
  totalCost: string;
  totalDue: string;
  compact?: boolean;
}>();

const { t } = useI18n();
</script>

<template>
  <div class="bg-surface rounded-xl border border-border" :class="compact ? 'mt-3 sm:mt-4 p-3' : 'mt-4 sm:mt-6 p-3 sm:p-5'">
    <h3 class="text-xs sm:text-sm font-semibold text-text-primary" :class="compact ? 'mb-2' : 'mb-2 sm:mb-3'">{{ t('stats') }}</h3>
    <div class="grid grid-cols-3" :class="compact ? 'gap-2' : 'gap-2 sm:gap-4'">
      <div>
        <p class="text-[10px] sm:text-sm text-text-muted">{{ t('active_subscriptions') }}</p>
        <p class="font-bold text-text-primary" :class="compact ? 'text-base sm:text-lg' : 'text-lg sm:text-xl'">{{ count }}</p>
      </div>
      <div>
        <p class="text-[10px] sm:text-sm text-text-muted">{{ t('total_cost') }}</p>
        <p class="font-bold text-text-primary" :class="compact ? 'text-base sm:text-lg' : 'text-lg sm:text-xl'">{{ totalCost }}</p>
      </div>
      <div>
        <p class="text-[10px] sm:text-sm text-text-muted">{{ t('amount_due') }}</p>
        <p class="font-bold text-text-primary" :class="compact ? 'text-base sm:text-lg' : 'text-lg sm:text-xl'">{{ totalDue }}</p>
      </div>
    </div>
  </div>
</template>
