<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch, type Component } from "vue";
import { tv } from "@/lib/tv";

export interface DropdownItem {
  id: string;
  label: string;
  icon?: Component;
  danger?: boolean;
  separator?: boolean;
  hidden?: boolean;
}

const props = defineProps<{
  items: DropdownItem[];
  open: boolean;
  anchorEl?: HTMLElement | null;
}>();

const emit = defineEmits<{
  select: [id: string];
  close: [];
}>();

const menuRef = ref<HTMLElement | null>(null);
const pos = ref({ top: 0, left: 0 });

function updatePosition() {
  if (!props.anchorEl) return;
  const rect = props.anchorEl.getBoundingClientRect();
  const menuWidth = 180;
  const menuHeight = 300;

  let top = rect.bottom + 4;
  let left = rect.right - menuWidth;

  if (left < 8) left = 8;
  if (top + menuHeight > window.innerHeight) {
    top = rect.top - menuHeight - 4;
    if (top < 8) top = 8;
  }

  pos.value = { top, left };
}

function onClickOutside(e: MouseEvent) {
  if (!props.open) return;
  const target = e.target as Node;
  if (menuRef.value && !menuRef.value.contains(target)) {
    emit("close");
  }
}

function onSelect(item: DropdownItem) {
  if (item.separator) return;
  emit("select", item.id);
  emit("close");
}

watch(
  () => props.open,
  async (val) => {
    if (val) {
      await nextTick();
      updatePosition();
    }
  }
);

onMounted(() => {
  document.addEventListener("mousedown", onClickOutside);
  window.addEventListener("scroll", updatePosition, true);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", onClickOutside);
  window.removeEventListener("scroll", updatePosition, true);
});

const dropdownTv = tv({
  slots: {
    menu: [
      "fixed z-[200] bg-[var(--color-surface)] border border-[var(--color-border)]",
      "rounded-xl shadow-xl py-1 min-w-[170px] origin-top-right",
    ],
    itemBtn: "w-full flex items-center gap-2.5 px-3 py-2 text-sm transition-colors",
    separator: "my-1 border-[var(--color-border)]",
  },
  variants: {
    danger: {
      true: { itemBtn: "text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20" },
      false: { itemBtn: "text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]" },
    },
  },
  defaultVariants: { danger: false },
});

const slots = dropdownTv();
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div
        v-if="open"
        ref="menuRef"
        :class="slots.menu()"
        :style="{ top: pos.top + 'px', left: pos.left + 'px' }"
      >
        <template v-for="item in items" :key="item.id">
          <hr v-if="item.separator" :class="slots.separator()" />
          <button
            v-else-if="!item.hidden"
            @click="onSelect(item)"
            :class="dropdownTv({ danger: item.danger || undefined }).itemBtn()"
          >
            <component v-if="item.icon" :is="item.icon" :size="14" class="shrink-0" />
            <span>{{ item.label }}</span>
          </button>
        </template>
      </div>
    </Transition>
  </Teleport>
</template>
