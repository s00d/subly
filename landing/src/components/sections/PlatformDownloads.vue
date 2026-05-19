<script setup lang="ts">
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { useReleaseDownloads } from "@/composables/useReleaseDownloads";
import type { MdiIconName } from "@/mdi-icons";
import type { PlatformKey } from "@/types/releases";

const { manifest, platformDownloadUrl } = useReleaseDownloads();

const cards: { key: PlatformKey; icon: MdiIconName }[] = [
  { key: "macos", icon: "apple" },
  { key: "windows", icon: "microsoft-windows" },
  { key: "linux", icon: "linux" },
];
</script>

<template>
  <div class="grid gap-4 sm:grid-cols-3">
    <RevealOnScroll
      v-for="(card, i) in cards"
      :key="card.key"
      tag="div"
      :stagger-index="i"
    >
      <a
        :href="platformDownloadUrl(card.key)"
        class="group flex h-full flex-col rounded-xl border border-border bg-surface p-5 no-underline transition-colors hover:border-accent/50"
        target="_blank"
        rel="noopener noreferrer"
      >
        <UiIcon :name="card.icon" size="lg" class="text-accent" />
        <span class="mt-3 font-semibold text-text">
          {{ manifest.platforms[card.key]?.label ?? card.key }}
        </span>
        <span v-if="manifest.version" class="mt-1 text-xs text-muted">
          v{{ manifest.version }}
        </span>
        <span
          v-if="manifest.platforms[card.key]?.fileName"
          class="mt-2 truncate text-xs text-muted"
        >
          {{ manifest.platforms[card.key]?.fileName }}
        </span>
        <span v-else class="mt-2 text-xs text-muted">View on GitHub Releases</span>
        <span class="mt-4 inline-flex items-center gap-1 text-sm font-medium text-accent">
          Download
          <UiIcon name="download" size="sm" />
        </span>
      </a>
    </RevealOnScroll>
  </div>
</template>
