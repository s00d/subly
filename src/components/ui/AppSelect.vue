<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted, nextTick } from "vue";
import { ChevronDown, Check, Search } from "lucide-vue-next";
import { tv } from "@/lib/tv";

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
  error?: string;
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

const selectTv = tv({
  slots: {
    root: "relative w-full",
    labelEl: "block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5",
    trigger: [
      "w-full flex items-center gap-2 rounded-lg border",
      "bg-[var(--color-surface)] text-left transition-shadow",
      "disabled:opacity-50 disabled:cursor-not-allowed",
    ],
    triggerText: "flex-1 truncate",
    chevron: "shrink-0 text-[var(--color-text-muted)] transition-transform",
    panel: [
      "fixed z-[200] bg-[var(--color-surface)] border border-[var(--color-border)]",
      "rounded-xl shadow-xl overflow-hidden",
    ],
    searchWrap: "p-2 border-b border-[var(--color-border)]",
    searchInput: [
      "w-full pl-8 pr-3 py-1.5 rounded-md border border-[var(--color-border)]",
      "bg-[var(--color-surface-secondary)] text-xs text-[var(--color-text-primary)]",
      "placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]",
    ],
    list: "max-h-56 overflow-y-auto py-1",
    emptyEl: "px-3 py-2 text-xs text-[var(--color-text-muted)] text-center",
    optionEl: "w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors disabled:opacity-40",
    errorEl: "mt-1 text-xs text-red-500",
  },
  variants: {
    size: {
      sm: { trigger: "px-2.5 py-1.5 text-xs" },
      md: { trigger: "px-3 py-2 text-sm" },
    },
    status: {
      error: { trigger: "border-red-500 ring-2 ring-red-500" },
      open: { trigger: "border-[var(--color-primary)] ring-2 ring-[var(--color-primary)]" },
      normal: { trigger: "border-[var(--color-border)] hover:border-[var(--color-text-muted)]" },
    },
    chevronOpen: {
      true: { chevron: "rotate-180" },
    },
    selected: {
      true: { triggerText: "text-[var(--color-text-primary)]" },
      false: { triggerText: "text-[var(--color-text-muted)]" },
    },
    optionActive: {
      true: { optionEl: "bg-[var(--color-primary-light)] text-[var(--color-primary)] font-medium" },
      false: { optionEl: "text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]" },
    },
  },
  defaultVariants: { size: "md", status: "normal" },
});

const slots = computed(() =>
  selectTv({
    size: props.size ?? "md",
    status: props.error ? "error" : isOpen.value ? "open" : "normal",
    chevronOpen: isOpen.value || undefined,
    selected: !!selected.value || undefined,
  }),
);
</script>

<template>
  <div :class="slots.root()">
    <label v-if="label" :class="slots.labelEl()">{{ label }}</label>

    <button
      ref="triggerRef"
      type="button"
      @click="toggle"
      :disabled="disabled"
      :class="slots.trigger()"
    >
      <span v-if="selected?.icon" class="shrink-0 text-base">
        <img v-if="selected.icon.startsWith('/') || selected.icon.startsWith('http') || selected.icon.startsWith('data:')" :src="selected.icon" class="w-5 h-5 object-contain inline-block" />
        <span v-else>{{ selected.icon }}</span>
      </span>
      <span :class="slots.triggerText()">
        {{ selected?.label || placeholder || '—' }}
      </span>
      <ChevronDown :size="14" :class="slots.chevron()" />
    </button>

    <Teleport to="body">
      <div
        v-if="isOpen"
        ref="dropdownRef"
        :class="slots.panel()"
        :style="{ top: pos.top + 'px', left: pos.left + 'px', width: pos.width + 'px' }"
      >
        <div v-if="searchable" :class="slots.searchWrap()">
          <div class="relative">
            <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
            <input
              v-model="search"
              type="text"
              :class="slots.searchInput()"
              placeholder="Search..."
              @click.stop
            />
          </div>
        </div>
        <div :class="slots.list()">
          <div v-if="filtered.length === 0" :class="slots.emptyEl()">No results</div>
          <button
            v-for="opt in filtered"
            :key="String(opt.value)"
            type="button"
            @click="select(opt)"
            :disabled="opt.disabled"
            :class="selectTv({ optionActive: String(opt.value) === String(modelValue) || undefined }).optionEl()"
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
    <p v-if="error" :class="slots.errorEl()">{{ error }}</p>
  </div>
</template>
