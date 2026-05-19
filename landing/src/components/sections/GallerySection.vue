<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import { gallery } from "@/content/site.en";
import { screenshotUrl } from "@/lib/assets";
</script>

<template>
  <SectionShell id="screens" :title="gallery.title" :subtitle="gallery.subtitle" muted>
    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      <RevealOnScroll
        v-for="(shot, i) in gallery.shots"
        :key="shot.file"
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
