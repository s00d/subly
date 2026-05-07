<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { Calculator } from "@lucide/vue";
import { ui, iconSize, statValue } from "@/lib/tv";

const { t } = useI18n();
const { fmt } = useCurrencyFormat();
const props = defineProps<{
  stats?: { avgAmount: number; count: number; total: number } | null;
}>();

const stats = computed(() => props.stats ?? null);
</script>

<template>
  <div v-if="stats && stats.count > 0" class="bg-surface rounded-xl border border-border p-2.5 sm:p-4">
    <div class="flex items-center gap-2 mb-3">
      <Calculator :size="iconSize.sm" class="text-primary" />
      <h2 :class="ui.sectionTitle()">{{ t('widget_avg_check') }}</h2>
    </div>
    <div class="grid grid-cols-3 gap-3 text-center">
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
  </div>
</template>
