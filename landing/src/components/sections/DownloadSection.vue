<script setup lang="ts">
import { Download, Heart, Play, Star } from "@lucide/vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import { downloadSection, siteMeta } from "@/content/site.en";
</script>

<template>
  <section id="download" class="scroll-mt-24 px-5 py-20 md:py-28">
    <RevealOnScroll
      tag="div"
      class="mx-auto max-w-3xl rounded-2xl border border-accent/40 bg-accent/10 p-8 text-center md:p-12"
    >
      <h2 class="text-3xl font-bold tracking-tight text-text md:text-4xl">
        {{ downloadSection.title }}
      </h2>
      <p class="mt-4 text-lg text-muted">{{ downloadSection.body }}</p>
      <div class="mt-8 flex flex-wrap items-center justify-center gap-3">
        <a
          :href="siteMeta.releasesUrl"
          class="inline-flex items-center gap-2 rounded-xl bg-accent px-5 py-3 text-sm font-semibold text-white no-underline transition-colors hover:bg-accent-hover"
        >
          <Download :size="18" aria-hidden="true" />
          {{ downloadSection.ctaPrimary }}
        </a>
        <a
          :href="siteMeta.repoUrl"
          class="inline-flex items-center gap-2 rounded-xl border border-border bg-surface px-5 py-3 text-sm font-semibold text-text no-underline hover:border-accent/60"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Star :size="18" aria-hidden="true" />
          {{ downloadSection.ctaSecondary }}
        </a>
        <a
          :href="siteMeta.videoUrl"
          class="inline-flex items-center gap-2 rounded-xl border border-border bg-surface px-5 py-3 text-sm font-semibold text-text no-underline hover:border-accent/60"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Play :size="18" aria-hidden="true" />
          {{ downloadSection.ctaVideo }}
        </a>
        <a
          :href="siteMeta.donateUrl"
          class="inline-flex items-center gap-2 rounded-xl border border-border bg-bg px-5 py-3 text-sm font-semibold text-muted no-underline hover:text-text"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Heart :size="18" aria-hidden="true" />
          {{ downloadSection.ctaDonate }}
        </a>
      </div>
    </RevealOnScroll>
  </section>
</template>
