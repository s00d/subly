<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { useSiteContent } from "@/composables/useSiteContent";
import { providerIconUrl } from "@/lib/assets";
import type { MdiIconName } from "@/mdi-icons";

const { content } = useSiteContent();
</script>

<template>
  <SectionShell
    id="sync"
    :eyebrow="content.syncSection.eyebrow"
    :eyebrow-icon="content.syncSection.icon"
    :title="content.syncSection.title"
    :subtitle="content.syncSection.body"
  >
    <RevealOnScroll tag="div">
      <ul class="flex flex-wrap gap-3">
        <li
          v-for="p in content.syncSection.providers"
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
          <UiIcon v-else :name="p.icon as MdiIconName" size="md" />
          {{ p.label }}
        </li>
      </ul>
      <ul class="mt-8 space-y-3 text-muted">
        <li
          v-for="(item, i) in content.syncSection.bullets"
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
