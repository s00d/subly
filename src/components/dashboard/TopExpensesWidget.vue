<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import type { Category, Expense } from "@/schemas/appData";
import { expenseToIsoDate } from "@/schemas/appData";
import { Receipt } from "@lucide/vue";

const props = defineProps<{
  categories?: Category[];
  items?: Array<Pick<Expense, "id" | "name" | "categoryId" | "amount" | "currencyId" | "createdAt">>;
}>();
const { t } = useI18n();
const { fmt } = useCurrencyFormat();
const categories = computed(() => props.categories ?? []);
const items = computed(() => (props.items ?? []).slice(0, 5));

function catName(id: string): string {
  return categories.value.find((c) => c.id === id)?.name ?? "";
}
</script>

<template>
  <div v-if="items.length > 0" class="bg-surface rounded-xl border border-border p-2.5 sm:p-4">
    <div class="flex items-center gap-2 mb-2.5">
      <Receipt :size="16" class="text-red-500" />
      <h2 class="text-sm sm:text-lg font-semibold text-text-primary">{{ t('widget_top_expenses') }}</h2>
    </div>
    <div class="space-y-2">
      <div
        v-for="(exp, i) in items"
        :key="exp.id"
        class="flex items-center gap-3 rounded-lg px-2 py-1.5"
      >
        <span class="text-xs font-bold text-text-muted w-5 text-center tabular-nums">{{ i + 1 }}</span>
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium text-text-primary truncate">{{ exp.name }}</p>
          <p class="text-[10px] text-text-muted">{{ catName(exp.categoryId) }} · {{ expenseToIsoDate(exp) }}</p>
        </div>
        <span class="text-sm font-bold text-text-primary tabular-nums shrink-0">{{ fmt(exp.amount, exp.currencyId) }}</span>
      </div>
    </div>
  </div>
</template>
