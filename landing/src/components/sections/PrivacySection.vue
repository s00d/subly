<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { privacySection, siteMeta } from "@/content/site.en";
</script>

<template>
  <SectionShell
    id="privacy"
    :eyebrow="privacySection.eyebrow"
    :eyebrow-icon="privacySection.icon"
    :title="privacySection.title"
    :subtitle="privacySection.body"
    muted
  >
    <RevealOnScroll tag="div" class="grid gap-8 md:grid-cols-[1fr_auto] md:items-start">
      <ul class="space-y-4">
        <li
          v-for="(item, i) in privacySection.bullets"
          :key="i"
          class="flex gap-3 text-sm text-muted md:text-base"
        >
          <UiIcon name="shield-check-outline" size="md" class="mt-0.5 shrink-0 text-accent" />
          {{ item }}
        </li>
      </ul>
      <div class="flex gap-4 rounded-xl border border-border bg-surface p-6">
        <UiIcon name="lock-outline" size="lg" class="text-accent" />
        <div>
          <p class="font-medium text-text">Read the full policy</p>
          <a
            :href="siteMeta.privacyUrl"
            class="mt-2 inline-flex items-center gap-1 text-sm text-accent no-underline hover:underline"
            target="_blank"
            rel="noopener noreferrer"
          >
            Privacy Policy on GitHub
            <UiIcon name="open-in-new" size="sm" />
          </a>
        </div>
      </div>
    </RevealOnScroll>
  </SectionShell>
</template>
