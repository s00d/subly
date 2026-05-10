<script setup lang="ts">
import { ref } from "vue";
import { ui } from "@/lib/tv";

const props = withDefaults(
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
  /** Second arg: row bounding rect for context-menu backdrop hole (no blur on this row). */
  contextmenu: [event: MouseEvent, anchorRect: DOMRect | null];
}>();

const rootEl = ref<HTMLElement | null>(null);

function getAnchorRect(): DOMRect | null {
  return rootEl.value?.getBoundingClientRect() ?? null;
}

/** Touch / pen long-press opens the same menu as desktop right-click (`contextmenu`). */
const LONG_PRESS_MS = 480;
const MOVE_CANCEL_PX = 12;
const DEDUPE_NATIVE_MS = 400;

let holdTimer: ReturnType<typeof setTimeout> | null = null;
let startX = 0;
let startY = 0;
let suppressClick = false;
let blockNativeContextUntil = 0;

function clearHold() {
  if (holdTimer) {
    clearTimeout(holdTimer);
    holdTimer = null;
  }
}

function onPointerDown(e: PointerEvent) {
  if (!props.clickable) return;
  if (e.pointerType === "mouse") return;
  if (e.button !== 0) return;
  startX = e.clientX;
  startY = e.clientY;
  clearHold();
  holdTimer = setTimeout(() => {
    holdTimer = null;
    suppressClick = true;
    blockNativeContextUntil = Date.now() + DEDUPE_NATIVE_MS;
    const syn = new MouseEvent("contextmenu", {
      bubbles: true,
      cancelable: true,
      clientX: e.clientX,
      clientY: e.clientY,
      button: 2,
      view: window,
    });
    emit("contextmenu", syn, getAnchorRect());
  }, LONG_PRESS_MS);
}

function onPointerMove(e: PointerEvent) {
  if (!holdTimer) return;
  if (e.pointerType === "mouse") return;
  const dx = Math.abs(e.clientX - startX);
  const dy = Math.abs(e.clientY - startY);
  if (dx > MOVE_CANCEL_PX || dy > MOVE_CANCEL_PX) {
    clearHold();
  }
}

function onPointerUp() {
  clearHold();
}

function onPointerCancel() {
  clearHold();
}

function onClick(e: MouseEvent) {
  if (!props.clickable) return;
  if (suppressClick) {
    suppressClick = false;
    e.preventDefault();
    e.stopPropagation();
    return;
  }
  emit("click", e);
}

function onContextMenu(e: MouseEvent) {
  e.preventDefault();
  if (Date.now() < blockNativeContextUntil) {
    return;
  }
  emit("contextmenu", e, getAnchorRect());
}
</script>

<template>
  <div
    ref="rootEl"
    :class="[
      mode === 'expanded' ? 'p-3 sm:p-4' : 'flex items-center',
      mode === 'compact' ? 'gap-2 px-3 py-2' : '',
      mode === 'default' ? `gap-2 sm:gap-3 ${ui.listRow()}` : '',
      clickable ? 'cursor-pointer' : '',
    ]"
    @click="onClick"
    @contextmenu.prevent="onContextMenu"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerCancel"
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
