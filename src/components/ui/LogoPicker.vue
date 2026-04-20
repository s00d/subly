<script setup lang="ts">
import { ref, computed } from "vue";
import { logoAssets } from "@/services/logoAssets";
import { useI18n } from "vue-i18n";
import { X, Upload, Image as ImageIcon, Search, Link as LinkIcon } from "lucide-vue-next";
import { tv } from "@/lib/tv";
import { useToast } from "@/composables/useToast";
import { useScrollLock } from "@/composables/useScrollLock";

const props = defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const { t } = useI18n();
const { toast } = useToast();
const showPicker = ref(false);
const searchQuery = ref("");
const activeTab = ref<"gallery" | "upload" | "url">("gallery");
const logoUrl = ref("");
const MAX_ICON_DIMENSION = 128;
const MAX_ICON_DATA_URL_BYTES = 200 * 1024;

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

function dataUrlSizeBytes(dataUrl: string): number {
  const base64 = dataUrl.split(",")[1] ?? "";
  return Math.ceil((base64.length * 3) / 4);
}

async function loadImageFromFile(file: File): Promise<HTMLImageElement> {
  return await new Promise((resolve, reject) => {
    const objectUrl = URL.createObjectURL(file);
    const img = new Image();
    img.onload = () => {
      URL.revokeObjectURL(objectUrl);
      resolve(img);
    };
    img.onerror = () => {
      URL.revokeObjectURL(objectUrl);
      reject(new Error("Invalid image"));
    };
    img.src = objectUrl;
  });
}

async function fileToDataUrl(file: File): Promise<string> {
  return await new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error("Read failed"));
    reader.readAsDataURL(file);
  });
}

async function compressImageToDataUrl(file: File): Promise<string> {
  if (file.type === "image/svg+xml") {
    return await fileToDataUrl(file);
  }

  const image = await loadImageFromFile(file);
  const scale = Math.min(1, MAX_ICON_DIMENSION / Math.max(image.width, image.height));
  const width = Math.max(1, Math.round(image.width * scale));
  const height = Math.max(1, Math.round(image.height * scale));

  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("Canvas not available");
  ctx.clearRect(0, 0, width, height);
  ctx.drawImage(image, 0, 0, width, height);

  let dataUrl = canvas.toDataURL("image/webp", 0.85);
  if (dataUrlSizeBytes(dataUrl) > MAX_ICON_DATA_URL_BYTES) {
    dataUrl = canvas.toDataURL("image/jpeg", 0.8);
  }
  if (dataUrlSizeBytes(dataUrl) > MAX_ICON_DATA_URL_BYTES) {
    dataUrl = canvas.toDataURL("image/jpeg", 0.65);
  }
  return dataUrl;
}

async function handleUpload(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    const dataUrl = await compressImageToDataUrl(file);
    if (dataUrlSizeBytes(dataUrl) > MAX_ICON_DATA_URL_BYTES) {
      toast("Image is too large after compression", "error");
      return;
    }
    emit("update:modelValue", dataUrl);
    showPicker.value = false;
  } catch {
    toast("Failed to process image", "error");
  } finally {
    input.value = "";
  }
}

function saveUrlLogo() {
  const value = logoUrl.value.trim();
  if (!value) return;
  emit("update:modelValue", value);
  showPicker.value = false;
}

function clear() {
  emit("update:modelValue", "");
}

const pickerTv = tv({
  slots: {
    previewBtn: [
      "w-12 h-12 rounded-xl border-2 border-dashed border-border",
      "flex items-center justify-center cursor-pointer",
      "hover:border-primary hover:bg-primary-light",
      "transition-all overflow-hidden shrink-0",
    ],
    clearBtn: [
      "p-1 rounded-md text-text-muted",
      "hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors",
    ],
    overlay: "fixed inset-0 z-[200] flex items-end sm:items-center justify-center sm:p-4",
    backdrop: "absolute inset-0 bg-black/50",
    panel: [
      "relative bg-surface w-full overflow-hidden",
      "rounded-t-2xl sm:rounded-2xl shadow-2xl max-h-[85vh] sm:max-h-none sm:max-w-lg",
    ],
    header: "flex items-center justify-between px-4 sm:px-5 py-3 sm:py-4 border-b border-border",
    headerTitle: "text-sm sm:text-base font-semibold text-text-primary",
    closeBtn: "p-1 rounded-lg hover:bg-surface-hover text-text-muted",
    tabBar: "flex border-b border-border",
    tab: "flex-1 px-4 py-2.5 text-sm font-medium transition-colors",
    searchInput: [
      "w-full pl-9 pr-3 py-2 rounded-lg border border-border",
      "bg-surface-secondary text-sm text-text-primary",
      "placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-primary",
    ],
    assetGrid: "grid grid-cols-6 sm:grid-cols-8 gap-1.5 sm:gap-2",
    assetBtn: "w-full aspect-square rounded-lg border flex items-center justify-center p-1 sm:p-1.5 transition-all hover:scale-105",
    sectionLabel: "text-xs font-medium text-text-muted mb-1.5 uppercase tracking-wide",
    uploadZone: [
      "flex flex-col items-center justify-center w-full h-40 rounded-xl",
      "border-2 border-dashed border-border cursor-pointer",
      "hover:border-primary hover:bg-primary-light transition-colors",
    ],
    urlInput: [
      "w-full px-3 py-2 rounded-lg border border-border",
      "bg-surface-secondary text-sm text-text-primary",
      "placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-primary",
    ],
    actionBtn: [
      "px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium",
      "hover:bg-primary-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
    ],
  },
  variants: {
    tabActive: {
      true: { tab: "text-primary border-b-2 border-primary" },
      false: { tab: "text-text-muted hover:text-text-secondary" },
    },
    assetSelected: {
      true: { assetBtn: "border-primary bg-primary-light ring-2 ring-primary" },
      false: { assetBtn: "border-border hover:border-text-muted bg-surface-secondary" },
    },
  },
});

const slots = pickerTv();
useScrollLock(showPicker);
</script>

<template>
  <div>
    <div class="flex items-center gap-2">
      <button type="button" @click="showPicker = true" :class="slots.previewBtn()">
        <img v-if="modelValue" :src="modelValue" class="w-full h-full object-contain p-1" />
        <ImageIcon v-else :size="20" class="text-text-muted" />
      </button>
      <button v-if="modelValue" type="button" @click="clear" :class="slots.clearBtn()">
        <X :size="14" />
      </button>
    </div>

    <Teleport to="body">
      <Transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div v-if="showPicker" :class="slots.overlay()">
          <div :class="slots.backdrop()" @click="showPicker = false" />
          <div :class="slots.panel()">
            <div :class="slots.header()">
              <h3 :class="slots.headerTitle()">{{ t('upload_logo') }}</h3>
              <button @click="showPicker = false" :class="slots.closeBtn()">
                <X :size="18" />
              </button>
            </div>

            <div :class="slots.tabBar()">
              <button
                type="button"
                @click="activeTab = 'gallery'"
                :class="pickerTv({ tabActive: activeTab === 'gallery' }).tab()"
              >
                <ImageIcon :size="14" class="inline mr-1.5" />
                {{ t('gallery') }}
              </button>
              <button
                type="button"
                @click="activeTab = 'upload'"
                :class="pickerTv({ tabActive: activeTab === 'upload' }).tab()"
              >
                <Upload :size="14" class="inline mr-1.5" />
                {{ t('upload_logo') }}
              </button>
              <button
                type="button"
                @click="activeTab = 'url'"
                :class="pickerTv({ tabActive: activeTab === 'url' }).tab()"
              >
                <LinkIcon :size="14" class="inline mr-1.5" />
                URL
              </button>
            </div>

            <div v-if="activeTab === 'gallery'" class="p-3 sm:p-4">
              <div class="relative mb-3">
                <Search :size="14" class="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
                <input
                  v-model="searchQuery"
                  type="text"
                  placeholder="Search icons..."
                  :class="slots.searchInput()"
                />
              </div>
              <div class="max-h-64 overflow-y-auto space-y-3">
                <div v-if="serviceAssets.length > 0">
                  <p :class="slots.sectionLabel()">{{ t('services') }}</p>
                  <div :class="slots.assetGrid()">
                    <button
                      v-for="asset in serviceAssets"
                      :key="asset.path"
                      type="button"
                      @click="selectAsset(asset.path)"
                      :class="pickerTv({ assetSelected: modelValue === asset.path }).assetBtn()"
                      :title="asset.name"
                    >
                      <img :src="asset.path" :alt="asset.name" class="w-full h-full object-contain" />
                    </button>
                  </div>
                </div>
                <div v-if="paymentAssets.length > 0">
                  <p :class="slots.sectionLabel()">{{ t('payment_methods') }}</p>
                  <div :class="slots.assetGrid()">
                    <button
                      v-for="asset in paymentAssets"
                      :key="asset.path"
                      type="button"
                      @click="selectAsset(asset.path)"
                      :class="pickerTv({ assetSelected: modelValue === asset.path }).assetBtn()"
                      :title="asset.name"
                    >
                      <img :src="asset.path" :alt="asset.name" class="w-full h-full object-contain" />
                    </button>
                  </div>
                </div>
              </div>
              <div v-if="filteredAssets.length === 0" class="text-center text-sm text-text-muted py-8">
                {{ t('no_results') }}
              </div>
            </div>

            <div v-else-if="activeTab === 'upload'" class="p-4 sm:p-6">
              <label :class="slots.uploadZone()">
                <Upload :size="32" class="text-text-muted mb-2" />
                <p class="text-sm text-text-secondary font-medium">{{ t('click_to_upload') }}</p>
                <p class="text-xs text-text-muted mt-1">{{ t('supported_formats') }}</p>
                <input type="file" accept="image/*" class="hidden" @change="handleUpload" />
              </label>
            </div>

            <div v-else class="p-4 sm:p-6 space-y-3">
              <label class="block text-sm font-medium text-text-primary">Image URL</label>
              <input
                v-model="logoUrl"
                type="url"
                placeholder="https://example.com/logo.png"
                :class="slots.urlInput()"
              />
              <p class="text-xs text-text-muted">PNG, JPG, SVG or favicon URL</p>
              <button type="button" :class="slots.actionBtn()" :disabled="!logoUrl.trim()" @click="saveUrlLogo">
                Use this icon
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
