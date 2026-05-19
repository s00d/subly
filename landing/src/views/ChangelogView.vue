<script setup lang="ts">
import MarkdownPage from "@/components/layout/MarkdownPage.vue";
import changelogRaw from "../../../CHANGELOG.md?raw";
</script>

<template>
  <MarkdownPage :markdown="changelogRaw" />
</template>
