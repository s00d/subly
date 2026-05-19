<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { syncSection } from "@/content/site.en";
import { providerIconUrl } from "@/lib/assets";
</script>

<template>
  <SectionShell
    id="sync"
    :eyebrow="syncSection.eyebrow"
    :eyebrow-icon="syncSection.icon"
    :title="syncSection.title"
    :subtitle="syncSection.body"
  >
    <RevealOnScroll tag="div">
      <ul class="flex flex-wrap gap-3">
        <li
          v-for="p in syncSection.providers"
          :key="p.id"
          class="flex items-center gap-2 rounded-xl border border-border bg-surface px-4 py-3 text-sm font-medium text-text"
        >
          <img
            v-if="p.brandIcon"
            :src="providerIconUrl(p.brandIcon)"
            :alt="p.label"
            width="22"
            height="22"
            class="opacity-90"
          />
          <UiIcon v-else :name="p.icon" size="md" />
          {{ p.label }}
        </li>
      </ul>
      <ul class="mt-8 space-y-3 text-muted">
        <li
          v-for="(item, i) in syncSection.bullets"
          :key="i"
          class="flex gap-3 text-sm md:text-base"
        >
          <UiIcon name="check-circle-outline" size="sm" class="mt-0.5 shrink-0 text-accent" />
          {{ item }}
        </li>
      </ul>
    </RevealOnScroll>
  </SectionShell>
</template>
