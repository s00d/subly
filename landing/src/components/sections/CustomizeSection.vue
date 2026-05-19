<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import ScreenshotFrame from "@/components/ui/ScreenshotFrame.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { useSiteContent } from "@/composables/useSiteContent";
import { screenshotUrl } from "@/lib/assets";

const { content } = useSiteContent();
</script>

<template>
  <SectionShell
    id="customize"
    :eyebrow="content.customizeSection.eyebrow"
    :eyebrow-icon="content.customizeSection.icon"
    :title="content.customizeSection.title"
    :subtitle="content.customizeSection.body"
  >
    <div class="grid items-center gap-10 md:grid-cols-2 md:gap-14">
      <RevealOnScroll tag="div">
        <ul class="space-y-3">
          <li
            v-for="(item, i) in content.customizeSection.bullets"
            :key="i"
            class="flex gap-3 text-sm text-muted md:text-base"
          >
            <UiIcon name="check-circle-outline" size="sm" class="mt-0.5 shrink-0 text-accent" />
            {{ item }}
          </li>
        </ul>
      </RevealOnScroll>
      <RevealOnScroll tag="div" :stagger-index="1">
        <ScreenshotFrame
          :src="screenshotUrl(content.customizeSection.screenshot)"
          :alt="content.customizeSection.alt"
        />
      </RevealOnScroll>
    </div>
  </SectionShell>
</template>
