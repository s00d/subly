<script setup lang="ts">
import { computed } from "vue";
import { X } from "@lucide/vue";
import { tv, ui } from "@/lib/tv";
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
    overlay: "fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4",
    backdrop: "absolute inset-0 bg-black/50",
    panel: [
      "relative bg-surface w-full overflow-hidden",
      "rounded-t-2xl sm:rounded-xl shadow-2xl max-h-[90vh] sm:max-h-none",
    ],
    header: "flex items-center justify-between px-4 sm:px-5 py-2.5 sm:py-3 border-b border-border",
    titleEl: ui.sectionTitle(),
    closeBtn: "p-1 rounded-lg hover:bg-surface-hover text-text-muted transition-colors",
    body: "px-4 sm:px-5 py-2.5 sm:py-3 max-h-[70vh] overflow-y-auto",
    footer: "px-4 sm:px-5 py-2.5 sm:py-3 border-t border-border flex justify-end gap-2.5",
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
