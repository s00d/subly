<script setup lang="ts">
import { ref, computed } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import Toast from "@/components/ui/Toast.vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { Trash2, Plus, ChevronUp, ChevronDown, Zap } from "lucide-vue-next";

const settingsStore = useSettingsStore();
const { t } = useI18n();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();

const newValue = ref("");

const presets = computed(() =>
  [...(settingsStore.settings.converterPresets ?? [])].sort((a, b) => a - b),
);

function addPreset() {
  const raw = newValue.value.replace(/[^\d.]/g, "");
  const val = parseFloat(raw);
  if (isNaN(val) || val <= 0) return;

  const current = settingsStore.settings.converterPresets ?? [];
  if (current.includes(val)) {
    toast(t("preset_exists"), "error");
    return;
  }

  settingsStore.updateSettings({
    converterPresets: [...current, val].sort((a, b) => a - b),
  });
  newValue.value = "";
  toast(t("success"));
}

function removePreset(val: number) {
  const current = settingsStore.settings.converterPresets ?? [];
  settingsStore.updateSettings({
    converterPresets: current.filter((v) => v !== val),
  });
}

function moveUp(val: number) {
  const arr = [...presets.value];
  const idx = arr.indexOf(val);
  if (idx <= 0) return;
  [arr[idx - 1], arr[idx]] = [arr[idx], arr[idx - 1]];
  settingsStore.updateSettings({ converterPresets: arr });
}

function moveDown(val: number) {
  const arr = [...presets.value];
  const idx = arr.indexOf(val);
  if (idx < 0 || idx >= arr.length - 1) return;
  [arr[idx], arr[idx + 1]] = [arr[idx + 1], arr[idx]];
  settingsStore.updateSettings({ converterPresets: arr });
}

function fmtNum(n: number): string {
  return n.toLocaleString(undefined, { maximumFractionDigits: 2 });
}
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-3 sm:p-5">
    <div class="flex items-center justify-between gap-2 mb-1">
      <div class="flex items-center gap-2 shrink-0">
        <Zap :size="16" class="text-[var(--color-primary)]" />
        <h2 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ t('converter_presets') }}</h2>
      </div>
    </div>
    <p class="text-xs text-[var(--color-text-muted)] mb-3">{{ t('converter_presets_desc') }}</p>

    <!-- Presets list -->
    <div class="space-y-1.5 max-h-60 overflow-y-auto overflow-x-hidden">
      <div
        v-for="(preset, idx) in presets"
        :key="preset"
        class="flex items-center gap-2 rounded-lg px-2 py-1 group"
      >
        <!-- Move buttons -->
        <div class="flex flex-row sm:flex-col shrink-0">
          <Tooltip :text="t('move_up')">
            <button @click="moveUp(preset)" :disabled="idx === 0" class="p-0.5 rounded text-[var(--color-text-muted)] hover:text-[var(--color-primary)] disabled:opacity-30 disabled:cursor-not-allowed transition-colors">
              <ChevronUp :size="14" />
            </button>
          </Tooltip>
          <Tooltip :text="t('move_down')">
            <button @click="moveDown(preset)" :disabled="idx === presets.length - 1" class="p-0.5 rounded text-[var(--color-text-muted)] hover:text-[var(--color-primary)] disabled:opacity-30 disabled:cursor-not-allowed transition-colors">
              <ChevronDown :size="14" />
            </button>
          </Tooltip>
        </div>

        <!-- Value -->
        <div class="flex-1 min-w-0">
          <span class="text-sm font-medium text-[var(--color-text-primary)] truncate block px-2 py-1 tabular-nums">
            {{ fmtNum(preset) }}
          </span>
        </div>

        <!-- Delete -->
        <Tooltip :text="t('delete')">
          <button @click="removePreset(preset)" class="p-1.5 rounded-lg text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors shrink-0">
            <Trash2 :size="14" />
          </button>
        </Tooltip>
      </div>
    </div>

    <p v-if="presets.length === 0" class="text-sm text-[var(--color-text-muted)] text-center py-3">—</p>

    <!-- Add new preset -->
    <div class="flex items-center gap-2 mt-3">
      <input
        v-model="newValue"
        type="text"
        inputmode="decimal"
        pattern="[0-9]*[.,]?[0-9]*"
        :placeholder="t('amount')"
        @keydown.enter="addPreset"
        class="flex-1 px-3 py-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] text-sm text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] transition-shadow tabular-nums"
      />
      <button @click="addPreset" class="px-3 py-1.5 rounded-lg bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)] transition-colors">
        <Plus :size="14" class="inline mr-1" /> {{ t('add') }}
      </button>
    </div>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </section>
</template>
