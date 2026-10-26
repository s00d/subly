<script setup lang="ts">
import UiCard from "@/components/ui/UiCard.vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import type { MoneyString } from "@/lib/money";
import { Calculator } from "@lucide/vue";
import { ui, iconSize, statValue } from "@/lib/tv";

const { t } = useI18n();
const { fmt } = useCurrencyFormat();
const props = defineProps<{
  stats?: { avgAmount: MoneyString; count: number; total: MoneyString } | null;
}>();

const stats = computed(() => props.stats ?? null);
</script>

<template>
  <UiCard v-if="stats && stats.count > 0" :title="t('widget_avg_check')" :icon="Calculator" tone="info">
    <div class="grid h-full grid-cols-3 content-center gap-3 text-center">
      <div>
        <p class="text-[10px] sm:text-xs text-text-muted">{{ t('avg_amount') }}</p>
        <p :class="statValue({ tone: 'primary' })">{{ fmt(stats.avgAmount) }}</p>
      </div>
      <div>
        <p class="text-[10px] sm:text-xs text-text-muted">{{ t('total') }}</p>
        <p :class="statValue()">{{ fmt(stats.total) }}</p>
      </div>
      <div>
        <p class="text-[10px] sm:text-xs text-text-muted">{{ t('count') }}</p>
        <p :class="statValue()">{{ stats.count }}</p>
      </div>
    </div>
  </UiCard>
</template>
