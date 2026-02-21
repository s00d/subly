<script setup lang="ts">
import { ref, onBeforeUnmount } from "vue";

const props = defineProps<{
  text: string;
  position?: "top" | "bottom" | "left" | "right";
}>();

const triggerRef = ref<HTMLElement | null>(null);
const visible = ref(false);
const coords = ref({ top: 0, left: 0 });

const GAP = 8;
let hideTimeout: ReturnType<typeof setTimeout> | null = null;

function show() {
  if (hideTimeout) {
    clearTimeout(hideTimeout);
    hideTimeout = null;
  }
  if (!triggerRef.value) return;
  const rect = triggerRef.value.getBoundingClientRect();
  const pos = props.position || "top";

  let top = 0;
  let left = 0;

  switch (pos) {
    case "top":
      top = rect.top - GAP;
      left = rect.left + rect.width / 2;
      break;
    case "bottom":
      top = rect.bottom + GAP;
      left = rect.left + rect.width / 2;
      break;
    case "left":
      top = rect.top + rect.height / 2;
      left = rect.left - GAP;
      break;
    case "right":
      top = rect.top + rect.height / 2;
      left = rect.right + GAP;
      break;
  }

  coords.value = { top, left };
  visible.value = true;
}

function hide() {
  hideTimeout = setTimeout(() => {
    visible.value = false;
  }, 80);
}

onBeforeUnmount(() => {
  if (hideTimeout) clearTimeout(hideTimeout);
});
</script>

<template>
  <div
    ref="triggerRef"
    class="inline-flex"
    @mouseenter="show"
    @mouseleave="hide"
    @focusin="show"
    @focusout="hide"
  >
    <slot />
  </div>

  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition duration-100 ease-in"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div
        v-if="visible"
        class="tooltip-bubble fixed z-[9999] pointer-events-none whitespace-nowrap px-2.5 py-1.5 rounded-lg text-[11px] font-medium shadow-lg"
        :class="{
          '-translate-x-1/2 -translate-y-full': !position || position === 'top',
          '-translate-x-1/2': position === 'bottom',
          '-translate-x-full -translate-y-1/2': position === 'left',
          '-translate-y-1/2': position === 'right',
        }"
        :style="{ top: coords.top + 'px', left: coords.left + 'px' }"
      >
        {{ text }}
        <div
          class="tooltip-arrow absolute w-2 h-2 rotate-45"
          :class="{
            'top-full left-1/2 -translate-x-1/2 -mt-1': !position || position === 'top',
            'bottom-full left-1/2 -translate-x-1/2 -mb-1': position === 'bottom',
            'left-full top-1/2 -translate-y-1/2 -ml-1': position === 'left',
            'right-full top-1/2 -translate-y-1/2 -mr-1': position === 'right',
          }"
        />
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.tooltip-bubble {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.tooltip-arrow {
  background: var(--color-surface-hover);
  border-right: 1px solid var(--color-border);
  border-bottom: 1px solid var(--color-border);
}

:global(.dark) .tooltip-bubble {
  background: var(--color-surface-secondary);
  color: var(--color-text-primary);
  border-color: var(--color-border);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

:global(.dark) .tooltip-arrow {
  background: var(--color-surface-secondary);
  border-color: var(--color-border);
}
</style>
