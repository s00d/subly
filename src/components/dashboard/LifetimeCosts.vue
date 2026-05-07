<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import type { LifetimeCost } from "@/services/dashboardClient";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import { Clock, ChevronDown, ChevronUp } from "@lucide/vue";

const props = defineProps<{
  costs: LifetimeCost[];
  fmt: (n: number) => string;
}>();

const { t } = useI18n();

const expanded = ref(false);

const visibleCosts = computed(() =>
  expanded.value ? props.costs : props.costs.slice(0, 5),
);

const totalLifetime = computed(() =>
  props.costs.reduce((s, c) => s + c.totalPaid, 0),
);
</script>

<template>
  <div class="bg-surface rounded-xl border border-border p-2.5 sm:p-4">
    <div class="flex items-center justify-between mb-2.5 sm:mb-3">
      <div class="flex items-center gap-2">
        <Clock :size="14" class="text-primary shrink-0" />
        <h3 class="text-xs sm:text-sm font-semibold text-text-primary">{{ t('lifetime_costs') }}</h3>
      </div>
      <span class="text-[10px] sm:text-xs font-medium text-primary">{{ t('total') }}: {{ fmt(totalLifetime) }}</span>
    </div>

    <div class="rounded-lg border border-border bg-surface-secondary overflow-hidden divide-y divide-border">
      <div
        v-for="item in visibleCosts"
        :key="item.subscriptionId"
        class="flex items-center gap-2 sm:gap-3 px-3 py-2.5 sm:py-3 transition-colors hover:bg-surface dark:hover:bg-white/6"
      >
        <IconDisplay :icon="item.logo" :size="24" class="sm:[&]:w-7 sm:[&]:h-7 shrink-0" />
        <div class="flex-1 min-w-0">
          <p class="text-xs sm:text-sm font-medium text-text-primary truncate">{{ item.name }}</p>
          <p class="text-[10px] sm:text-[11px] text-text-muted">
            {{ item.monthsActive }} {{ t('months_active') }} · {{ t('monthly') }}: {{ fmt(item.monthlyEquivalent) }}
          </p>
        </div>
        <p class="text-xs sm:text-sm font-bold text-text-primary whitespace-nowrap shrink-0 tabular-nums">{{ fmt(item.totalPaid) }}</p>
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
  </div>
</template>
