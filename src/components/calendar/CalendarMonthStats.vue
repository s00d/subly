<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { CalendarCheck, Clock, Repeat } from "@lucide/vue";

const props = defineProps<{
  count: number;
  totalCost: string;
  totalDue: string;
  compact?: boolean;
}>();

const { t } = useI18n();

const cells = computed(() => [
  { key: "count", label: t("active_subscriptions"), value: String(props.count), icon: Repeat, chip: "bg-primary/14 text-primary" },
  { key: "cost", label: t("total_cost"), value: props.totalCost, icon: CalendarCheck, chip: "bg-violet-500/15 text-violet-500 dark:text-violet-400" },
  { key: "due", label: t("amount_due"), value: props.totalDue, icon: Clock, chip: "bg-warning/15 text-warning" },
]);
</script>

<template>
  <section class="grid grid-cols-3 rounded-xl border border-border bg-surface shadow-sm" :class="compact ? 'mt-3' : 'mt-4'">
    <div
      v-for="c in cells"
      :key="c.key"
      class="flex min-w-0 items-start gap-2.5 border-border [&:not(:last-child)]:border-r"
      :class="compact ? 'px-3 py-2.5' : 'px-4 py-3.5 sm:px-5'"
    >
      <span class="mt-0.5 hidden h-7 w-7 shrink-0 items-center justify-center rounded-lg sm:flex" :class="c.chip">
        <component :is="c.icon" :size="14" :stroke-width="2.2" />
      </span>
      <div class="min-w-0">
        <p class="line-clamp-2 text-[11px] font-medium leading-tight text-text-muted">{{ c.label }}</p>
        <p class="truncate font-semibold tabular-nums tracking-[-0.015em] text-text-primary" :class="compact ? 'text-sm' : 'text-base sm:text-lg'">
          {{ c.value }}
        </p>
      </div>
    </div>
  </section>
</template>
