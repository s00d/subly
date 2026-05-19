<script setup lang="ts">
import { RouterLink } from "vue-router";
import { useSiteContent } from "@/composables/useSiteContent";
import { assetUrl } from "@/lib/assets";
import UiIcon from "@/components/ui/UiIcon.vue";
import type { MdiIconName } from "@/mdi-icons";

const { content } = useSiteContent();
</script>

<template>
  <footer class="border-t border-border px-5 py-12">
    <div class="mx-auto flex max-w-6xl flex-col gap-8 md:flex-row md:items-center md:justify-between">
      <div class="flex items-center gap-3">
        <img :src="assetUrl('brand-logo.svg')" alt="" width="40" height="40" class="rounded-lg" />
        <div>
          <p class="font-medium text-text">{{ content.footer.tagline }}</p>
          <p class="text-sm text-muted">{{ content.footer.license }}</p>
        </div>
      </div>
      <nav class="flex flex-wrap gap-x-6 gap-y-2" aria-label="Footer">
        <template v-for="link in content.footer.links" :key="link.label">
          <RouterLink
            v-if="!link.external && 'to' in link"
            :to="link.to"
            class="inline-flex items-center gap-1.5 text-sm text-muted no-underline transition-colors hover:text-accent"
          >
            <UiIcon :name="link.icon as MdiIconName" size="sm" />
            {{ link.label }}
          </RouterLink>
          <a
            v-else
            :href="link.href"
            class="inline-flex items-center gap-1.5 text-sm text-muted no-underline transition-colors hover:text-accent"
            target="_blank"
            rel="noopener noreferrer"
          >
            <UiIcon :name="link.icon as MdiIconName" size="sm" />
            {{ link.label }}
          </a>
        </template>
        <a
          :href="content.siteMeta.donateUrl"
          class="inline-flex items-center gap-1.5 text-sm text-muted no-underline transition-colors hover:text-accent"
          target="_blank"
          rel="noopener noreferrer"
        >
          <UiIcon name="heart-outline" size="sm" />
          {{ content.footer.donateLabel }}
        </a>
      </nav>
    </div>
  </footer>
</template>
