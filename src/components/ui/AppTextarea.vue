<script setup lang="ts">
import { computed } from "vue";
import { tv, ui } from "@/lib/tv";

const props = defineProps<{
  modelValue: string;
  placeholder?: string;
  disabled?: boolean;
  rows?: number;
  label?: string;
  error?: string;
  class?: string;
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
    labelEl: ui.fieldLabel(),
    textareaEl: [
      ui.field(),
      "px-3 py-2 text-sm resize-none",
    ],
    errorEl: ui.fieldError(),
  },
  variants: {
    status: {
      error: {
        textareaEl: "border-red-500 hover:border-red-500 focus:ring-red-500/20 focus:border-red-500",
      },
      normal: {},
    },
    disabled: {
      true: { textareaEl: "bg-surface-secondary" },
    },
  },
  defaultVariants: { status: "normal" },
});

const slots = computed(() =>
  textareaTv({
    status: props.error ? "error" : "normal",
    disabled: props.disabled || undefined,
  }),
);
</script>

<template>
  <div :class="slots.root({ class: props.class })">
    <label v-if="label" :class="slots.labelEl()">{{ label }}</label>
    <textarea
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :rows="rows || 3"
      @input="onInput"
      :class="slots.textareaEl()"
    />
    <p v-if="error" :class="slots.errorEl()">{{ error }}</p>
  </div>
</template>
