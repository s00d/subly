<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted, nextTick } from "vue";
import { ChevronDown, Check, Search } from "@lucide/vue";
import { tv, ui } from "@/lib/tv";

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
  const panelRect = dropdownRef.value?.getBoundingClientRect();
  const panelHeight = panelRect?.height || 280;
  let top = rect.bottom + 4;
  if (top + panelHeight > window.innerHeight - 8) {
    top = Math.max(8, rect.top - panelHeight - 4);
  }
  let left = rect.left;
  const width = rect.width;
  if (left + width > window.innerWidth - 8) {
    left = Math.max(8, window.innerWidth - width - 8);
  }
  pos.value = {
    top,
    left,
    width,
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

function onResize() {
  if (!isOpen.value) return;
  updatePosition();
}

onMounted(() => {
  document.addEventListener("mousedown", onClickOutside);
  window.addEventListener("scroll", onScroll, true);
  window.addEventListener("resize", onResize);
});
onUnmounted(() => {
  document.removeEventListener("mousedown", onClickOutside);
  window.removeEventListener("scroll", onScroll, true);
  window.removeEventListener("resize", onResize);
});

const selectTv = tv({
  slots: {
    root: "relative w-full",
    labelEl: ui.fieldLabel(),
    trigger: [
      ui.field(),
      "flex items-center gap-2 text-left",
    ],
    triggerText: "flex-1 truncate",
    chevron: "shrink-0 text-text-muted transition-transform",
    panel: ["fixed z-[200] bg-surface border border-border rounded-xl shadow-xl overflow-hidden"],
    searchWrap: "p-2 border-b border-border",
    searchInput: [
      "w-full pl-8 pr-3 py-1.5 rounded-md border border-border bg-surface text-xs text-text-primary",
      "placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary",
    ],
    list: "max-h-56 overflow-y-auto py-1",
    emptyEl: "px-3 py-2 text-xs text-text-muted text-center",
    optionEl: "w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors disabled:opacity-40",
    errorEl: ui.fieldError(),
  },
  variants: {
    size: {
      sm: { trigger: "px-2.5 py-1.5 text-xs" },
      md: { trigger: "px-3 py-2 text-sm" },
    },
    status: {
      error: { trigger: "border-red-500 ring-2 ring-red-500/20" },
      open: { trigger: "border-primary ring-2 ring-primary/20" },
      normal: {},
    },
    chevronOpen: {
      true: { chevron: "rotate-180" },
    },
    selected: {
      true: { triggerText: "text-text-primary" },
      false: { triggerText: "text-text-muted" },
    },
    optionActive: {
      true: { optionEl: "bg-primary-light text-primary font-medium" },
      false: { optionEl: "text-text-secondary hover:bg-surface dark:hover:bg-white/6" },
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
      <Transition
        enter-active-class="transition ease-out duration-150"
        enter-from-class="opacity-0 scale-95 origin-top"
        enter-to-class="opacity-100 scale-100 origin-top"
        leave-active-class="transition ease-in duration-100"
        leave-from-class="opacity-100 scale-100 origin-top"
        leave-to-class="opacity-0 scale-95 origin-top"
      >
        <div
          v-if="isOpen"
          ref="dropdownRef"
          :class="slots.panel()"
          :style="{ top: pos.top + 'px', left: pos.left + 'px', width: pos.width + 'px' }"
        >
        <div v-if="searchable" :class="slots.searchWrap()">
          <div class="relative">
            <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
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
            <Check v-if="String(opt.value) === String(modelValue)" :size="14" class="shrink-0 text-primary" />
          </button>
        </div>
        </div>
      </Transition>
    </Teleport>
    <p v-if="error" :class="slots.errorEl()">{{ error }}</p>
  </div>
</template>
