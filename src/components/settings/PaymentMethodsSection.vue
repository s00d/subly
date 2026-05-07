<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { upsertPaymentMethod, deletePaymentMethod as deletePaymentMethodApi, maxSortOrder } from "@/services/catalogClient";
import type { PaymentMethod, Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import AppInput from "@/components/ui/AppInput.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import IconPickerModal from "@/components/ui/IconPickerModal.vue";
import { Trash2, Plus, ChevronUp, ChevronDown, Star, Search, Eye, EyeOff } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { ui } from "@/lib/tv";

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
  Object.assign(pm, next);
}
async function setDefaultPaymentMethod(id: string) {
  if (!settings.value) return;
  const next = { ...settings.value, defaultPaymentMethodId: id };
  settings.value = next;
  await metaStore.updateSettings(next);
}
async function reorderPaymentMethods(ids: string[]) {
  for (let i = 0; i < ids.length; i += 1) {
    const pm = paymentMethods.value.find((p) => p.id === ids[i]);
    if (!pm) continue;
    const next = { ...pm, sortOrder: i };
    await upsertPaymentMethod(next);
    Object.assign(pm, next);
  }
}

async function addPm() {
  const order = await maxSortOrder("paymentMethods");
  const pm: PaymentMethod = { id: crypto.randomUUID(), name: "Payment method", icon: "/assets/card-generic.svg", enabled: true, sortOrder: order + 1, i18nKey: "" };
  await upsertPaymentMethod(pm);
  paymentMethods.value.push(pm);
}
function savePmName(id: string, name: string) { void updatePaymentMethod(id, { name }); }
async function removePm(id: string) {
  if (isUsedPayment(id)) {
    toast(t("payment_method_cannot_delete"), "error");
    return;
  }
  await deletePaymentMethodApi(id);
  paymentMethods.value = paymentMethods.value.filter((p) => p.id !== id);
  toast(t("success"));
}
function togglePm(id: string) {
  const pm = paymentMethods.value.find((p) => p.id === id);
  if (!pm) return;
  void updatePaymentMethod(id, { enabled: !pm.enabled });
}
function openEditIconPicker(id: string) { editIconPmId.value = id; showEditIconPicker.value = true; }
function onEditIconSelect(icon: string) { if (editIconPmId.value) void updatePaymentMethod(editIconPmId.value, { icon }); }
function movePmUp(id: string) {
  const ids = sortedPaymentMethods.value.map((p) => p.id);
  const idx = ids.indexOf(id);
  if (idx <= 0) return;
  [ids[idx - 1], ids[idx]] = [ids[idx], ids[idx - 1]];
  void reorderPaymentMethods(ids);
}
function movePmDown(id: string) {
  const ids = sortedPaymentMethods.value.map((p) => p.id);
  const idx = ids.indexOf(id);
  if (idx < 0 || idx >= ids.length - 1) return;
  [ids[idx], ids[idx + 1]] = [ids[idx + 1], ids[idx]];
  void reorderPaymentMethods(ids);
}
</script>

<template>
  <section class="bg-surface rounded-xl border border-border p-3 sm:p-5">
    <div class="flex items-center justify-between gap-2 mb-3">
      <h2 :class="[ui.sectionTitle(), 'shrink-0']">{{ t('payment_methods') }}</h2>
      <div class="relative w-32 sm:w-44">
        <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
        <input v-model="pmSearch" type="text" :placeholder="t('search')" class="w-full pl-8 pr-3 py-1.5 rounded-lg border border-border bg-surface text-xs text-text-primary placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-primary transition-shadow" />
      </div>
    </div>

    <!-- PMs list -->
    <div class="space-y-1.5 max-h-72 overflow-y-auto overflow-x-hidden">
      <div
        v-for="(pm, idx) in filteredPaymentMethods"
        :key="pm.id"
        class="flex items-center gap-2 rounded-lg px-2 py-1"
        :class="[!pm.enabled ? 'opacity-40' : '', pm.id === settings?.defaultPaymentMethodId ? 'bg-primary-light/50' : '']"
      >
        <!-- Move buttons -->
        <div v-if="!isPmSearching" class="flex flex-row sm:flex-col shrink-0">
          <Tooltip :text="t('move_up')"><button @click="movePmUp(pm.id)" :disabled="idx === 0" class="p-0.5 rounded text-text-muted hover:text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronUp :size="14" /></button></Tooltip>
          <Tooltip :text="t('move_down')"><button @click="movePmDown(pm.id)" :disabled="idx === sortedPaymentMethods.length - 1" class="p-0.5 rounded text-text-muted hover:text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronDown :size="14" /></button></Tooltip>
        </div>
        <!-- Default star -->
        <Tooltip :text="t('set_as_primary')">
          <button @click="setDefaultPaymentMethod(pm.id)" class="p-1 rounded-lg transition-colors shrink-0" :class="pm.id === settings?.defaultPaymentMethodId ? 'text-yellow-500' : 'text-text-muted hover:text-yellow-500'">
            <Star :size="14" :fill="pm.id === settings?.defaultPaymentMethodId ? 'currentColor' : 'none'" />
          </button>
        </Tooltip>
        <!-- Clickable icon -->
        <Tooltip :text="t('change_icon')">
          <button @click="openEditIconPicker(pm.id)" class="w-8 h-8 rounded-lg border border-border flex items-center justify-center shrink-0 hover:border-primary transition-colors cursor-pointer">
            <IconDisplay :icon="pm.icon" :size="20" />
          </button>
        </Tooltip>
        <!-- Name: read-only for default, editable for user-added -->
        <div class="flex-1 min-w-0">
          <span v-if="isDefaultItem(pm)" class="text-sm text-text-primary truncate block px-2 py-1">{{ pm.name }}</span>
          <AppInput v-else :modelValue="pm.name" @update:modelValue="(v: string | number) => savePmName(pm.id, String(v))" size="sm" />
        </div>
        <!-- Toggle -->
        <Tooltip :text="pm.enabled ? t('enabled') : t('off')">
          <button @click="togglePm(pm.id)" class="p-1.5 rounded-lg transition-colors shrink-0" :class="pm.enabled ? 'text-green-600 hover:bg-green-50 dark:hover:bg-green-900/20' : 'text-text-muted hover:bg-surface-hover'">
            <Eye v-if="pm.enabled" :size="14" /><EyeOff v-else :size="14" />
          </button>
        </Tooltip>
        <!-- Delete -->
        <Tooltip :text="t('delete')">
          <button @click="removePm(pm.id)" :disabled="isUsedPayment(pm.id)" class="p-1.5 rounded-lg transition-colors shrink-0" :class="isUsedPayment(pm.id) ? 'text-text-muted cursor-not-allowed' : 'text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20'"><Trash2 :size="14" /></button>
        </Tooltip>
      </div>
    </div>

    <!-- Add button (same style as Categories) -->
    <button @click="addPm" class="mt-3 px-3 py-1.5 rounded-lg bg-primary text-white text-sm hover:bg-primary-hover transition-colors">
      <Plus :size="14" class="inline mr-1" /> {{ t('add') }}
    </button>

    <!-- Icon picker for editing existing PM -->
    <IconPickerModal
      :show="showEditIconPicker"
      :modelValue="editIconPmId ? (paymentMethods.find((p) => p.id === editIconPmId)?.icon || '') : ''"
      group="payment"
      @update:modelValue="onEditIconSelect"
      @close="showEditIconPicker = false; editIconPmId = null"
    />
  </section>
</template>
