<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { upsertPaymentMethod, deletePaymentMethod as deletePaymentMethodApi, maxSortOrder } from "@/services/catalogClient";
import type { PaymentMethod, Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { useCatalogsUsageStore } from "@/stores/catalogsUsageStore";
import { formatErrorForToast } from "@/utils/formatError";
import AppInput from "@/components/ui/AppInput.vue";
import UiCard from "@/components/ui/UiCard.vue";
import SavedIndicator, { useAutoSave } from "@/components/settings/SavedIndicator.vue";
import SettingsSearch from "@/components/settings/SettingsSearch.vue";
import ReorderButtons from "@/components/settings/ReorderButtons.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import IconPickerModal from "@/components/ui/IconPickerModal.vue";
import { Trash2, Plus, Eye, EyeOff } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { button } from "@/lib/tv";

const props = defineProps<{
  lookupData: {
    paymentMethods: PaymentMethod[];
    settings: Settings;
    paymentMethodUsage: Record<string, number>;
  } | null;
}>();
const { t } = useI18n();
const { toast } = useToast();
const metaStore = useAppMetaStore();
const catalogsUsageStore = useCatalogsUsageStore();
const { savedAt, run, debounce } = useAutoSave();
const iconBtn = button({ intent: "ghost", size: "icon", class: "h-7 w-7" });
const paymentMethods = ref<PaymentMethod[]>([]);
const settings = ref<Settings | null>(null);
const paymentMethodUsage = ref<Record<string, number>>({});
watch(
  () => props.lookupData,
  (lookup) => {
    paymentMethods.value = lookup?.paymentMethods ?? [];
    settings.value = lookup?.settings ?? null;
    paymentMethodUsage.value = lookup?.paymentMethodUsage ?? {};
  },
  { immediate: true, deep: true },
);

const pmSearch = ref("");
const isPmSearching = computed(() => pmSearch.value.length > 0);
const editIconPmId = ref<string | null>(null);
const showEditIconPicker = ref(false);

/** The default method is pinned on top by the sort: it can't move and nothing moves above it. */
const pinnedCount = computed(() =>
  paymentMethods.value.some((p) => p.id === settings.value?.defaultPaymentMethodId) ? 1 : 0,
);
const sortedPaymentMethods = computed(() => {
  const defId = settings.value?.defaultPaymentMethodId;
  return paymentMethods.value.slice().sort((a, b) => {
    if (a.id === defId && b.id !== defId) return -1;
    if (b.id === defId && a.id !== defId) return 1;
    return a.sortOrder - b.sortOrder;
  });
});

const filteredPaymentMethods = computed(() => {
  if (!pmSearch.value) return sortedPaymentMethods.value;
  const q = pmSearch.value.toLowerCase();
  return sortedPaymentMethods.value.filter((pm) => pm.name.toLowerCase().includes(q));
});

const isUsedPayment = (id: string) => (paymentMethodUsage.value[id] ?? 0) > 0;
const isDefaultItem = (pm: { i18nKey?: string }) => !!pm.i18nKey;

async function updatePaymentMethod(id: string, updates: Partial<PaymentMethod>) {
  const pm = paymentMethods.value.find((p) => p.id === id);
  if (!pm) return;
  const next = { ...pm, ...updates };
  await upsertPaymentMethod(next);
  metaStore.refreshSoon();
  Object.assign(pm, next);
}
function setDefaultPaymentMethod(id: string) {
  if (!settings.value) return;
  const next = { ...settings.value, defaultPaymentMethodId: id };
  settings.value = next;
  void run(() => metaStore.updateSettings(next));
}
async function reorderPaymentMethods(ids: string[]) {
  for (let i = 0; i < ids.length; i += 1) {
    const pm = paymentMethods.value.find((p) => p.id === ids[i]);
    if (!pm) continue;
    const next = { ...pm, sortOrder: i };
    await upsertPaymentMethod(next);
    metaStore.refreshSoon();
    Object.assign(pm, next);
  }
}

function addPm() {
  void run(async () => {
    const order = await maxSortOrder("paymentMethods");
    const pm: PaymentMethod = { id: crypto.randomUUID(), name: "Payment method", icon: "/assets/card-generic.svg", enabled: true, sortOrder: order + 1, i18nKey: "" };
    await upsertPaymentMethod(pm);
    metaStore.refreshSoon();
    paymentMethods.value.push(pm);
  });
}
/** Name edits: update the row now, persist once typing pauses. */
function savePmName(id: string, name: string) {
  const pm = paymentMethods.value.find((p) => p.id === id);
  if (!pm) return;
  pm.name = name;
  debounce(`name:${id}`, () => updatePaymentMethod(id, { name: pm.name }));
}
async function removePm(id: string) {
  try {
    await deletePaymentMethodApi(id);
    metaStore.refreshSoon();
    paymentMethods.value = paymentMethods.value.filter((p) => p.id !== id);
    await catalogsUsageStore.refresh();
    toast(t("success"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}
function togglePm(id: string) {
  const pm = paymentMethods.value.find((p) => p.id === id);
  if (!pm) return;
  void run(() => updatePaymentMethod(id, { enabled: !pm.enabled }));
}
function openEditIconPicker(id: string) { editIconPmId.value = id; showEditIconPicker.value = true; }
function onEditIconSelect(icon: string) {
  const id = editIconPmId.value;
  if (id) void run(() => updatePaymentMethod(id, { icon }));
}
function movePmUp(id: string) {
  const ids = sortedPaymentMethods.value.map((p) => p.id);
  const idx = ids.indexOf(id);
  if (idx <= 0) return;
  [ids[idx - 1], ids[idx]] = [ids[idx], ids[idx - 1]];
  void run(() => reorderPaymentMethods(ids));
}
function movePmDown(id: string) {
  const ids = sortedPaymentMethods.value.map((p) => p.id);
  const idx = ids.indexOf(id);
  if (idx < 0 || idx >= ids.length - 1) return;
  [ids[idx], ids[idx + 1]] = [ids[idx + 1], ids[idx]];
  void run(() => reorderPaymentMethods(ids));
}
</script>

<template>
  <UiCard :title="t('payment_methods')">
    <template #actions>
      <SavedIndicator :at="savedAt" />
      <SettingsSearch v-model="pmSearch" class="w-32 sm:w-44" />
    </template>

    <!-- PMs list -->
    <div class="space-y-1 max-h-72 overflow-y-auto overflow-x-hidden">
      <div
        v-for="(pm, idx) in filteredPaymentMethods"
        :key="pm.id"
        class="flex items-center gap-1.5 rounded-lg px-1.5 py-1"
        :class="[!pm.enabled ? 'opacity-40' : '', pm.id === settings?.defaultPaymentMethodId ? 'bg-primary-light/50' : '']"
      >
        <!-- Move buttons -->
        <ReorderButtons
          v-if="!isPmSearching && idx >= pinnedCount"
          :up-disabled="idx <= pinnedCount"
          :down-disabled="idx === sortedPaymentMethods.length - 1"
          @up="movePmUp(pm.id)"
          @down="movePmDown(pm.id)"
        />
        <div v-else-if="!isPmSearching" class="w-14 shrink-0" />
        <!-- Default (radio) -->
        <Tooltip :text="t('set_as_primary')">
          <button
            type="button"
            role="radio"
            :aria-checked="pm.id === settings?.defaultPaymentMethodId"
            :aria-label="t('set_as_primary')"
            :class="iconBtn"
            @click="setDefaultPaymentMethod(pm.id)"
          >
            <span class="flex h-3.5 w-3.5 items-center justify-center rounded-full border-2" :class="pm.id === settings?.defaultPaymentMethodId ? 'border-primary' : 'border-border-strong'">
              <span v-if="pm.id === settings?.defaultPaymentMethodId" class="h-1.5 w-1.5 rounded-full bg-primary" />
            </span>
          </button>
        </Tooltip>
        <!-- Clickable icon -->
        <Tooltip :text="t('change_icon')">
          <button type="button" @click="openEditIconPicker(pm.id)" class="w-8 h-8 rounded-lg border border-border flex items-center justify-center shrink-0 hover:border-primary transition-colors cursor-pointer">
            <IconDisplay :icon="pm.icon" :size="20" />
          </button>
        </Tooltip>
        <!-- Name: read-only for default, editable for user-added -->
        <div class="flex-1 min-w-0">
          <span v-if="isDefaultItem(pm)" class="text-xs text-text-primary truncate block px-2.5 py-1 border border-transparent">{{ pm.name }}</span>
          <AppInput v-else :modelValue="pm.name" @update:modelValue="(v: string | number) => savePmName(pm.id, String(v))" size="sm" variant="inline" />
        </div>
        <span v-if="pm.id === settings?.defaultPaymentMethodId" class="shrink-0 rounded-full bg-primary/12 px-1.5 py-0.5 text-[10px] font-medium text-primary">{{ t('catalog_default') }}</span>
        <!-- Toggle -->
        <Tooltip :text="pm.enabled ? t('enabled') : t('off')">
          <button
            type="button"
            :aria-pressed="pm.enabled"
            :aria-label="pm.enabled ? t('enabled') : t('off')"
            @click="togglePm(pm.id)"
            :class="[iconBtn, pm.enabled ? 'text-success hover:bg-success/12 hover:text-success' : 'text-text-muted']"
          >
            <Eye v-if="pm.enabled" :size="14" /><EyeOff v-else :size="14" />
          </button>
        </Tooltip>
        <!-- Delete -->
        <Tooltip :text="t('delete')">
          <button type="button" :aria-label="t('delete')" @click="removePm(pm.id)" :class="[iconBtn, 'text-danger hover:bg-danger/12 hover:text-danger']"><Trash2 :size="14" /></button>
        </Tooltip>
      </div>
    </div>

    <button type="button" @click="addPm" :class="button({ intent: 'secondary', size: 'sm', class: 'mt-3' })">
      <Plus :size="14" /> {{ t('add') }}
    </button>

    <!-- Icon picker for editing existing PM -->
    <IconPickerModal
      :show="showEditIconPicker"
      :modelValue="editIconPmId ? (paymentMethods.find((p) => p.id === editIconPmId)?.icon || '') : ''"
      group="payment"
      @update:modelValue="onEditIconSelect"
      @close="showEditIconPicker = false; editIconPmId = null"
    />
  </UiCard>
</template>
