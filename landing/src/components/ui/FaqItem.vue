<script setup lang="ts">
import RevealOnScroll from "./RevealOnScroll.vue";

defineProps<{
  question: string;
  answer: string;
  staggerIndex?: number;
}>();
</script>

<template>
  <RevealOnScroll tag="details" :stagger-index="staggerIndex ?? 0" class="group rounded-xl border border-border bg-surface">
    <summary
      class="cursor-pointer list-none px-5 py-4 text-base font-medium text-text transition-colors hover:text-accent [&::-webkit-details-marker]:hidden"
    >
      <span class="flex items-center justify-between gap-3">
        {{ question }}
        <span
          class="shrink-0 text-muted transition-transform group-open:rotate-45"
          aria-hidden="true"
        >+</span>
      </span>
    </summary>
    <p class="border-t border-border px-5 pb-4 pt-3 text-sm leading-relaxed text-muted">
      {{ answer }}
    </p>
  </RevealOnScroll>
</template>
