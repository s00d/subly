<script setup lang="ts">
import { ref, computed } from "vue";
import { useCatalogStore } from "@/stores/catalog";
import { useSettingsStore } from "@/stores/settings";
import { useSubscriptionsStore } from "@/stores/subscriptions";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import IconPickerModal from "@/components/ui/IconPickerModal.vue";
import { Trash2, Plus, ChevronUp, ChevronDown, Star, Search, Eye, EyeOff } from "lucide-vue-next";
import Tooltip from "@/components/ui/Tooltip.vue";

const catalogStore = useCatalogStore();
const settingsStore = useSettingsStore();
const subsStore = useSubscriptionsStore();
const { t } = useI18n();
const { toast } = useToast();

const pmSearch = ref("");
const isPmSearching = computed(() => pmSearch.value.length > 0);

// Edit PM icon
const editIconPmId = ref<string | null>(null);
const showEditIconPicker = ref(false);

/** Sorted: default/primary first, then by order */
const sortedPaymentMethods = computed(() => {
  const defId = settingsStore.settings.defaultPaymentMethodId;
  return catalogStore.paymentMethods.slice().sort((a, b) => {
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

const isUsedPayment = (id: string) => subsStore.subscriptions.some((s) => s.paymentMethodId === id);
const isDefaultItem = (pm: { i18nKey?: string }) => !!pm.i18nKey;

function addPm() {
  catalogStore.addPaymentMethod("Payment method", "/assets/card-generic.svg");
}

function savePmName(id: string, name: string) {
  catalogStore.updatePaymentMethod(id, { name });
}

function removePm(id: string) {
  if (!catalogStore.deletePaymentMethod(id)) toast(t("error"), "error");
  else toast(t("success"));
}

function togglePm(id: string) { catalogStore.togglePaymentMethod(id); }

// Icon picker for existing PM
function openEditIconPicker(id: string) {
  editIconPmId.value = id;
  showEditIconPicker.value = true;
}

function onEditIconSelect(icon: string) {
  if (editIconPmId.value) {
    catalogStore.updatePaymentMethod(editIconPmId.value, { icon });
  }
}

// Reorder
function movePmUp(id: string) {
  const ids = sortedPaymentMethods.value.map((p) => p.id);
  const idx = ids.indexOf(id);
  if (idx <= 0) return;
  [ids[idx - 1], ids[idx]] = [ids[idx], ids[idx - 1]];
  catalogStore.reorderPaymentMethods(ids);
}

function movePmDown(id: string) {
  const ids = sortedPaymentMethods.value.map((p) => p.id);
  const idx = ids.indexOf(id);
  if (idx < 0 || idx >= ids.length - 1) return;
  [ids[idx], ids[idx + 1]] = [ids[idx + 1], ids[idx]];
  catalogStore.reorderPaymentMethods(ids);
}
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
    <div class="flex items-center justify-between gap-2 mb-3">
      <h2 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)] shrink-0">{{ t('payment_methods') }}</h2>
      <div class="relative w-32 sm:w-44">
        <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
        <input v-model="pmSearch" type="text" :placeholder="t('search')" class="w-full pl-8 pr-3 py-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] text-xs text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] transition-shadow" />
      </div>
    </div>

    <!-- PMs list -->
    <div class="space-y-1.5 max-h-72 overflow-y-auto overflow-x-hidden">
      <div
        v-for="(pm, idx) in filteredPaymentMethods"
        :key="pm.id"
        class="flex items-center gap-2 rounded-lg px-2 py-1"
        :class="[!pm.enabled ? 'opacity-40' : '', pm.id === settingsStore.settings.defaultPaymentMethodId ? 'bg-[var(--color-primary-light)]/50' : '']"
      >
        <!-- Move buttons -->
        <div v-if="!isPmSearching" class="flex flex-row sm:flex-col shrink-0">
          <Tooltip :text="t('move_up')"><button @click="movePmUp(pm.id)" :disabled="idx === 0" class="p-0.5 rounded text-[var(--color-text-muted)] hover:text-[var(--color-primary)] disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronUp :size="14" /></button></Tooltip>
          <Tooltip :text="t('move_down')"><button @click="movePmDown(pm.id)" :disabled="idx === sortedPaymentMethods.length - 1" class="p-0.5 rounded text-[var(--color-text-muted)] hover:text-[var(--color-primary)] disabled:opacity-30 disabled:cursor-not-allowed transition-colors"><ChevronDown :size="14" /></button></Tooltip>
        </div>
        <!-- Default star -->
        <Tooltip :text="t('set_as_primary')">
          <button @click="settingsStore.updateSettings({ defaultPaymentMethodId: pm.id })" class="p-1 rounded-lg transition-colors shrink-0" :class="pm.id === settingsStore.settings.defaultPaymentMethodId ? 'text-yellow-500' : 'text-[var(--color-text-muted)] hover:text-yellow-500'">
            <Star :size="14" :fill="pm.id === settingsStore.settings.defaultPaymentMethodId ? 'currentColor' : 'none'" />
          </button>
        </Tooltip>
        <!-- Clickable icon -->
        <Tooltip :text="t('change_icon')">
          <button @click="openEditIconPicker(pm.id)" class="w-8 h-8 rounded-lg border border-[var(--color-border)] flex items-center justify-center shrink-0 hover:border-[var(--color-primary)] transition-colors cursor-pointer">
            <IconDisplay :icon="pm.icon" :size="20" />
          </button>
        </Tooltip>
        <!-- Name: read-only for default, editable for user-added -->
        <div class="flex-1 min-w-0">
          <span v-if="isDefaultItem(pm)" class="text-sm text-[var(--color-text-primary)] truncate block px-2 py-1">{{ pm.name }}</span>
          <AppInput v-else :modelValue="pm.name" @update:modelValue="(v: string | number) => savePmName(pm.id, String(v))" size="sm" />
        </div>
        <!-- Toggle -->
        <Tooltip :text="pm.enabled ? t('enabled') : t('off')">
          <button @click="togglePm(pm.id)" class="p-1.5 rounded-lg transition-colors shrink-0" :class="pm.enabled ? 'text-green-600 hover:bg-green-50 dark:hover:bg-green-900/20' : 'text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)]'">
            <Eye v-if="pm.enabled" :size="14" /><EyeOff v-else :size="14" />
          </button>
        </Tooltip>
        <!-- Delete -->
        <Tooltip :text="t('delete')">
          <button @click="removePm(pm.id)" :disabled="isUsedPayment(pm.id)" class="p-1.5 rounded-lg transition-colors shrink-0" :class="isUsedPayment(pm.id) ? 'text-[var(--color-text-muted)] cursor-not-allowed' : 'text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20'"><Trash2 :size="14" /></button>
        </Tooltip>
      </div>
    </div>

    <!-- Add button (same style as Categories) -->
    <button @click="addPm" class="mt-3 px-3 py-1.5 rounded-lg bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)] transition-colors">
      <Plus :size="14" class="inline mr-1" /> {{ t('add') }}
    </button>

    <!-- Icon picker for editing existing PM -->
    <IconPickerModal
      :show="showEditIconPicker"
      :modelValue="editIconPmId ? (catalogStore.paymentMethods.find((p) => p.id === editIconPmId)?.icon || '') : ''"
      group="payment"
      @update:modelValue="onEditIconSelect"
      @close="showEditIconPicker = false; editIconPmId = null"
    />
  </section>
</template>
