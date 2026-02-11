<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "@/i18n";
import { useHeaderActions } from "@/composables/useHeaderActions";

const route = useRoute();
const { t } = useI18n();
const { actions } = useHeaderActions();

const pageTitle = computed(() => {
  const names: Record<string, string> = {
    dashboard: t("dashboard"),
    subscriptions: t("subscriptions"),
    calendar: t("calendar"),
    settings: t("settings"),
  };
  return names[route.name as string] || "Subly";
});
</script>

<template>
  <header class="h-14 sm:h-16 bg-[var(--color-surface)] border-b border-[var(--color-border)] flex items-center px-3 sm:px-6 shrink-0 gap-2">
    <!-- Mobile logo (shown only on mobile where sidebar is hidden) -->
    <div class="flex items-center gap-2 md:hidden shrink-0">
      <div class="w-7 h-7 rounded-lg bg-[var(--color-primary)] flex items-center justify-center">
        <span class="text-white font-bold text-xs">W</span>
      </div>
    </div>
    <h1 class="text-base sm:text-xl font-semibold text-[var(--color-text-primary)] truncate">{{ pageTitle }}</h1>
    <div class="flex items-center gap-2 ml-auto sm:ml-3">
      <button
        v-for="action in actions"
        :key="action.id"
        @click="action.onClick"
        class="w-8 h-8 rounded-lg bg-[var(--color-primary)] text-white flex items-center justify-center hover:bg-[var(--color-primary-hover)] shadow-sm transition-colors shrink-0"
        :title="action.title"
      >
        <component :is="action.icon" :size="18" />
      </button>
    </div>
  </header>
</template>
