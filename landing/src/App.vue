<script setup lang="ts">
import { watch } from "vue";
import { useRoute } from "vue-router";
import SiteFooter from "@/components/layout/SiteFooter.vue";
import SiteHeader from "@/components/layout/SiteHeader.vue";
import JsonLd from "@/components/seo/JsonLd.vue";
import { useSiteContent } from "@/composables/useSiteContent";
import { LOCALE_META } from "@/content/locales";
import { RouterView } from "vue-router";

const route = useRoute();
const { content, locale, ready, loading, loadError } = useSiteContent();

watch(
  [() => route.meta.title, content, locale, ready],
  () => {
    const meta = LOCALE_META[locale.value];
    document.documentElement.lang = meta.htmlLang;
    document.documentElement.dir = locale.value === "ar" ? "rtl" : "ltr";

    if (!ready.value || !content.value) return;

    document.title =
      typeof route.meta.title === "string"
        ? route.meta.title
        : content.value.siteMeta.title;
  },
  { immediate: true },
);
</script>

<template>
  <div v-if="ready && content" class="motion-safe min-h-screen bg-bg">
    <JsonLd />
    <SiteHeader />
    <main>
      <RouterView />
    </main>
    <SiteFooter />
  </div>
  <div
    v-else-if="loading"
    class="flex min-h-screen items-center justify-center bg-bg text-sm text-muted"
    aria-busy="true"
  >
    …
  </div>
  <div
    v-else
    class="flex min-h-screen items-center justify-center bg-bg px-6 text-center text-sm text-muted"
  >
    {{ loadError ?? "Failed to load page content." }}
  </div>
</template>
