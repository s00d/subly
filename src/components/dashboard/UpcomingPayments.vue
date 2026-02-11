<script setup lang="ts">
import type { Subscription } from "@/schemas/appData";
import { useI18n } from "@/i18n";
import { ArrowRight } from "lucide-vue-next";

defineProps<{
  subscriptions: Subscription[];
  fmt: (amount: number, currencyId?: string) => string;
}>();

const emit = defineEmits<{ navigate: [] }>();
const { t } = useI18n();

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString(undefined, { month: "short", day: "numeric" });
}
</script>

<template>
  <div class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold text-[var(--color-text-primary)]">{{ t('upcoming_payments') }}</h2>
      <button @click="emit('navigate')" class="flex items-center gap-1 text-sm text-[var(--color-primary)] hover:text-[var(--color-primary-hover)] font-medium">
        {{ t('your_subscriptions') }} <ArrowRight :size="14" />
      </button>
    </div>
    <div v-if="subscriptions.length === 0" class="text-sm text-[var(--color-text-muted)] py-4">{{ t('no_upcoming_payments') }}</div>
    <div v-else class="space-y-2">
      <div v-for="sub in subscriptions" :key="sub.id" class="flex items-center gap-3 p-3 rounded-lg hover:bg-[var(--color-surface-hover)]">
        <div class="w-10 h-10 rounded-lg bg-[var(--color-primary-light)] flex items-center justify-center text-sm font-bold text-[var(--color-primary)] shrink-0 overflow-hidden">
          <img v-if="sub.logo" :src="sub.logo" class="w-8 h-8 object-contain rounded" />
          <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-sm font-medium text-[var(--color-text-primary)] truncate">{{ sub.name }}</p>
          <p class="text-xs text-[var(--color-text-muted)]">{{ formatDate(sub.nextPayment) }}</p>
        </div>
        <span class="text-sm font-semibold text-[var(--color-text-primary)]">{{ fmt(sub.price, sub.currencyId) }}</span>
      </div>
    </div>
  </div>
</template>
