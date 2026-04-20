<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { logoAssets, type LogoAsset } from "@/services/logoAssets";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import { X, Search, Upload } from "lucide-vue-next";
import { tv } from "@/lib/tv";
import { useScrollLock } from "@/composables/useScrollLock";

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

const filterGroup = computed(() => props.group || "all");

const filteredAssets = computed(() => {
  let items: LogoAsset[] = filterGroup.value === "all"
    ? logoAssets
    : logoAssets.filter((a) => a.group === filterGroup.value);

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

function onBackdropClick() {
  emit("close");
}

const pickerTv = tv({
  slots: {
    overlay: "fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4",
    backdrop: "absolute inset-0 bg-black/50",
    panel: [
      "relative bg-[var(--color-surface)] w-full max-w-md overflow-hidden",
      "rounded-t-2xl sm:rounded-xl shadow-2xl max-h-[85vh] sm:max-h-none",
    ],
    header: "flex items-center justify-between px-4 sm:px-5 py-3 sm:py-4 border-b border-[var(--color-border)]",
    headerTitle: "text-sm sm:text-base font-semibold text-[var(--color-text-primary)]",
    closeBtn: "p-1 rounded-lg hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]",
    searchRow: "px-4 sm:px-5 pt-3 sm:pt-4 pb-2 flex gap-2",
    searchInput: [
      "w-full pl-8 pr-3 py-2 rounded-lg border border-[var(--color-border)]",
      "bg-[var(--color-surface)] text-sm text-[var(--color-text-primary)]",
      "placeholder-[var(--color-text-muted)]",
      "focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] transition-shadow",
    ],
    uploadBtn: [
      "flex items-center gap-1.5 px-3 py-2 rounded-lg border border-[var(--color-border)]",
      "text-xs sm:text-sm text-[var(--color-text-secondary)]",
      "hover:border-[var(--color-primary)] hover:text-[var(--color-primary)] transition-colors shrink-0",
    ],
    currentWrap: "px-5 pb-2",
    currentInner: "flex items-center gap-2 p-2 rounded-lg bg-[var(--color-primary-light)] border border-[var(--color-primary)]/20",
    gridWrap: "px-4 sm:px-5 pb-4 max-h-[50vh] overflow-y-auto",
    sectionLabel: "text-[10px] uppercase tracking-wider text-[var(--color-text-muted)] font-medium mb-2",
    grid: "grid grid-cols-8 gap-1.5",
    iconBtn: "w-10 h-10 rounded-lg border flex items-center justify-center text-lg hover:bg-[var(--color-surface-hover)] transition-colors",
    emptyText: "text-center py-8 text-sm text-[var(--color-text-muted)]",
  },
  variants: {
    selected: {
      true: { iconBtn: "border-[var(--color-primary)] bg-[var(--color-primary-light)]" },
      false: { iconBtn: "border-[var(--color-border)]" },
    },
  },
});

const slots = pickerTv();
useScrollLock(computed(() => props.show));
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="show" :class="slots.overlay()">
        <div :class="slots.backdrop()" @click="onBackdropClick" />
        <div :class="slots.panel()">
          <div :class="slots.header()">
            <h3 :class="slots.headerTitle()">{{ t('choose_icon') }}</h3>
            <button @click="emit('close')" :class="slots.closeBtn()">
              <X :size="20" />
            </button>
          </div>

          <div :class="slots.searchRow()">
            <div class="relative flex-1">
              <Search :size="14" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" />
              <input
                v-model="search"
                type="text"
                :placeholder="t('search')"
                :class="slots.searchInput()"
              />
            </div>
            <button @click="fileInput?.click()" :class="slots.uploadBtn()">
              <Upload :size="14" />
              <span class="hidden sm:inline">{{ t('upload') }}</span>
              <span class="sm:hidden">+</span>
            </button>
            <input ref="fileInput" type="file" accept="image/*" class="hidden" @change="handleUpload" />
          </div>

          <div v-if="modelValue" :class="slots.currentWrap()">
            <div :class="slots.currentInner()">
              <IconDisplay :icon="modelValue" :size="24" />
              <span class="text-xs text-[var(--color-primary)] font-medium">{{ t('current') }}</span>
            </div>
          </div>

          <div :class="slots.gridWrap()">
            <div v-for="group in filteredEmojiGroups" :key="group.label" class="mb-3">
              <p :class="slots.sectionLabel()">{{ group.label }}</p>
              <div :class="slots.grid()">
                <button
                  v-for="item in group.items"
                  :key="item.emoji"
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
                  @click="select(asset.path)"
                  :class="pickerTv({ selected: modelValue === asset.path }).iconBtn()"
                  :title="asset.name"
                >
                  <img :src="asset.path" class="w-6 h-6 object-contain" />
                </button>
              </div>
            </div>

            <div v-if="filteredAssets.length === 0 && filteredEmojiGroups.length === 0" :class="slots.emptyText()">
              {{ t('no_results') }}
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
