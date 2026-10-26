<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from "vue";
import { X } from "@lucide/vue";
import { tv } from "@/lib/tv";
import { useScrollLock } from "@/composables/useScrollLock";

const props = defineProps<{
  title: string;
  show: boolean;
  maxWidth?: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

const modalTv = tv({
  slots: {
    overlay: "fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4 sm:p-6",
    backdrop: "absolute inset-0 bg-black/45 backdrop-blur-[2px]",
    dragHandle: "sm:hidden h-1.5 w-12 rounded-full bg-border mx-auto mt-3 mb-1 shrink-0",
    panel: [
      "relative bg-surface w-full overflow-hidden flex flex-col",
      "rounded-t-[18px] sm:rounded-[14px] border border-border shadow-lg max-h-[92vh] sm:max-h-none",
    ],
    header: "flex items-center justify-between gap-3 px-5 py-3 border-b border-border shrink-0",
    titleEl: "text-[15px] font-semibold text-text-primary tracking-[-0.01em]",
    closeBtn:
      "-mr-1.5 h-9 w-9 sm:h-8 sm:w-8 rounded-lg hover:bg-surface-hover text-text-muted hover:text-text-primary transition-colors inline-flex items-center justify-center",
    body: "px-5 py-4 flex-1 min-h-0 max-h-[70vh] overflow-y-auto",
    footer: "px-5 py-3 border-t border-border flex justify-end gap-2 shrink-0",
  },
});

const slots = modalTv();

useScrollLock(computed(() => props.show));

// Escape closes the topmost open modal (stacked dialogs register in order; last one wins).
const openStack: symbol[] = ((globalThis as { __sublyModalStack?: symbol[] }).__sublyModalStack ??= []);
const self = Symbol("modal");
function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && openStack[openStack.length - 1] === self) {
    e.stopPropagation();
    emit("close");
  }
}
watch(
  () => props.show,
  (open) => {
    const i = openStack.indexOf(self);
    if (open && i < 0) openStack.push(self);
    if (!open && i >= 0) openStack.splice(i, 1);
  },
  { immediate: true },
);
onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  const i = openStack.indexOf(self);
  if (i >= 0) openStack.splice(i, 1);
});
</script>

<template>
  <Teleport to="body">
    <Transition name="app-modal">
      <div v-if="show" :class="slots.overlay()">
        <div :class="[slots.backdrop(), 'app-modal-backdrop']" @click="emit('close')" />
        <div
          :class="[slots.panel(), 'app-modal-panel']"
          :style="{ maxWidth: maxWidth || '32rem' }"
        >
          <div :class="slots.dragHandle()" aria-hidden="true" />
          <div :class="slots.header()">
            <h3 :class="slots.titleEl()">{{ title }}</h3>
            <button @click="emit('close')" :class="slots.closeBtn()">
              <X :size="18" />
            </button>
          </div>
          <div :class="slots.body()">
            <slot />
          </div>
          <div v-if="$slots.footer" :class="slots.footer()">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
