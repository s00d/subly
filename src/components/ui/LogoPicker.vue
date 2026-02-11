<script setup lang="ts">
import { ref, computed } from "vue";
import { logoAssets, type LogoAsset } from "@/services/logoAssets";
import { useI18n } from "@/i18n";
import { X, Upload, Image as ImageIcon, Search } from "lucide-vue-next";

const props = defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const { t } = useI18n();
const showPicker = ref(false);
const searchQuery = ref("");
const activeTab = ref<"gallery" | "upload">("gallery");

const filteredAssets = computed(() => {
  let assets = logoAssets;
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    assets = assets.filter((a) => a.name.toLowerCase().includes(q));
  }
  return assets;
});

const serviceAssets = computed(() => filteredAssets.value.filter((a) => a.group === "service"));
const paymentAssets = computed(() => filteredAssets.value.filter((a) => a.group === "payment"));

function selectAsset(path: string) {
  emit("update:modelValue", path);
  showPicker.value = false;
}

function handleUpload(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => {
    emit("update:modelValue", reader.result as string);
    showPicker.value = false;
  };
  reader.readAsDataURL(file);
}

function clear() {
  emit("update:modelValue", "");
}
</script>

<template>
  <div>
    <!-- Preview / trigger -->
    <div class="flex items-center gap-2">
      <button
        type="button"
        @click="showPicker = true"
        class="w-12 h-12 rounded-xl border-2 border-dashed border-[var(--color-border)] flex items-center justify-center cursor-pointer hover:border-[var(--color-primary)] hover:bg-[var(--color-primary-light)] transition-all overflow-hidden shrink-0"
      >
        <img
          v-if="modelValue"
          :src="modelValue"
          class="w-full h-full object-contain p-1"
        />
        <ImageIcon v-else :size="20" class="text-[var(--color-text-muted)]" />
      </button>
      <button
        v-if="modelValue"
        type="button"
        @click="clear"
        class="p-1 rounded-md text-[var(--color-text-muted)] hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
      >
        <X :size="14" />
      </button>
    </div>

    <!-- Picker Modal -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div v-if="showPicker" class="fixed inset-0 z-[200] flex items-center justify-center p-4">
          <div class="absolute inset-0 bg-black/50" @click="showPicker = false" />
          <div class="relative bg-[var(--color-surface)] rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden">
            <!-- Header -->
            <div class="flex items-center justify-between px-5 py-4 border-b border-[var(--color-border)]">
              <h3 class="text-base font-semibold text-[var(--color-text-primary)]">{{ t('upload_logo') }}</h3>
              <button
                @click="showPicker = false"
                class="p-1 rounded-lg hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]"
              >
                <X :size="18" />
              </button>
            </div>

            <!-- Tabs -->
            <div class="flex border-b border-[var(--color-border)]">
              <button
                type="button"
                @click="activeTab = 'gallery'"
                class="flex-1 px-4 py-2.5 text-sm font-medium transition-colors"
                :class="activeTab === 'gallery'
                  ? 'text-[var(--color-primary)] border-b-2 border-[var(--color-primary)]'
                  : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)]'"
              >
                <ImageIcon :size="14" class="inline mr-1.5" />
                {{ t('gallery') }}
              </button>
              <button
                type="button"
                @click="activeTab = 'upload'"
                class="flex-1 px-4 py-2.5 text-sm font-medium transition-colors"
                :class="activeTab === 'upload'
                  ? 'text-[var(--color-primary)] border-b-2 border-[var(--color-primary)]'
                  : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)]'"
              >
                <Upload :size="14" class="inline mr-1.5" />
                {{ t('upload_logo') }}
              </button>
            </div>

            <!-- Gallery tab -->
            <div v-if="activeTab === 'gallery'" class="p-4">
              <!-- Search -->
              <div class="relative mb-3">
                <Search :size="14" class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
                <input
                  v-model="searchQuery"
                  type="text"
                  placeholder="Search icons..."
                  class="w-full pl-9 pr-3 py-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-secondary)] text-sm text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
                />
              </div>
              <!-- Grid: Services -->
              <div class="max-h-64 overflow-y-auto space-y-3">
                <div v-if="serviceAssets.length > 0">
                  <p class="text-xs font-medium text-[var(--color-text-muted)] mb-1.5 uppercase tracking-wide">{{ t('services') }}</p>
                  <div class="grid grid-cols-8 gap-2">
                    <button
                      v-for="asset in serviceAssets"
                      :key="asset.path"
                      type="button"
                      @click="selectAsset(asset.path)"
                      class="w-full aspect-square rounded-lg border flex items-center justify-center p-1.5 transition-all hover:scale-105"
                      :class="modelValue === asset.path
                        ? 'border-[var(--color-primary)] bg-[var(--color-primary-light)] ring-2 ring-[var(--color-primary)]'
                        : 'border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-surface-secondary)]'"
                      :title="asset.name"
                    >
                      <img :src="asset.path" :alt="asset.name" class="w-full h-full object-contain" />
                    </button>
                  </div>
                </div>
                <div v-if="paymentAssets.length > 0">
                  <p class="text-xs font-medium text-[var(--color-text-muted)] mb-1.5 uppercase tracking-wide">{{ t('payment_methods') }}</p>
                  <div class="grid grid-cols-8 gap-2">
                    <button
                      v-for="asset in paymentAssets"
                      :key="asset.path"
                      type="button"
                      @click="selectAsset(asset.path)"
                      class="w-full aspect-square rounded-lg border flex items-center justify-center p-1.5 transition-all hover:scale-105"
                      :class="modelValue === asset.path
                        ? 'border-[var(--color-primary)] bg-[var(--color-primary-light)] ring-2 ring-[var(--color-primary)]'
                        : 'border-[var(--color-border)] hover:border-[var(--color-text-muted)] bg-[var(--color-surface-secondary)]'"
                      :title="asset.name"
                    >
                      <img :src="asset.path" :alt="asset.name" class="w-full h-full object-contain" />
                    </button>
                  </div>
                </div>
              </div>
              <div v-if="filteredAssets.length === 0" class="text-center text-sm text-[var(--color-text-muted)] py-8">
                {{ t('no_results') }}
              </div>
            </div>

            <!-- Upload tab -->
            <div v-else class="p-6">
              <label
                class="flex flex-col items-center justify-center w-full h-40 rounded-xl border-2 border-dashed border-[var(--color-border)] cursor-pointer hover:border-[var(--color-primary)] hover:bg-[var(--color-primary-light)] transition-colors"
              >
                <Upload :size="32" class="text-[var(--color-text-muted)] mb-2" />
                <p class="text-sm text-[var(--color-text-secondary)] font-medium">{{ t('click_to_upload') }}</p>
                <p class="text-xs text-[var(--color-text-muted)] mt-1">{{ t('supported_formats') }}</p>
                <input type="file" accept="image/*" class="hidden" @change="handleUpload" />
              </label>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
