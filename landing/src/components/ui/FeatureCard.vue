<script setup lang="ts">
import RevealOnScroll from "./RevealOnScroll.vue";

defineProps<{
  title: string;
  body: string;
  staggerIndex?: number;
}>();
</script>

<template>
  <RevealOnScroll
    tag="article"
    :stagger-index="staggerIndex ?? 0"
    class="rounded-xl border border-border bg-surface p-6 shadow-lg shadow-black/20 transition-colors hover:border-accent/50"
  >
    <h3 class="text-lg font-semibold text-text">{{ title }}</h3>
    <p class="mt-2 text-sm leading-relaxed text-muted">{{ body }}</p>
  </RevealOnScroll>
</template>
