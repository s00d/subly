<script lang="ts">
import { onBeforeUnmount, ref, watch } from "vue";
import { Check } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { formatErrorForToast } from "@/utils/formatError";

/**
 * Save-on-change for settings cards: `run` saves now, `debounce` coalesces keystrokes per key.
 * Success only bumps `savedAt` (feeds <SavedIndicator>); errors still toast. A save fn may
 * return `false` to say "failed, error already shown". Pending saves flush on unmount.
 */
export function useAutoSave(delay = 400) {
  const { t } = useI18n();
  const { toast } = useToast();
  const savedAt = ref(0);
  const pending = new Map<string, { timer: ReturnType<typeof setTimeout>; fn: () => Promise<unknown> }>();

  async function run(fn: () => Promise<unknown>): Promise<boolean> {
    try {
      const ok = (await fn()) !== false;
      if (ok) savedAt.value = Date.now();
      return ok;
    } catch (e) {
      console.error("Settings save failed:", e);
      toast(formatErrorForToast(e, t), "error");
      return false;
    }
  }

  function debounce(key: string, fn: () => Promise<unknown>) {
    const prev = pending.get(key);
    if (prev) clearTimeout(prev.timer);
    const timer = setTimeout(() => {
      pending.delete(key);
      void run(fn);
    }, delay);
    pending.set(key, { timer, fn });
  }

  onBeforeUnmount(() => {
    for (const p of pending.values()) {
      clearTimeout(p.timer);
      void run(p.fn);
    }
    pending.clear();
  });

  return { savedAt, run, debounce };
}
</script>

<script setup lang="ts">
/** Small "Saved ✓" that appears whenever `at` changes and fades out. */
const props = defineProps<{ at: number }>();
const { t: tr } = useI18n();
const visible = ref(false);
let hideTimer: ReturnType<typeof setTimeout> | undefined;

watch(
  () => props.at,
  (v) => {
    if (!v) return;
    visible.value = true;
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => (visible.value = false), 1800);
  },
);
onBeforeUnmount(() => clearTimeout(hideTimer));
</script>

<template>
  <Transition
    enter-active-class="transition-opacity duration-150"
    enter-from-class="opacity-0"
    leave-active-class="transition-opacity duration-500"
    leave-to-class="opacity-0"
  >
    <span v-if="visible" role="status" class="inline-flex items-center gap-1 text-xs text-text-muted">
      <Check :size="12" class="text-success" />{{ tr("settings_autosaved") }}
    </span>
  </Transition>
</template>
