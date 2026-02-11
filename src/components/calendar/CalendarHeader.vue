<script setup lang="ts">
import { useI18n } from "@/i18n";
import { ChevronLeft, ChevronRight, CalendarDays } from "lucide-vue-next";

defineProps<{
  monthName: string;
  year: number;
  isCurrentMonth: boolean;
}>();

const emit = defineEmits<{
  prevMonth: [];
  nextMonth: [];
  resetMonth: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="flex items-center justify-between mb-4 sm:mb-6">
    <div class="flex items-center gap-2 sm:gap-3">
      <button
        v-if="!isCurrentMonth"
        @click="emit('resetMonth')"
        class="p-1.5 sm:p-2 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)]"
        :title="t('reset')"
      >
        <CalendarDays :size="16" class="text-[var(--color-text-secondary)]" />
      </button>
      <button
        v-if="!isCurrentMonth"
        @click="emit('prevMonth')"
        class="p-1.5 sm:p-2 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)]"
      >
        <ChevronLeft :size="16" class="text-[var(--color-text-secondary)]" />
      </button>
      <h2 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)]">
        {{ monthName }} {{ year }}
      </h2>
      <button
        @click="emit('nextMonth')"
        class="p-1.5 sm:p-2 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)]"
      >
        <ChevronRight :size="16" class="text-[var(--color-text-secondary)]" />
      </button>
    </div>
  </div>
</template>
