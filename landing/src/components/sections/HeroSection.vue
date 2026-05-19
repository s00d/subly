<script setup lang="ts">
import { onMounted, ref } from "vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import ScreenshotFrame from "@/components/ui/ScreenshotFrame.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { preferredDownloadUrl } from "@/composables/useReleaseDownloads";
import { useSiteContent } from "@/composables/useSiteContent";
import { assetUrl, screenshotUrl } from "@/lib/assets";

const { content } = useSiteContent();
const downloadUrl = ref(content.value.siteMeta.releasesUrl);

onMounted(() => {
  downloadUrl.value = preferredDownloadUrl();
});
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
          {{ content.hero.headline }}
        </h1>
        <p class="mt-5 max-w-xl text-lg text-muted">
          {{ content.hero.subline }}
        </p>
        <div class="mt-8 flex flex-wrap gap-3">
          <a
            :href="downloadUrl"
            class="inline-flex items-center gap-2 rounded-xl bg-accent px-5 py-3 text-sm font-semibold text-white no-underline transition-colors hover:bg-accent-hover"
            target="_blank"
            rel="noopener noreferrer"
          >
            {{ content.hero.ctaPrimary }}
            <UiIcon name="arrow-right" size="sm" />
          </a>
          <a
            :href="content.siteMeta.repoUrl"
            class="inline-flex items-center gap-2 rounded-xl border border-border bg-surface px-5 py-3 text-sm font-semibold text-text no-underline transition-colors hover:border-accent/60"
            target="_blank"
            rel="noopener noreferrer"
          >
            <UiIcon name="open-in-new" size="sm" />
            {{ content.hero.ctaSecondary }}
          </a>
          <a
            href="#video"
            class="inline-flex items-center gap-2 rounded-xl border border-border bg-bg px-5 py-3 text-sm font-semibold text-muted no-underline transition-colors hover:border-accent/60 hover:text-text"
          >
            <UiIcon name="play-circle-outline" size="sm" />
            {{ content.hero.watchDemo }}
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
