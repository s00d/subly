<script setup lang="ts">
import { ref, computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import Toast from "@/components/ui/Toast.vue";
import { Trash2, Plus, ChevronUp, ChevronDown, Star, Search, Hash } from "lucide-vue-next";

const store = useAppStore();
const { t } = useI18n();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();

const tagSearch = ref("");
const isSearching = computed(() => tagSearch.value.length > 0);

/** Sorted: favorites first, then by order */
const sortedTags = computed(() =>
  [...store.state.tags].sort((a, b) => {
    if (a.favorite && !b.favorite) return -1;
    if (!a.favorite && b.favorite) return 1;
    return a.order - b.order;
  })
);

const filteredTags = computed(() => {
  if (!tagSearch.value) return sortedTags.value;
  const q = tagSearch.value.toLowerCase();
  return sortedTags.value.filter((tag) => tag.name.toLowerCase().includes(q));
});

const isUsedTag = (name: string) =>
  store.state.subscriptions.some((s) => s.tags?.includes(name)) ||
  store.state.expenses.some((e) => e.tags?.includes(name));

const isDefaultItem = (tag: { i18nKey?: string }) => !!tag.i18nKey;

function addTag() {
  store.addTag("Tag");
}

function saveTagName(id: string, name: string) {
  store.updateTag(id, { name });
}

function removeTag(id: string) {
  store.deleteTag(id);
  toast(t("success"));
}

function toggleFavorite(id: string) {
  store.toggleTagFavorite(id);
}

// Reorder
function moveUp(id: string) {
  const ids = sortedTags.value.map((t) => t.id);
  const idx = ids.indexOf(id);
  if (idx <= 0) return;
  [ids[idx - 1], ids[idx]] = [ids[idx], ids[idx - 1]];
  store.reorderTags(ids);
}

function moveDown(id: string) {
  const ids = sortedTags.value.map((t) => t.id);
  const idx = ids.indexOf(id);
  if (idx < 0 || idx >= ids.length - 1) return;
  [ids[idx], ids[idx + 1]] = [ids[idx + 1], ids[idx]];
  store.reorderTags(ids);
}
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <Hash :size="18" class="text-[var(--color-primary)]" />
        <h2 class="text-lg font-semibold text-[var(--color-text-primary)]">{{ t('manage_tags') }}</h2>
      </div>
      <div class="relative w-44">
        <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
        <input v-model="tagSearch" type="text" :placeholder="t('search')" class="w-full pl-8 pr-3 py-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] text-xs text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] transition-shadow" />
      </div>
    </div>

    <!-- Tags list -->
    <div class="space-y-1.5 max-h-72 overflow-y-auto overflow-x-hidden">
      <div
        v-for="(tag, idx) in filteredTags"
        :key="tag.id"
        class="flex items-center gap-2 rounded-lg px-2 py-1 group"
        :class="[tag.favorite ? 'bg-[var(--color-primary-light)]/50' : '']"
      >
        <!-- Move buttons -->
        <div v-if="!isSearching" class="flex flex-col shrink-0">
          <button @click="moveUp(tag.id)" :disabled="idx === 0" class="p-0.5 rounded text-[var(--color-text-muted)] hover:text-[var(--color-primary)] disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronUp :size="14" /></button>
          <button @click="moveDown(tag.id)" :disabled="idx === sortedTags.length - 1" class="p-0.5 rounded text-[var(--color-text-muted)] hover:text-[var(--color-primary)] disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronDown :size="14" /></button>
        </div>

        <!-- Favorite star -->
        <button @click="toggleFavorite(tag.id)" class="p-1 rounded-lg transition-colors shrink-0" :class="tag.favorite ? 'text-yellow-500' : 'text-[var(--color-text-muted)] hover:text-yellow-500'" :title="t('favorites')">
          <Star :size="14" :fill="tag.favorite ? 'currentColor' : 'none'" />
        </button>

        <!-- Hash icon -->
        <Hash :size="13" class="text-[var(--color-text-muted)] shrink-0" />

        <!-- Name: read-only for default, editable inline for user-added -->
        <div class="flex-1 min-w-0">
          <span v-if="isDefaultItem(tag)" class="text-sm text-[var(--color-text-primary)] truncate block px-2 py-1">{{ tag.name }}</span>
          <AppInput v-else :modelValue="tag.name" @update:modelValue="(v: any) => saveTagName(tag.id, String(v))" size="sm" />
        </div>

        <!-- Usage count -->
        <span class="text-[10px] text-[var(--color-text-muted)] shrink-0">
          {{ store.state.subscriptions.filter((s) => s.tags?.includes(tag.name)).length + store.state.expenses.filter((e) => e.tags?.includes(tag.name)).length }}
        </span>

        <!-- Delete -->
        <button @click="removeTag(tag.id)" :disabled="isUsedTag(tag.name)" class="p-1.5 rounded-lg transition-colors shrink-0" :class="isUsedTag(tag.name) ? 'text-[var(--color-text-muted)] cursor-not-allowed' : 'text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20'">
          <Trash2 :size="14" />
        </button>
      </div>
    </div>
    <p v-if="filteredTags.length === 0" class="text-sm text-[var(--color-text-muted)] text-center py-3">{{ t('no_tags') }}</p>

    <!-- Add button (same style as Categories) -->
    <button @click="addTag" class="mt-3 px-3 py-1.5 rounded-lg bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)] transition-colors">
      <Plus :size="14" class="inline mr-1" /> {{ t('add') }}
    </button>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </section>
</template>
