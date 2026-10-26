<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import type { Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import Toast from "@/components/ui/Toast.vue";
import UiCard from "@/components/ui/UiCard.vue";
import SavedIndicator, { useAutoSave } from "@/components/settings/SavedIndicator.vue";
import ReorderButtons from "@/components/settings/ReorderButtons.vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { Trash2, Plus, Zap } from "@lucide/vue";
import { ui, button } from "@/lib/tv";

const props = defineProps<{ lookupSettings: Settings | null }>();
const { t } = useI18n();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();
const metaStore = useAppMetaStore();
const settings = ref<Settings | null>(null);
const { savedAt, run } = useAutoSave();

const newValue = ref("");
watch(
  () => props.lookupSettings,
  (value) => {
    settings.value = value;
  },
  { immediate: true, deep: true },
);

// Saved order (the converter shows them in this order); sorting here made up/down a no-op.
const presets = computed(() => settings.value?.converterPresets ?? []);

function updateSettings(updates: Partial<Settings>) {
  if (!settings.value) return;
  const next = { ...settings.value, ...updates };
  settings.value = next;
  void run(() => metaStore.updateSettings(next));
}

function addPreset() {
  const raw = newValue.value.replace(",", ".").replace(/[^\d.]/g, "");
  const val = parseFloat(raw);
  if (isNaN(val) || val <= 0) return;

  const current = settings.value?.converterPresets ?? [];
  if (current.includes(val)) {
    toast(t("preset_exists"), "error");
    return;
  }

  updateSettings({
    converterPresets: [...current, val].sort((a, b) => a - b),
  });
  newValue.value = "";
}

function removePreset(val: number) {
  const current = settings.value?.converterPresets ?? [];
  updateSettings({
    converterPresets: current.filter((v) => v !== val),
  });
}

function moveUp(val: number) {
  const arr = [...presets.value];
  const idx = arr.indexOf(val);
  if (idx <= 0) return;
  [arr[idx - 1], arr[idx]] = [arr[idx], arr[idx - 1]];
  updateSettings({ converterPresets: arr });
}

function moveDown(val: number) {
  const arr = [...presets.value];
  const idx = arr.indexOf(val);
  if (idx < 0 || idx >= arr.length - 1) return;
  [arr[idx], arr[idx + 1]] = [arr[idx + 1], arr[idx]];
  updateSettings({ converterPresets: arr });
}

function fmtNum(n: number): string {
  return n.toLocaleString(undefined, { maximumFractionDigits: 2 });
}
</script>

<template>
  <UiCard :title="t('converter_presets')" :icon="Zap" icon-tone="text-primary">
    <template #actions>
      <SavedIndicator :at="savedAt" />
    </template>

    <p class="text-xs text-text-muted mb-3">{{ t('converter_presets_desc') }}</p>

    <!-- Presets list -->
    <div class="space-y-1 max-h-60 overflow-y-auto overflow-x-hidden">
      <div
        v-for="(preset, idx) in presets"
        :key="preset"
        class="flex items-center gap-1.5 rounded-lg px-1.5 py-1 group"
      >
        <ReorderButtons
          :up-disabled="idx === 0"
          :down-disabled="idx === presets.length - 1"
          @up="moveUp(preset)"
          @down="moveDown(preset)"
        />

        <!-- Value -->
        <div class="flex-1 min-w-0">
          <span class="text-sm font-medium text-text-primary truncate block px-2 py-1 tabular-nums">
            {{ fmtNum(preset) }}
          </span>
        </div>

        <!-- Delete -->
        <Tooltip :text="t('delete')">
          <button
            type="button"
            :aria-label="t('delete')"
            @click="removePreset(preset)"
            :class="button({ intent: 'ghost', size: 'icon', class: 'h-7 w-7 text-danger hover:bg-danger/12 hover:text-danger' })"
          >
            <Trash2 :size="14" />
          </button>
        </Tooltip>
      </div>
    </div>

    <p v-if="presets.length === 0" class="text-sm text-text-muted text-center py-3">—</p>

    <!-- Add new preset -->
    <div class="flex items-center gap-2 mt-3">
      <input
        v-model="newValue"
        type="text"
        inputmode="decimal"
        pattern="[0-9]*[.,]?[0-9]*"
        :placeholder="t('amount')"
        @keydown.enter="addPreset"
        :class="ui.field({ class: 'flex-1 h-8 py-1 tabular-nums' })"
      />
      <button type="button" @click="addPreset" :class="button({ intent: 'secondary' })">
        <Plus :size="14" /> {{ t('add') }}
      </button>
    </div>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </UiCard>
</template>
