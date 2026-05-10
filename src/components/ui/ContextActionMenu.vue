<script setup lang="ts">
import type { Component } from "vue";
import { ref, computed, watch, onUnmounted, nextTick } from "vue";

export type ContextMenuRow =
  | { kind: "separator" }
  | {
      kind: "button";
      label: string;
      danger?: boolean;
      /** Lucide icon component (e.g. from `lucide/vue`). */
      icon?: Component;
      run: () => void | Promise<void>;
    };

export type ContextMenuExcludeRect = {
  left: number;
  top: number;
  right: number;
  bottom: number;
};

const props = defineProps<{
  open: boolean;
  x: number;
  y: number;
  rows: ContextMenuRow[];
  /** When set, backdrop blur/dim splits around this viewport rect (e.g. the list row). */
  excludeRect?: ContextMenuExcludeRect | null;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
}>();

const panelRef = ref<HTMLElement | null>(null);
const panelPos = ref({ left: 0, top: 0 });
const positioned = ref(false);

function close() {
  emit("update:open", false);
}

function clampToViewport() {
  const el = panelRef.value;
  if (!el) return;
  const pad = 8;
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  const rect = el.getBoundingClientRect();
  let left = props.x;
  let top = props.y + 6;
  if (left + rect.width > vw - pad) {
    left = vw - pad - rect.width;
  }
  if (left < pad) {
    left = pad;
  }
  if (top + rect.height > vh - pad) {
    top = props.y - rect.height - 6;
  }
  if (top < pad) {
    top = pad;
  }
  panelPos.value = { left, top };
  positioned.value = true;
}

async function layoutPanel() {
  positioned.value = false;
  panelPos.value = { left: props.x, top: props.y + 6 };
  await nextTick();
  requestAnimationFrame(() => {
    clampToViewport();
  });
}

watch(
  () => props.open,
  async (v) => {
    if (v) {
      await layoutPanel();
      window.addEventListener("keydown", onKeydown);
    } else {
      positioned.value = false;
      window.removeEventListener("keydown", onKeydown);
    }
  },
);

watch(
  () => [props.x, props.y, props.rows],
  async () => {
    if (props.open) {
      await layoutPanel();
    }
  },
);

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    close();
  }
}

onUnmounted(() => window.removeEventListener("keydown", onKeydown));

async function onRowClick(row: ContextMenuRow) {
  if (row.kind !== "button") return;
  try {
    await row.run();
  } finally {
    close();
  }
}

const SCRIM_CLASS =
  "ctx-scrim absolute bg-black/35 backdrop-blur-[3px] [-webkit-backdrop-filter:blur(3px)]";

/** Four viewport-fixed strips around `excludeRect`; empty → use single full scrim. */
const holeScrimStrips = computed(() => {
  const raw = props.excludeRect;
  if (!raw || !props.open) return null;
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  const left = Math.max(0, raw.left);
  const top = Math.max(0, raw.top);
  const right = Math.min(vw, raw.right);
  const bottom = Math.min(vh, raw.bottom);
  if (right - left < 2 || bottom - top < 2) return null;
  return [
    { top: 0, left: 0, width: vw, height: top },
    { top: bottom, left: 0, width: vw, height: Math.max(0, vh - bottom) },
    { top, left: 0, width: left, height: bottom - top },
    { top, left: right, width: Math.max(0, vw - right), height: bottom - top },
  ];
});
</script>

<template>
  <Teleport to="body">
    <Transition name="ctx-scrim">
      <div
        v-if="open"
        class="ctx-layer fixed inset-0 z-[300]"
        style="touch-action: none"
        aria-modal="true"
        role="presentation"
        @click.self="close"
      >
        <template v-if="holeScrimStrips">
          <div
            v-for="(strip, si) in holeScrimStrips"
            :key="si"
            :class="SCRIM_CLASS"
            aria-hidden="true"
            :style="{
              top: `${strip.top}px`,
              left: `${strip.left}px`,
              width: `${strip.width}px`,
              height: `${strip.height}px`,
            }"
            @click="close"
          />
        </template>
        <div
          v-else
          :class="SCRIM_CLASS + ' inset-0'"
          aria-hidden="true"
          @click="close"
        />
        <div
          ref="panelRef"
          class="ctx-panel fixed z-[301] min-w-44 max-w-[min(calc(100vw-1rem),20rem)] overflow-hidden rounded-xl border border-border bg-surface py-1.5 shadow-2xl outline-none ring-1 ring-black/[0.05] dark:bg-surface dark:ring-white/10"
          :class="{ 'ctx-panel--visible': positioned }"
          :style="{ left: `${panelPos.left}px`, top: `${panelPos.top}px` }"
          role="menu"
          tabindex="-1"
          @click.stop
        >
          <template v-for="(row, i) in rows" :key="i">
            <div v-if="row.kind === 'separator'" class="mx-2 my-1.5 h-px bg-border/90" role="separator" />
            <button
              v-else
              type="button"
              role="menuitem"
              class="ctx-item mx-1.5 flex w-[calc(100%-0.75rem)] cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-2 text-left text-sm font-medium text-text-primary outline-none transition-[transform,background-color,color] duration-150 ease-out select-none hover:bg-surface-hover active:scale-[0.98] active:bg-surface-secondary focus-visible:bg-surface-hover focus-visible:ring-2 focus-visible:ring-primary/40 dark:hover:bg-white/[0.07] dark:active:bg-white/[0.1] dark:focus-visible:ring-primary/50"
              :class="
                row.danger
                  ? 'text-red-600 hover:bg-red-500/12 hover:text-red-700 active:bg-red-500/18 dark:text-red-400 dark:hover:bg-red-500/15 dark:hover:text-red-300'
                  : ''
              "
              @click="onRowClick(row)"
            >
              <span
                v-if="row.icon"
                class="ctx-item-icon flex size-[18px] shrink-0 items-center justify-center text-text-muted [.ctx-item:hover_&]:text-text-secondary"
                :class="row.danger ? '!text-red-500 dark:!text-red-400 [.ctx-item:hover_&]:!text-red-600' : ''"
                aria-hidden="true"
              >
                <component :is="row.icon" :size="16" :stroke-width="2" class="shrink-0" />
              </span>
              <span class="min-w-0 flex-1 leading-snug">{{ row.label }}</span>
            </button>
          </template>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.ctx-layer {
  pointer-events: auto;
}

.ctx-scrim-enter-active,
.ctx-scrim-leave-active {
  transition: opacity 0.22s ease;
}

.ctx-scrim-enter-from,
.ctx-scrim-leave-to {
  opacity: 0;
}

.ctx-scrim-enter-active .ctx-scrim,
.ctx-scrim-leave-active .ctx-scrim {
  transition: inherit;
}

.ctx-panel {
  transform-origin: top left;
  opacity: 0;
  transform: scale(0.94) translateY(-6px);
  transition:
    opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1),
    transform 0.22s cubic-bezier(0.16, 1, 0.3, 1);
  will-change: transform, opacity;
}

.ctx-panel.ctx-panel--visible {
  opacity: 1;
  transform: scale(1) translateY(0);
}

.ctx-item:active {
  transition-duration: 0.08s;
}

@media (prefers-reduced-motion: reduce) {
  .ctx-scrim-enter-active,
  .ctx-scrim-leave-active {
    transition-duration: 0.01ms !important;
  }

  .ctx-panel {
    transition-duration: 0.01ms !important;
    transform: none !important;
  }

  .ctx-panel.ctx-panel--visible {
    transform: none !important;
  }
}
</style>
