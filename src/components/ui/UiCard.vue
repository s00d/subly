<script setup lang="ts">
import { computed, type Component } from "vue";

/**
 * The one card surface: 12px radius, hairline border, 16px padding, optional header row
 * (icon · title · subtitle · actions). `flush` drops body padding for edge-to-edge lists.
 */
type Tone = "primary" | "success" | "warning" | "danger" | "info" | "violet" | "pink" | "neutral";

const props = withDefaults(
  defineProps<{
    title?: string;
    subtitle?: string;
    icon?: Component;
    /** Colour of the header icon chip — gives each widget its own recognisable accent. */
    tone?: Tone;
    /** Raw text-colour class for the icon; overrides `tone` (e.g. a dynamic up/down arrow). */
    iconTone?: string;
    flush?: boolean;
  }>(),
  { tone: "primary" },
);

const TONES: Record<Tone, string> = {
  primary: "bg-primary/14 text-primary",
  success: "bg-success/14 text-success",
  warning: "bg-warning/15 text-warning",
  danger: "bg-danger/14 text-danger",
  info: "bg-info/14 text-info",
  violet: "bg-violet-500/15 text-violet-500 dark:text-violet-400",
  pink: "bg-pink-500/14 text-pink-500 dark:text-pink-400",
  neutral: "bg-surface-hover text-text-secondary",
};
const chip = computed(() => (props.iconTone ? `bg-surface-hover ${props.iconTone}` : TONES[props.tone]));
</script>

<template>
  <section class="flex flex-col rounded-xl border border-border bg-surface shadow-sm min-w-0">
    <header
      v-if="title || $slots.actions"
      class="flex items-center gap-2 px-4 pt-3.5"
      :class="flush ? 'pb-2' : 'pb-1'"
    >
      <span v-if="icon" class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg" :class="chip">
        <component :is="icon" :size="15" :stroke-width="2.2" />
      </span>
      <div class="min-w-0 flex-1">
        <h2 class="text-[15px] font-semibold text-text-primary tracking-[-0.01em] truncate">{{ title }}</h2>
        <p v-if="subtitle" class="text-xs text-text-muted truncate">{{ subtitle }}</p>
      </div>
      <div v-if="$slots.actions" class="flex items-center gap-1 shrink-0">
        <slot name="actions" />
      </div>
    </header>
    <div class="flex-1 min-h-0" :class="flush ? '' : title || $slots.actions ? 'px-4 pb-4 pt-2' : 'p-4'">
      <slot />
    </div>
  </section>
</template>
