<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useSiteContent } from "@/composables/useSiteContent";
import UiIcon from "@/components/ui/UiIcon.vue";
import type { LocaleId } from "@/content/index";

const { locale, content, switchLocale } = useSiteContent();
const open = ref(false);
const root = ref<HTMLElement | null>(null);

const currentLabel = () =>
  content.value.localeSwitcher.options.find((o) => o.id === locale.value)?.label ?? locale.value;

function pick(target: LocaleId) {
  switchLocale(target);
  open.value = false;
}

function onDocClick(e: MouseEvent) {
  if (!open.value || !root.value?.contains(e.target as Node)) {
    open.value = false;
  }
}

onMounted(() => document.addEventListener("click", onDocClick));
onUnmounted(() => document.removeEventListener("click", onDocClick));
</script>

<template>
  <div ref="root" class="relative">
    <button
      type="button"
      class="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 py-2 text-sm text-muted transition-colors hover:border-accent/50 hover:text-text"
      :aria-expanded="open"
      :aria-label="content.localeSwitcher.ariaLabel"
      @click.stop="open = !open"
    >
      <UiIcon name="translate" size="sm" />
      <span class="hidden sm:inline">{{ currentLabel() }}</span>
      <UiIcon
        name="chevron-down"
        size="sm"
        class="transition-transform"
        :class="open ? 'rotate-180' : ''"
      />
    </button>
    <ul
      v-if="open"
      class="absolute right-0 top-full z-50 mt-1 min-w-[9rem] overflow-hidden rounded-lg border border-border bg-surface py-1 shadow-lg shadow-black/40"
      role="listbox"
    >
      <li v-for="opt in content.localeSwitcher.options" :key="opt.id" role="option">
        <button
          type="button"
          class="flex w-full items-center px-3 py-2 text-left text-sm transition-colors hover:bg-bg"
          :class="locale === opt.id ? 'font-semibold text-accent' : 'text-muted'"
          @click="pick(opt.id)"
        >
          {{ opt.label }}
        </button>
      </li>
    </ul>
  </div>
</template>
