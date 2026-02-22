<script setup lang="ts">
import { computed } from "vue";
import { Check } from "lucide-vue-next";
import { tv } from "@/lib/tv";

const props = defineProps<{
  modelValue: boolean;
  label?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const checkboxTv = tv({
  slots: {
    root: "inline-flex items-center gap-2.5 cursor-pointer select-none",
    box: [
      "w-[18px] h-[18px] shrink-0 rounded-[5px] border-2",
      "flex items-center justify-center transition-all duration-150",
    ],
    labelEl: "text-sm text-[var(--color-text-secondary)]",
  },
  variants: {
    disabled: {
      true: { root: "opacity-50 cursor-not-allowed" },
    },
    checked: {
      true: { box: "bg-[var(--color-primary)] border-[var(--color-primary)]" },
      false: { box: "border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-surface)]" },
    },
  },
  defaultVariants: { checked: false },
});

const slots = computed(() =>
  checkboxTv({
    disabled: props.disabled || undefined,
    checked: props.modelValue,
  }),
);
</script>

<template>
  <label :class="slots.root()">
    <button
      type="button"
      role="checkbox"
      :aria-checked="modelValue"
      :disabled="disabled"
      @click="emit('update:modelValue', !modelValue)"
      :class="slots.box()"
    >
      <Check v-if="modelValue" :size="12" class="text-white" stroke-width="3" />
    </button>
    <span v-if="label" :class="slots.labelEl()">{{ label }}</span>
    <span v-if="$slots.default" :class="slots.labelEl()"><slot /></span>
  </label>
</template>
