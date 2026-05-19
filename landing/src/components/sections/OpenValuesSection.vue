<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import FeatureCard from "@/components/ui/FeatureCard.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { openValuesSection, siteMeta } from "@/content/site.en";
</script>

<template>
  <SectionShell
    id="open"
    :eyebrow="openValuesSection.eyebrow"
    :eyebrow-icon="openValuesSection.icon"
    :title="openValuesSection.title"
  >
    <RevealOnScroll tag="div" class="max-w-3xl">
      <p class="text-lg leading-relaxed text-muted md:text-xl">
        {{ openValuesSection.lead }}
      </p>
    </RevealOnScroll>

    <div class="mt-12 grid gap-5 lg:grid-cols-3">
      <FeatureCard
        v-for="(pillar, i) in openValuesSection.pillars"
        :key="pillar.title"
        :title="pillar.title"
        :body="pillar.body"
        :icon="pillar.icon"
        :stagger-index="i"
      />
    </div>

    <RevealOnScroll tag="div" :stagger-index="3" class="mt-12 grid gap-8 lg:grid-cols-[1fr_auto] lg:items-center">
      <ul class="grid gap-3 sm:grid-cols-2">
        <li
          v-for="(item, i) in openValuesSection.productHighlights"
          :key="i"
          class="flex gap-3 rounded-xl border border-border bg-surface/80 px-4 py-3 text-sm text-muted"
        >
          <UiIcon name="check-circle-outline" size="sm" class="mt-0.5 shrink-0 text-accent" />
          {{ item }}
        </li>
      </ul>
      <a
        :href="siteMeta.repoUrl"
        class="inline-flex items-center justify-center gap-2 rounded-xl bg-accent px-6 py-3.5 text-sm font-semibold text-white no-underline transition-colors hover:bg-accent-hover lg:shrink-0"
        target="_blank"
        rel="noopener noreferrer"
      >
        <UiIcon name="github" size="sm" />
        {{ openValuesSection.ctaLabel }}
        <UiIcon name="open-in-new" size="sm" />
      </a>
    </RevealOnScroll>
  </SectionShell>
</template>
