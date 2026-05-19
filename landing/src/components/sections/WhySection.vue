<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import FeatureCard from "@/components/ui/FeatureCard.vue";
import { useSiteContent } from "@/composables/useSiteContent";

const { content } = useSiteContent();
</script>

<template>
  <SectionShell
    id="why"
    :title="content.whySection.title"
    :subtitle="content.whySection.subtitle"
  >
    <div class="grid gap-5 sm:grid-cols-2">
      <FeatureCard
        v-for="(card, i) in content.whyCards"
        :key="card.title"
        :title="card.title"
        :body="card.body"
        :icon="card.icon"
        :stagger-index="i"
      />
    </div>
  </SectionShell>
</template>
