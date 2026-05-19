<script setup lang="ts">
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { appStore } from "@/content/site.en";
</script>

<template>
  <RevealOnScroll
    tag="div"
    class="mx-auto mt-10 max-w-3xl rounded-2xl border border-border bg-surface p-8 text-center md:p-10"
  >
    <UiIcon name="apple" size="lg" class="mx-auto text-text" />
    <h3 class="mt-4 text-xl font-semibold text-text">{{ appStore.title }}</h3>
    <p class="mt-3 text-muted">{{ appStore.body }}</p>
    <p class="mt-2 text-sm text-muted">
      {{ appStore.platforms.join(" · ") }}
    </p>
    <a
      :href="appStore.url"
      class="mt-6 inline-flex items-center gap-2 rounded-xl border border-border bg-bg px-5 py-3 text-sm font-semibold text-text no-underline transition-colors hover:border-accent/60"
      target="_blank"
      rel="noopener noreferrer"
    >
      <UiIcon name="apple" size="sm" />
      View on the App Store
      <UiIcon name="open-in-new" size="sm" />
    </a>
  </RevealOnScroll>
</template>
