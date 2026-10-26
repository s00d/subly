<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { getLogoAssets, type LogoAsset } from "@/services/logoClient";
import { useI18n } from "vue-i18n";
import { X, Upload, Image as ImageIcon, Search, Link as LinkIcon } from "@lucide/vue";
import { tv, ui, button } from "@/lib/tv";
import { useToast } from "@/composables/useToast";
import { formatErrorForToast } from "@/utils/formatError";
import Modal from "@/components/ui/Modal.vue";

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
const logoAssets = ref<LogoAsset[]>([]);
const MAX_ICON_DIMENSION = 128;
const MAX_ICON_DATA_URL_BYTES = 200 * 1024;

const filteredAssets = computed(() => {
  let assets = logoAssets.value;
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
      toast(t("logo_too_large"), "error");
      return;
    }
    emit("update:modelValue", dataUrl);
    showPicker.value = false;
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
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
      "w-12 h-12 rounded-xl border-2 border-dashed border-border-strong",
      "flex items-center justify-center cursor-pointer",
      "hover:border-primary hover:bg-surface-secondary",
      "transition-all overflow-hidden shrink-0",
    ],
    tabBar: "flex -mx-5 -mt-4 mb-4 border-b border-border",
    tab: "flex-1 inline-flex items-center justify-center gap-1.5 px-4 py-2.5 text-sm font-medium transition-colors",
    searchInput: ui.field({ class: "pl-9" }),
    assetGrid: "grid grid-cols-6 sm:grid-cols-8 gap-1.5 sm:gap-2",
    assetBtn: "w-full aspect-square rounded-lg border flex items-center justify-center p-1 sm:p-1.5 transition-all hover:scale-105",
    sectionLabel: "text-xs font-medium text-text-muted mb-1.5 uppercase tracking-wide",
    uploadZone: [
      "flex flex-col items-center justify-center w-full h-40 rounded-xl",
      "border-2 border-dashed border-border-strong cursor-pointer",
      "hover:border-primary hover:bg-surface-secondary transition-colors",
    ],
  },
  variants: {
    tabActive: {
      true: { tab: "text-primary border-b-2 border-primary" },
      false: { tab: "text-text-muted hover:text-text-secondary" },
    },
    assetSelected: {
      true: { assetBtn: "border-primary bg-primary-light ring-2 ring-primary/20" },
      false: { assetBtn: "border-border bg-surface-secondary hover:bg-surface-hover" },
    },
  },
});

const slots = pickerTv();

onMounted(async () => {
  logoAssets.value = await getLogoAssets();
});
</script>

<template>
  <div>
    <div class="flex items-center gap-2">
      <button type="button" @click="showPicker = true" :class="slots.previewBtn()" :aria-label="t('upload_logo')">
        <img v-if="modelValue" :src="modelValue" alt="" class="w-full h-full object-contain p-1" />
        <ImageIcon v-else :size="20" class="text-text-muted" />
      </button>
      <button
        v-if="modelValue"
        type="button"
        @click="clear"
        :aria-label="t('delete')"
        :class="button({ intent: 'ghost', size: 'icon', class: 'text-text-muted hover:text-danger hover:bg-danger/12' })"
      >
        <X :size="14" />
      </button>
    </div>

    <Modal :show="showPicker" :title="t('upload_logo')" @close="showPicker = false">
      <div :class="slots.tabBar()">
        <button
          type="button"
          @click="activeTab = 'gallery'"
          :class="pickerTv({ tabActive: activeTab === 'gallery' }).tab()"
        >
          <ImageIcon :size="14" />
          {{ t('gallery') }}
        </button>
        <button
          type="button"
          @click="activeTab = 'upload'"
          :class="pickerTv({ tabActive: activeTab === 'upload' }).tab()"
        >
          <Upload :size="14" />
          {{ t('upload_logo') }}
        </button>
        <button
          type="button"
          @click="activeTab = 'url'"
          :class="pickerTv({ tabActive: activeTab === 'url' }).tab()"
        >
          <LinkIcon :size="14" />
          {{ t('url') }}
        </button>
      </div>

      <div v-if="activeTab === 'gallery'">
        <div class="relative mb-3">
          <Search :size="14" class="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('search')"
            :class="slots.searchInput()"
          />
        </div>
        <div class="space-y-3">
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

      <div v-else-if="activeTab === 'upload'" class="py-2">
        <label :class="slots.uploadZone()">
          <Upload :size="32" class="text-text-muted mb-2" />
          <p class="text-sm text-text-secondary font-medium">{{ t('click_to_upload') }}</p>
          <p class="text-xs text-text-muted mt-1">{{ t('supported_formats') }}</p>
          <input type="file" accept="image/*" class="hidden" @change="handleUpload" />
        </label>
      </div>

      <form v-else class="space-y-3 py-2" @submit.prevent="saveUrlLogo">
        <label :class="ui.fieldLabel()">{{ t('logo_image_url') }}</label>
        <input
          v-model="logoUrl"
          type="url"
          placeholder="https://example.com/logo.png"
          :class="ui.field()"
        />
        <p class="text-xs text-text-muted">{{ t('logo_url_hint') }}</p>
        <button type="submit" :class="button({ intent: 'primary' })" :disabled="!logoUrl.trim()">
          {{ t('logo_use_this') }}
        </button>
      </form>
    </Modal>
  </div>
</template>
