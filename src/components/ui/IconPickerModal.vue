<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { getLogoAssets, type LogoAsset } from "@/services/logoClient";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import { Search, Upload } from "@lucide/vue";
import { tv, ui, button } from "@/lib/tv";
import Modal from "@/components/ui/Modal.vue";

const props = defineProps<{
  show: boolean;
  modelValue: string;
  group?: "payment" | "service" | "all";
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
  close: [];
}>();

const { t } = useI18n();
const search = ref("");
const fileInput = ref<HTMLInputElement | null>(null);
const logoAssets = ref<LogoAsset[]>([]);

const filterGroup = computed(() => props.group || "all");

const filteredAssets = computed(() => {
  let items: LogoAsset[] = filterGroup.value === "all"
    ? logoAssets.value
    : logoAssets.value.filter((a) => a.group === filterGroup.value);

  if (search.value) {
    const q = search.value.toLowerCase();
    items = items.filter((a) => a.name.toLowerCase().includes(q));
  }
  return items;
});

interface EmojiGroup {
  label: string;
  items: { emoji: string; keywords: string }[];
}

const emojiGroups: EmojiGroup[] = [
  {
    label: "💰 Payment & Money",
    items: [
      { emoji: "💵", keywords: "cash money dollar bill" },
      { emoji: "💴", keywords: "yen money japan" },
      { emoji: "💶", keywords: "euro money europe" },
      { emoji: "💷", keywords: "pound money uk sterling" },
      { emoji: "💸", keywords: "money wings fly spending" },
      { emoji: "💳", keywords: "card credit debit payment" },
      { emoji: "💰", keywords: "money bag gold rich" },
      { emoji: "🪙", keywords: "coin gold money" },
      { emoji: "🏦", keywords: "bank building finance" },
      { emoji: "🧾", keywords: "receipt bill invoice" },
      { emoji: "💎", keywords: "gem diamond premium" },
      { emoji: "🏧", keywords: "atm cash withdraw" },
      { emoji: "💲", keywords: "dollar sign money price" },
      { emoji: "🤑", keywords: "money face rich" },
      { emoji: "📈", keywords: "chart up growth profit" },
      { emoji: "📉", keywords: "chart down loss" },
    ],
  },
  {
    label: "$ Currency Symbols",
    items: [
      { emoji: "$", keywords: "dollar usd currency" },
      { emoji: "€", keywords: "euro eur currency" },
      { emoji: "£", keywords: "pound gbp currency sterling" },
      { emoji: "¥", keywords: "yen jpy cny currency" },
      { emoji: "₽", keywords: "ruble rub currency russia" },
      { emoji: "₹", keywords: "rupee inr currency india" },
      { emoji: "₩", keywords: "won krw currency korea" },
      { emoji: "₿", keywords: "bitcoin btc crypto currency" },
      { emoji: "₺", keywords: "lira try currency turkey" },
      { emoji: "₴", keywords: "hryvnia uah currency ukraine" },
      { emoji: "₸", keywords: "tenge kzt currency kazakhstan" },
      { emoji: "₮", keywords: "tugrik mnt currency mongolia" },
      { emoji: "₫", keywords: "dong vnd currency vietnam" },
      { emoji: "₦", keywords: "naira ngn currency nigeria" },
      { emoji: "₱", keywords: "peso php currency philippines" },
      { emoji: "฿", keywords: "baht thb currency thailand" },
    ],
  },
  {
    label: "👤 Users & People",
    items: [
      { emoji: "👤", keywords: "user person profile" },
      { emoji: "👥", keywords: "users people group" },
      { emoji: "👨", keywords: "man male person" },
      { emoji: "👩", keywords: "woman female person" },
      { emoji: "👨‍💻", keywords: "developer coder programmer man" },
      { emoji: "👩‍💻", keywords: "developer coder programmer woman" },
      { emoji: "👨‍💼", keywords: "office worker business man" },
      { emoji: "👩‍💼", keywords: "office worker business woman" },
      { emoji: "👪", keywords: "family household group" },
      { emoji: "🧑‍🤝‍🧑", keywords: "people together couple" },
      { emoji: "🤝", keywords: "handshake deal agreement" },
      { emoji: "👋", keywords: "wave hello greeting" },
    ],
  },
  {
    label: "📱 Services & Tech",
    items: [
      { emoji: "📱", keywords: "phone mobile smartphone" },
      { emoji: "💻", keywords: "laptop computer" },
      { emoji: "🖥️", keywords: "desktop monitor screen" },
      { emoji: "⌨️", keywords: "keyboard typing" },
      { emoji: "🎮", keywords: "game controller gaming" },
      { emoji: "🎵", keywords: "music audio song" },
      { emoji: "🎬", keywords: "movie film cinema" },
      { emoji: "📺", keywords: "tv television streaming" },
      { emoji: "📡", keywords: "satellite internet connection" },
      { emoji: "☁️", keywords: "cloud storage hosting" },
      { emoji: "🔒", keywords: "lock security vpn" },
      { emoji: "🔑", keywords: "key password access" },
      { emoji: "📧", keywords: "email mail message" },
      { emoji: "🌐", keywords: "web internet globe" },
      { emoji: "📰", keywords: "news newspaper article" },
      { emoji: "📚", keywords: "books reading education" },
    ],
  },
  {
    label: "🏷️ Categories",
    items: [
      { emoji: "🏠", keywords: "home house rent mortgage" },
      { emoji: "🚗", keywords: "car auto transport vehicle" },
      { emoji: "🏥", keywords: "hospital health medical" },
      { emoji: "💊", keywords: "medicine pill health" },
      { emoji: "🏋️", keywords: "gym fitness workout sport" },
      { emoji: "🎓", keywords: "education school university" },
      { emoji: "✈️", keywords: "travel flight airplane" },
      { emoji: "🍽️", keywords: "food restaurant dining" },
      { emoji: "🛒", keywords: "shopping cart store" },
      { emoji: "👕", keywords: "clothing fashion clothes" },
      { emoji: "🐾", keywords: "pet animal" },
      { emoji: "🎁", keywords: "gift present subscription box" },
      { emoji: "⚡", keywords: "electric power utility" },
      { emoji: "💧", keywords: "water utility bill" },
      { emoji: "🔥", keywords: "fire hot gas heating" },
      { emoji: "📞", keywords: "phone call telecom" },
    ],
  },
];

const filteredEmojiGroups = computed(() => {
  if (!search.value) return emojiGroups;
  const q = search.value.toLowerCase();
  const result: EmojiGroup[] = [];
  for (const group of emojiGroups) {
    const filtered = group.items.filter(
      (item) => item.emoji.includes(q) || item.keywords.includes(q),
    );
    if (filtered.length > 0) {
      result.push({ label: group.label, items: filtered });
    }
  }
  return result;
});

function select(value: string) {
  emit("update:modelValue", value);
  emit("close");
}

function handleUpload(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => {
    emit("update:modelValue", reader.result as string);
    emit("close");
  };
  reader.readAsDataURL(file);
  input.value = "";
}

const pickerTv = tv({
  slots: {
    searchRow: "pb-3 flex gap-2",
    searchInput: ui.field({ class: "pl-8" }),
    currentInner: "mb-3 flex items-center gap-2 p-2 rounded-lg bg-surface-secondary border border-border",
    sectionLabel: "text-[10px] uppercase tracking-wider text-text-muted font-medium mb-2",
    grid: "grid grid-cols-8 gap-1.5",
    iconBtn: "w-full aspect-square max-w-10 rounded-lg border flex items-center justify-center text-lg transition-colors hover:bg-surface-hover",
    emptyText: "text-center py-8 text-sm text-text-muted",
  },
  variants: {
    selected: {
      true: { iconBtn: "border-primary bg-primary-light" },
      false: { iconBtn: "border-border" },
    },
  },
});

const slots = pickerTv();

onMounted(async () => {
  logoAssets.value = await getLogoAssets();
});
</script>

<template>
  <Modal :show="show" :title="t('choose_icon')" maxWidth="28rem" @close="emit('close')">
    <div :class="slots.searchRow()">
      <div class="relative flex-1">
        <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
        <input
          v-model="search"
          type="text"
          :placeholder="t('search')"
          :class="slots.searchInput()"
        />
      </div>
      <button type="button" @click="fileInput?.click()" :aria-label="t('upload')" :class="button({ size: 'lg', class: 'shrink-0' })">
        <Upload :size="14" />
        <span class="hidden sm:inline">{{ t('upload') }}</span>
      </button>
      <input ref="fileInput" type="file" accept="image/*" class="hidden" @change="handleUpload" />
    </div>

    <div v-if="modelValue" :class="slots.currentInner()">
      <IconDisplay :icon="modelValue" :size="24" />
      <span class="text-xs text-primary font-medium">{{ t('current') }}</span>
    </div>

    <div v-for="group in filteredEmojiGroups" :key="group.label" class="mb-3">
      <p :class="slots.sectionLabel()">{{ group.label }}</p>
      <div :class="slots.grid()">
        <button
          v-for="item in group.items"
          :key="item.emoji"
          type="button"
          @click="select(item.emoji)"
          :class="pickerTv({ selected: modelValue === item.emoji }).iconBtn()"
          :title="item.keywords"
        >
          {{ item.emoji }}
        </button>
      </div>
    </div>

    <div v-if="filteredAssets.length > 0">
      <p :class="slots.sectionLabel()">{{ t('icons') }}</p>
      <div :class="slots.grid()">
        <button
          v-for="asset in filteredAssets"
          :key="asset.path"
          type="button"
          @click="select(asset.path)"
          :class="pickerTv({ selected: modelValue === asset.path }).iconBtn()"
          :title="asset.name"
        >
          <img :src="asset.path" :alt="asset.name" class="w-6 h-6 object-contain" />
        </button>
      </div>
    </div>

    <div v-if="filteredAssets.length === 0 && filteredEmojiGroups.length === 0" :class="slots.emptyText()">
      {{ t('no_results') }}
    </div>
  </Modal>
</template>
