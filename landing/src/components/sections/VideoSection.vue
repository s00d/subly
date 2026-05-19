<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { siteMeta, videoSection } from "@/content/site.en";
import { screenshotUrl } from "@/lib/assets";
</script>

<template>
  <SectionShell
    id="video"
    :eyebrow="videoSection.eyebrow"
    eyebrow-icon="play-circle-outline"
    :title="videoSection.title"
    :subtitle="videoSection.subtitle"
    muted
  >
    <RevealOnScroll tag="div" class="mx-auto max-w-4xl">
      <div class="overflow-hidden rounded-xl border border-border bg-black shadow-xl shadow-black/30">
        <video
          class="aspect-video w-full"
          controls
          playsinline
          preload="metadata"
          :poster="screenshotUrl('01_dashboard_overview.png')"
        >
          <source :src="siteMeta.videoUrl" type="video/mp4" />
          Your browser does not support embedded video.
        </video>
      </div>
      <p class="mt-4 text-center text-sm text-muted">
        <a
          :href="siteMeta.videoUrl"
          class="inline-flex items-center gap-1 text-accent no-underline hover:underline"
          target="_blank"
          rel="noopener noreferrer"
        >
          <UiIcon name="open-in-new" size="sm" />
          {{ videoSection.fallbackLabel }}
        </a>
      </p>
    </RevealOnScroll>
  </SectionShell>
</template>
