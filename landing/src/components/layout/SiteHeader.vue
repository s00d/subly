<script setup lang="ts">
import { Menu, X } from "@lucide/vue";
import { ref } from "vue";
import { navLinks, siteMeta } from "@/content/site.en";
import { assetUrl } from "@/lib/assets";
import { useActiveSection } from "@/composables/useActiveSection";

const sectionIds = ["why", "subscriptions", "dashboard", "ai", "sync", "faq"] as const;
const active = useActiveSection(sectionIds);
const menuOpen = ref(false);

function closeMenu() {
  menuOpen.value = false;
}
</script>

<template>
  <header
    class="sticky top-0 z-50 border-b border-border bg-bg/90 backdrop-blur-md"
  >
    <div class="mx-auto flex max-w-6xl items-center justify-between gap-4 px-5 py-3">
      <a href="#top" class="flex items-center gap-2.5 no-underline" @click="closeMenu">
        <img
          :src="assetUrl('app-icon.svg')"
          alt=""
          width="36"
          height="36"
          class="rounded-lg"
        />
        <span class="text-lg font-semibold text-text">Subly</span>
      </a>

      <nav class="hidden items-center gap-1 md:flex" aria-label="Primary">
        <a
          v-for="link in navLinks"
          :key="link.id"
          :href="`#${link.id}`"
          class="rounded-lg px-3 py-2 text-sm text-muted no-underline transition-colors hover:bg-surface hover:text-text"
          :class="active === link.id ? 'bg-surface text-text' : ''"
        >
          {{ link.label }}
        </a>
      </nav>

      <div class="flex items-center gap-2">
        <a
          :href="siteMeta.releasesUrl"
          class="hidden rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white no-underline transition-colors hover:bg-accent-hover sm:inline-block"
        >
          Download
        </a>
        <button
          type="button"
          class="inline-flex rounded-lg border border-border bg-surface p-2 text-text md:hidden"
          :aria-expanded="menuOpen"
          aria-label="Toggle menu"
          @click="menuOpen = !menuOpen"
        >
          <Menu v-if="!menuOpen" :size="20" />
          <X v-else :size="20" />
        </button>
      </div>
    </div>

    <nav
      v-if="menuOpen"
      class="border-t border-border bg-bg px-5 py-3 md:hidden"
      aria-label="Mobile"
    >
      <a
        v-for="link in navLinks"
        :key="link.id"
        :href="`#${link.id}`"
        class="block rounded-lg px-3 py-2 text-sm text-muted no-underline hover:bg-surface hover:text-text"
        @click="closeMenu"
      >
        {{ link.label }}
      </a>
      <a
        :href="siteMeta.releasesUrl"
        class="mt-2 block rounded-lg bg-accent px-3 py-2 text-center text-sm font-medium text-white no-underline"
        @click="closeMenu"
      >
        Download
      </a>
    </nav>
  </header>
</template>
