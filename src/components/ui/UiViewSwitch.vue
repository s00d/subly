<script setup lang="ts">
import { LayoutGrid, LayoutList, Rows3 } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import Tooltip from "@/components/ui/Tooltip.vue";

type ViewMode = "compact" | "default" | "expanded";

/** All three list layouts visible at once (segmented), instead of a button that cycles through them. */
const props = defineProps<{ modelValue: ViewMode; labels?: Partial<Record<ViewMode, string>> }>();
const emit = defineEmits<{ "update:modelValue": [mode: ViewMode] }>();
const { t } = useI18n();

const modes = [
  { id: "compact", icon: Rows3, key: "view_compact" },
  { id: "default", icon: LayoutList, key: "view_default" },
  { id: "expanded", icon: LayoutGrid, key: "view_expanded" },
] as const;

const label = (id: ViewMode, key: string) => props.labels?.[id] ?? t(key);
</script>

<template>
  <div class="inline-flex shrink-0 items-center gap-0.5 rounded-lg bg-surface-hover p-0.5" role="radiogroup">
    <Tooltip v-for="m in modes" :key="m.id" :text="label(m.id, m.key)" position="bottom">
      <button
        type="button"
        role="radio"
        data-compact
        :aria-checked="modelValue === m.id"
        :aria-label="label(m.id, m.key)"
        class="inline-flex h-8 w-8 items-center justify-center rounded-md transition-colors"
        :class="modelValue === m.id ? 'bg-surface text-text-primary shadow-sm' : 'text-text-muted hover:text-text-primary'"
        @click="emit('update:modelValue', m.id)"
      >
        <component :is="m.icon" :size="15" />
      </button>
    </Tooltip>
  </div>
</template>
