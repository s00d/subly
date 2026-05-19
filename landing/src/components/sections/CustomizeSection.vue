<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import ScreenshotFrame from "@/components/ui/ScreenshotFrame.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { customizeSection } from "@/content/site.en";
import { screenshotUrl } from "@/lib/assets";
</script>

<template>
  <SectionShell
    id="customize"
    :eyebrow="customizeSection.eyebrow"
    :eyebrow-icon="customizeSection.icon"
    :title="customizeSection.title"
    :subtitle="customizeSection.body"
  >
    <div class="grid items-center gap-10 md:grid-cols-2 md:gap-14">
      <RevealOnScroll tag="div">
        <ul class="space-y-3 text-muted">
          <li
            v-for="(item, i) in customizeSection.bullets"
            :key="i"
            class="flex gap-3 text-sm md:text-base"
          >
            <UiIcon name="check-circle-outline" size="sm" class="mt-0.5 shrink-0 text-accent" />
            {{ item }}
          </li>
        </ul>
      </RevealOnScroll>
      <RevealOnScroll tag="div" :stagger-index="1">
        <ScreenshotFrame
          :src="screenshotUrl(customizeSection.screenshot)"
          :alt="customizeSection.alt"
        />
      </RevealOnScroll>
    </div>
  </SectionShell>
</template>
