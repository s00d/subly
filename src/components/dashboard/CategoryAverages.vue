<script setup lang="ts">
import UiCard from "@/components/ui/UiCard.vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { CategoryAverage } from "@/services/dashboardClient";
import { parseMoney, type MoneyString } from "@/lib/money";
import type { Category } from "@/schemas/appData";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import { Tag } from "@lucide/vue";

const props = defineProps<{
  averages: CategoryAverage[];
  fmt: (n: MoneyString) => string;
  categories?: Category[];
}>();

const categories = computed<Category[]>(() => props.categories ?? []);
const { t } = useI18n();

function getCatIcon(id: string): string {
  return categories.value.find((c) => c.id === id)?.icon || "";
}
</script>

<template>
  <UiCard :title="t('category_averages')" :icon="Tag" tone="pink">

    <div class="space-y-2.5 sm:space-y-3">
      <div
        v-for="cat in averages"
        :key="cat.categoryId"
        class="flex items-center gap-2 sm:gap-3"
      >
        <div class="flex-1 min-w-0">
          <div class="flex items-center justify-between mb-1">
            <span class="text-xs sm:text-sm font-medium text-text-primary truncate flex items-center gap-1.5">
              <IconDisplay v-if="getCatIcon(cat.categoryId)" :icon="getCatIcon(cat.categoryId)" :size="14" />
              {{ cat.categoryName }}
            </span>
            <span class="text-[10px] sm:text-xs text-text-muted whitespace-nowrap ml-2">
              {{ cat.subscriptionCount }} {{ cat.subscriptionCount === 1 ? t('subscription_single') : t('subscriptions') }}
            </span>
          </div>

          <!-- Progress bar showing proportion -->
          <div class="w-full bg-surface-hover rounded-full h-1.5 mb-1">
            <div
              class="h-1.5 rounded-full bg-primary transition-all"
              :style="{
                width:
                  Math.min(
                    100,
                    (parseMoney(cat.totalMonthly)
                      .dividedBy(parseMoney(averages[0]?.totalMonthly || '1'))
                      .toNumber()) *
                      100,
                  ) + '%',
              }"
            />
          </div>

          <div class="flex items-center justify-between text-[11px] text-text-muted">
            <span>{{ t('total') }}: {{ fmt(cat.totalMonthly) }}/{{ t('mo') }}</span>
            <span>{{ t('avg') }}: {{ fmt(cat.averageMonthly) }}/{{ t('mo') }}</span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="averages.length === 0" class="text-center py-4 text-sm text-text-muted">
      {{ t('no_data') }}
    </div>
  </UiCard>
</template>
