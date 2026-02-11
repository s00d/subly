<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted, nextTick } from "vue";
import { ChevronDown, Check, Search } from "lucide-vue-next";

export interface SelectOption {
  value: string | number;
  label: string;
  icon?: string;
  disabled?: boolean;
}

const props = defineProps<{
  modelValue: string | number;
  options: SelectOption[];
  placeholder?: string;
  label?: string;
  disabled?: boolean;
  searchable?: boolean;
  size?: "sm" | "md";
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string | number];
}>();

const isOpen = ref(false);
const search = ref("");
const dropdownRef = ref<HTMLElement | null>(null);
const triggerRef = ref<HTMLElement | null>(null);
const pos = ref({ top: 0, left: 0, width: 200 });

const selected = computed(() =>
  props.options.find((o) => String(o.value) === String(props.modelValue))
);

const filtered = computed(() => {
  if (!search.value) return props.options;
  const q = search.value.toLowerCase();
  return props.options.filter((o) => o.label.toLowerCase().includes(q));
});

function updatePosition() {
  if (!triggerRef.value) return;
  const rect = triggerRef.value.getBoundingClientRect();
  pos.value = {
    top: rect.bottom + 4,
    left: rect.left,
    width: rect.width,
  };
}

function toggle() {
  if (props.disabled) return;
  isOpen.value = !isOpen.value;
  if (isOpen.value) {
    search.value = "";
    nextTick(updatePosition);
  }
}

function select(opt: SelectOption) {
  if (opt.disabled) return;
  emit("update:modelValue", opt.value);
  isOpen.value = false;
}

function onClickOutside(e: MouseEvent) {
  const target = e.target as Node;
  if (
    dropdownRef.value && !dropdownRef.value.contains(target) &&
    triggerRef.value && !triggerRef.value.contains(target)
  ) {
    isOpen.value = false;
  }
}

function onScroll() {
  if (!isOpen.value) return;
  updatePosition();
}

onMounted(() => {
  document.addEventListener("mousedown", onClickOutside);
  window.addEventListener("scroll", onScroll, true);
});
onUnmounted(() => {
  document.removeEventListener("mousedown", onClickOutside);
  window.removeEventListener("scroll", onScroll, true);
});
</script>

<template>
  <div class="relative w-full">
    <label v-if="label" class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{{ label }}</label>

    <button
      ref="triggerRef"
      type="button"
      @click="toggle"
      :disabled="disabled"
      class="w-full flex items-center gap-2 rounded-lg border bg-[var(--color-surface)] text-left transition-shadow disabled:opacity-50 disabled:cursor-not-allowed"
      :class="[
        isOpen ? 'border-[var(--color-primary)] ring-2 ring-[var(--color-primary)]' : 'border-[var(--color-border)] hover:border-[var(--color-text-muted)]',
        size === 'sm' ? 'px-2.5 py-1.5 text-xs' : 'px-3 py-2 text-sm',
      ]"
    >
      <span v-if="selected?.icon" class="shrink-0 text-base">
        <img v-if="selected.icon.startsWith('/') || selected.icon.startsWith('http') || selected.icon.startsWith('data:')" :src="selected.icon" class="w-5 h-5 object-contain inline-block" />
        <span v-else>{{ selected.icon }}</span>
      </span>
      <span class="flex-1 truncate" :class="selected ? 'text-[var(--color-text-primary)]' : 'text-[var(--color-text-muted)]'">
        {{ selected?.label || placeholder || '—' }}
      </span>
      <ChevronDown
        :size="14"
        class="shrink-0 text-[var(--color-text-muted)] transition-transform"
        :class="{ 'rotate-180': isOpen }"
      />
    </button>

    <Teleport to="body">
      <div
        v-if="isOpen"
        ref="dropdownRef"
        class="fixed z-[200] bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl shadow-xl overflow-hidden"
        :style="{ top: pos.top + 'px', left: pos.left + 'px', width: pos.width + 'px' }"
      >
        <div v-if="searchable" class="p-2 border-b border-[var(--color-border)]">
          <div class="relative">
            <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
            <input
              v-model="search"
              type="text"
              class="w-full pl-8 pr-3 py-1.5 rounded-md border border-[var(--color-border)] bg-[var(--color-surface-secondary)] text-xs text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]"
              placeholder="Search..."
              @click.stop
            />
          </div>
        </div>
        <div class="max-h-56 overflow-y-auto py-1">
          <div v-if="filtered.length === 0" class="px-3 py-2 text-xs text-[var(--color-text-muted)] text-center">No results</div>
          <button
            v-for="opt in filtered"
            :key="String(opt.value)"
            type="button"
            @click="select(opt)"
            :disabled="opt.disabled"
            class="w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors disabled:opacity-40"
            :class="[
              String(opt.value) === String(modelValue)
                ? 'bg-[var(--color-primary-light)] text-[var(--color-primary)] font-medium'
                : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]',
            ]"
          >
            <span v-if="opt.icon" class="shrink-0 text-base">
              <img v-if="opt.icon.startsWith('/') || opt.icon.startsWith('http') || opt.icon.startsWith('data:')" :src="opt.icon" class="w-5 h-5 object-contain inline-block" />
              <span v-else>{{ opt.icon }}</span>
            </span>
            <span class="flex-1 truncate text-left">{{ opt.label }}</span>
            <Check v-if="String(opt.value) === String(modelValue)" :size="14" class="shrink-0 text-[var(--color-primary)]" />
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>
