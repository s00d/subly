<script setup lang="ts">
import { Cloud } from "@lucide/vue";
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import { syncSection } from "@/content/site.en";
import { providerIconUrl } from "@/lib/assets";
</script>

<template>
  <SectionShell
    id="sync"
    :eyebrow="syncSection.eyebrow"
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
            v-if="p.icon"
            :src="providerIconUrl(p.icon)"
            :alt="p.label"
            width="22"
            height="22"
            class="opacity-90"
          />
          <Cloud v-else :size="20" class="text-accent" aria-hidden="true" />
          {{ p.label }}
        </li>
      </ul>
      <ul class="mt-8 space-y-3 text-muted">
        <li
          v-for="(item, i) in syncSection.bullets"
          :key="i"
          class="flex gap-3 text-sm md:text-base"
        >
          <span class="mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-accent" aria-hidden="true" />
          {{ item }}
        </li>
      </ul>
    </RevealOnScroll>
  </SectionShell>
</template>
