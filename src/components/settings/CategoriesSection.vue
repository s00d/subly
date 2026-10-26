<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { upsertCategory, deleteCategory as deleteCategoryApi, maxSortOrder } from "@/services/catalogClient";
import type { Category, Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { useCatalogsUsageStore } from "@/stores/catalogsUsageStore";
import { formatErrorForToast } from "@/utils/formatError";
import AppInput from "@/components/ui/AppInput.vue";
import UiCard from "@/components/ui/UiCard.vue";
import SavedIndicator, { useAutoSave } from "@/components/settings/SavedIndicator.vue";
import SettingsSearch from "@/components/settings/SettingsSearch.vue";
import ReorderButtons from "@/components/settings/ReorderButtons.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import IconPickerModal from "@/components/ui/IconPickerModal.vue";
import { Trash2, Plus, ImageIcon } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { button } from "@/lib/tv";

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
const catalogsUsageStore = useCatalogsUsageStore();
const { savedAt, run, debounce } = useAutoSave();
const iconBtn = button({ intent: "ghost", size: "icon", class: "h-7 w-7" });
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
/** Rows the sort pins on top ("No category", default): they can't move and nothing moves above them. */
const pinnedCount = computed(() => {
  const defId = settings.value?.defaultCategoryId;
  return 1 + (defId && defId !== "cat-1" && categories.value.some((c) => c.id === defId) ? 1 : 0);
});
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
    if (cat) void run(() => updateCategory(cat.id, cat.name, icon));
  }
  showIconPicker.value = false;
  iconPickerCatId.value = null;
}

function addCat() {
  void run(async () => {
    const order = await maxSortOrder("categories");
    const cat: Category = { id: crypto.randomUUID(), name: "Category", icon: "", sortOrder: order + 1, i18nKey: "" };
    await upsertCategory(cat);
    metaStore.refreshSoon();
    categories.value.push(cat);
  });
}
async function updateCategory(id: string, name: string, icon?: string) {
  const cat = categories.value.find((c) => c.id === id);
  if (!cat) return;
  const next = { ...cat, name, icon: icon ?? cat.icon };
  await upsertCategory(next);
  metaStore.refreshSoon();
  Object.assign(cat, next);
}
/** Name edits: update the row now, persist once typing pauses. */
function saveCat(id: string, name: string) {
  const cat = categories.value.find((c) => c.id === id);
  if (!cat) return;
  cat.name = name;
  debounce(`name:${id}`, () => updateCategory(id, cat.name));
}
async function removeCat(id: string) {
  if (id === "cat-1") {
    toast(t("category_cannot_delete"), "error");
    return;
  }
  try {
    await deleteCategoryApi(id);
    metaStore.refreshSoon();
    categories.value = categories.value.filter((c) => c.id !== id);
    await catalogsUsageStore.refresh();
    toast(t("success"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

async function moveCatUp(id: string) {
  const ids = sortedCategories.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx <= 0) return;
  [ids[idx - 1], ids[idx]] = [ids[idx], ids[idx - 1]];
  await run(() => reorderCategories(ids));
}

async function moveCatDown(id: string) {
  const ids = sortedCategories.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx < 0 || idx >= ids.length - 1) return;
  [ids[idx], ids[idx + 1]] = [ids[idx + 1], ids[idx]];
  await run(() => reorderCategories(ids));
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
    metaStore.refreshSoon();
    Object.assign(cat, next);
  }
}
function setDefaultCategory(id: string) {
  if (!settings.value) return;
  const next = { ...settings.value, defaultCategoryId: id };
  settings.value = next;
  void run(() => metaStore.updateSettings(next));
}
</script>

<template>
  <UiCard :title="t('categories')">
    <template #actions>
      <SavedIndicator :at="savedAt" />
      <SettingsSearch v-model="catSearch" class="w-32 sm:w-44" />
    </template>
    <div class="space-y-1 max-h-72 overflow-y-auto overflow-x-hidden">
      <div
        v-for="(c, idx) in filteredCategories"
        :key="c.id"
        class="flex gap-1.5 items-center rounded-lg px-1.5 py-1"
        :class="c.id === settings?.defaultCategoryId ? 'bg-primary-light/50' : ''"
      >
        <!-- Move buttons (not for cat-1 "No category") -->
        <ReorderButtons
          v-if="!isCatSearching && idx >= pinnedCount"
          :up-disabled="idx <= pinnedCount"
          :down-disabled="idx === sortedCategories.length - 1"
          @up="moveCatUp(c.id)"
          @down="moveCatDown(c.id)"
        />
        <div v-else-if="!isCatSearching" class="w-14 shrink-0" />

        <!-- Default (radio) -->
        <Tooltip :text="t('set_as_primary')">
          <button
            type="button"
            role="radio"
            :aria-checked="c.id === settings?.defaultCategoryId"
            :aria-label="t('set_as_primary')"
            :class="iconBtn"
            @click="setDefaultCategory(c.id)"
          >
            <span class="flex h-3.5 w-3.5 items-center justify-center rounded-full border-2" :class="c.id === settings?.defaultCategoryId ? 'border-primary' : 'border-border-strong'">
              <span v-if="c.id === settings?.defaultCategoryId" class="h-1.5 w-1.5 rounded-full bg-primary" />
            </span>
          </button>
        </Tooltip>

        <!-- Icon button -->
        <Tooltip :text="t('choose_icon')">
          <button
            type="button"
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
          <span v-if="isDefaultItem(c)" class="text-xs text-text-primary truncate block px-2.5 py-1 border border-transparent">{{ c.name }}</span>
          <AppInput v-else :modelValue="c.name" @update:modelValue="(v: string | number) => saveCat(c.id, String(v))" size="sm" variant="inline" />
        </div>
        <span v-if="c.id === settings?.defaultCategoryId" class="shrink-0 rounded-full bg-primary/12 px-1.5 py-0.5 text-[10px] font-medium text-primary">{{ t('catalog_default') }}</span>

        <!-- Delete (not for cat-1) -->
        <Tooltip v-if="c.id !== 'cat-1'" :text="t('delete')">
          <button type="button" :aria-label="t('delete')" @click="removeCat(c.id)" :class="[iconBtn, 'text-danger hover:bg-danger/12 hover:text-danger']"><Trash2 :size="14" /></button>
        </Tooltip>
        <div v-else class="w-7 shrink-0" />
      </div>
    </div>
    <button type="button" @click="addCat" :class="button({ intent: 'secondary', size: 'sm', class: 'mt-3' })">
      <Plus :size="14" /> {{ t('add') }}
    </button>

    <!-- Icon Picker Modal -->
    <IconPickerModal
      :show="showIconPicker"
      :modelValue="iconPickerValue"
      group="all"
      @update:modelValue="onIconSelected"
      @close="showIconPicker = false"
    />
  </UiCard>
</template>
