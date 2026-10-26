<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { ChevronLeft, ChevronRight, CalendarDays } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { typo, iconSize, button } from "@/lib/tv";

defineProps<{
  monthName: string;
  year: number;
  isCurrentMonth: boolean;
  compact?: boolean;
}>();

const emit = defineEmits<{
  prevMonth: [];
  nextMonth: [];
  resetMonth: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="flex items-center justify-between" :class="compact ? 'mb-3 sm:mb-4' : 'mb-4 sm:mb-6'">
    <div class="flex items-center gap-2 sm:gap-3">
      <Tooltip :text="t('reset')">
        <button :class="button({ size: 'icon' })" :aria-label="t('reset')" @click="emit('resetMonth')">
          <CalendarDays :size="iconSize.sm" class="text-text-secondary" />
        </button>
      </Tooltip>
      <Tooltip :text="t('previous_month')">
        <button :class="button({ size: 'icon' })" :aria-label="t('previous_month')" @click="emit('prevMonth')">
          <ChevronLeft :size="iconSize.sm" class="text-text-secondary" />
        </button>
      </Tooltip>
      <h2 :class="typo.screenTitle()">
        {{ monthName }}
      </h2>
      <Tooltip :text="t('next_month')">
        <button :class="button({ size: 'icon' })" :aria-label="t('next_month')" @click="emit('nextMonth')">
          <ChevronRight :size="iconSize.sm" class="text-text-secondary" />
        </button>
      </Tooltip>
    </div>
  </div>
</template>
