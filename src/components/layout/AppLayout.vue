<script setup lang="ts">
import { ref, provide } from "vue";
import AppSidebar from "./AppSidebar.vue";
import AppHeader from "./AppHeader.vue";
import MobileTabBar from "./MobileTabBar.vue";
import { tv } from "@/lib/tv";

const sidebarOpen = ref(false);
provide("sidebarOpen", sidebarOpen);

const layoutTv = tv({
  slots: {
    root: "flex h-screen overflow-hidden",
    sidebar: "hidden md:flex",
    content: "flex-1 flex flex-col overflow-hidden min-w-0",
    main: "flex-1 overflow-y-auto p-3 sm:p-4 md:p-6 bg-[var(--color-surface-secondary)] pb-20 md:pb-6",
    tabBar: "md:hidden",
  },
});

const slots = layoutTv();
</script>

<template>
  <div :class="slots.root()">
    <AppSidebar :class="slots.sidebar()" />

    <!-- Mobile sidebar overlay -->
    <Teleport to="body">
      <Transition name="sidebar-fade">
        <div v-if="sidebarOpen" class="fixed inset-0 z-50 md:hidden" @click.self="sidebarOpen = false">
          <div class="absolute inset-0 bg-black/40" @click="sidebarOpen = false" />
          <Transition name="sidebar-slide">
            <div v-if="sidebarOpen" class="absolute inset-y-0 left-0 w-64">
              <AppSidebar class="flex w-full" :mobile="true" @navigate="sidebarOpen = false" />
            </div>
          </Transition>
        </div>
      </Transition>
    </Teleport>

    <div :class="slots.content()">
      <AppHeader @toggle-sidebar="sidebarOpen = !sidebarOpen" />
      <main :class="slots.main()">
        <slot />
      </main>
    </div>
    <MobileTabBar :class="slots.tabBar()" />
  </div>
</template>

<style scoped>
.sidebar-fade-enter-active,
.sidebar-fade-leave-active {
  transition: opacity 0.2s ease;
}
.sidebar-fade-enter-from,
.sidebar-fade-leave-to {
  opacity: 0;
}

.sidebar-slide-enter-active {
  transition: transform 0.25s ease-out;
}
.sidebar-slide-leave-active {
  transition: transform 0.2s ease-in;
}
.sidebar-slide-enter-from,
.sidebar-slide-leave-to {
  transform: translateX(-100%);
}
</style>
