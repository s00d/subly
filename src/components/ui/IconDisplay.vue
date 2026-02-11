<script setup lang="ts">
import { isImageIcon } from "@/services/logoAssets";

defineProps<{
  icon: string;
  size?: number;
}>();
</script>

<template>
  <img
    v-if="isImageIcon(icon)"
    :src="icon"
    :width="size || 20"
    :height="size || 20"
    class="object-contain inline-block shrink-0"
  />
  <span v-else class="inline-block shrink-0">{{ icon }}</span>
</template>
