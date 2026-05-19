<script setup lang="ts">
import UiIcon from "@/components/ui/UiIcon.vue";
import { useSiteContent } from "@/composables/useSiteContent";
import type { MdiIconName } from "@/mdi-icons";

const { content } = useSiteContent();
</script>

<template>
  <section class="border-y border-border bg-surface/30 px-5 py-10">
    <div class="mx-auto max-w-6xl text-center">
      <p class="text-sm font-semibold uppercase tracking-widest text-muted">
        {{ content.platforms.title }}
      </p>
      <ul class="mt-6 flex flex-wrap items-center justify-center gap-6 md:gap-10">
        <li
          v-for="p in content.platforms.items"
          :key="p.name"
          class="inline-flex items-center gap-2 text-text"
        >
          <UiIcon :name="p.icon as MdiIconName" size="md" class="text-accent" />
          <span class="font-medium">{{ p.name }}</span>
        </li>
      </ul>
      <p class="mt-6 text-sm text-muted">{{ content.platforms.tagline }}</p>
    </div>
  </section>
</template>
