<script setup lang="ts">
import { onMounted, ref } from "vue";
import { RouterLink } from "vue-router";
import { navLinks, siteMeta } from "@/content/site.en";
import { useActiveSection } from "@/composables/useActiveSection";
import { preferredDownloadUrl } from "@/composables/useReleaseDownloads";
import { assetUrl } from "@/lib/assets";
import UiIcon from "@/components/ui/UiIcon.vue";

const sectionIds = navLinks.map((l) => l.id);
const active = useActiveSection(sectionIds);
const menuOpen = ref(false);
const downloadUrl = ref(siteMeta.releasesUrl);

onMounted(() => {
  downloadUrl.value = preferredDownloadUrl();
});

function closeMenu() {
  menuOpen.value = false;
}
</script>

<template>
  <header
    class="sticky top-0 z-50 border-b border-border bg-bg/90 backdrop-blur-md"
  >
    <div class="mx-auto flex max-w-6xl items-center justify-between gap-4 px-5 py-3">
      <RouterLink to="/" class="flex items-center gap-2.5 no-underline" @click="closeMenu">
        <img
          :src="assetUrl('app-icon.svg')"
          alt=""
          width="36"
          height="36"
          class="rounded-lg"
        />
        <span class="text-lg font-semibold text-text">Subly</span>
      </RouterLink>

      <nav class="hidden items-center gap-0.5 lg:flex" aria-label="Primary">
        <a
          v-for="link in navLinks"
          :key="link.id"
          :href="`#${link.id}`"
          class="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-2 text-sm text-muted no-underline transition-colors hover:bg-surface hover:text-text"
          :class="active === link.id ? 'bg-surface text-text' : ''"
        >
          <UiIcon :name="link.icon" size="sm" />
          {{ link.label }}
        </a>
        <a
          href="#download"
          class="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-2 text-sm text-muted no-underline transition-colors hover:bg-surface hover:text-text"
        >
          <UiIcon name="download" size="sm" />
          Download
        </a>
      </nav>

      <div class="flex items-center gap-2">
        <a
          :href="downloadUrl"
          class="hidden rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white no-underline transition-colors hover:bg-accent-hover sm:inline-flex sm:items-center sm:gap-1.5"
          target="_blank"
          rel="noopener noreferrer"
        >
          Download
        </a>
        <button
          type="button"
          class="inline-flex rounded-lg border border-border bg-surface p-2 text-text lg:hidden"
          :aria-expanded="menuOpen"
          aria-label="Toggle menu"
          @click="menuOpen = !menuOpen"
        >
          <UiIcon :name="menuOpen ? 'close' : 'menu'" size="md" />
        </button>
      </div>
    </div>

    <nav
      v-if="menuOpen"
      class="border-t border-border bg-bg px-5 py-3 lg:hidden"
      aria-label="Mobile"
    >
      <a
        v-for="link in navLinks"
        :key="link.id"
        :href="`#${link.id}`"
        class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm text-muted no-underline hover:bg-surface hover:text-text"
        @click="closeMenu"
      >
        <UiIcon :name="link.icon" size="sm" />
        {{ link.label }}
      </a>
      <a
        :href="downloadUrl"
        class="mt-2 flex items-center justify-center gap-2 rounded-lg bg-accent px-3 py-2 text-sm font-medium text-white no-underline"
        target="_blank"
        rel="noopener noreferrer"
        @click="closeMenu"
      >
        Download
      </a>
    </nav>
  </header>
</template>
