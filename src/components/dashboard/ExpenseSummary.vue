<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { Wallet, TrendingUp, ArrowRight } from "lucide-vue-next";
import { useRouter } from "vue-router";

const store = useAppStore();
const { t } = useI18n();
const router = useRouter();

const now = new Date();
const currentMonth = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
const currentYear = String(now.getFullYear());

function convertToMain(amount: number, currencyId: string): number {
  const cur = store.state.currencies.find((c) => c.id === currencyId);
  const main = store.mainCurrency.value;
  if (!cur || !main || cur.id === main.id) return amount;
  if (cur.rate && main.rate) return amount * main.rate / cur.rate;
  return amount;
}

const monthExpenses = computed(() =>
  store.state.expenses
    .filter((e) => e.date.startsWith(currentMonth))
    .reduce((s, e) => s + convertToMain(e.amount, e.currencyId), 0)
);

const yearExpenses = computed(() =>
  store.state.expenses
    .filter((e) => e.date.startsWith(currentYear))
    .reduce((s, e) => s + convertToMain(e.amount, e.currencyId), 0)
);

const recentExpenses = computed(() =>
  [...store.state.expenses]
    .sort((a, b) => b.date.localeCompare(a.date))
    .slice(0, 5)
);

const symbol = computed(() => store.mainCurrency.value?.symbol || "$");

function getCategoryName(id: string) {
  return store.state.categories.find((c) => c.id === id)?.name || "";
}
</script>

<template>
  <div class="rounded-xl bg-[var(--color-surface)] border border-[var(--color-border)] p-4 sm:p-5">
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

    <!-- Summary cards -->
    <div class="grid grid-cols-2 gap-3 mb-4">
      <div class="p-3 rounded-lg bg-[var(--color-surface-hover)]">
        <p class="text-xs text-[var(--color-text-muted)]">{{ t('this_month') }}</p>
        <p class="text-lg font-bold text-[var(--color-text-primary)]">{{ symbol }}{{ monthExpenses.toFixed(2) }}</p>
      </div>
      <div class="p-3 rounded-lg bg-[var(--color-surface-hover)]">
        <p class="text-xs text-[var(--color-text-muted)]">{{ t('this_year') }}</p>
        <p class="text-lg font-bold text-[var(--color-text-primary)]">{{ symbol }}{{ yearExpenses.toFixed(2) }}</p>
      </div>
    </div>

    <!-- Recent list -->
    <div v-if="recentExpenses.length > 0" class="space-y-2">
      <p class="text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wide">{{ t('recent_expenses') }}</p>
      <div v-for="exp in recentExpenses" :key="exp.id"
        class="flex items-center justify-between py-1.5 text-sm">
        <div class="min-w-0">
          <span class="text-[var(--color-text-primary)] truncate block">{{ exp.name }}</span>
          <span class="text-xs text-[var(--color-text-muted)]">{{ exp.date }} · {{ getCategoryName(exp.categoryId) }}</span>
        </div>
        <span class="shrink-0 font-medium text-[var(--color-text-primary)] ml-3">
          {{ store.state.currencies.find(c => c.id === exp.currencyId)?.symbol }}{{ exp.amount.toFixed(2) }}
        </span>
      </div>
    </div>
    <div v-else class="text-center text-sm text-[var(--color-text-muted)] py-4">
      {{ t('no_expenses_yet') }}
    </div>
  </div>
</template>
