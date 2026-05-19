<script setup lang="ts">
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { platforms } from "@/content/site.en";
</script>

<template>
  <section id="platforms" class="border-y border-border bg-surface/50 px-5 py-10">
    <RevealOnScroll tag="div" class="mx-auto max-w-6xl text-center">
      <p class="text-sm font-medium uppercase tracking-widest text-muted">
        {{ platforms.title }}
      </p>
      <ul class="mt-6 flex flex-wrap items-center justify-center gap-4 md:gap-8">
        <li
          v-for="item in platforms.items"
          :key="item.name"
          class="flex items-center gap-2 rounded-full border border-border bg-bg px-4 py-2 text-sm font-medium text-text"
        >
          <UiIcon :name="item.icon" size="sm" class="text-accent" />
          {{ item.name }}
        </li>
      </ul>
      <p class="mt-6 text-base text-muted">{{ platforms.tagline }}</p>
    </RevealOnScroll>
  </section>
</template>
