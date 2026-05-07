<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { upsertHouseholdMember, deleteHouseholdMember as deleteHouseholdMemberApi, maxSortOrder } from "@/services/catalogClient";
import type { HouseholdMember } from "@/schemas/appData";
import AppInput from "@/components/ui/AppInput.vue";
import { Trash2, Plus } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { ui } from "@/lib/tv";

const props = defineProps<{ lookupHousehold: HouseholdMember[] | null }>();
const { t } = useI18n();
const { toast } = useToast();
const household = ref<HouseholdMember[]>([]);
watch(
  () => props.lookupHousehold,
  (value) => {
    household.value = (value ?? []).map((member, index) => ({
      ...member,
      email: member.email ?? "",
      sortOrder: member.sortOrder ?? index + 1,
    }));
  },
  { immediate: true, deep: true },
);

const sortedHousehold = computed(() => household.value);

async function addMember() {
  const order = await maxSortOrder("householdMembers");
  const member: HouseholdMember = { id: crypto.randomUUID(), name: "Member", email: "", sortOrder: order + 1 };
  await upsertHouseholdMember(member);
  household.value.push(member);
}
async function saveMember(id: string, name: string, email: string) {
  const member = household.value.find((m) => m.id === id);
  if (!member) return;
  const next = { ...member, name, email };
  await upsertHouseholdMember(next);
  Object.assign(member, next);
  toast(t("success"));
}
async function removeMember(id: string) {
  if (household.value.length <= 1) {
    toast(t("household_keep_one_member"), "error");
    return;
  }
  await deleteHouseholdMemberApi(id);
  household.value = household.value.filter((m) => m.id !== id);
  toast(t("success"));
}
</script>

<template>
  <section class="bg-surface rounded-xl border border-border p-3 sm:p-5">
    <h2 :class="[ui.sectionTitle(), 'mb-3 sm:mb-4']">{{ t('household') }}</h2>
    <div class="space-y-2">
      <div v-for="m in sortedHousehold" :key="m.id" class="flex gap-2 items-center">
        <div class="flex-1 min-w-0">
          <AppInput :modelValue="m.name" @update:modelValue="(v: string | number) => saveMember(m.id, String(v), m.email ?? '')" />
        </div>
        <div class="flex-1 min-w-0">
          <AppInput :modelValue="m.email ?? ''" @update:modelValue="(v: string | number) => saveMember(m.id, m.name, String(v))" :placeholder="t('email')" />
        </div>
        <Tooltip v-if="sortedHousehold.length > 1" :text="t('delete')">
          <button @click="removeMember(m.id)" class="p-2 rounded-lg text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors shrink-0"><Trash2 :size="16" /></button>
        </Tooltip>
      </div>
    </div>
    <button @click="addMember" class="mt-3 px-3 py-1.5 rounded-lg bg-primary text-white text-sm hover:bg-primary-hover transition-colors">
      <Plus :size="14" class="inline mr-1" /> {{ t('add') }}
    </button>
  </section>
</template>
