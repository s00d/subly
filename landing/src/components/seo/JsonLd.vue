<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import {
  faqItems,
  siteMeta,
  videoSection,
} from "@/content/site.en";

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
      softwareVersion: siteMeta.softwareVersion,
      offers: {
        "@type": "Offer",
        price: "0",
        priceCurrency: "USD",
      },
      downloadUrl: siteMeta.releasesUrl,
      url: siteMeta.siteUrl,
    },
    {
      "@type": "VideoObject",
      "@id": `${siteMeta.siteUrl}#demo-video`,
      name: videoSection.title,
      description: videoSection.subtitle,
      contentUrl: siteMeta.videoUrl,
      thumbnailUrl: siteMeta.ogImage,
      uploadDate: "2026-05-07",
    },
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
