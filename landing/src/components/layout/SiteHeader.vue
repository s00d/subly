<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { RouterLink } from "vue-router";
import LocaleSwitcher from "@/components/ui/LocaleSwitcher.vue";
import { useActiveSection } from "@/composables/useActiveSection";
import { preferredDownloadUrl } from "@/composables/useReleaseDownloads";
import { useSiteContent } from "@/composables/useSiteContent";
import { assetUrl } from "@/lib/assets";
import UiIcon from "@/components/ui/UiIcon.vue";
import type { MdiIconName } from "@/mdi-icons";

const { content, homePath, sectionLink } = useSiteContent();
const active = useActiveSection([
  "why",
  "subscriptions",
  "dashboard",
  "video",
  "ai",
  "sync",
  "faq",
]);
const menuOpen = ref(false);
const moreOpen = ref(false);
const moreRoot = ref<HTMLElement | null>(null);
const downloadUrl = ref(content.value?.siteMeta.releasesUrl ?? "");

const moreMenuActive = computed(() =>
  (content.value?.navMoreLinks ?? []).some((link) => active.value === link.id),
);

function closeMenu() {
  menuOpen.value = false;
}

function onDocClick(e: MouseEvent) {
  if (moreOpen.value && !moreRoot.value?.contains(e.target as Node)) {
    moreOpen.value = false;
  }
}

onMounted(() => {
  downloadUrl.value = preferredDownloadUrl();
  document.addEventListener("click", onDocClick);
});
onUnmounted(() => document.removeEventListener("click", onDocClick));

function navLinkClass(id: string) {
  return [
    "inline-flex items-center gap-1.5 rounded-lg px-2.5 py-2 text-sm text-muted no-underline transition-colors hover:bg-surface hover:text-text",
    active.value === id ? "bg-surface text-text" : "",
  ];
}
</script>

<template>
  <header class="sticky top-0 z-50 border-b border-border bg-bg/90 backdrop-blur-md">
    <div class="mx-auto flex max-w-6xl items-center justify-between gap-4 px-5 py-3">
      <RouterLink :to="homePath" class="flex shrink-0 items-center gap-2.5 no-underline" @click="closeMenu">
        <img :src="assetUrl('app-icon.svg')" alt="" width="36" height="36" class="rounded-lg" />
        <span class="text-lg font-semibold text-text">Subly</span>
      </RouterLink>

      <nav class="hidden min-w-0 items-center gap-0.5 lg:flex" aria-label="Primary">
        <RouterLink
          v-for="link in content.navPrimaryLinks"
          :key="link.id"
          :to="sectionLink(link.id)"
          :class="navLinkClass(link.id)"
        >
          <UiIcon :name="link.icon as MdiIconName" size="sm" class="shrink-0" />
          <span class="truncate">{{ link.label }}</span>
        </RouterLink>

        <div v-if="content.navMoreLinks.length" ref="moreRoot" class="relative">
          <button
            type="button"
            class="inline-flex items-center gap-1 rounded-lg border border-transparent px-2.5 py-2 text-sm text-muted transition-colors hover:bg-surface hover:text-text"
            :class="moreMenuActive || moreOpen ? 'border-border bg-surface text-text' : ''"
            :aria-expanded="moreOpen"
            :aria-label="content.headerNav.moreMenuAria"
            @click.stop="moreOpen = !moreOpen"
          >
            <UiIcon name="dots-horizontal" size="sm" />
            <span class="max-w-[4.5rem] truncate">{{ content.headerNav.moreLabel }}</span>
            <UiIcon
              name="chevron-down"
              size="sm"
              class="shrink-0 transition-transform"
              :class="moreOpen ? 'rotate-180' : ''"
            />
          </button>
          <ul
            v-if="moreOpen"
            class="absolute right-0 top-full z-50 mt-1 min-w-[11rem] overflow-hidden rounded-lg border border-border bg-surface py-1 shadow-lg shadow-black/40"
            role="menu"
          >
            <li v-for="link in content.navMoreLinks" :key="link.id" role="none">
              <RouterLink
                :to="sectionLink(link.id)"
                role="menuitem"
                class="flex items-center gap-2 px-3 py-2.5 text-sm text-muted no-underline transition-colors hover:bg-bg hover:text-text"
                :class="active === link.id ? 'bg-bg font-medium text-text' : ''"
                @click="moreOpen = false"
              >
                <UiIcon :name="link.icon as MdiIconName" size="sm" />
                {{ link.label }}
              </RouterLink>
            </li>
          </ul>
        </div>
      </nav>

      <div class="flex shrink-0 items-center gap-2">
        <LocaleSwitcher />
        <a
          :href="downloadUrl"
          class="hidden rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white no-underline transition-colors hover:bg-accent-hover sm:inline-flex sm:items-center sm:gap-1.5"
          target="_blank"
          rel="noopener noreferrer"
        >
          {{ content.headerNav.download }}
        </a>
        <button
          type="button"
          class="inline-flex rounded-lg border border-border bg-surface p-2 text-text lg:hidden"
          :aria-expanded="menuOpen"
          :aria-label="content.headerNav.menuToggle"
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
      <RouterLink
        v-for="link in content.navLinks"
        :key="link.id"
        :to="sectionLink(link.id)"
        class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm text-muted no-underline hover:bg-surface hover:text-text"
        :class="active === link.id ? 'bg-surface text-text' : ''"
        @click="closeMenu"
      >
        <UiIcon :name="link.icon as MdiIconName" size="sm" />
        {{ link.label }}
      </RouterLink>
      <a
        :href="downloadUrl"
        class="mt-2 flex items-center justify-center gap-2 rounded-lg bg-accent px-3 py-2 text-sm font-medium text-white no-underline"
        target="_blank"
        rel="noopener noreferrer"
        @click="closeMenu"
      >
        {{ content.headerNav.download }}
      </a>
    </nav>
  </header>
</template>
