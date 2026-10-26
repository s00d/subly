<script setup lang="ts">
import type { Component } from "vue";

defineProps<{ icon?: Component; title: string; description?: string }>();
</script>

<template>
  <div class="flex flex-col items-center text-center px-6 py-12">
    <div v-if="icon" class="mb-3 flex h-11 w-11 items-center justify-center rounded-xl bg-surface-hover text-text-muted">
      <component :is="icon" :size="20" />
    </div>
    <p class="text-sm font-medium text-text-primary">{{ title }}</p>
    <p v-if="description" class="mt-1 max-w-sm text-xs text-text-muted">{{ description }}</p>
    <div v-if="$slots.default" class="mt-4">
      <slot />
    </div>
  </div>
</template>
