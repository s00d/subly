<script setup lang="ts">
import { ref, computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import IconPickerModal from "@/components/ui/IconPickerModal.vue";
import { Trash2, Plus, ChevronUp, ChevronDown, Star, Search, ImageIcon } from "lucide-vue-next";

const store = useAppStore();
const { t } = useI18n();
const { toast } = useToast();

const catSearch = ref("");
const isCatSearching = computed(() => catSearch.value.length > 0);

/** Sorted: default category first, then by order */
const sortedCategories = computed(() => {
  const defId = store.state.settings.defaultCategoryId;
  return [...store.state.categories].sort((a, b) => {
    // "No category" always first
    if (a.id === "cat-1" && b.id !== "cat-1") return -1;
    if (b.id === "cat-1" && a.id !== "cat-1") return 1;
    // Default/primary second
    if (a.id === defId && b.id !== defId) return -1;
    if (b.id === defId && a.id !== defId) return 1;
    return a.order - b.order;
  });
});

const filteredCategories = computed(() => {
  if (!catSearch.value) return sortedCategories.value;
  const q = catSearch.value.toLowerCase();
  return sortedCategories.value.filter((c) => c.name.toLowerCase().includes(q));
});

const isUsedCategory = (id: string) => store.state.subscriptions.some((s) => s.categoryId === id);
const isDefaultItem = (c: { i18nKey?: string }) => !!c.i18nKey;

// Icon picker
const showIconPicker = ref(false);
const iconPickerCatId = ref<string | null>(null);
const iconPickerValue = ref("");

function openIconPicker(catId: string) {
  const cat = store.state.categories.find((c) => c.id === catId);
  iconPickerCatId.value = catId;
  iconPickerValue.value = cat?.icon || "";
  showIconPicker.value = true;
}

function onIconSelected(icon: string) {
  if (iconPickerCatId.value) {
    const cat = store.state.categories.find((c) => c.id === iconPickerCatId.value);
    if (cat) {
      store.updateCategory(cat.id, cat.name, icon);
      toast(t("success"));
    }
  }
  showIconPicker.value = false;
  iconPickerCatId.value = null;
}

function addCat() { store.addCategory("Category"); }
function saveCat(id: string, name: string) { store.updateCategory(id, name); toast(t("success")); }
function removeCat(id: string) {
  if (!store.deleteCategory(id)) toast(t("error"), "error");
  else toast(t("success"));
}

function moveCatUp(id: string) {
  const ids = sortedCategories.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx <= 0) return;
  [ids[idx - 1], ids[idx]] = [ids[idx], ids[idx - 1]];
  store.reorderCategories(ids);
}

function moveCatDown(id: string) {
  const ids = sortedCategories.value.map((c) => c.id);
  const idx = ids.indexOf(id);
  if (idx < 0 || idx >= ids.length - 1) return;
  [ids[idx], ids[idx + 1]] = [ids[idx + 1], ids[idx]];
  store.reorderCategories(ids);
}

function getCatIcon(id: string): string {
  return store.state.categories.find((c) => c.id === id)?.icon || "";
}
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
    <div class="flex items-center justify-between mb-3">
      <h2 class="text-lg font-semibold text-[var(--color-text-primary)]">{{ t('categories') }}</h2>
      <div class="relative w-44">
        <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
        <input v-model="catSearch" type="text" :placeholder="t('search')" class="w-full pl-8 pr-3 py-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] text-xs text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] transition-shadow" />
      </div>
    </div>
    <div class="space-y-1.5 max-h-72 overflow-y-auto overflow-x-hidden">
      <div
        v-for="(c, idx) in filteredCategories"
        :key="c.id"
        class="flex gap-2 items-center rounded-lg px-2 py-1"
        :class="c.id === store.state.settings.defaultCategoryId ? 'bg-[var(--color-primary-light)]/50' : ''"
      >
        <!-- Move buttons (not for cat-1 "No category") -->
        <div v-if="!isCatSearching && c.id !== 'cat-1'" class="flex flex-col shrink-0">
          <button @click="moveCatUp(c.id)" :disabled="idx === 0" class="p-0.5 rounded text-[var(--color-text-muted)] hover:text-[var(--color-primary)] disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronUp :size="14" /></button>
          <button @click="moveCatDown(c.id)" :disabled="idx === sortedCategories.length - 1" class="p-0.5 rounded text-[var(--color-text-muted)] hover:text-[var(--color-primary)] disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronDown :size="14" /></button>
        </div>
        <div v-else-if="c.id === 'cat-1'" class="w-[22px] shrink-0" />

        <!-- Primary star -->
        <button @click="store.updateSettings({ defaultCategoryId: c.id })" class="p-1 rounded-lg transition-colors shrink-0" :class="c.id === store.state.settings.defaultCategoryId ? 'text-yellow-500' : 'text-[var(--color-text-muted)] hover:text-yellow-500'" :title="t('set_as_primary')">
          <Star :size="14" :fill="c.id === store.state.settings.defaultCategoryId ? 'currentColor' : 'none'" />
        </button>

        <!-- Icon button -->
        <button
          @click="openIconPicker(c.id)"
          class="w-8 h-8 rounded-lg border flex items-center justify-center shrink-0 transition-colors hover:border-[var(--color-primary)]"
          :class="getCatIcon(c.id) ? 'border-[var(--color-primary)]/30 bg-[var(--color-primary-light)]' : 'border-[var(--color-border)]'"
          :title="t('choose_icon')"
        >
          <IconDisplay v-if="getCatIcon(c.id)" :icon="getCatIcon(c.id)" :size="18" />
          <ImageIcon v-else :size="14" class="text-[var(--color-text-muted)]" />
        </button>

        <!-- Name: read-only for default, editable for user-added -->
        <div class="flex-1 min-w-0">
          <span v-if="isDefaultItem(c)" class="text-sm text-[var(--color-text-primary)] truncate block px-2 py-1">{{ c.name }}</span>
          <AppInput v-else :modelValue="c.name" @update:modelValue="(v: any) => saveCat(c.id, String(v))" size="sm" />
        </div>

        <!-- Delete (not for cat-1) -->
        <button v-if="c.id !== 'cat-1'" @click="removeCat(c.id)" :disabled="isUsedCategory(c.id)" class="p-1.5 rounded-lg transition-colors shrink-0" :class="isUsedCategory(c.id) ? 'text-[var(--color-text-muted)] cursor-not-allowed' : 'text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20'"><Trash2 :size="14" /></button>
        <div v-else class="w-[34px] shrink-0" />
      </div>
    </div>
    <button @click="addCat" class="mt-3 px-3 py-1.5 rounded-lg bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)] transition-colors">
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
