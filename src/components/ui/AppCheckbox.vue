<script setup lang="ts">
import { Check } from "lucide-vue-next";

defineProps<{
  modelValue: boolean;
  label?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();
</script>

<template>
  <label
    class="inline-flex items-center gap-2.5 cursor-pointer select-none"
    :class="{ 'opacity-50 cursor-not-allowed': disabled }"
  >
    <button
      type="button"
      role="checkbox"
      :aria-checked="modelValue"
      :disabled="disabled"
      @click="emit('update:modelValue', !modelValue)"
      class="w-[18px] h-[18px] shrink-0 rounded-[5px] border-2 flex items-center justify-center transition-all duration-150"
      :class="[
        modelValue
          ? 'bg-[var(--color-primary)] border-[var(--color-primary)]'
          : 'border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-surface)]',
      ]"
    >
      <Check v-if="modelValue" :size="12" class="text-white" stroke-width="3" />
    </button>
    <span v-if="label" class="text-sm text-[var(--color-text-secondary)]">{{ label }}</span>
    <span v-if="$slots.default" class="text-sm text-[var(--color-text-secondary)]"><slot /></span>
  </label>
</template>
