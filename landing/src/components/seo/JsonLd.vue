<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import {
  appStore,
  faqItems,
  siteMeta,
  videoSection,
} from "@/content/site.en";
import releases from "@/generated/releases.json";
import type { ReleasesManifest } from "@/types/releases";

const manifest = releases as ReleasesManifest;

const downloadUrls = [
  manifest.platforms.macos?.url,
  manifest.platforms.windows?.url,
  manifest.platforms.linux?.url,
  manifest.fallbackUrl,
].filter((url): url is string => Boolean(url));

const graph = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "WebSite",
      "@id": `${siteMeta.siteUrl}#website`,
      url: siteMeta.siteUrl,
      name: "Subly",
      description: siteMeta.description,
      inLanguage: "en",
    },
    {
      "@type": "SoftwareApplication",
      "@id": `${siteMeta.siteUrl}#app`,
      name: "Subly",
      applicationCategory: siteMeta.applicationCategory,
      operatingSystem: siteMeta.operatingSystems.join(", "),
      description: siteMeta.description,
      softwareVersion: manifest.version || siteMeta.softwareVersion,
      offers: [
        {
          "@type": "Offer",
          price: "0",
          priceCurrency: "USD",
          url: manifest.fallbackUrl,
        },
      ],
      downloadUrl: downloadUrls.length ? downloadUrls : siteMeta.releasesUrl,
      url: siteMeta.siteUrl,
      sameAs: [siteMeta.repoUrl, appStore.url],
    },
    ...videoSection.demos.map((demo) => ({
      "@type": "VideoObject",
      "@id": `${siteMeta.siteUrl}#demo-video-${demo.id}`,
      name: `${videoSection.title} — ${demo.label}`,
      description: demo.description,
      contentUrl: demo.url,
      thumbnailUrl: siteMeta.ogImage,
      uploadDate: "2026-05-07",
    })),
    {
      "@type": "FAQPage",
      "@id": `${siteMeta.siteUrl}#faq`,
      mainEntity: faqItems.map((item) => ({
        "@type": "Question",
        name: item.q,
        acceptedAnswer: {
          "@type": "Answer",
          text: item.a,
        },
      })),
    },
  ],
};

let scriptEl: HTMLScriptElement | null = null;

onMounted(() => {
  scriptEl = document.createElement("script");
  scriptEl.type = "application/ld+json";
  scriptEl.textContent = JSON.stringify(graph);
  document.head.appendChild(scriptEl);
});

onUnmounted(() => {
  scriptEl?.remove();
  scriptEl = null;
});
</script>

<template />
