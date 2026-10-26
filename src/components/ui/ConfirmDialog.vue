<script setup lang="ts">
import { AlertTriangle } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import Modal from "@/components/ui/Modal.vue";
import { button } from "@/lib/tv";

/** Destructive-action confirmation on top of the shared Modal (one look for every "delete?" prompt). */
withDefaults(
  defineProps<{ show: boolean; title: string; message: string; confirmLabel?: string; danger?: boolean }>(),
  { danger: true },
);
const emit = defineEmits<{ confirm: []; cancel: [] }>();
const { t } = useI18n();
</script>

<template>
  <Modal :show="show" :title="title" max-width="24rem" @close="emit('cancel')">
    <div class="flex gap-3">
      <div v-if="danger" class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-danger/12 text-danger">
        <AlertTriangle :size="17" />
      </div>
      <p class="pt-1.5 text-sm text-text-secondary">{{ message }}</p>
    </div>
    <template #footer>
      <button :class="button()" @click="emit('cancel')">{{ t("cancel") }}</button>
      <button :class="button({ intent: danger ? 'danger' : 'primary' })" @click="emit('confirm')">
        {{ confirmLabel ?? t("delete") }}
      </button>
    </template>
  </Modal>
</template>
