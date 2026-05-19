<script setup lang="ts">
import { useIntersectionObserver } from "@vueuse/core";
import { ref } from "vue";
import { useReducedMotion } from "@/composables/useReducedMotion";

const props = withDefaults(
  defineProps<{
    tag?: string;
    staggerIndex?: number;
  }>(),
  { tag: "div", staggerIndex: 0 },
);

const root = ref<HTMLElement | null>(null);
const visible = ref(false);
const reducedMotion = useReducedMotion();

useIntersectionObserver(
  root,
  ([entry]) => {
    if (entry?.isIntersecting) visible.value = true;
  },
  { threshold: 0.15, rootMargin: "0px 0px -8% 0px" },
);

const delayMs = () => (reducedMotion.value ? 0 : props.staggerIndex * 80);
</script>

<template>
  <component
    :is="tag"
    ref="root"
    class="transition-[opacity,transform] duration-700 ease-out"
    :class="
      visible || reducedMotion
        ? 'opacity-100 translate-y-0'
        : 'opacity-0 translate-y-6'
    "
    :style="{ transitionDelay: `${delayMs()}ms` }"
  >
    <slot />
  </component>
</template>
