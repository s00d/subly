<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { CreditCard, TrendingUp, TrendingDown, Wallet, BarChart3, Star } from "@lucide/vue";
import { iconSize } from "@/lib/tv";

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

const tileWrap =
  "bg-surface rounded-2xl border border-border p-4 shadow-sm transition-shadow duration-200 hover:shadow-md";
const tileHead = "flex items-center gap-3 mb-3";
const tileLabel = "text-xs font-medium text-text-secondary leading-tight";
const numBase = "text-xl font-black tabular-nums tracking-tight";
const iconBox = (tone: string) =>
  `flex shrink-0 items-center justify-center rounded-xl p-2 ${tone}`;
</script>

<template>
  <div class="grid grid-cols-2 gap-3 sm:gap-4 lg:grid-cols-4">
    <div :class="tileWrap">
      <div :class="tileHead">
        <div :class="iconBox('bg-blue-500/10 text-blue-600 dark:text-blue-400')">
          <CreditCard :size="iconSize.md" stroke-width="2.25" />
        </div>
        <span :class="tileLabel">{{ t("active_subscriptions") }}</span>
      </div>
      <p :class="[numBase, 'text-text-primary']">{{ activeCount }}</p>
    </div>

    <div :class="tileWrap">
      <div :class="tileHead">
        <div :class="iconBox('bg-emerald-500/10 text-emerald-600 dark:text-emerald-400')">
          <TrendingUp :size="iconSize.md" stroke-width="2.25" />
        </div>
        <span :class="tileLabel">{{ t("monthly_cost") }}</span>
      </div>
      <p :class="[numBase, 'text-primary']">{{ fmt(totalMonthly) }}</p>
    </div>

    <div :class="tileWrap">
      <div :class="tileHead">
        <div :class="iconBox('bg-violet-500/10 text-violet-600 dark:text-violet-400')">
          <Wallet :size="iconSize.md" stroke-width="2.25" />
        </div>
        <span :class="tileLabel">{{ t("yearly_cost") }}</span>
      </div>
      <p :class="[numBase, 'text-text-primary']">{{ fmt(totalYearly) }}</p>
    </div>

    <div :class="tileWrap">
      <div :class="tileHead">
        <div :class="iconBox('bg-orange-500/10 text-orange-600 dark:text-orange-400')">
          <TrendingDown :size="iconSize.md" stroke-width="2.25" />
        </div>
        <span :class="tileLabel">{{ t("amount_due") }}</span>
      </div>
      <p :class="[numBase, 'text-text-primary']">{{ fmt(amountDue) }}</p>
    </div>

    <div :class="tileWrap">
      <div :class="tileHead">
        <div :class="iconBox('bg-cyan-500/10 text-cyan-600 dark:text-cyan-400')">
          <BarChart3 :size="iconSize.md" stroke-width="2.25" />
        </div>
        <span :class="tileLabel">{{ t("average_monthly") }}</span>
      </div>
      <p :class="[numBase, 'text-text-primary']">{{ fmt(avgMonthly) }}</p>
    </div>

    <div v-if="mostExpensive" :class="tileWrap">
      <div :class="tileHead">
        <div :class="iconBox('bg-amber-500/10 text-amber-600 dark:text-amber-400')">
          <Star :size="iconSize.md" stroke-width="2.25" />
        </div>
        <span :class="tileLabel">{{ t("most_expensive") }}</span>
      </div>
      <p :class="[numBase, 'text-primary']">{{ fmt(mostExpensive.price) }}</p>
      <p class="mt-1 truncate text-[11px] font-semibold text-primary">{{ mostExpensive.name }}</p>
    </div>

    <div v-if="budgetUsed !== null" :class="tileWrap">
      <div :class="tileHead">
        <div :class="iconBox('bg-rose-500/10 text-rose-600 dark:text-rose-400')">
          <Wallet :size="iconSize.md" stroke-width="2.25" />
        </div>
        <span :class="tileLabel">{{ t("percentage_budget_used") }}</span>
      </div>
      <p :class="[numBase, 'text-text-primary']">{{ fmtPercent(budgetUsed || 0) }}</p>
    </div>

    <div v-if="totalSavings > 0" :class="tileWrap">
      <div :class="tileHead">
        <div :class="iconBox('bg-teal-500/10 text-teal-600 dark:text-teal-400')">
          <TrendingDown :size="iconSize.md" stroke-width="2.25" />
        </div>
        <span :class="tileLabel">{{ t("monthly_savings") }}</span>
      </div>
      <p :class="[numBase, 'text-green-600 dark:text-green-400']">{{ fmt(totalSavings) }}</p>
    </div>
  </div>
</template>
