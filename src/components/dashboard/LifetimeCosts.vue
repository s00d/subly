<script setup lang="ts">
import { avatarTone } from "@/lib/tv";
import UiCard from "@/components/ui/UiCard.vue";
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import type { LifetimeCost } from "@/services/dashboardClient";
import { moneyFromDecimal, sumMoney, type MoneyString } from "@/lib/money";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import { Clock, ChevronDown, ChevronUp } from "@lucide/vue";

const props = defineProps<{
  costs: LifetimeCost[];
  fmt: (n: MoneyString) => string;
}>();

const { t } = useI18n();

const expanded = ref(false);

const visibleCosts = computed(() =>
  expanded.value ? props.costs : props.costs.slice(0, 5),
);

const totalLifetime = computed(() => sumMoney(props.costs.map((c) => c.totalPaid)));
</script>

<template>
  <UiCard :title="t('lifetime_costs')" :icon="Clock" tone="violet">
    <template #actions>
    <span class="text-xs font-medium text-text-secondary tabular-nums">{{ t('total') }}: {{ fmt(moneyFromDecimal(totalLifetime)) }}</span>
    </template>

    <div class="-mx-2 divide-y divide-border">
      <div
        v-for="item in visibleCosts"
        :key="item.subscriptionId"
        class="flex items-center gap-3 px-2 py-2.5 transition-colors rounded-lg hover:bg-surface-hover"
      >
        <div
          class="w-8 h-8 rounded-[9px] flex items-center justify-center text-[13px] font-semibold shrink-0 overflow-hidden"
          :class="item.logo ? 'bg-surface ring-1 ring-border' : avatarTone(item.name)"
        >
          <IconDisplay v-if="item.logo" :icon="item.logo" :size="22" />
          <span v-else>{{ item.name.charAt(0).toUpperCase() }}</span>
        </div>
        <div class="flex-1 min-w-0">
          <p class="text-[13px] sm:text-sm font-medium text-text-primary truncate">{{ item.name }}</p>
          <p class="text-[11px] text-text-muted">
            {{ item.monthsActive }} {{ t('months_active') }} · {{ t('monthly') }}: {{ fmt(item.monthlyEquivalent) }}
          </p>
        </div>
        <p class="text-[13px] sm:text-sm font-semibold text-text-primary whitespace-nowrap shrink-0 tabular-nums">{{ fmt(item.totalPaid) }}</p>
      </div>
    </div>

    <button
      v-if="costs.length > 5"
      type="button"
      @click="expanded = !expanded"
      class="w-full mt-3 flex items-center justify-center gap-1 text-xs text-primary hover:underline py-1.5 rounded-lg hover:bg-surface-hover/50 transition-colors"
    >
      <component :is="expanded ? ChevronUp : ChevronDown" :size="14" />
      {{ expanded ? t('show_less') : t('show_all') }} ({{ costs.length }})
    </button>
  </UiCard>
</template>
