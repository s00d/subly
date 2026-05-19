<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import FaqItem from "@/components/ui/FaqItem.vue";
import { useSiteContent } from "@/composables/useSiteContent";

const { content } = useSiteContent();
</script>

<template>
  <SectionShell id="faq" :title="content.faqSection.title" muted>
    <div class="mx-auto max-w-3xl divide-y divide-border">
      <FaqItem
        v-for="(item, i) in content.faqItems"
        :key="i"
        :question="item.q"
        :answer="item.a"
        :stagger-index="i"
      />
    </div>
  </SectionShell>
</template>
