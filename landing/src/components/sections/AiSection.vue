<script setup lang="ts">
import { Sparkles } from "@lucide/vue";
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import ScreenshotFrame from "@/components/ui/ScreenshotFrame.vue";
import { aiSection } from "@/content/site.en";
import { screenshotUrl } from "@/lib/assets";
</script>

<template>
  <SectionShell
    id="ai"
    :eyebrow="aiSection.eyebrow"
    :title="aiSection.title"
    :subtitle="aiSection.body"
    muted
  >
    <div class="grid items-center gap-10 md:grid-cols-2 md:gap-14">
      <RevealOnScroll tag="div">
        <ul class="space-y-3">
          <li
            v-for="(item, i) in aiSection.bullets"
            :key="i"
            class="flex gap-3 text-sm text-muted md:text-base"
          >
            <Sparkles :size="18" class="mt-0.5 shrink-0 text-accent" aria-hidden="true" />
            {{ item }}
          </li>
        </ul>
      </RevealOnScroll>
      <RevealOnScroll tag="div" :stagger-index="1">
        <ScreenshotFrame
          :src="screenshotUrl(aiSection.screenshot)"
          :alt="aiSection.alt"
        />
      </RevealOnScroll>
    </div>
  </SectionShell>
</template>
