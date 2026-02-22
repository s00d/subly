<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useCatalogStore } from "@/stores/catalog";
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { dbGetExpenseAggregations, type ExpenseAggregation } from "@/services/database";
import { Wallet, ArrowRight } from "lucide-vue-next";
import { useRouter } from "vue-router";

const catalogStore = useCatalogStore();
const { t } = useI18n();
const { fmtCurrency, fmtDateShort } = useLocaleFormat();
const router = useRouter();

const now = new Date();
const monthPrefix = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
const yearPrefix = String(now.getFullYear());

const agg = ref<ExpenseAggregation>({ monthTotal: 0, yearTotal: 0, recentExpenses: [] });

onMounted(async () => {
  agg.value = await dbGetExpenseAggregations(monthPrefix, yearPrefix);
});

const mainCode = () => catalogStore.mainCurrency?.code || "USD";

function getCategoryName(id: string) {
  return catalogStore.categories.find((c) => c.id === id)?.name || "";
}
</script>

<template>
  <div class="rounded-xl bg-[var(--color-surface)] border border-[var(--color-border)] p-3 sm:p-5">
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-2">
        <Wallet :size="18" class="text-[var(--color-primary)]" />
        <h3 class="font-semibold text-[var(--color-text-primary)]">{{ t('widget_expenses') }}</h3>
      </div>
      <button @click="router.push('/expenses')"
        class="flex items-center gap-1 text-xs text-[var(--color-primary)] hover:underline">
        {{ t('view_all') }} <ArrowRight :size="12" />
      </button>
    </div>

    <div class="grid grid-cols-2 gap-3 mb-4">
      <div class="p-3 rounded-lg bg-[var(--color-surface-hover)]">
        <p class="text-xs text-[var(--color-text-muted)]">{{ t('this_month') }}</p>
        <p class="text-lg font-bold text-[var(--color-text-primary)]">{{ fmtCurrency(agg.monthTotal, mainCode()) }}</p>
      </div>
      <div class="p-3 rounded-lg bg-[var(--color-surface-hover)]">
        <p class="text-xs text-[var(--color-text-muted)]">{{ t('this_year') }}</p>
        <p class="text-lg font-bold text-[var(--color-text-primary)]">{{ fmtCurrency(agg.yearTotal, mainCode()) }}</p>
      </div>
    </div>

    <div v-if="agg.recentExpenses.length > 0" class="space-y-2">
      <p class="text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wide">{{ t('recent_expenses') }}</p>
      <div v-for="exp in agg.recentExpenses" :key="exp.id"
        class="flex items-center justify-between py-1.5 text-sm">
        <div class="min-w-0">
          <span class="text-[var(--color-text-primary)] truncate block">{{ exp.name }}</span>
          <span class="text-xs text-[var(--color-text-muted)]">{{ fmtDateShort(exp.date) }} · {{ getCategoryName(exp.categoryId) }}</span>
        </div>
        <span class="shrink-0 font-medium text-[var(--color-text-primary)] ml-3">
          {{ fmtCurrency(exp.amount, catalogStore.currencies.find(c => c.id === exp.currencyId)?.code || 'USD') }}
        </span>
      </div>
    </div>
    <div v-else class="text-center text-sm text-[var(--color-text-muted)] py-4">
      {{ t('no_expenses_yet') }}
    </div>
  </div>
</template>
