<script setup lang="ts">
import { useAppMetaStore } from "@/stores/appMetaStore";
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { upsertTag, deleteTag as deleteTagApi, maxSortOrder } from "@/services/catalogClient";
import type { Tag } from "@/schemas/appData";
import AppInput from "@/components/ui/AppInput.vue";
import UiCard from "@/components/ui/UiCard.vue";
import SavedIndicator, { useAutoSave } from "@/components/settings/SavedIndicator.vue";
import SettingsSearch from "@/components/settings/SettingsSearch.vue";
import ReorderButtons from "@/components/settings/ReorderButtons.vue";
import Toast from "@/components/ui/Toast.vue";
import { Trash2, Plus, Star, Hash } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { useCatalogsUsageStore } from "@/stores/catalogsUsageStore";
import { formatErrorForToast } from "@/utils/formatError";
import { button } from "@/lib/tv";

const props = defineProps<{
  lookupData: {
    tags: Tag[];
    tagUsage: Record<string, number>;
  } | null;
}>();
const metaStore = useAppMetaStore();
const { t } = useI18n();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();
const catalogsUsageStore = useCatalogsUsageStore();
const { savedAt, run, debounce } = useAutoSave();
const iconBtn = button({ intent: "ghost", size: "icon", class: "h-7 w-7" });
const tags = ref<Tag[]>([]);
const tagUsage = ref<Record<string, number>>({});
watch(
  () => props.lookupData,
  (lookup) => {
    tags.value = lookup?.tags ?? [];
    tagUsage.value = lookup?.tagUsage ?? {};
  },
  { immediate: true, deep: true },
);

const tagSearch = ref("");
const isSearching = computed(() => tagSearch.value.length > 0);
const sortedTags = computed(() =>
  [...tags.value].sort((a, b) => {
    if (a.favorite && !b.favorite) return -1;
    if (!a.favorite && b.favorite) return 1;
    return a.sortOrder - b.sortOrder;
  })
);
const filteredTags = computed(() => {
  if (!tagSearch.value) return sortedTags.value;
  const q = tagSearch.value.toLowerCase();
  return sortedTags.value.filter((tag) => tag.name.toLowerCase().includes(q));
});

const isUsedTag = (name: string) => (tagUsage.value[name] ?? 0) > 0;
const isDefaultItem = (tag: { i18nKey?: string }) => !!tag.i18nKey;

async function updateTag(id: string, updates: Partial<Tag>) {
  const tag = tags.value.find((t) => t.id === id);
  if (!tag) return;
  const next = { ...tag, ...updates };
  await upsertTag(next);
  metaStore.refreshSoon();
  Object.assign(tag, next);
}
async function reorderTags(ids: string[]) {
  for (let i = 0; i < ids.length; i += 1) {
    const tag = tags.value.find((t) => t.id === ids[i]);
    if (!tag) continue;
    const next = { ...tag, sortOrder: i };
    await upsertTag(next);
    metaStore.refreshSoon();
    Object.assign(tag, next);
  }
}
function addTag() {
  void run(async () => {
    const order = await maxSortOrder("tags");
    const tag: Tag = { id: crypto.randomUUID(), name: "Tag", favorite: false, sortOrder: order + 1, i18nKey: "" };
    await upsertTag(tag);
    metaStore.refreshSoon();
    tags.value.push(tag);
  });
}
/** Name edits: update the row now, persist once typing pauses. */
function saveTagName(id: string, name: string) {
  const tag = tags.value.find((t) => t.id === id);
  if (!tag) return;
  tag.name = name;
  debounce(`name:${id}`, () => updateTag(id, { name: tag.name }));
}
async function removeTag(id: string) {
  try {
    await deleteTagApi(id);
    metaStore.refreshSoon();
    tags.value = tags.value.filter((t) => t.id !== id);
    await catalogsUsageStore.refresh();
    toast(t("success"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}
function toggleFavorite(id: string) {
  const tag = tags.value.find((t) => t.id === id);
  if (!tag) return;
  void run(() => updateTag(id, { favorite: !tag.favorite }));
}
function moveUp(id: string) {
  const ids = sortedTags.value.map((t) => t.id);
  const idx = ids.indexOf(id);
  if (idx <= 0) return;
  [ids[idx - 1], ids[idx]] = [ids[idx], ids[idx - 1]];
  void run(() => reorderTags(ids));
}
function moveDown(id: string) {
  const ids = sortedTags.value.map((t) => t.id);
  const idx = ids.indexOf(id);
  if (idx < 0 || idx >= ids.length - 1) return;
  [ids[idx], ids[idx + 1]] = [ids[idx + 1], ids[idx]];
  void run(() => reorderTags(ids));
}
</script>

<template>
  <UiCard :title="t('manage_tags')" :icon="Hash" icon-tone="text-primary">
    <template #actions>
      <SavedIndicator :at="savedAt" />
      <SettingsSearch v-model="tagSearch" class="w-32 sm:w-44" />
    </template>

    <!-- Tags list -->
    <div class="space-y-1 max-h-72 overflow-y-auto overflow-x-hidden">
      <div
        v-for="(tag, idx) in filteredTags"
        :key="tag.id"
        class="flex items-center gap-1.5 rounded-lg px-1.5 py-1 group"
        :class="[tag.favorite ? 'bg-primary-light/50' : '']"
      >
        <!-- Move buttons -->
        <ReorderButtons
          v-if="!isSearching"
          :up-disabled="idx === 0"
          :down-disabled="idx === sortedTags.length - 1"
          @up="moveUp(tag.id)"
          @down="moveDown(tag.id)"
        />

        <!-- Favorite star -->
        <Tooltip :text="t('favorites')">
          <button
            type="button"
            :aria-pressed="tag.favorite"
            :aria-label="t('favorites')"
            @click="toggleFavorite(tag.id)"
            :class="[iconBtn, tag.favorite ? 'text-warning hover:text-warning' : 'text-text-muted hover:text-warning']"
          >
            <Star :size="14" :fill="tag.favorite ? 'currentColor' : 'none'" />
          </button>
        </Tooltip>

        <!-- Hash icon -->
        <Hash :size="13" class="text-text-muted shrink-0" />

        <!-- Name: read-only for default, editable inline for user-added -->
        <div class="flex-1 min-w-0">
          <span v-if="isDefaultItem(tag)" class="text-xs text-text-primary truncate block px-2.5 py-1 border border-transparent">{{ tag.name }}</span>
          <AppInput v-else :modelValue="tag.name" @update:modelValue="(v: string | number) => saveTagName(tag.id, String(v))" size="sm" variant="inline" />
        </div>

        <!-- Usage count -->
        <span class="text-[10px] text-text-muted shrink-0 tabular-nums">
          {{ tagUsage[tag.name] ?? 0 }}
        </span>

        <!-- Delete -->
        <Tooltip :text="t('delete')">
          <button
            type="button"
            :aria-label="t('delete')"
            @click="removeTag(tag.id)"
            :disabled="isUsedTag(tag.name)"
            :class="[iconBtn, 'text-danger hover:bg-danger/12 hover:text-danger disabled:text-text-muted']"
          >
            <Trash2 :size="14" />
          </button>
        </Tooltip>
      </div>
    </div>
    <p v-if="filteredTags.length === 0" class="text-sm text-text-muted text-center py-3">{{ t('no_tags') }}</p>

    <button type="button" @click="addTag" :class="button({ intent: 'secondary', size: 'sm', class: 'mt-3' })">
      <Plus :size="14" /> {{ t('add') }}
    </button>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </UiCard>
</template>
