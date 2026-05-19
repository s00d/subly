<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from "vue";
import { useSiteContent } from "@/composables/useSiteContent";
import releases from "@/generated/releases.json";
import type { ReleasesManifest } from "@/types/releases";

const { content } = useSiteContent();
const manifest = releases as ReleasesManifest;

const graph = computed(() => {
  const meta = content.value.siteMeta;
  const downloadUrls = [
    manifest.platforms.macos?.url,
    manifest.platforms.windows?.url,
    manifest.platforms.linux?.url,
    manifest.fallbackUrl,
  ].filter((url): url is string => Boolean(url));

  return {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "WebSite",
        "@id": `${meta.siteUrl}#website`,
        url: meta.siteUrl,
        name: "Subly",
        description: meta.description,
        inLanguage: meta.locale.startsWith("ru") ? "ru" : "en",
      },
      {
        "@type": "SoftwareApplication",
        "@id": `${meta.siteUrl}#app`,
        name: "Subly",
        applicationCategory: meta.applicationCategory,
        operatingSystem: meta.operatingSystems.join(", "),
        description: meta.description,
        softwareVersion: manifest.version || meta.softwareVersion,
        offers: [
          {
            "@type": "Offer",
            price: "0",
            priceCurrency: "USD",
            url: manifest.fallbackUrl,
          },
        ],
        downloadUrl: downloadUrls.length ? downloadUrls : meta.releasesUrl,
        url: meta.siteUrl,
        sameAs: [meta.repoUrl, content.value.appStore.url],
      },
      ...content.value.videoSection.demos.map((demo) => ({
        "@type": "VideoObject",
        "@id": `${meta.siteUrl}#demo-video-${demo.id}`,
        name: `${content.value.videoSection.title} — ${demo.label}`,
        description: demo.description,
        contentUrl: demo.url,
        thumbnailUrl: meta.ogImage,
        uploadDate: "2026-05-07",
      })),
      {
        "@type": "FAQPage",
        "@id": `${meta.siteUrl}#faq`,
        mainEntity: content.value.faqItems.map((item) => ({
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
});

let scriptEl: HTMLScriptElement | null = null;

function syncJsonLd() {
  scriptEl?.remove();
  scriptEl = document.createElement("script");
  scriptEl.type = "application/ld+json";
  scriptEl.textContent = JSON.stringify(graph.value);
  document.head.appendChild(scriptEl);
}

onMounted(syncJsonLd);
watch(graph, syncJsonLd);

onUnmounted(() => {
  scriptEl?.remove();
  scriptEl = null;
});
</script>

<template />
