<script setup lang="ts">
defineProps<{
  modelValue: string | number;
  type?: string;
  placeholder?: string;
  disabled?: boolean;
  required?: boolean;
  min?: number | string;
  max?: number | string;
  step?: number | string;
  label?: string;
  size?: "sm" | "md";
  error?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string | number];
}>();

function onInput(e: Event) {
  const target = e.target as HTMLInputElement;
  emit("update:modelValue", target.value);
}
</script>

<template>
  <div class="w-full">
    <label v-if="label" class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{{ label }}</label>
    <input
      :type="type || 'text'"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :required="required"
      :min="min"
      :max="max"
      :step="step"
      @input="onInput"
      class="w-full rounded-lg border bg-[var(--color-surface)] text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed transition-shadow"
      :class="[
        size === 'sm' ? 'px-2.5 py-1.5 text-xs' : 'px-3 py-2 text-sm',
        error ? 'border-red-500 hover:border-red-500 focus:ring-red-500' : disabled ? 'border-[var(--color-border)] bg-[var(--color-surface-hover)]' : 'border-[var(--color-border)] hover:border-[var(--color-text-muted)]',
      ]"
    />
    <p v-if="error" class="mt-1 text-xs text-red-500">{{ error }}</p>
  </div>
</template>
