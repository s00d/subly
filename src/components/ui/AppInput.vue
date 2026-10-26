<script setup lang="ts">
import { computed } from "vue";
import { tv, ui } from "@/lib/tv";

/** HTML `inputmode` (мобильная клавиатура). */
type InputModeAttr = "none" | "text" | "decimal" | "numeric" | "tel" | "search" | "email" | "url";

const props = defineProps<{
  modelValue: string | number;
  type?: string;
  placeholder?: string;
  /** Нативный `title` на `<input>` (подсказка при наведении). */
  tooltip?: string;
  disabled?: boolean;
  required?: boolean;
  min?: number | string;
  max?: number | string;
  step?: number | string;
  /** Нативный `inputmode` (например `decimal`, `numeric`) для мобильных клавиатур. */
  inputmode?: InputModeAttr;
  label?: string;
  size?: "sm" | "md";
  /** `inline`: borderless until hover/focus — for editable names inside list rows. */
  variant?: "inline";
  error?: string;
  class?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string | number];
}>();

function onInput(e: Event) {
  const target = e.target as HTMLInputElement;
  if (props.type === "number") {
    if (target.value === "") {
      emit("update:modelValue", "");
      return;
    }
    const parsed = Number(target.value);
    emit("update:modelValue", Number.isFinite(parsed) ? parsed : "");
    return;
  }
  emit("update:modelValue", target.value);
}

const inputTv = tv({
  slots: {
    root: "w-full",
    labelEl: ui.fieldLabel(),
    inputEl: [
      ui.field(),
    ],
    errorEl: ui.fieldError(),
  },
  variants: {
    size: {
      sm: { inputEl: "min-h-8 px-2.5 py-1 text-xs" },
      md: { inputEl: "px-3 py-1.5 text-[13px]" },
    },
    status: {
      error: { inputEl: "border-danger hover:border-danger focus:ring-danger/20 focus:border-danger" },
      normal: {},
    },
    disabled: {
      true: { inputEl: "bg-surface-secondary" },
    },
    inline: {
      true: { inputEl: "border-transparent bg-transparent shadow-none hover:border-border" },
    },
  },
  defaultVariants: { size: "md", status: "normal" },
});

const slots = computed(() =>
  inputTv({
    size: props.size ?? "md",
    status: props.error ? "error" : "normal",
    disabled: props.disabled || undefined,
    inline: props.variant === "inline" || undefined,
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
      :title="tooltip"
      :disabled="disabled"
      :required="required"
      :min="min"
      :max="max"
      :step="step"
      :inputmode="inputmode"
      @input="onInput"
      :class="slots.inputEl()"
    />
    <p v-if="error" :class="slots.errorEl()">{{ error }}</p>
  </div>
</template>
