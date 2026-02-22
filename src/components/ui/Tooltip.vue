<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { tv } from "@/lib/tv";

const props = defineProps<{
  text: string;
  position?: "top" | "bottom" | "left" | "right";
}>();

const triggerRef = ref<HTMLElement | null>(null);
const visible = ref(false);
const coords = ref({ top: 0, left: 0 });

const GAP = 8;
const LONG_PRESS_MS = 500;
const AUTO_HIDE_MS = 2000;
const HOVER_MAX_MS = 4000;
let hideTimeout: ReturnType<typeof setTimeout> | null = null;
let longPressTimeout: ReturnType<typeof setTimeout> | null = null;
let autoHideTimeout: ReturnType<typeof setTimeout> | null = null;

function clearAllTimers() {
  if (hideTimeout) { clearTimeout(hideTimeout); hideTimeout = null; }
  if (longPressTimeout) { clearTimeout(longPressTimeout); longPressTimeout = null; }
  if (autoHideTimeout) { clearTimeout(autoHideTimeout); autoHideTimeout = null; }
}

function computeCoords() {
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
}

function show() {
  clearAllTimers();
  computeCoords();
  visible.value = true;
  autoHideTimeout = setTimeout(dismiss, HOVER_MAX_MS);
}

function hide() {
  clearAllTimers();
  hideTimeout = setTimeout(dismiss, 60);
}

function dismiss() {
  clearAllTimers();
  visible.value = false;
}

function onTouchStart() {
  longPressTimeout = setTimeout(() => {
    show();
    autoHideTimeout = setTimeout(dismiss, AUTO_HIDE_MS);
  }, LONG_PRESS_MS);
}

function onTouchEnd() {
  if (longPressTimeout) { clearTimeout(longPressTimeout); longPressTimeout = null; }
}

function onTouchMove() {
  if (longPressTimeout) { clearTimeout(longPressTimeout); longPressTimeout = null; }
}

function onGlobalDismiss() {
  if (visible.value) dismiss();
}

onMounted(() => {
  window.addEventListener("scroll", onGlobalDismiss, true);
  window.addEventListener("pointerdown", onGlobalDismiss, true);
});

onBeforeUnmount(() => {
  clearAllTimers();
  window.removeEventListener("scroll", onGlobalDismiss, true);
  window.removeEventListener("pointerdown", onGlobalDismiss, true);
});

const tooltipTv = tv({
  slots: {
    trigger: "inline-flex",
    bubble: [
      "tooltip-bubble fixed z-[9999] pointer-events-none whitespace-nowrap",
      "px-2.5 py-1.5 rounded-lg text-[11px] font-medium shadow-lg",
    ],
    arrow: "tooltip-arrow absolute w-2 h-2 rotate-45",
  },
  variants: {
    position: {
      top: {
        bubble: "-translate-x-1/2 -translate-y-full",
        arrow: "top-full left-1/2 -translate-x-1/2 -mt-1",
      },
      bottom: {
        bubble: "-translate-x-1/2",
        arrow: "bottom-full left-1/2 -translate-x-1/2 -mb-1",
      },
      left: {
        bubble: "-translate-x-full -translate-y-1/2",
        arrow: "left-full top-1/2 -translate-y-1/2 -ml-1",
      },
      right: {
        bubble: "-translate-y-1/2",
        arrow: "right-full top-1/2 -translate-y-1/2 -mr-1",
      },
    },
  },
  defaultVariants: { position: "top" },
});

const slots = tooltipTv({ position: props.position || "top" });
</script>

<template>
  <div
    ref="triggerRef"
    :class="slots.trigger()"
    @mouseenter="show"
    @mouseleave="hide"
    @focusin="show"
    @focusout="hide"
    @touchstart.passive="onTouchStart"
    @touchend.passive="onTouchEnd"
    @touchmove.passive="onTouchMove"
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
        :class="slots.bubble()"
        :style="{ top: coords.top + 'px', left: coords.left + 'px' }"
      >
        {{ text }}
        <div :class="slots.arrow()" />
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
