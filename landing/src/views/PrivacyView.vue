<script setup lang="ts">
import MarkdownPage from "@/components/layout/MarkdownPage.vue";
import privacyRaw from "../../../PRIVACY.md?raw";
</script>

<template>
  <MarkdownPage :markdown="privacyRaw" />
</template>
