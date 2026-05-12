<script setup lang="ts">
import { computed, ref } from "vue";
import AppInput from "@/components/ui/AppInput.vue";

/**
 * Password-style input for values that are already stored somewhere we can't
 * (or don't want to) read back into the UI — keyring secrets, sync passwords,
 * API tokens, …
 *
 * Behavior:
 *   - When `hasSavedValue` is `true` and the user hasn't started editing
 *     yet, the field renders a static mask (`••••••••`). The mask is purely
 *     decorative; `modelValue` stays empty.
 *   - On the first focus/click the field clears itself (the mask disappears)
 *     so the user can type a fresh value. No need to validate “did the user
 *     type after the mask?” edge cases — the field is just empty again.
 *   - From there it behaves like a normal password input. Whatever the user
 *     types is reflected in `modelValue`.
 *
 * Callers can use `modelValue.length > 0` as "user supplied a new secret to
 * save" without any sentinel-string acrobatics.
 */
const props = withDefaults(
  defineProps<{
    modelValue: string;
    /** True ⇔ a value is already persisted (keyring, backend, …). */
    hasSavedValue?: boolean;
    label?: string;
    placeholder?: string;
    disabled?: boolean;
    size?: "sm" | "md";
    /** Width of the visual mask. Default mirrors typical password length. */
    maskLength?: number;
    /** Optional CSS class forwarded to the underlying input wrapper. */
    class?: string;
  }>(),
  {
    hasSavedValue: false,
    disabled: false,
    maskLength: 16,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
  /** Emitted the moment the user starts editing (mask cleared). */
  edit: [];
}>();

const editing = ref(false);

const maskValue = computed(() => "\u2022".repeat(props.maskLength));

/** What the underlying `<input>` shows. */
const displayValue = computed(() => {
  if (editing.value) return props.modelValue;
  if (props.hasSavedValue) return maskValue.value;
  return props.modelValue;
});

function startEditing() {
  if (editing.value || props.disabled) return;
  editing.value = true;
  if (props.modelValue !== "") emit("update:modelValue", "");
  emit("edit");
}

function onInput(v: string | number) {
  if (!editing.value) editing.value = true;
  emit("update:modelValue", String(v));
}

/**
 * Re-arm the mask. Useful after a successful save when the parent wants the
 * field to look "filled again" without exposing the freshly saved secret.
 */
function reset() {
  editing.value = false;
  if (props.modelValue !== "") emit("update:modelValue", "");
}

defineExpose({ reset });
</script>

<template>
  <AppInput
    type="password"
    :modelValue="displayValue"
    :label="label"
    :placeholder="placeholder"
    :disabled="disabled"
    :size="size"
    :class="props.class"
    @update:modelValue="onInput"
    @focusin="startEditing"
    @mousedown="startEditing"
  />
</template>
