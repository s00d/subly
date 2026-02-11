<script setup lang="ts">
import { watch } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { CheckCircle, XCircle, Info, AlertTriangle, X, Copy } from "lucide-vue-next";

const props = defineProps<{
  message: string;
  type?: "success" | "error" | "info" | "warning";
  show: boolean;
  duration?: number;
}>();

const emit = defineEmits<{ close: [] }>();

const icons = {
  success: CheckCircle,
  error: XCircle,
  info: Info,
  warning: AlertTriangle,
};

const colors = {
  success: "bg-green-50 border-green-200 text-green-800 dark:bg-green-900/30 dark:border-green-800 dark:text-green-300",
  error: "bg-red-50 border-red-200 text-red-800 dark:bg-red-900/30 dark:border-red-800 dark:text-red-300",
  info: "bg-blue-50 border-blue-200 text-blue-800 dark:bg-blue-900/30 dark:border-blue-800 dark:text-blue-300",
  warning: "bg-yellow-50 border-yellow-200 text-yellow-800 dark:bg-yellow-900/30 dark:border-yellow-800 dark:text-yellow-300",
};

let timer: ReturnType<typeof setTimeout> | null = null;

watch(() => props.show, (val) => {
  if (timer) { clearTimeout(timer); timer = null; }
  if (val) {
    timer = setTimeout(() => emit("close"), props.duration || 4000);
  }
});

async function copyMessage() {
  try {
    await writeText(props.message);
  } catch {
    try { await navigator.clipboard.writeText(props.message); } catch { /* ignore */ }
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition ease-out duration-300 transform"
      enter-from-class="translate-y-2 opacity-0"
      enter-to-class="translate-y-0 opacity-100"
      leave-active-class="transition ease-in duration-200 transform"
      leave-from-class="translate-y-0 opacity-100"
      leave-to-class="translate-y-2 opacity-0"
    >
      <div
        v-if="show"
        class="fixed bottom-6 right-6 z-[100] flex items-center gap-2.5 px-4 py-3 rounded-lg border shadow-lg max-w-md"
        :class="colors[type || 'info']"
      >
        <component :is="icons[type || 'info']" :size="18" class="shrink-0" />
        <span class="text-sm font-medium flex-1 min-w-0">{{ message }}</span>
        <button
          @click="copyMessage"
          class="p-1 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors shrink-0 opacity-60 hover:opacity-100"
          title="Copy"
        >
          <Copy :size="13" />
        </button>
        <button
          @click="emit('close')"
          class="p-1 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors shrink-0 opacity-60 hover:opacity-100"
          title="Close"
        >
          <X :size="14" />
        </button>
      </div>
    </Transition>
  </Teleport>
</template>
