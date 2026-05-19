<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import ScreenshotFrame from "@/components/ui/ScreenshotFrame.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { screenshotUrl } from "@/lib/assets";

defineProps<{
  id: string;
  eyebrow: string;
  icon: string;
  title: string;
  body: string;
  bullets: readonly string[];
  screenshot: string;
  alt: string;
  reverse?: boolean;
  muted?: boolean;
}>();
</script>

<template>
  <SectionShell
    :id="id"
    :eyebrow="eyebrow"
    :eyebrow-icon="icon"
    :title="title"
    :subtitle="body"
    :muted="muted"
  >
    <div
      class="grid items-center gap-10 md:grid-cols-2 md:gap-14"
      :class="reverse ? 'md:[&>*:first-child]:order-2' : ''"
    >
      <RevealOnScroll tag="div">
        <ul class="space-y-3 text-muted">
          <li
            v-for="(item, i) in bullets"
            :key="i"
            class="flex gap-3 text-sm leading-relaxed md:text-base"
          >
            <UiIcon name="check-circle-outline" size="sm" class="mt-0.5 shrink-0 text-accent" />
            {{ item }}
          </li>
        </ul>
        <slot name="extra" />
      </RevealOnScroll>
      <RevealOnScroll tag="div" :stagger-index="1">
        <ScreenshotFrame :src="screenshotUrl(screenshot)" :alt="alt" />
      </RevealOnScroll>
    </div>
  </SectionShell>
</template>
