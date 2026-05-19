<script setup lang="ts">
import { computed, ref, watch } from "vue";
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { useSiteContent } from "@/composables/useSiteContent";
import { screenshotUrl } from "@/lib/assets";
import type { MdiIconName } from "@/mdi-icons";

const { content } = useSiteContent();
const activeId = ref(content.value.videoSection.demos[0]?.id ?? "desktop");
const videoKey = ref(0);

const activeDemo = computed(
  () =>
    content.value.videoSection.demos.find((d) => d.id === activeId.value) ??
    content.value.videoSection.demos[0],
);

watch(activeId, () => {
  videoKey.value += 1;
});
</script>

<template>
  <SectionShell
    id="video"
    :eyebrow="content.videoSection.eyebrow"
    eyebrow-icon="play-circle-outline"
    :title="content.videoSection.title"
    :subtitle="content.videoSection.subtitle"
    muted
  >
    <div class="mb-8 flex flex-wrap justify-center gap-2">
      <button
        v-for="tab in content.videoSection.demos"
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
      {{ activeDemo.description }}
    </p>

    <RevealOnScroll
      tag="div"
      class="mx-auto max-w-4xl"
      :class="activeDemo.aspect === 'portrait' ? 'max-w-sm' : ''"
    >
      <div class="overflow-hidden rounded-xl border border-border bg-black shadow-xl shadow-black/30">
        <video
          :key="videoKey"
          class="w-full"
          :class="activeDemo.aspect === 'portrait' ? 'aspect-[9/16]' : 'aspect-video'"
          controls
          playsinline
          preload="metadata"
          :poster="screenshotUrl(activeDemo.poster)"
        >
          <source :src="activeDemo.url" type="video/mp4" />
          Your browser does not support embedded video.
        </video>
      </div>
      <p class="mt-4 text-center text-sm text-muted">
        <a
          :href="activeDemo.url"
          class="inline-flex items-center gap-1 text-accent no-underline hover:underline"
          target="_blank"
          rel="noopener noreferrer"
        >
          <UiIcon name="open-in-new" size="sm" />
          {{ content.videoSection.fallbackLabel }}
        </a>
      </p>
    </RevealOnScroll>
  </SectionShell>
</template>
