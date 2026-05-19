<script setup lang="ts">
import RevealOnScroll from "./RevealOnScroll.vue";
import UiIcon from "./UiIcon.vue";

defineProps<{
  question: string;
  answer: string;
  staggerIndex?: number;
}>();
</script>

<template>
  <RevealOnScroll
    tag="details"
    :stagger-index="staggerIndex ?? 0"
    class="group rounded-xl border border-border bg-surface"
  >
    <summary
      class="flex cursor-pointer list-none items-center justify-between gap-3 px-5 py-4 text-base font-medium text-text transition-colors hover:text-accent [&::-webkit-details-marker]:hidden"
    >
      <span>{{ question }}</span>
      <UiIcon
        name="chevron-down"
        size="md"
        class="shrink-0 text-muted transition-transform duration-300 group-open:rotate-180"
      />
    </summary>
    <p class="border-t border-border px-5 pb-4 pt-3 text-sm leading-relaxed text-muted">
      {{ answer }}
    </p>
  </RevealOnScroll>
</template>
