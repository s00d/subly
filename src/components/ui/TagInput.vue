<script setup lang="ts">
import { ref, computed } from "vue";
import { useCatalogStore } from "@/stores/catalog";
import { useI18n } from "vue-i18n";
import { X, Plus } from "lucide-vue-next";
import { tv } from "@/lib/tv";

const props = withDefaults(defineProps<{
  modelValue?: string[];
  label?: string;
}>(), {
  modelValue: () => [],
});

const emit = defineEmits<{
  "update:modelValue": [value: string[]];
}>();

const catalogStore = useCatalogStore();
const { t } = useI18n();

const inputValue = ref("");
const showSuggestions = ref(false);
const safeModelValue = computed(() => (Array.isArray(props.modelValue) ? props.modelValue : []));

const quickTags = computed(() =>
  catalogStore.favoriteTags
    .filter((tag) => !safeModelValue.value.includes(tag.name))
    .map((tag) => tag.name),
);

const suggestions = computed(() => {
  const allTags = catalogStore.tags;
  if (!inputValue.value.trim()) {
    return allTags
      .filter((tag) => !safeModelValue.value.includes(tag.name))
      .map((tag) => tag.name);
  }
  const q = inputValue.value.toLowerCase();
  return allTags
    .filter((tag) => tag.name.toLowerCase().includes(q) && !safeModelValue.value.includes(tag.name))
    .map((tag) => tag.name);
});

function addTag(tagName: string) {
  const n = tagName.trim();
  if (!n || safeModelValue.value.includes(n)) return;
  emit("update:modelValue", [...safeModelValue.value, n]);
  if (!catalogStore.tags.some((t) => t.name === n)) catalogStore.addTag(n);
  inputValue.value = "";
  showSuggestions.value = false;
}

function removeTag(tagName: string) {
  emit("update:modelValue", safeModelValue.value.filter((t) => t !== tagName));
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    e.preventDefault();
    if (inputValue.value.trim()) addTag(inputValue.value);
  }
}

function onBlur() {
  setTimeout(() => { showSuggestions.value = false; }, 150);
}

const tagInputTv = tv({
  slots: {
    root: "",
    labelEl: "block text-xs font-medium text-text-secondary mb-1",
    tagList: "flex flex-wrap gap-1.5 mb-2",
    tag: [
      "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium",
      "bg-primary-light text-primary",
    ],
    tagRemoveBtn: "p-0.5 rounded-full hover:bg-primary hover:text-white transition-colors",
    inputWrap: "relative",
    inputRow: "flex items-center gap-1.5",
    inputEl: [
      "flex-1 px-3 py-1.5 rounded-lg border border-border",
      "bg-surface text-xs text-text-primary",
      "placeholder-text-muted",
      "focus:outline-none focus:ring-2 focus:ring-primary transition-shadow",
    ],
    addBtn: [
      "p-1.5 rounded-lg bg-primary text-white",
      "disabled:opacity-30 hover:bg-primary-hover transition-colors shrink-0",
    ],
    dropdown: [
      "absolute left-0 right-0 top-full mt-1",
      "bg-surface border border-border",
      "rounded-lg shadow-lg z-10 max-h-28 overflow-auto",
    ],
    suggestionBtn: [
      "block w-full text-left px-3 py-1.5 text-xs",
      "text-text-primary hover:bg-surface-hover transition-colors",
    ],
    quickTagsWrap: "flex flex-wrap gap-1.5 mt-2",
    quickTag: [
      "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium",
      "border border-dashed border-border text-text-secondary",
      "hover:border-primary hover:text-primary",
      "hover:bg-primary-light transition-colors",
    ],
  },
});

const slots = tagInputTv();
</script>

<template>
  <div :class="slots.root()">
    <label v-if="label" :class="slots.labelEl()">{{ label }}</label>

    <div :class="slots.tagList()" v-if="safeModelValue.length > 0">
      <span v-for="tag in safeModelValue" :key="tag" :class="slots.tag()">
        {{ tag }}
        <button type="button" @click="removeTag(tag)" :class="slots.tagRemoveBtn()">
          <X :size="10" />
        </button>
      </span>
    </div>

    <div :class="slots.inputWrap()">
      <div :class="slots.inputRow()">
        <input
          v-model="inputValue"
          type="text"
          :placeholder="t('tag_placeholder')"
          :class="slots.inputEl()"
          @keydown="onKeyDown"
          @focus="showSuggestions = true"
          @blur="onBlur"
        />
        <button
          type="button"
          @click="addTag(inputValue)"
          :disabled="!inputValue.trim()"
          :class="slots.addBtn()"
        >
          <Plus :size="14" />
        </button>
      </div>

      <div
        v-if="showSuggestions && suggestions.length > 0"
        :class="slots.dropdown()"
      >
        <button
          v-for="tagName in suggestions"
          :key="tagName"
          type="button"
          @mousedown.prevent="addTag(tagName)"
          @touchend.prevent="addTag(tagName)"
          :class="slots.suggestionBtn()"
        >
          {{ tagName }}
        </button>
      </div>
    </div>

    <div v-if="quickTags.length > 0" :class="slots.quickTagsWrap()">
      <button
        v-for="tagName in quickTags"
        :key="tagName"
        type="button"
        @click="addTag(tagName)"
        :class="slots.quickTag()"
      >
        <Plus :size="10" />
        {{ tagName }}
      </button>
    </div>
  </div>
</template>
