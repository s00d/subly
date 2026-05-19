<script setup lang="ts">
import { computed } from "vue";
import type { MdiIconName } from "@/mdi-icons";

const props = withDefaults(
  defineProps<{
    name: MdiIconName | (string & {});
    size?: "sm" | "md" | "lg";
    label?: string;
  }>(),
  { size: "md" },
);

const sizeClass = computed(() => {
  if (props.size === "sm") return "icon-sm";
  if (props.size === "lg") return "icon-lg";
  return "icon-md";
});

const iconClass = computed(() => `i-mdi-${props.name}`);
</script>

<template>
  <span
    :class="[iconClass, sizeClass]"
    :aria-hidden="label ? undefined : true"
    :aria-label="label"
    role="img"
  />
</template>
