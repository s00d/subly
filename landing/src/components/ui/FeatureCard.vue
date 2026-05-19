<script setup lang="ts">
import RevealOnScroll from "./RevealOnScroll.vue";
import UiIcon from "./UiIcon.vue";

defineProps<{
  title: string;
  body: string;
  icon: string;
  staggerIndex?: number;
}>();
</script>

<template>
  <RevealOnScroll
    tag="article"
    :stagger-index="staggerIndex ?? 0"
    class="rounded-xl border border-border bg-surface p-6 shadow-lg shadow-black/20 transition-colors hover:border-accent/50"
  >
    <div class="mb-3 flex items-center gap-2">
      <span class="flex size-9 items-center justify-center rounded-lg border border-border bg-bg">
        <UiIcon :name="icon" size="md" />
      </span>
      <h3 class="text-lg font-semibold text-text">{{ title }}</h3>
    </div>
    <p class="text-sm leading-relaxed text-muted">{{ body }}</p>
  </RevealOnScroll>
</template>
