<script setup lang="ts">
import { computed } from "vue";
import { tv } from "@/lib/tv";

const props = defineProps<{
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
  class?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string | number];
}>();

function onInput(e: Event) {
  const target = e.target as HTMLInputElement;
  emit("update:modelValue", target.value);
}

const inputTv = tv({
  slots: {
    root: "w-full",
    labelEl: "block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5",
    inputEl: [
      "w-full rounded-lg border bg-[var(--color-surface)]",
      "text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)]",
      "focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent",
      "disabled:opacity-50 disabled:cursor-not-allowed transition-shadow",
    ],
    errorEl: "mt-1 text-xs text-red-500",
  },
  variants: {
    size: {
      sm: { inputEl: "px-2.5 py-1.5 text-xs" },
      md: { inputEl: "px-3 py-2 text-sm" },
    },
    status: {
      error: { inputEl: "border-red-500 hover:border-red-500 focus:ring-red-500" },
      normal: { inputEl: "border-[var(--color-border)] hover:border-[var(--color-text-muted)]" },
    },
    disabled: {
      true: { inputEl: "border-[var(--color-border)] bg-[var(--color-surface-hover)]" },
    },
  },
  defaultVariants: { size: "md", status: "normal" },
});

const slots = computed(() =>
  inputTv({
    size: props.size ?? "md",
    status: props.error ? "error" : "normal",
    disabled: props.disabled || undefined,
  }),
);
</script>

<template>
  <div :class="slots.root({ class: props.class })">
    <label v-if="label" :class="slots.labelEl()">{{ label }}</label>
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
      :class="slots.inputEl()"
    />
    <p v-if="error" :class="slots.errorEl()">{{ error }}</p>
  </div>
</template>
