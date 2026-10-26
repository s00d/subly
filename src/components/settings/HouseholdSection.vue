<script setup lang="ts">
import { useAppMetaStore } from "@/stores/appMetaStore";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { upsertHouseholdMember, deleteHouseholdMember as deleteHouseholdMemberApi, maxSortOrder } from "@/services/catalogClient";
import type { HouseholdMember } from "@/schemas/appData";
import { formatErrorForToast } from "@/utils/formatError";
import AppInput from "@/components/ui/AppInput.vue";
import UiCard from "@/components/ui/UiCard.vue";
import SavedIndicator, { useAutoSave } from "@/components/settings/SavedIndicator.vue";
import { Trash2, Plus } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { button } from "@/lib/tv";

const props = defineProps<{ lookupHousehold: HouseholdMember[] | null }>();
const metaStore = useAppMetaStore();
const { t } = useI18n();
const { toast } = useToast();
const { savedAt, run, debounce } = useAutoSave();
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

function addMember() {
  void run(async () => {
    const order = await maxSortOrder("householdMembers");
    const member: HouseholdMember = { id: crypto.randomUUID(), name: "Member", email: "", sortOrder: order + 1 };
    await upsertHouseholdMember(member);
    metaStore.refreshSoon();
    household.value.push(member);
  });
}
/** Field edits: update the row now, persist once typing pauses. */
function saveMember(id: string, field: "name" | "email", value: string) {
  const member = household.value.find((m) => m.id === id);
  if (!member) return;
  member[field] = value;
  debounce(`member:${id}`, () => upsertHouseholdMember({ ...member }).then(() => metaStore.refreshSoon()));
}
async function removeMember(id: string) {
  if (household.value.length <= 1) {
    toast(t("household_keep_one_member"), "error");
    return;
  }
  try {
    await deleteHouseholdMemberApi(id);
    metaStore.refreshSoon();
    household.value = household.value.filter((m) => m.id !== id);
    toast(t("success"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}
</script>

<template>
  <UiCard :title="t('household')">
    <template #actions>
      <SavedIndicator :at="savedAt" />
    </template>
    <div class="space-y-2">
      <div v-for="m in sortedHousehold" :key="m.id" class="flex gap-2 items-center">
        <div class="flex-1 min-w-0">
          <AppInput :modelValue="m.name" @update:modelValue="(v: string | number) => saveMember(m.id, 'name', String(v))" />
        </div>
        <div class="flex-1 min-w-0">
          <AppInput :modelValue="m.email ?? ''" @update:modelValue="(v: string | number) => saveMember(m.id, 'email', String(v))" :placeholder="t('email')" />
        </div>
        <Tooltip v-if="sortedHousehold.length > 1" :text="t('delete')">
          <button
            type="button"
            :aria-label="t('delete')"
            @click="removeMember(m.id)"
            :class="button({ intent: 'ghost', size: 'icon', class: 'text-danger hover:bg-danger/12 hover:text-danger' })"
          ><Trash2 :size="16" /></button>
        </Tooltip>
      </div>
    </div>
    <button type="button" @click="addMember" :class="button({ intent: 'secondary', size: 'sm', class: 'mt-3' })">
      <Plus :size="14" /> {{ t('add') }}
    </button>
  </UiCard>
</template>
