<script setup lang="ts">
import { computed } from "vue";
import { tv } from "@/lib/tv";

const props = defineProps<{
  modelValue: boolean;
  label?: string;
  description?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const toggleTv = tv({
  slots: {
    root: "flex items-center justify-between gap-3 cursor-pointer select-none",
    content: "flex-1 min-w-0",
    labelEl: "text-sm text-[var(--color-text-secondary)]",
    descEl: "text-xs text-[var(--color-text-muted)] mt-0.5",
    track: "relative w-10 h-[22px] shrink-0 rounded-full transition-colors duration-200",
    thumb: "absolute top-[2px] left-[2px] w-[18px] h-[18px] rounded-full bg-white shadow-sm transition-transform duration-200",
  },
  variants: {
    disabled: {
      true: { root: "opacity-50 cursor-not-allowed" },
    },
    checked: {
      true: {
        track: "bg-[var(--color-primary)]",
        thumb: "translate-x-[18px]",
      },
      false: {
        track: "bg-[var(--color-border)]",
      },
    },
  },
  defaultVariants: { checked: false },
});

const slots = computed(() =>
  toggleTv({
    disabled: props.disabled || undefined,
    checked: props.modelValue,
  }),
);
</script>

<template>
  <label :class="slots.root()">
    <div :class="slots.content()">
      <p v-if="label" :class="slots.labelEl()">{{ label }}</p>
      <p v-if="description" :class="slots.descEl()">{{ description }}</p>
    </div>
    <button
      type="button"
      role="switch"
      :aria-checked="modelValue"
      :disabled="disabled"
      @click="emit('update:modelValue', !modelValue)"
      :class="slots.track()"
    >
      <span :class="slots.thumb()" />
    </button>
  </label>
</template>
