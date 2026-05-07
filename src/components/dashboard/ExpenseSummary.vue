<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import type { Category, Currency, Expense } from "@/schemas/appData";
import { expenseToIsoDate } from "@/schemas/appData";
import { Wallet, ArrowRight } from "@lucide/vue";
import { useRouter } from "vue-router";

const props = defineProps<{
  categories?: Category[];
  currencies?: Currency[];
  mainCurrencyId?: string;
  agg?: { monthTotal: number; yearTotal: number; recentExpenses: Array<Pick<Expense, "id" | "name" | "categoryId" | "amount" | "currencyId" | "createdAt">> };
}>();
const categories = computed(() => props.categories ?? []);
const currencies = computed(() => props.currencies ?? []);
const mainCurrencyId = computed(() => props.mainCurrencyId ?? "cur-2");
const { t } = useI18n();
const { fmtCurrency, fmtDateShort } = useLocaleFormat();
const router = useRouter();

const agg = computed(
  () =>
    props.agg ?? {
      monthTotal: 0,
      yearTotal: 0,
      recentExpenses: [] as Array<Pick<Expense, "id" | "name" | "categoryId" | "amount" | "currencyId" | "createdAt">>,
    },
);

const mainCode = () => currencies.value.find((c) => c.id === mainCurrencyId.value)?.code || "USD";

function getCategoryName(id: string) {
  return categories.value.find((c) => c.id === id)?.name || "";
}
</script>

<template>
  <div class="rounded-xl bg-surface border border-border p-2.5 sm:p-4">
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <Wallet :size="18" class="text-primary" />
        <h3 class="font-semibold text-text-primary">{{ t('widget_expenses') }}</h3>
      </div>
      <button @click="router.push('/expenses')"
        class="flex items-center gap-1 text-xs text-primary hover:underline">
        {{ t('view_all') }} <ArrowRight :size="12" />
      </button>
    </div>

    <div class="grid grid-cols-2 gap-2.5 mb-3">
      <div class="p-2.5 rounded-lg border border-border bg-surface-secondary">
        <p class="text-xs text-text-muted">{{ t('this_month') }}</p>
        <p class="text-base sm:text-lg font-bold text-text-primary tabular-nums">{{ fmtCurrency(agg.monthTotal, mainCode()) }}</p>
      </div>
      <div class="p-2.5 rounded-lg border border-border bg-surface-secondary">
        <p class="text-xs text-text-muted">{{ t('this_year') }}</p>
        <p class="text-base sm:text-lg font-bold text-text-primary tabular-nums">{{ fmtCurrency(agg.yearTotal, mainCode()) }}</p>
      </div>
    </div>

    <div v-if="agg.recentExpenses.length > 0" class="space-y-2">
      <p class="text-xs font-medium text-text-muted uppercase tracking-wide">{{ t('recent_expenses') }}</p>
      <div class="rounded-lg border border-border bg-surface-secondary overflow-hidden divide-y divide-border">
        <div
          v-for="exp in agg.recentExpenses"
          :key="exp.id"
          class="flex items-center justify-between gap-3 px-3 py-2.5 text-sm transition-colors hover:bg-surface dark:hover:bg-white/6"
        >
          <div class="min-w-0">
            <span class="text-text-primary truncate block">{{ exp.name }}</span>
            <span class="text-xs text-text-muted">{{ fmtDateShort(expenseToIsoDate(exp)) }} · {{ getCategoryName(exp.categoryId) }}</span>
          </div>
          <span class="shrink-0 font-semibold text-text-primary tabular-nums ml-3">
            {{ fmtCurrency(exp.amount, currencies.find(c => c.id === exp.currencyId)?.code || 'USD') }}
          </span>
        </div>
      </div>
    </div>
    <div v-else class="text-center text-sm text-text-muted py-4">
      {{ t('no_expenses_yet') }}
    </div>
  </div>
</template>
