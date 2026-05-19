<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import FeatureCard from "@/components/ui/FeatureCard.vue";
import { whyCards } from "@/content/site.en";
</script>

<template>
  <SectionShell id="why" title="Why people choose Subly" subtitle="Clarity for recurring bills and daily spending — without trading away privacy.">
    <div class="grid gap-5 sm:grid-cols-2">
      <FeatureCard
        v-for="(card, i) in whyCards"
        :key="card.title"
        :title="card.title"
        :body="card.body"
        :stagger-index="i"
      />
    </div>
  </SectionShell>
</template>
