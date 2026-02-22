<script setup lang="ts">
import { X } from "lucide-vue-next";
import { tv } from "@/lib/tv";

defineProps<{
  title: string;
  show: boolean;
  maxWidth?: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

const modalTv = tv({
  slots: {
    overlay: "fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4",
    backdrop: "absolute inset-0 bg-black/50",
    panel: [
      "relative bg-[var(--color-surface)] w-full overflow-hidden",
      "rounded-t-2xl sm:rounded-xl shadow-2xl max-h-[90vh] sm:max-h-none",
    ],
    header: "flex items-center justify-between px-4 sm:px-6 py-3 sm:py-4 border-b border-[var(--color-border)]",
    titleEl: "text-base sm:text-lg font-semibold text-[var(--color-text-primary)]",
    closeBtn: "p-1 rounded-lg hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]",
    body: "px-4 sm:px-6 py-3 sm:py-4 max-h-[70vh] overflow-y-auto",
    footer: "px-4 sm:px-6 py-3 sm:py-4 border-t border-[var(--color-border)] flex justify-end gap-3",
  },
});

const slots = modalTv();
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="show" :class="slots.overlay()">
        <div :class="slots.backdrop()" @click="emit('close')" />
        <div
          :class="slots.panel()"
          :style="{ maxWidth: maxWidth || '32rem' }"
        >
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
