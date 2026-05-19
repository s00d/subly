<script setup lang="ts">
import { computed } from "vue";
import { marked } from "marked";
const props = defineProps<{
  markdown: string;
}>();

const html = computed(() =>
  marked.parse(props.markdown, { gfm: true, breaks: false }),
);
</script>

<template>
  <article class="min-h-[60vh]">
    <div class="md-content" v-html="html" />
  </article>
</template>
