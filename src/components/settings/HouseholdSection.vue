<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import { Trash2, Plus } from "lucide-vue-next";

const store = useAppStore();
const { t } = useI18n();
const { toast } = useToast();

const sortedHousehold = computed(() =>
  [...store.state.household].sort((a, b) => a.order - b.order)
);

function addMember() { store.addHouseholdMember("Member"); }
function saveMember(id: string, name: string, email: string) {
  store.updateHouseholdMember(id, name, email);
  toast(t("success"));
}
function removeMember(id: string) {
  if (!store.deleteHouseholdMember(id)) toast(t("error"), "error");
  else toast(t("success"));
}
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
    <h2 class="text-lg font-semibold text-[var(--color-text-primary)] mb-4">{{ t('household') }}</h2>
    <div class="space-y-2">
      <div v-for="m in sortedHousehold" :key="m.id" class="flex gap-2 items-center">
        <div class="flex-1 min-w-0">
          <AppInput :modelValue="m.name" @update:modelValue="(v: any) => saveMember(m.id, String(v), m.email)" />
        </div>
        <div class="flex-1 min-w-0">
          <AppInput :modelValue="m.email" @update:modelValue="(v: any) => saveMember(m.id, m.name, String(v))" :placeholder="t('email')" />
        </div>
        <button v-if="sortedHousehold.length > 1" @click="removeMember(m.id)" class="p-2 rounded-lg text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors shrink-0"><Trash2 :size="16" /></button>
      </div>
    </div>
    <button @click="addMember" class="mt-3 px-3 py-1.5 rounded-lg bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)] transition-colors">
      <Plus :size="14" class="inline mr-1" /> {{ t('add') }}
    </button>
  </section>
</template>
