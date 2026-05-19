<script setup lang="ts">
import SiteFooter from "@/components/layout/SiteFooter.vue";
import SiteHeader from "@/components/layout/SiteHeader.vue";
import { RouterView } from "vue-router";
</script>

<template>
  <div class="motion-safe min-h-screen bg-bg">
    <SiteHeader />
    <main>
      <RouterView />
    </main>
    <SiteFooter />
  </div>
</template>
