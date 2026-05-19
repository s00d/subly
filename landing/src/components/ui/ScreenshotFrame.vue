<script setup lang="ts">
defineProps<{
  src: string;
  alt: string;
  float?: boolean;
}>();
</script>

<template>
  <figure
    class="overflow-hidden rounded-xl border border-border bg-surface shadow-xl shadow-black/30"
    :class="float ? 'hero-mockup-float' : ''"
  >
    <img
      :src="src"
      :alt="alt"
      class="block h-auto w-full transition-transform duration-500 hover:scale-[1.02]"
      loading="lazy"
      decoding="async"
    />
  </figure>
</template>
