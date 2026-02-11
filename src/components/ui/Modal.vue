<script setup lang="ts">
import { X } from "lucide-vue-next";

defineProps<{
  title: string;
  show: boolean;
  maxWidth?: string;
}>();

const emit = defineEmits<{
  close: [];
}>();
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
      <div v-if="show" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/50" @click="emit('close')" />

        <!-- Panel -->
        <div
          class="relative bg-[var(--color-surface)] w-full overflow-hidden rounded-t-2xl sm:rounded-xl shadow-2xl max-h-[90vh] sm:max-h-none"
          :style="{ maxWidth: maxWidth || '32rem' }"
        >
          <!-- Header -->
          <div class="flex items-center justify-between px-4 sm:px-6 py-3 sm:py-4 border-b border-[var(--color-border)]">
            <h3 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ title }}</h3>
            <button
              @click="emit('close')"
              class="p-1 rounded-lg hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]"
            >
              <X :size="20" />
            </button>
          </div>

          <!-- Body -->
          <div class="px-4 sm:px-6 py-3 sm:py-4 max-h-[70vh] overflow-y-auto">
            <slot />
          </div>

          <!-- Footer -->
          <div v-if="$slots.footer" class="px-4 sm:px-6 py-3 sm:py-4 border-t border-[var(--color-border)] flex justify-end gap-3">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
