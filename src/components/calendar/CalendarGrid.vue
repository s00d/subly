<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";

export interface CalendarCell {
  day: number;
  isEmpty: boolean;
  isToday: boolean;
  subs: { id: string; name: string; price: number; currencyId: string }[];
  expenses?: { id: string; name: string; amount: number; currencyId: string }[];
}

defineProps<{
  cells: CalendarCell[];
}>();

const emit = defineEmits<{
  selectDay: [cell: CalendarCell];
}>();

const { t } = useI18n();
const { fmt } = useCurrencyFormat();

const weekDays = () => [t("mon"), t("tue"), t("wed"), t("thu"), t("fri"), t("sat"), t("sun")];
</script>

<template>
  <div class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] overflow-hidden">
    <!-- Day headers -->
    <div class="grid grid-cols-7 border-b border-[var(--color-border)]">
      <div
        v-for="day in weekDays()"
        :key="day"
        class="py-1.5 sm:py-2 text-center text-[10px] sm:text-xs font-medium text-[var(--color-text-muted)] uppercase"
      >{{ day }}</div>
    </div>

    <!-- Cells -->
    <div class="grid grid-cols-7">
      <div
        v-for="(cell, idx) in cells"
        :key="idx"
        class="min-h-[48px] sm:min-h-[80px] p-0.5 sm:p-1.5 border-b border-r border-[var(--color-border)]"
        :class="{
          'bg-[var(--color-surface-hover)]/50': cell.isEmpty,
          'bg-blue-50/50 dark:bg-blue-900/10': cell.isToday,
          'cursor-pointer hover:bg-[var(--color-surface-hover)]': !cell.isEmpty && (cell.subs.length > 0 || (cell.expenses && cell.expenses.length > 0)),
        }"
        @click="!cell.isEmpty && (cell.subs.length > 0 || (cell.expenses && cell.expenses.length > 0)) && emit('selectDay', cell)"
      >
        <div v-if="!cell.isEmpty">
          <span
            class="text-[10px] sm:text-xs font-medium inline-block mb-0.5 sm:mb-1 w-5 h-5 sm:w-6 sm:h-6 rounded-full text-center leading-5 sm:leading-6"
            :class="cell.isToday ? 'bg-[var(--color-primary)] text-white' : 'text-[var(--color-text-secondary)]'"
          >{{ cell.day }}</span>
          <!-- Mobile: just show dot indicators -->
          <div class="sm:hidden flex gap-0.5 flex-wrap">
            <div v-for="sub in cell.subs.slice(0, 3)" :key="sub.id" class="w-1.5 h-1.5 rounded-full bg-[var(--color-primary)]" />
            <div v-for="exp in (cell.expenses || []).slice(0, 3)" :key="exp.id" class="w-1.5 h-1.5 rounded-full bg-orange-400" />
            <div v-if="cell.subs.length + (cell.expenses?.length || 0) > 6" class="text-[8px] text-[var(--color-text-muted)] leading-none">+{{ cell.subs.length + (cell.expenses?.length || 0) - 6 }}</div>
          </div>
          <!-- Desktop: show names + amounts -->
          <div class="hidden sm:block">
            <div v-for="sub in cell.subs.slice(0, 2)" :key="sub.id"
              class="flex items-center gap-0.5 text-[10px] leading-tight px-1 py-0.5 rounded bg-blue-600 text-white dark:bg-blue-500 font-medium mb-0.5">
              <span class="truncate">{{ sub.name }}</span>
              <span class="ml-auto shrink-0 opacity-80">{{ fmt(sub.price, sub.currencyId) }}</span>
            </div>
            <div v-for="exp in (cell.expenses || []).slice(0, 2)" :key="exp.id"
              class="flex items-center gap-0.5 text-[10px] leading-tight px-1 py-0.5 rounded bg-orange-100 text-orange-600 dark:bg-orange-900/30 dark:text-orange-400 font-medium mb-0.5">
              <span class="truncate">{{ exp.name }}</span>
              <span class="ml-auto shrink-0 opacity-80">{{ fmt(exp.amount, exp.currencyId) }}</span>
            </div>
            <div v-if="cell.subs.length + (cell.expenses?.length || 0) > 4" class="text-[10px] text-[var(--color-text-muted)]">
              +{{ cell.subs.length + (cell.expenses?.length || 0) - 4 }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
