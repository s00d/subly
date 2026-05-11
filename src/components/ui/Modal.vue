<script setup lang="ts">
import { computed } from "vue";
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
    backdrop: "absolute inset-0 bg-slate-900/45 backdrop-blur-sm",
    dragHandle: "sm:hidden h-1.5 w-12 rounded-full bg-border mx-auto mt-3 mb-1 shrink-0",
    panel: [
      "relative bg-surface w-full overflow-hidden flex flex-col",
      "rounded-t-[24px] sm:rounded-2xl shadow-xl max-h-[92vh] sm:max-h-none",
    ],
    header: "flex items-center justify-between px-4 sm:px-6 py-3 border-b border-border shrink-0",
    titleEl: "text-base font-semibold text-text-primary tracking-tight",
    closeBtn:
      "p-2.5 -mr-1 rounded-full hover:bg-surface-hover text-text-muted transition-colors min-w-[44px] min-h-[44px] inline-flex items-center justify-center",
    body: "px-4 sm:px-6 py-4 flex-1 min-h-0 max-h-[70vh] overflow-y-auto",
    footer:
      "px-4 sm:px-6 py-4 border-t border-border bg-surface-secondary/60 flex justify-end gap-2.5 shrink-0",
  },
});

const slots = modalTv();

useScrollLock(computed(() => props.show));
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
              <X :size="20" />
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
