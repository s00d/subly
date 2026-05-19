<script setup lang="ts">
import { Lock, Shield } from "@lucide/vue";
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import { privacySection, siteMeta } from "@/content/site.en";
</script>

<template>
  <SectionShell
    id="privacy"
    :eyebrow="privacySection.eyebrow"
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
          <Shield :size="20" class="mt-0.5 shrink-0 text-accent" aria-hidden="true" />
          {{ item }}
        </li>
      </ul>
      <div class="flex gap-4 rounded-xl border border-border bg-surface p-6">
        <Lock :size="32" class="text-accent" aria-hidden="true" />
        <div>
          <p class="font-medium text-text">Read the full policy</p>
          <a
            :href="siteMeta.privacyUrl"
            class="mt-2 inline-block text-sm text-accent no-underline hover:underline"
            target="_blank"
            rel="noopener noreferrer"
          >
            Privacy Policy on GitHub →
          </a>
        </div>
      </div>
    </RevealOnScroll>
  </SectionShell>
</template>
