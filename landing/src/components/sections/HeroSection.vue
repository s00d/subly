<script setup lang="ts">
import { ArrowRight, ExternalLink } from "@lucide/vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import ScreenshotFrame from "@/components/ui/ScreenshotFrame.vue";
import { hero, siteMeta } from "@/content/site.en";
import { assetUrl, screenshotUrl } from "@/lib/assets";
</script>

<template>
  <section id="top" class="scroll-mt-0 px-5 pb-16 pt-12 md:pb-24 md:pt-16">
    <div class="mx-auto grid max-w-6xl items-center gap-12 md:grid-cols-2 md:gap-16">
      <RevealOnScroll tag="div">
        <img
          :src="assetUrl('brand-logo.svg')"
          alt="Subly"
          width="72"
          height="72"
          class="mb-6 rounded-2xl"
        />
        <h1 class="text-4xl font-bold tracking-tight text-text md:text-5xl lg:text-[3.25rem] lg:leading-tight">
          {{ hero.headline }}
        </h1>
        <p class="mt-5 max-w-xl text-lg text-muted">
          {{ hero.subline }}
        </p>
        <div class="mt-8 flex flex-wrap gap-3">
          <a
            :href="siteMeta.releasesUrl"
            class="inline-flex items-center gap-2 rounded-xl bg-accent px-5 py-3 text-sm font-semibold text-white no-underline transition-colors hover:bg-accent-hover"
          >
            {{ hero.ctaPrimary }}
            <ArrowRight :size="18" aria-hidden="true" />
          </a>
          <a
            :href="siteMeta.repoUrl"
            class="inline-flex items-center gap-2 rounded-xl border border-border bg-surface px-5 py-3 text-sm font-semibold text-text no-underline transition-colors hover:border-accent/60"
            target="_blank"
            rel="noopener noreferrer"
          >
            <ExternalLink :size="18" aria-hidden="true" />
            {{ hero.ctaSecondary }}
          </a>
        </div>
      </RevealOnScroll>
      <RevealOnScroll tag="div" :stagger-index="1">
        <ScreenshotFrame
          :src="screenshotUrl('01_dashboard_overview.png')"
          alt="Subly dashboard"
          float
        />
      </RevealOnScroll>
    </div>
  </section>
</template>
