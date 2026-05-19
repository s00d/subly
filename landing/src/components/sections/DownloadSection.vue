<script setup lang="ts">
import { onMounted, ref } from "vue";
import { RouterLink } from "vue-router";
import AppStoreSection from "@/components/sections/AppStoreSection.vue";
import PlatformDownloads from "@/components/sections/PlatformDownloads.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { preferredDownloadUrl } from "@/composables/useReleaseDownloads";
import { downloadSection, siteMeta } from "@/content/site.en";

const primaryUrl = ref(siteMeta.releasesUrl);

onMounted(() => {
  primaryUrl.value = preferredDownloadUrl();
});
</script>

<template>
  <section id="download" class="scroll-mt-24 px-5 py-20 md:py-28">
    <RevealOnScroll
      tag="div"
      class="mx-auto max-w-4xl rounded-2xl border border-accent/40 bg-accent/10 p-8 text-center md:p-12"
    >
      <h2 class="text-3xl font-bold tracking-tight text-text md:text-4xl">
        {{ downloadSection.title }}
      </h2>
      <p class="mt-4 text-lg text-muted">{{ downloadSection.body }}</p>

      <p class="mt-10 text-sm font-semibold uppercase tracking-wide text-muted">
        {{ downloadSection.githubHeading }}
      </p>
      <PlatformDownloads class="mt-6 text-left" />

      <AppStoreSection />

      <div class="mt-10 flex flex-wrap items-center justify-center gap-3">
        <a
          :href="primaryUrl"
          class="inline-flex items-center gap-2 rounded-xl bg-accent px-5 py-3 text-sm font-semibold text-white no-underline transition-colors hover:bg-accent-hover"
          target="_blank"
          rel="noopener noreferrer"
        >
          <UiIcon name="download" size="sm" />
          {{ downloadSection.ctaPrimary }}
        </a>
        <a
          :href="siteMeta.repoUrl"
          class="inline-flex items-center gap-2 rounded-xl border border-border bg-surface px-5 py-3 text-sm font-semibold text-text no-underline hover:border-accent/60"
          target="_blank"
          rel="noopener noreferrer"
        >
          <UiIcon name="star-outline" size="sm" />
          {{ downloadSection.ctaSecondary }}
        </a>
        <RouterLink
          to="/changelog"
          class="inline-flex items-center gap-2 rounded-xl border border-border bg-surface px-5 py-3 text-sm font-semibold text-text no-underline hover:border-accent/60"
        >
          <UiIcon name="history" size="sm" />
          Changelog
        </RouterLink>
        <a
          href="#video"
          class="inline-flex items-center gap-2 rounded-xl border border-border bg-surface px-5 py-3 text-sm font-semibold text-text no-underline hover:border-accent/60"
        >
          <UiIcon name="play-circle-outline" size="sm" />
          {{ downloadSection.ctaVideo }}
        </a>
        <a
          :href="siteMeta.donateUrl"
          class="inline-flex items-center gap-2 rounded-xl border border-border bg-bg px-5 py-3 text-sm font-semibold text-muted no-underline hover:text-text"
          target="_blank"
          rel="noopener noreferrer"
        >
          <UiIcon name="heart-outline" size="sm" />
          {{ downloadSection.ctaDonate }}
        </a>
      </div>
    </RevealOnScroll>
  </section>
</template>
