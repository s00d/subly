<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useCatalogStore } from "@/stores/catalog";
import type { CategoryAverage } from "@/services/analytics";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import { Tag } from "lucide-vue-next";

defineProps<{
  averages: CategoryAverage[];
  fmt: (n: number) => string;
}>();

const catalogStore = useCatalogStore();
const { t } = useI18n();

function getCatIcon(id: string): string {
  return catalogStore.categories.find((c) => c.id === id)?.icon || "";
}
</script>

<template>
  <div class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
    <div class="flex items-center gap-2 mb-3 sm:mb-4">
      <Tag :size="14" class="text-[var(--color-primary)] shrink-0" />
      <h3 class="text-xs sm:text-sm font-semibold text-[var(--color-text-primary)]">{{ t('category_averages') }}</h3>
    </div>

    <div class="space-y-2.5 sm:space-y-3">
      <div
        v-for="cat in averages"
        :key="cat.categoryId"
        class="flex items-center gap-2 sm:gap-3"
      >
        <div class="flex-1 min-w-0">
          <div class="flex items-center justify-between mb-1">
            <span class="text-xs sm:text-sm font-medium text-[var(--color-text-primary)] truncate flex items-center gap-1.5">
              <IconDisplay v-if="getCatIcon(cat.categoryId)" :icon="getCatIcon(cat.categoryId)" :size="14" />
              {{ cat.categoryName }}
            </span>
            <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)] whitespace-nowrap ml-2">
              {{ cat.subscriptionCount }} {{ cat.subscriptionCount === 1 ? t('subscription_single') : t('subscriptions') }}
            </span>
          </div>

          <!-- Progress bar showing proportion -->
          <div class="w-full bg-[var(--color-surface-hover)] rounded-full h-1.5 mb-1">
            <div
              class="h-1.5 rounded-full bg-[var(--color-primary)] transition-all"
              :style="{ width: Math.min(100, (cat.totalMonthly / (averages[0]?.totalMonthly || 1)) * 100) + '%' }"
            />
          </div>

          <div class="flex items-center justify-between text-[11px] text-[var(--color-text-muted)]">
            <span>{{ t('total') }}: {{ fmt(cat.totalMonthly) }}/{{ t('mo') }}</span>
            <span>{{ t('avg') }}: {{ fmt(cat.averageMonthly) }}/{{ t('mo') }}</span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="averages.length === 0" class="text-center py-4 text-sm text-[var(--color-text-muted)]">
      {{ t('no_data') }}
    </div>
  </div>
</template>
