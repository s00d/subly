<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { watch } from "vue";
import { useClipboard } from "@/composables/useClipboard";
import { CheckCircle, XCircle, Info, AlertTriangle, X, Copy } from "@lucide/vue";
import { tv } from "@/lib/tv";

const props = defineProps<{
  message: string;
  type?: "success" | "error" | "info" | "warning";
  show: boolean;
  duration?: number;
}>();

const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();

const icons = {
  success: CheckCircle,
  error: XCircle,
  info: Info,
  warning: AlertTriangle,
};

let timer: ReturnType<typeof setTimeout> | null = null;
const { copyToClipboard } = useClipboard();

watch(
  () => [props.show, props.message] as const,
  ([show]) => {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    if (show) {
      timer = setTimeout(() => emit("close"), props.duration || 4000);
    }
  },
);

async function copyMessage() {
  await copyToClipboard(props.message);
}

const toastTv = tv({
  slots: {
    root: [
      "fixed z-[250] flex items-center gap-2.5 pl-3.5 pr-2 py-2.5 rounded-xl border border-border-strong",
      "bg-surface text-text-primary shadow-lg",
      "bottom-20 left-3 right-3 sm:bottom-6 sm:left-auto sm:right-6 sm:max-w-sm",
    ],
    icon: "shrink-0",
  },
  variants: {
    type: {
      success: { icon: "text-success" },
      error: { icon: "text-danger" },
      info: { icon: "text-primary" },
      warning: { icon: "text-warning" },
    },
  },
  defaultVariants: { type: "info" },
});

const actionBtnClass =
  "p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-surface-hover transition-colors shrink-0";
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
        :class="toastTv({ type: type || 'info' }).root()"
      >
        <component :is="icons[type || 'info']" :size="17" :class="toastTv({ type: type || 'info' }).icon()" />
        <span class="text-[13px] font-medium flex-1 min-w-0">{{ message }}</span>
        <button @click="copyMessage" :class="actionBtnClass" :title="t('copy')" :aria-label="t('copy')">
          <Copy :size="13" />
        </button>
        <button @click="emit('close')" :class="actionBtnClass" :title="t('close')" :aria-label="t('close')">
          <X :size="14" />
        </button>
      </div>
    </Transition>
  </Teleport>
</template>
