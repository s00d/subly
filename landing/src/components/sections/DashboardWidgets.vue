<script setup lang="ts">
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import { dashboardWidgets } from "@/content/site.en";
</script>

<template>
  <RevealOnScroll tag="div" class="mt-6 flex flex-wrap gap-2">
    <span
      v-for="(label, i) in dashboardWidgets"
      :key="label"
      class="rounded-full border border-border bg-bg px-3 py-1 text-xs text-muted"
      :style="{ transitionDelay: `${i * 40}ms` }"
    >
      {{ label }}
    </span>
  </RevealOnScroll>
</template>
