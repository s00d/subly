<script setup lang="ts">
import { ref, computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { X, Plus } from "lucide-vue-next";

const props = defineProps<{
  modelValue: string[];
  label?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string[]];
}>();

const store = useAppStore();
const { t } = useI18n();

const inputValue = ref("");
const showSuggestions = ref(false);

/** All tag names not yet selected */
const suggestions = computed(() => {
  const allTags = store.state.tags;
  if (!inputValue.value.trim()) {
    return allTags
      .filter((tag) => !props.modelValue.includes(tag.name))
      .map((tag) => tag.name);
  }
  const q = inputValue.value.toLowerCase();
  return allTags
    .filter((tag) => tag.name.toLowerCase().includes(q) && !props.modelValue.includes(tag.name))
    .map((tag) => tag.name);
});

function addTag(tagName: string) {
  const n = tagName.trim();
  if (!n || props.modelValue.includes(n)) return;
  emit("update:modelValue", [...props.modelValue, n]);
  // Also add to global tags if new
  if (!store.state.tags.some((t) => t.name === n)) store.addTag(n);
  inputValue.value = "";
  showSuggestions.value = false;
}

function removeTag(tagName: string) {
  emit("update:modelValue", props.modelValue.filter((t) => t !== tagName));
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    e.preventDefault();
    if (inputValue.value.trim()) addTag(inputValue.value);
  }
}

function onBlur() {
  // Delay to allow click on suggestion
  setTimeout(() => { showSuggestions.value = false; }, 150);
}
</script>

<template>
  <div>
    <label v-if="label" class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">{{ label }}</label>

    <!-- Current tags -->
    <div class="flex flex-wrap gap-1.5 mb-2" v-if="modelValue.length > 0">
      <span
        v-for="tag in modelValue"
        :key="tag"
        class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-[var(--color-primary-light)] text-[var(--color-primary)]"
      >
        {{ tag }}
        <button type="button" @click="removeTag(tag)" class="p-0.5 rounded-full hover:bg-[var(--color-primary)] hover:text-white transition-colors">
          <X :size="10" />
        </button>
      </span>
    </div>

    <!-- Input -->
    <div class="relative">
      <div class="flex items-center gap-1.5">
        <input
          v-model="inputValue"
          type="text"
          :placeholder="t('tag_placeholder')"
          class="flex-1 px-3 py-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] text-xs text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] transition-shadow"
          @keydown="onKeyDown"
          @focus="showSuggestions = true"
          @blur="onBlur"
        />
        <button
          type="button"
          @click="addTag(inputValue)"
          :disabled="!inputValue.trim()"
          class="p-1.5 rounded-lg bg-[var(--color-primary)] text-white disabled:opacity-30 hover:bg-[var(--color-primary-hover)] transition-colors shrink-0"
        >
          <Plus :size="14" />
        </button>
      </div>

      <!-- Suggestions -->
      <div
        v-if="showSuggestions && suggestions.length > 0"
        class="absolute left-0 right-0 top-full mt-1 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg shadow-lg z-10 max-h-28 overflow-auto"
      >
        <button
          v-for="tagName in suggestions"
          :key="tagName"
          type="button"
          @mousedown.prevent="addTag(tagName)"
          class="block w-full text-left px-3 py-1.5 text-xs text-[var(--color-text-primary)] hover:bg-[var(--color-surface-hover)] transition-colors"
        >
          {{ tagName }}
        </button>
      </div>
    </div>
  </div>
</template>
