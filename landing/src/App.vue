<script setup lang="ts">
import { watch } from "vue";
import { useRoute } from "vue-router";
import SiteFooter from "@/components/layout/SiteFooter.vue";
import SiteHeader from "@/components/layout/SiteHeader.vue";
import JsonLd from "@/components/seo/JsonLd.vue";
import { useSiteContent } from "@/composables/useSiteContent";
import { RouterView } from "vue-router";

const route = useRoute();
const { content, locale } = useSiteContent();

watch(
  [() => route.meta.title, content, locale],
  () => {
    const title =
      typeof route.meta.title === "string"
        ? route.meta.title
        : content.value.siteMeta.title;
    document.title = title;
    document.documentElement.lang = locale.value === "ru" ? "ru" : "en";
  },
  { immediate: true },
);
</script>

<template>
  <div class="motion-safe min-h-screen bg-bg">
    <JsonLd />
    <SiteHeader />
    <main>
      <RouterView />
    </main>
    <SiteFooter />
  </div>
</template>
