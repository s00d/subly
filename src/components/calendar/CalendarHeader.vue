<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { ChevronLeft, ChevronRight, CalendarDays } from "lucide-vue-next";
import Tooltip from "@/components/ui/Tooltip.vue";

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
      <Tooltip v-if="!isCurrentMonth" :text="t('reset')">
        <button
          @click="emit('resetMonth')"
          class="p-1.5 sm:p-2 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)]"
        >
          <CalendarDays :size="16" class="text-[var(--color-text-secondary)]" />
        </button>
      </Tooltip>
      <Tooltip v-if="!isCurrentMonth" :text="t('previous_month')">
        <button
          @click="emit('prevMonth')"
          class="p-1.5 sm:p-2 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)]"
        >
          <ChevronLeft :size="16" class="text-[var(--color-text-secondary)]" />
        </button>
      </Tooltip>
      <h2 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)]">
        {{ monthName }}
      </h2>
      <Tooltip :text="t('next_month')">
        <button
          @click="emit('nextMonth')"
          class="p-1.5 sm:p-2 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)]"
        >
          <ChevronRight :size="16" class="text-[var(--color-text-secondary)]" />
        </button>
      </Tooltip>
    </div>
  </div>
</template>
