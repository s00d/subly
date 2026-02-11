<script setup lang="ts">
defineProps<{
  modelValue: boolean;
  label?: string;
  description?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();
</script>

<template>
  <label
    class="flex items-center justify-between gap-3 cursor-pointer select-none"
    :class="{ 'opacity-50 cursor-not-allowed': disabled }"
  >
    <div class="flex-1 min-w-0">
      <p v-if="label" class="text-sm text-[var(--color-text-secondary)]">{{ label }}</p>
      <p v-if="description" class="text-xs text-[var(--color-text-muted)] mt-0.5">{{ description }}</p>
    </div>
    <button
      type="button"
      role="switch"
      :aria-checked="modelValue"
      :disabled="disabled"
      @click="emit('update:modelValue', !modelValue)"
      class="relative w-10 h-[22px] shrink-0 rounded-full transition-colors duration-200"
      :class="[
        modelValue ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-border)]',
      ]"
    >
      <span
        class="absolute top-[2px] left-[2px] w-[18px] h-[18px] rounded-full bg-white shadow-sm transition-transform duration-200"
        :class="{ 'translate-x-[18px]': modelValue }"
      />
    </button>
  </label>
</template>
