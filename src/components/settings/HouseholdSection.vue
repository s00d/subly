<script setup lang="ts">
import { computed } from "vue";
import { useCatalogStore } from "@/stores/catalog";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import { Trash2, Plus } from "lucide-vue-next";
import Tooltip from "@/components/ui/Tooltip.vue";

const catalogStore = useCatalogStore();
const { t } = useI18n();
const { toast } = useToast();

const sortedHousehold = computed(() => catalogStore.household);

function addMember() { catalogStore.addHouseholdMember("Member"); }
function saveMember(id: string, name: string, email: string) {
  catalogStore.updateHouseholdMember(id, name, email);
  toast(t("success"));
}
function removeMember(id: string) {
  if (!catalogStore.deleteHouseholdMember(id)) toast(t("error"), "error");
  else toast(t("success"));
}
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
    <h2 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)] mb-3 sm:mb-4">{{ t('household') }}</h2>
    <div class="space-y-2">
      <div v-for="m in sortedHousehold" :key="m.id" class="flex gap-2 items-center">
        <div class="flex-1 min-w-0">
          <AppInput :modelValue="m.name" @update:modelValue="(v: string | number) => saveMember(m.id, String(v), m.email)" />
        </div>
        <div class="flex-1 min-w-0">
          <AppInput :modelValue="m.email" @update:modelValue="(v: string | number) => saveMember(m.id, m.name, String(v))" :placeholder="t('email')" />
        </div>
        <Tooltip v-if="sortedHousehold.length > 1" :text="t('delete')">
          <button @click="removeMember(m.id)" class="p-2 rounded-lg text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors shrink-0"><Trash2 :size="16" /></button>
        </Tooltip>
      </div>
    </div>
    <button @click="addMember" class="mt-3 px-3 py-1.5 rounded-lg bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)] transition-colors">
      <Plus :size="14" class="inline mr-1" /> {{ t('add') }}
    </button>
  </section>
</template>
