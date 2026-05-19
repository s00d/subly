<script setup lang="ts">
import { computed, ref } from "vue";
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { deviceGalleries, deviceGalleryMeta } from "@/content/site.en";
import { screenshotUrl } from "@/lib/assets";
import type { MdiIconName } from "@/mdi-icons";

const activeId = ref(deviceGalleries[0]?.id ?? "desktop");

const activeGallery = computed(
  () => deviceGalleries.find((g) => g.id === activeId.value) ?? deviceGalleries[0],
);
</script>

<template>
  <SectionShell
    id="screens"
    :title="deviceGalleryMeta.title"
    :subtitle="deviceGalleryMeta.subtitle"
    muted
  >
    <div class="mb-8 flex flex-wrap justify-center gap-2">
      <button
        v-for="tab in deviceGalleries"
        :key="tab.id"
        type="button"
        class="inline-flex items-center gap-2 rounded-xl border px-4 py-2.5 text-sm font-medium transition-colors"
        :class="
          activeId === tab.id
            ? 'border-accent bg-accent/15 text-text'
            : 'border-border bg-surface text-muted hover:text-text'
        "
        @click="activeId = tab.id"
      >
        <UiIcon :name="tab.icon as MdiIconName" size="sm" />
        {{ tab.label }}
      </button>
    </div>

    <p class="mb-8 text-center text-muted">
      {{ activeGallery.description }}
    </p>

    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      <RevealOnScroll
        v-for="(shot, i) in activeGallery.shots"
        :key="`${activeId}-${shot.file}`"
        tag="figure"
        :stagger-index="i"
        class="overflow-hidden rounded-xl border border-border bg-surface transition-transform hover:scale-[1.02]"
      >
        <img
          :src="screenshotUrl(shot.file)"
          :alt="`Subly ${shot.label}`"
          class="block w-full"
          loading="lazy"
        />
        <figcaption class="border-t border-border px-3 py-2 text-center text-sm text-muted">
          {{ shot.label }}
        </figcaption>
      </RevealOnScroll>
    </div>
  </SectionShell>
</template>
