<script setup lang="ts">
import { ui } from "@/lib/tv";

withDefaults(
  defineProps<{
    mode?: "compact" | "default" | "expanded";
    clickable?: boolean;
  }>(),
  {
    mode: "default",
    clickable: true,
  },
);

const emit = defineEmits<{
  click: [event: MouseEvent];
  contextmenu: [event: MouseEvent];
}>();
</script>

<template>
  <div
    :class="[
      mode === 'expanded' ? 'p-3 sm:p-4' : 'flex items-center',
      mode === 'compact' ? 'gap-2 px-3 py-2' : '',
      mode === 'default' ? `gap-2 sm:gap-3 ${ui.listRow()}` : '',
      clickable ? 'cursor-pointer' : '',
    ]"
    @click="emit('click', $event)"
    @contextmenu.prevent="emit('contextmenu', $event)"
  >
    <template v-if="mode === 'expanded'">
      <slot name="expanded" />
    </template>
    <template v-else>
      <slot name="selection" />
      <slot name="leading" />
      <div class="min-w-0 flex-1">
        <slot name="main" />
      </div>
      <slot name="meta" />
      <slot name="value" />
      <slot name="trailing" />
    </template>
  </div>
  <slot name="after" />
</template>
