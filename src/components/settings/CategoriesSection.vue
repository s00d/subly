<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { upsertCategory, deleteCategory as deleteCategoryApi, maxSortOrder } from "@/services/catalogClient";
import type { Category, Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import AppInput from "@/components/ui/AppInput.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import IconPickerModal from "@/components/ui/IconPickerModal.vue";
import { Trash2, Plus, ChevronUp, ChevronDown, Star, Search, ImageIcon } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { ui } from "@/lib/tv";

const props = defineProps<{
  lookupData: {
    categories: Category[];
    settings: Settings;
    categoryUsage: Record<string, number>;
  } | null;
}>();
const { t } = useI18n();
const { toast } = useToast();
const metaStore = useAppMetaStore();
const categories = ref<Category[]>([]);
const settings = ref<Settings | null>(null);
const categoryUsage = ref<Record<string, number>>({});
watch(
  () => props.lookupData,
  (lookup) => {
    categories.value = lookup?.categories ?? [];
    settings.value = lookup?.settings ?? null;
    categoryUsage.value = lookup?.categoryUsage ?? {};
  },
  { immediate: true, deep: true },
);

const catSearch = ref("");
const isCatSearching = computed(() => catSearch.value.length > 0);

/** Sorted: default category first, then by order */
const sortedCategories = computed(() => {
  const defId = settings.value?.defaultCategoryId;
  return [...categories.value].sort((a, b) => {
    // "No category" always first
    if (a.id === "cat-1" && b.id !== "cat-1") return -1;
    if (b.id === "cat-1" && a.id !== "cat-1") return 1;
    // Default/primary second
    if (a.id === defId && b.id !== defId) return -1;
    if (b.id === defId && a.id !== defId) return 1;
    return a.sortOrder - b.sortOrder;
  });
});

const filteredCategories = computed(() => {
  if (!catSearch.value) return sortedCategories.value;
  const q = catSearch.value.toLowerCase();
  return sortedCategories.value.filter((c) => c.name.toLowerCase().includes(q));
});

const isUsedCategory = (id: string) => (categoryUsage.value[id] ?? 0) > 0;
const isDefaultItem = (c: { i18nKey?: string }) => !!c.i18nKey;

// Icon picker
const showIconPicker = ref(false);
const iconPickerCatId = ref<string | null>(null);
const iconPickerValue = ref("");

function openIconPicker(catId: string) {
  const cat = categories.value.find((c) => c.id === catId);
  iconPickerCatId.value = catId;
  iconPickerValue.value = cat?.icon || "";
  showIconPicker.value = true;
}

function onIconSelected(icon: string) {
  if (iconPickerCatId.value) {
    const cat = categories.value.find((c) => c.id === iconPickerCatId.value);
    if (cat) {
      updateCategory(cat.id, cat.name, icon);
      toast(t("success"));
    }
  }
  showIconPicker.value = false;
  iconPickerCatId.value = null;
}

async function addCat() {
  const order = await maxSortOrder("categories");
  const cat: Category = { id: crypto.randomUUID(), name: "Category", icon: "", sortOrder: order + 1, i18nKey: "" };
  await upsertCategory(cat);
  categories.value.push(cat);
}
async function updateCategory(id: string, name: string, icon?: string) {
  const cat = categories.value.find((c) => c.id === id);
  if (!cat) return;
  const next = { ...cat, name, icon: icon ?? cat.icon };
  await upsertCategory(next);
  Object.assign(cat, next);
}
async function saveCat(id: string, name: string) { await updateCategory(id, name); toast(t("success")); }
async function removeCat(id: string) {
  if (isUsedCategory(id) || id === "cat-1") {
    toast(t("category_cannot_delete"), "error");
    return;
  }
  await deleteCategoryApi(id);
  categories.value = categories.value.filter((c) => c.id !== id);
  toast(t("success"));
}

async function moveCatUp(id: string) {
  const ids = sortedCategories.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx <= 0) return;
  [ids[idx - 1], ids[idx]] = [ids[idx], ids[idx - 1]];
  await reorderCategories(ids);
}

async function moveCatDown(id: string) {
  const ids = sortedCategories.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx < 0 || idx >= ids.length - 1) return;
  [ids[idx], ids[idx + 1]] = [ids[idx + 1], ids[idx]];
  await reorderCategories(ids);
}

function getCatIcon(id: string): string {
  return categories.value.find((c) => c.id === id)?.icon || "";
}
async function reorderCategories(ids: string[]) {
  for (let i = 0; i < ids.length; i += 1) {
    const cat = categories.value.find((c) => c.id === ids[i]);
    if (!cat) continue;
    const next = { ...cat, sortOrder: i };
    await upsertCategory(next);
    Object.assign(cat, next);
  }
}
async function setDefaultCategory(id: string) {
  if (!settings.value) return;
  const next = { ...settings.value, defaultCategoryId: id };
  settings.value = next;
  await metaStore.updateSettings(next);
}
</script>

<template>
  <section class="bg-surface rounded-xl border border-border p-3 sm:p-5">
    <div class="flex items-center justify-between gap-2 mb-3">
      <h2 :class="[ui.sectionTitle(), 'shrink-0']">{{ t('categories') }}</h2>
      <div class="relative w-32 sm:w-44">
        <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
        <input v-model="catSearch" type="text" :placeholder="t('search')" class="w-full pl-8 pr-3 py-1.5 rounded-lg border border-border bg-surface text-xs text-text-primary placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-primary transition-shadow" />
      </div>
    </div>
    <div class="space-y-1.5 max-h-72 overflow-y-auto overflow-x-hidden">
      <div
        v-for="(c, idx) in filteredCategories"
        :key="c.id"
        class="flex gap-2 items-center rounded-lg px-2 py-1"
        :class="c.id === settings?.defaultCategoryId ? 'bg-primary-light/50' : ''"
      >
        <!-- Move buttons (not for cat-1 "No category") -->
        <div v-if="!isCatSearching && c.id !== 'cat-1'" class="flex flex-row sm:flex-col shrink-0">
          <Tooltip :text="t('move_up')"><button @click="moveCatUp(c.id)" :disabled="idx === 0" class="p-0.5 rounded text-text-muted hover:text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronUp :size="14" /></button></Tooltip>
          <Tooltip :text="t('move_down')"><button @click="moveCatDown(c.id)" :disabled="idx === sortedCategories.length - 1" class="p-0.5 rounded text-text-muted hover:text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronDown :size="14" /></button></Tooltip>
        </div>
        <div v-else-if="c.id === 'cat-1'" class="w-[22px] shrink-0" />

        <!-- Primary star -->
        <Tooltip :text="t('set_as_primary')">
          <button @click="setDefaultCategory(c.id)" class="p-1 rounded-lg transition-colors shrink-0" :class="c.id === settings?.defaultCategoryId ? 'text-yellow-500' : 'text-text-muted hover:text-yellow-500'">
            <Star :size="14" :fill="c.id === settings?.defaultCategoryId ? 'currentColor' : 'none'" />
          </button>
        </Tooltip>

        <!-- Icon button -->
        <Tooltip :text="t('choose_icon')">
          <button
            @click="openIconPicker(c.id)"
            class="w-8 h-8 rounded-lg border flex items-center justify-center shrink-0 transition-colors hover:border-primary"
            :class="getCatIcon(c.id) ? 'border-primary/30 bg-primary-light' : 'border-border'"
          >
          <IconDisplay v-if="getCatIcon(c.id)" :icon="getCatIcon(c.id)" :size="18" />
          <ImageIcon v-else :size="14" class="text-text-muted" />
        </button>
        </Tooltip>

        <!-- Name: read-only for default, editable for user-added -->
        <div class="flex-1 min-w-0">
          <span v-if="isDefaultItem(c)" class="text-sm text-text-primary truncate block px-2 py-1">{{ c.name }}</span>
          <AppInput v-else :modelValue="c.name" @update:modelValue="(v: string | number) => saveCat(c.id, String(v))" size="sm" />
        </div>

        <!-- Delete (not for cat-1) -->
        <Tooltip v-if="c.id !== 'cat-1'" :text="t('delete')">
          <button @click="removeCat(c.id)" :disabled="isUsedCategory(c.id)" class="p-1.5 rounded-lg transition-colors shrink-0" :class="isUsedCategory(c.id) ? 'text-text-muted cursor-not-allowed' : 'text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20'"><Trash2 :size="14" /></button>
        </Tooltip>
        <div v-else class="w-[34px] shrink-0" />
      </div>
    </div>
    <button @click="addCat" class="mt-3 px-3 py-1.5 rounded-lg bg-primary text-white text-sm hover:bg-primary-hover transition-colors">
      <Plus :size="14" class="inline mr-1" /> {{ t('add') }}
    </button>

    <!-- Icon Picker Modal -->
    <IconPickerModal
      :show="showIconPicker"
      :modelValue="iconPickerValue"
      group="all"
      @update:modelValue="onIconSelected"
      @close="showIconPicker = false"
    />
  </section>
</template>
