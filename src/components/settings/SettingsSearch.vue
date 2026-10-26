<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { Search } from "@lucide/vue";
import { ui } from "@/lib/tv";

/** Compact catalog search field (shared `ui.field()` look with a leading icon). */
const model = defineModel<string>({ required: true });
const { t } = useI18n();
</script>

<template>
  <div class="relative">
    <Search :size="14" class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
    <input
      v-model="model"
      type="search"
      :placeholder="t('search')"
      :aria-label="t('search')"
      :class="ui.field({ class: 'pl-8' })"
    />
  </div>
</template>
