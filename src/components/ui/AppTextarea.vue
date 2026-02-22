<script setup lang="ts">
import { tv } from "@/lib/tv";

defineProps<{
  modelValue: string;
  placeholder?: string;
  disabled?: boolean;
  rows?: number;
  label?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

function onInput(e: Event) {
  emit("update:modelValue", (e.target as HTMLTextAreaElement).value);
}

const textareaTv = tv({
  slots: {
    root: "w-full",
    labelEl: "block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5",
    textareaEl: [
      "w-full px-3 py-2 rounded-lg border border-[var(--color-border)]",
      "bg-[var(--color-surface)] text-[var(--color-text-primary)] text-sm",
      "placeholder-[var(--color-text-muted)] resize-none",
      "hover:border-[var(--color-text-muted)]",
      "focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent",
      "disabled:opacity-50 transition-shadow",
    ],
  },
});

const slots = textareaTv();
</script>

<template>
  <div :class="slots.root()">
    <label v-if="label" :class="slots.labelEl()">{{ label }}</label>
    <textarea
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :rows="rows || 3"
      @input="onInput"
      :class="slots.textareaEl()"
    />
  </div>
</template>
