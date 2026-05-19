<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import FaqItem from "@/components/ui/FaqItem.vue";
import { faqItems } from "@/content/site.en";
</script>

<template>
  <SectionShell id="faq" title="Frequently asked questions">
    <div class="mx-auto max-w-3xl space-y-3">
      <FaqItem
        v-for="(item, i) in faqItems"
        :key="item.q"
        :question="item.q"
        :answer="item.a"
        :stagger-index="i"
      />
    </div>
  </SectionShell>
</template>
