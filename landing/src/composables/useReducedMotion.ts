import { useMediaQuery } from "@vueuse/core";
import { computed } from "vue";

export function useReducedMotion() {
  const prefersReducedMotion = useMediaQuery("(prefers-reduced-motion: reduce)");
  return computed(() => prefersReducedMotion.value);
}
