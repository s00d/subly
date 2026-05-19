<script setup lang="ts">
import { watch } from "vue";
import { useRoute } from "vue-router";
import SiteFooter from "@/components/layout/SiteFooter.vue";
import SiteHeader from "@/components/layout/SiteHeader.vue";
import JsonLd from "@/components/seo/JsonLd.vue";
import { siteMeta } from "@/content/site.en";
import { RouterView } from "vue-router";

const route = useRoute();

watch(
  () => route.meta.title,
  (title) => {
    document.title = typeof title === "string" ? title : siteMeta.title;
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
