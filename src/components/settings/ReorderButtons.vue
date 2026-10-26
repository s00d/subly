<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { ChevronUp, ChevronDown } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { button } from "@/lib/tv";

/** Up/down pair for catalog rows — 28px hit targets. */
defineProps<{ upDisabled?: boolean; downDisabled?: boolean }>();
const emit = defineEmits<{ up: []; down: [] }>();
const { t } = useI18n();
const btn = button({ intent: "ghost", size: "icon", class: "h-7 w-7 text-text-muted hover:text-primary disabled:opacity-30" });
</script>

<template>
  <div class="flex shrink-0">
    <Tooltip :text="t('move_up')">
      <button type="button" :class="btn" :disabled="upDisabled" :aria-label="t('move_up')" @click="emit('up')">
        <ChevronUp :size="16" />
      </button>
    </Tooltip>
    <Tooltip :text="t('move_down')">
      <button type="button" :class="btn" :disabled="downDisabled" :aria-label="t('move_down')" @click="emit('down')">
        <ChevronDown :size="16" />
      </button>
    </Tooltip>
  </div>
</template>
