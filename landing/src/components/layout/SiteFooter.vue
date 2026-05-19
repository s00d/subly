<script setup lang="ts">
import { footer, siteMeta } from "@/content/site.en";
import { assetUrl } from "@/lib/assets";
</script>

<template>
  <footer class="border-t border-border px-5 py-12">
    <div class="mx-auto flex max-w-6xl flex-col gap-8 md:flex-row md:items-center md:justify-between">
      <div class="flex items-center gap-3">
        <img :src="assetUrl('brand-logo.svg')" alt="" width="40" height="40" class="rounded-lg" />
        <div>
          <p class="font-medium text-text">{{ footer.tagline }}</p>
          <p class="text-sm text-muted">{{ footer.license }}</p>
        </div>
      </div>
      <nav class="flex flex-wrap gap-x-6 gap-y-2" aria-label="Footer">
        <a
          v-for="link in footer.links"
          :key="link.href"
          :href="link.href"
          class="text-sm text-muted no-underline transition-colors hover:text-accent"
          target="_blank"
          rel="noopener noreferrer"
        >
          {{ link.label }}
        </a>
        <a
          :href="siteMeta.donateUrl"
          class="text-sm text-muted no-underline transition-colors hover:text-accent"
          target="_blank"
          rel="noopener noreferrer"
        >
          Donate
        </a>
      </nav>
    </div>
  </footer>
</template>
