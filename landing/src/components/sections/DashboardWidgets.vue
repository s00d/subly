<script setup lang="ts">
import { useSiteContent } from "@/composables/useSiteContent";

const { content } = useSiteContent();
</script>

<template>
  <ul class="mt-6 flex flex-wrap gap-2">
    <li
      v-for="(label, i) in content.dashboardWidgets"
      :key="i"
      class="rounded-lg border border-border bg-bg px-3 py-1.5 text-xs text-muted"
    >
      {{ label }}
    </li>
  </ul>
</template>
