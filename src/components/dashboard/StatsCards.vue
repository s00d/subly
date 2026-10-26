<script setup lang="ts">
import { computed, type Component } from "vue";
import { BarChart3, CalendarRange, Clock, Crown, PiggyBank, Wallet } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { moneyGt, parseMoney, toDisplayNumber, type MoneyString } from "@/lib/money";

const props = defineProps<{
  activeCount: number;
  totalMonthly: MoneyString;
  totalYearly: MoneyString;
  amountDue: MoneyString;
  avgMonthly: MoneyString;
  mostExpensive: { name: string; price: MoneyString } | null;
  budgetUsed: MoneyString | null;
  totalSavings: MoneyString;
  fmt: (amount: MoneyString) => string;
}>();

const { t } = useI18n();
const { fmtPercent } = useLocaleFormat();

/** Secondary KPIs — one strip, one cell each; optional ones only when they carry data. */
const cells = computed(() => {
  const out: { key: string; label: string; value: string; hint?: string; tone?: string; icon: Component; chip: string }[] = [
    { key: "yearly", label: t("yearly_cost"), value: props.fmt(props.totalYearly), icon: CalendarRange, chip: "bg-violet-500/15 text-violet-500 dark:text-violet-400" },
    { key: "due", label: t("amount_due"), value: props.fmt(props.amountDue), icon: Clock, chip: "bg-warning/15 text-warning" },
    { key: "avg", label: t("average_monthly"), value: props.fmt(props.avgMonthly), icon: BarChart3, chip: "bg-info/14 text-info" },
  ];
  if (props.mostExpensive) {
    out.push({
      key: "top",
      label: t("most_expensive"),
      value: props.fmt(props.mostExpensive.price),
      hint: props.mostExpensive.name,
      icon: Crown,
      chip: "bg-pink-500/14 text-pink-500 dark:text-pink-400",
    });
  }
  if (props.budgetUsed !== null) {
    out.push({
      key: "budget",
      label: t("percentage_budget_used"),
      value: fmtPercent(toDisplayNumber(parseMoney(props.budgetUsed ?? "0"))),
      tone: moneyGt(props.budgetUsed ?? "0", "100") ? "text-danger" : undefined,
      icon: Wallet,
      chip: "bg-primary/14 text-primary",
    });
  }
  if (moneyGt(props.totalSavings, "0")) {
    out.push({ key: "savings", label: t("monthly_savings"), value: props.fmt(props.totalSavings), tone: "text-success", icon: PiggyBank, chip: "bg-success/14 text-success" });
  }
  return out;
});
</script>

<template>
  <section class="relative overflow-hidden rounded-xl border border-border bg-surface shadow-sm">
    <!-- Accent wash behind the headline number -->
    <div
      aria-hidden="true"
      class="pointer-events-none absolute inset-0 bg-[radial-gradient(120%_140%_at_0%_0%,color-mix(in_srgb,var(--color-primary)_22%,transparent)_0%,transparent_55%)]"
    />
    <div class="relative flex flex-col gap-4 p-5 sm:flex-row sm:items-end sm:justify-between sm:p-6">
      <div class="min-w-0">
        <p class="text-xs font-semibold uppercase tracking-wider text-primary">{{ t("monthly_cost") }}</p>
        <p class="mt-1.5 text-[34px] sm:text-[40px] leading-none font-bold tabular-nums tracking-[-0.035em] text-text-primary">
          {{ fmt(totalMonthly) }}
        </p>
      </div>
      <div class="flex items-center gap-2 rounded-full border border-border bg-surface/70 px-3 py-1.5 text-xs font-medium text-text-secondary backdrop-blur">
        <span class="h-2 w-2 rounded-full bg-success shadow-[0_0_0_3px_color-mix(in_srgb,var(--color-success)_22%,transparent)]" />
        <span class="tabular-nums font-semibold text-text-primary">{{ activeCount }}</span>
        {{ t("active_subscriptions") }}
      </div>
    </div>
    <div class="relative grid grid-cols-2 border-t border-border sm:grid-cols-3 lg:[grid-template-columns:repeat(auto-fit,minmax(150px,1fr))]">
      <div
        v-for="c in cells"
        :key="c.key"
        class="flex min-w-0 items-start gap-2.5 px-4 py-3.5 sm:px-5 border-border [&:not(:last-child)]:border-r max-sm:[&:nth-child(2n)]:border-r-0 max-sm:[&:nth-child(n+3)]:border-t"
      >
        <span class="mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-lg" :class="c.chip">
          <component :is="c.icon" :size="14" :stroke-width="2.2" />
        </span>
        <div class="min-w-0">
          <p class="text-[11px] font-medium text-text-muted truncate">{{ c.label }}</p>
          <p class="text-base font-semibold tabular-nums tracking-[-0.015em] truncate" :class="c.tone ?? 'text-text-primary'">
            {{ c.value }}
          </p>
          <p v-if="c.hint" class="text-[11px] text-text-muted truncate">{{ c.hint }}</p>
        </div>
      </div>
    </div>
  </section>
</template>
