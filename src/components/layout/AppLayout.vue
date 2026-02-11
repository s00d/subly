<script setup lang="ts">
import AppSidebar from "./AppSidebar.vue";
import AppHeader from "./AppHeader.vue";
import MobileTabBar from "./MobileTabBar.vue";
</script>

<template>
  <div class="flex h-screen overflow-hidden">
    <!-- Desktop sidebar -->
    <AppSidebar class="hidden md:flex" />
    <div class="flex-1 flex flex-col overflow-hidden min-w-0">
      <AppHeader />
      <main class="flex-1 overflow-y-auto p-3 sm:p-4 md:p-6 bg-[var(--color-surface-secondary)] pb-20 md:pb-6">
        <slot />
      </main>
    </div>
    <!-- Mobile bottom tab bar -->
    <MobileTabBar class="md:hidden" />
  </div>
</template>
