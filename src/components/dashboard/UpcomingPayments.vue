<script setup lang="ts">
import UiCard from "@/components/ui/UiCard.vue";
import type { Subscription } from "@/schemas/appData";
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import type { MoneyString } from "@/lib/money";
import { ArrowRight, CalendarClock } from "@lucide/vue";
import { avatarTone } from "@/lib/tv";

defineProps<{
  subscriptions: Subscription[];
  fmt: (amount: MoneyString, currencyId?: string) => string;
}>();

const emit = defineEmits<{ navigate: [] }>();
const { t } = useI18n();
const { fmtDateShort: formatDate } = useLocaleFormat();
</script>

<template>
  <UiCard :title="t('upcoming_payments')" :icon="CalendarClock" tone="primary">
    <template #actions>
    <button @click="emit('navigate')" class="inline-flex items-center gap-1 text-xs font-medium text-primary hover:text-primary-hover">
      {{ t('your_subscriptions') }} <ArrowRight :size="14" />
    </button>
    </template>
    <div v-if="subscriptions.length === 0" class="text-xs text-text-muted py-6 text-center">{{ t('no_upcoming_payments') }}</div>
    <div v-else class="-mx-2 divide-y divide-border">
      <div
        v-for="sub in subscriptions"
        :key="sub.id"
        class="flex items-center gap-3 px-2 py-2.5 transition-colors rounded-lg hover:bg-surface-hover"
      >
        <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-[10px] flex items-center justify-center text-sm font-semibold shrink-0 overflow-hidden" :class="sub.logo ? 'bg-surface ring-1 ring-border' : avatarTone(sub.name)">
          <img v-if="sub.logo" :src="sub.logo" class="w-6 h-6 sm:w-7 sm:h-7 object-contain rounded" />
          <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-xs sm:text-sm font-medium text-text-primary truncate">{{ sub.name }}</p>
          <p class="text-[10px] sm:text-[11px] text-text-muted">{{ formatDate(sub.nextPayment) }}</p>
        </div>
        <span class="text-xs sm:text-sm font-semibold text-text-primary tabular-nums whitespace-nowrap shrink-0">
          {{ fmt(sub.price, sub.currencyId) }}
        </span>
      </div>
    </div>
  </UiCard>
</template>
