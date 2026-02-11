<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "@/i18n";
import type { LifetimeCost } from "@/services/analytics";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import { Clock, ChevronDown, ChevronUp } from "lucide-vue-next";

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
  <div class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-2">
        <Clock :size="16" class="text-[var(--color-primary)]" />
        <h3 class="text-sm font-semibold text-[var(--color-text-primary)]">{{ t('lifetime_costs') }}</h3>
      </div>
      <span class="text-xs font-medium text-[var(--color-primary)]">{{ t('total') }}: {{ fmt(totalLifetime) }}</span>
    </div>

    <div class="space-y-2">
      <div
        v-for="item in visibleCosts"
        :key="item.subscriptionId"
        class="flex items-center gap-3 py-2 px-3 rounded-lg bg-[var(--color-surface-hover)] hover:bg-[var(--color-border)] transition-colors"
      >
        <IconDisplay :icon="item.logo" :size="28" />
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium text-[var(--color-text-primary)] truncate">{{ item.name }}</p>
          <p class="text-[11px] text-[var(--color-text-muted)]">
            {{ item.monthsActive }} {{ t('months_active') }} · {{ t('monthly') }}: {{ fmt(item.monthlyEquivalent) }}
          </p>
        </div>
        <p class="text-sm font-bold text-[var(--color-text-primary)] whitespace-nowrap">{{ fmt(item.totalPaid) }}</p>
      </div>
    </div>

    <button
      v-if="costs.length > 5"
      @click="expanded = !expanded"
      class="w-full mt-3 flex items-center justify-center gap-1 text-xs text-[var(--color-primary)] hover:underline py-1"
    >
      <component :is="expanded ? ChevronUp : ChevronDown" :size="14" />
      {{ expanded ? t('show_less') : t('show_all') }} ({{ costs.length }})
    </button>
  </div>
</template>
