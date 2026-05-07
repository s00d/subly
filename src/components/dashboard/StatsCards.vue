<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { CreditCard, TrendingUp, TrendingDown, Wallet, BarChart3, Star } from "@lucide/vue";
import { iconSize, statValue } from "@/lib/tv";

defineProps<{
  activeCount: number;
  totalMonthly: number;
  totalYearly: number;
  amountDue: number;
  avgMonthly: number;
  mostExpensive: { name: string; price: number } | null;
  budgetUsed: number | null;
  totalSavings: number;
  fmt: (amount: number) => string;
}>();

const { t } = useI18n();
const { fmtPercent } = useLocaleFormat();
</script>

<template>
  <div class="grid grid-cols-2 gap-2.5 sm:grid-cols-3 sm:gap-3 lg:grid-cols-4">
    <div class="bg-surface rounded-xl border border-border p-2.5 sm:p-3.5">
      <div class="flex items-center gap-2 sm:gap-3 mb-2">
        <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-lg bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center shrink-0"><CreditCard :size="iconSize.sm" class="text-blue-600 dark:text-blue-400" /></div>
        <span class="text-[10px] sm:text-xs text-text-muted leading-tight">{{ t('active_subscriptions') }}</span>
      </div>
      <p :class="statValue()">{{ activeCount }}</p>
    </div>
    <div class="bg-surface rounded-xl border border-border p-2.5 sm:p-3.5">
      <div class="flex items-center gap-2 sm:gap-3 mb-2">
        <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-lg bg-green-100 dark:bg-green-900/30 flex items-center justify-center shrink-0"><TrendingUp :size="iconSize.sm" class="text-green-600 dark:text-green-400" /></div>
        <span class="text-[10px] sm:text-xs text-text-muted leading-tight">{{ t('monthly_cost') }}</span>
      </div>
      <p :class="statValue()">{{ fmt(totalMonthly) }}</p>
    </div>
    <div class="bg-surface rounded-xl border border-border p-2.5 sm:p-3.5">
      <div class="flex items-center gap-2 sm:gap-3 mb-2">
        <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-lg bg-purple-100 dark:bg-purple-900/30 flex items-center justify-center shrink-0"><Wallet :size="iconSize.sm" class="text-purple-600 dark:text-purple-400" /></div>
        <span class="text-[10px] sm:text-xs text-text-muted leading-tight">{{ t('yearly_cost') }}</span>
      </div>
      <p :class="statValue()">{{ fmt(totalYearly) }}</p>
    </div>
    <div class="bg-surface rounded-xl border border-border p-2.5 sm:p-3.5">
      <div class="flex items-center gap-2 sm:gap-3 mb-2">
        <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-lg bg-orange-100 dark:bg-orange-900/30 flex items-center justify-center shrink-0"><TrendingDown :size="iconSize.sm" class="text-orange-600 dark:text-orange-400" /></div>
        <span class="text-[10px] sm:text-xs text-text-muted leading-tight">{{ t('amount_due') }}</span>
      </div>
      <p :class="statValue()">{{ fmt(amountDue) }}</p>
    </div>
    <div class="bg-surface rounded-xl border border-border p-2.5 sm:p-3.5">
      <div class="flex items-center gap-2 sm:gap-3 mb-2">
        <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-lg bg-cyan-100 dark:bg-cyan-900/30 flex items-center justify-center shrink-0"><BarChart3 :size="iconSize.sm" class="text-cyan-600 dark:text-cyan-400" /></div>
        <span class="text-[10px] sm:text-xs text-text-muted leading-tight">{{ t('average_monthly') }}</span>
      </div>
      <p :class="statValue()">{{ fmt(avgMonthly) }}</p>
    </div>
    <div v-if="mostExpensive" class="bg-surface rounded-xl border border-border p-2.5 sm:p-3.5">
      <div class="flex items-center gap-2 sm:gap-3 mb-2">
        <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-lg bg-yellow-100 dark:bg-yellow-900/30 flex items-center justify-center shrink-0"><Star :size="iconSize.sm" class="text-yellow-600 dark:text-yellow-400" /></div>
        <span class="text-[10px] sm:text-xs text-text-muted leading-tight">{{ t('most_expensive') }}</span>
      </div>
      <p :class="statValue()">{{ fmt(mostExpensive.price) }}</p>
      <p class="text-[10px] sm:text-xs text-primary font-medium truncate">{{ mostExpensive.name }}</p>
    </div>
    <div v-if="budgetUsed !== null" class="bg-surface rounded-xl border border-border p-2.5 sm:p-3.5">
      <div class="flex items-center gap-2 sm:gap-3 mb-2">
        <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-lg bg-rose-100 dark:bg-rose-900/30 flex items-center justify-center shrink-0"><Wallet :size="iconSize.sm" class="text-rose-600 dark:text-rose-400" /></div>
        <span class="text-[10px] sm:text-xs text-text-muted leading-tight">{{ t('percentage_budget_used') }}</span>
      </div>
      <p :class="statValue()">{{ fmtPercent(budgetUsed || 0) }}</p>
    </div>
    <div v-if="totalSavings > 0" class="bg-surface rounded-xl border border-border p-2.5 sm:p-3.5">
      <div class="flex items-center gap-2 sm:gap-3 mb-2">
        <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-lg bg-emerald-100 dark:bg-emerald-900/30 flex items-center justify-center shrink-0"><TrendingDown :size="iconSize.sm" class="text-emerald-600 dark:text-emerald-400" /></div>
        <span class="text-[10px] sm:text-xs text-text-muted leading-tight">{{ t('monthly_savings') }}</span>
      </div>
      <p :class="statValue({ tone: 'green' })">{{ fmt(totalSavings) }}</p>
    </div>
  </div>
</template>
