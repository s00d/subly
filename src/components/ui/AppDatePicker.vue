<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { useI18n } from "vue-i18n";
import {
  Calendar as CalendarIcon,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
} from "lucide-vue-next";
import { tv } from "@/lib/tv";

const props = defineProps<{
  modelValue: string;
  label?: string;
  disabled?: boolean;
  error?: string;
  min?: string;
  max?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const { t, locale } = useI18n();

const isOpen = ref(false);
const popoverRef = ref<HTMLElement | null>(null);
const triggerRef = ref<HTMLElement | null>(null);
const viewMode = ref<"days" | "months" | "years">("days");
const viewDate = ref(new Date());
const pos = ref({ top: 0, left: 0 });

watch(
  () => props.modelValue,
  (newVal) => {
    if (!newVal || isOpen.value) return;
    const parsed = new Date(newVal);
    if (!Number.isNaN(parsed.getTime())) {
      viewDate.value = parsed;
    }
  },
  { immediate: true },
);

const currentYear = computed(() => viewDate.value.getFullYear());
const currentMonth = computed(() => viewDate.value.getMonth());

const weekDays = computed(() => {
  const formatter = new Intl.DateTimeFormat(locale.value, { weekday: "short" });
  const monday = new Date(Date.UTC(2024, 0, 1));
  return Array.from({ length: 7 }).map((_, i) => {
    const d = new Date(monday);
    d.setUTCDate(monday.getUTCDate() + i);
    return formatter.format(d);
  });
});

const monthNames = computed(() => {
  const formatter = new Intl.DateTimeFormat(locale.value, { month: "short" });
  return Array.from({ length: 12 }).map((_, i) => formatter.format(new Date(2024, i, 1)));
});

const calendarDays = computed(() => {
  const year = currentYear.value;
  const month = currentMonth.value;
  const firstDayOfMonth = new Date(year, month, 1).getDay();
  const firstDayOffset = firstDayOfMonth === 0 ? 6 : firstDayOfMonth - 1;
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const daysInPrevMonth = new Date(year, month, 0).getDate();
  const days: Array<{ date: Date; isCurrentMonth: boolean }> = [];

  for (let i = firstDayOffset - 1; i >= 0; i--) {
    days.push({ date: new Date(year, month - 1, daysInPrevMonth - i), isCurrentMonth: false });
  }
  for (let i = 1; i <= daysInMonth; i++) {
    days.push({ date: new Date(year, month, i), isCurrentMonth: true });
  }
  while (days.length < 42) {
    const day = days.length - (firstDayOffset + daysInMonth) + 1;
    days.push({ date: new Date(year, month + 1, day), isCurrentMonth: false });
  }
  return days;
});

const yearsList = computed(() => {
  const base = currentYear.value;
  return Array.from({ length: 25 }).map((_, i) => base - 12 + i);
});

function formatIso(d: Date) {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function formatDisplayDate(dateStr: string) {
  if (!dateStr) return "";
  const d = new Date(dateStr);
  if (Number.isNaN(d.getTime())) return "";
  return new Intl.DateTimeFormat(locale.value, {
    day: "numeric",
    month: "long",
    year: "numeric",
  }).format(d);
}

function inRange(d: Date) {
  const iso = formatIso(d);
  if (props.min && iso < props.min) return false;
  if (props.max && iso > props.max) return false;
  return true;
}

const isToday = (d: Date) => formatIso(d) === formatIso(new Date());
const isSelected = (d: Date) => formatIso(d) === props.modelValue;

function toggleOpen() {
  if (props.disabled) return;
  isOpen.value = !isOpen.value;
  if (!isOpen.value) return;
  viewMode.value = "days";
  const source = props.modelValue ? new Date(props.modelValue) : new Date();
  viewDate.value = Number.isNaN(source.getTime()) ? new Date() : source;
  nextTick(updatePosition);
}

function selectDay(d: Date) {
  if (!inRange(d)) return;
  emit("update:modelValue", formatIso(d));
  isOpen.value = false;
}

function setQuickDate(offset: number) {
  const d = new Date();
  d.setDate(d.getDate() + offset);
  if (!inRange(d)) return;
  emit("update:modelValue", formatIso(d));
  isOpen.value = false;
}

function navigateMonth(dir: number) {
  viewDate.value = new Date(currentYear.value, currentMonth.value + dir, 1);
}

function navigateYear(dir: number) {
  viewDate.value = new Date(currentYear.value + dir, currentMonth.value, 1);
}

function selectMonth(monthIndex: number) {
  viewDate.value = new Date(currentYear.value, monthIndex, 1);
  viewMode.value = "days";
}

function selectYear(year: number) {
  viewDate.value = new Date(year, currentMonth.value, 1);
  viewMode.value = "months";
}

function updatePosition() {
  if (!triggerRef.value || !popoverRef.value) return;
  const rect = triggerRef.value.getBoundingClientRect();
  const popRect = popoverRef.value.getBoundingClientRect();
  const margin = 8;
  let top = rect.bottom + 4;
  let left = rect.left;

  if (top + popRect.height > window.innerHeight - margin) {
    top = rect.top - popRect.height - 4;
  }
  if (top < margin) {
    top = margin;
  }
  if (left + popRect.width > window.innerWidth - margin) {
    left = window.innerWidth - popRect.width - margin;
  }
  if (left < margin) {
    left = margin;
  }
  pos.value = { top, left };
}

function onClickOutside(e: MouseEvent) {
  const target = e.target as Node;
  if (
    isOpen.value &&
    popoverRef.value &&
    !popoverRef.value.contains(target) &&
    triggerRef.value &&
    !triggerRef.value.contains(target)
  ) {
    isOpen.value = false;
  }
}

onMounted(() => {
  document.addEventListener("mousedown", onClickOutside);
  window.addEventListener("scroll", updatePosition, true);
  window.addEventListener("resize", updatePosition);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", onClickOutside);
  window.removeEventListener("scroll", updatePosition, true);
  window.removeEventListener("resize", updatePosition);
});

const datePickerTv = tv({
  slots: {
    root: "relative w-full",
    labelEl: "block text-xs font-medium text-text-secondary mb-1.5",
    trigger: [
      "w-full flex items-center gap-2.5 px-3 py-2 rounded-lg border",
      "bg-surface text-sm text-left transition-shadow cursor-pointer",
      "disabled:opacity-50 disabled:cursor-not-allowed",
    ],
    popover: "fixed z-[200] w-72 bg-surface border border-border rounded-xl shadow-xl overflow-hidden flex flex-col",
    header: "flex items-center justify-between px-3 py-2 border-b border-border bg-surface-secondary",
    navBtn: "p-1.5 rounded hover:bg-border text-text-secondary transition-colors",
    viewToggleBtn: "text-sm font-semibold hover:bg-border px-2 py-1 rounded transition-colors text-text-primary",
    gridDays: "grid grid-cols-7 gap-1 p-3",
    gridMonthsYears: "grid grid-cols-3 gap-2 p-3",
    weekday: "text-center text-[10px] font-semibold text-text-muted uppercase mb-1",
    cellBtn: "h-8 rounded-lg text-sm flex items-center justify-center transition-colors",
    bigCellBtn: "py-3 rounded-lg text-sm font-medium transition-colors",
    quickActions: "flex items-center gap-2 p-3 border-t border-border bg-surface-secondary",
    quickBtn: "flex-1 py-1.5 text-xs font-medium rounded-md border border-border bg-surface hover:bg-surface-hover hover:text-primary transition-colors text-text-secondary",
    errorEl: "mt-1 text-xs text-red-500",
  },
  variants: {
    status: {
      error: { trigger: "border-red-500 ring-2 ring-red-500/20" },
      open: { trigger: "border-primary ring-2 ring-primary/20" },
      normal: { trigger: "border-border hover:border-text-muted" },
    },
    cellState: {
      selected: { cellBtn: "bg-primary text-white font-bold shadow-md", bigCellBtn: "bg-primary text-white font-bold shadow-md" },
      today: { cellBtn: "bg-primary-light text-primary font-bold", bigCellBtn: "bg-primary-light text-primary font-bold" },
      normal: { cellBtn: "text-text-primary hover:bg-surface-hover", bigCellBtn: "text-text-primary hover:bg-surface-hover" },
      muted: { cellBtn: "text-text-muted hover:bg-surface-hover opacity-50", bigCellBtn: "text-text-muted hover:bg-surface-hover opacity-50" },
      disabled: { cellBtn: "text-text-muted opacity-30 cursor-not-allowed", bigCellBtn: "text-text-muted opacity-30 cursor-not-allowed" },
    },
  },
});

const slots = computed(() =>
  datePickerTv({
    status: props.error ? "error" : isOpen.value ? "open" : "normal",
  }),
);

function getCellState(day: { date: Date; isCurrentMonth: boolean }) {
  if (!inRange(day.date)) return "disabled";
  if (isSelected(day.date)) return "selected";
  if (isToday(day.date)) return "today";
  if (!day.isCurrentMonth) return "muted";
  return "normal";
}
</script>

<template>
  <div :class="slots.root()">
    <label v-if="label" :class="slots.labelEl()">{{ label }}</label>

    <button ref="triggerRef" type="button" @click="toggleOpen" :disabled="disabled" :class="slots.trigger()">
      <CalendarIcon :size="16" class="text-text-muted shrink-0" />
      <span class="flex-1 truncate text-text-primary" :class="{ 'text-text-muted': !modelValue }">
        {{ modelValue ? formatDisplayDate(modelValue) : (t("select_date") || "Select date") }}
      </span>
    </button>

    <p v-if="error" :class="slots.errorEl()">{{ error }}</p>

    <Teleport to="body">
      <Transition
        enter-active-class="transition ease-out duration-150"
        enter-from-class="opacity-0 scale-95 -translate-y-2"
        enter-to-class="opacity-100 scale-100 translate-y-0"
        leave-active-class="transition ease-in duration-100"
        leave-from-class="opacity-100 scale-100"
        leave-to-class="opacity-0 scale-95"
      >
        <div
          v-if="isOpen"
          ref="popoverRef"
          :class="slots.popover()"
          :style="{ top: `${pos.top}px`, left: `${pos.left}px` }"
          @click.stop
        >
          <div :class="slots.header()">
            <div class="flex gap-1">
              <button type="button" @click="navigateYear(-1)" :class="slots.navBtn()" title="Prev year">
                <ChevronsLeft :size="16" />
              </button>
              <button type="button" @click="navigateMonth(-1)" :class="slots.navBtn()" title="Prev month">
                <ChevronLeft :size="16" />
              </button>
            </div>

            <button
              type="button"
              @click="viewMode = viewMode === 'days' ? 'months' : (viewMode === 'months' ? 'years' : 'days')"
              :class="slots.viewToggleBtn()"
            >
              {{ monthNames[currentMonth] }} {{ currentYear }}
            </button>

            <div class="flex gap-1">
              <button type="button" @click="navigateMonth(1)" :class="slots.navBtn()" title="Next month">
                <ChevronRight :size="16" />
              </button>
              <button type="button" @click="navigateYear(1)" :class="slots.navBtn()" title="Next year">
                <ChevronsRight :size="16" />
              </button>
            </div>
          </div>

          <div v-if="viewMode === 'days'" :class="slots.gridDays()">
            <div v-for="wd in weekDays" :key="wd" :class="slots.weekday()">{{ wd }}</div>

            <button
              v-for="(day, idx) in calendarDays"
              :key="idx"
              type="button"
              :disabled="!inRange(day.date)"
              @click="selectDay(day.date)"
              :class="[slots.cellBtn(), datePickerTv({ cellState: getCellState(day) }).cellBtn()]"
            >
              {{ day.date.getDate() }}
            </button>
          </div>

          <div v-else-if="viewMode === 'months'" :class="slots.gridMonthsYears()">
            <button
              v-for="(mName, idx) in monthNames"
              :key="idx"
              type="button"
              @click="selectMonth(idx)"
              :class="[slots.bigCellBtn(), datePickerTv({ cellState: currentMonth === idx ? 'today' : 'normal' }).bigCellBtn()]"
            >
              {{ mName }}
            </button>
          </div>

          <div v-else class="max-h-64 overflow-y-auto" :class="slots.gridMonthsYears()">
            <button
              v-for="year in yearsList"
              :key="year"
              type="button"
              @click="selectYear(year)"
              :class="[slots.bigCellBtn(), datePickerTv({ cellState: currentYear === year ? 'today' : 'normal' }).bigCellBtn()]"
            >
              {{ year }}
            </button>
          </div>

          <div v-if="viewMode === 'days'" :class="slots.quickActions()">
            <button type="button" @click="setQuickDate(-1)" :class="slots.quickBtn()">
              {{ t("yesterday") || "Yesterday" }}
            </button>
            <button type="button" @click="setQuickDate(0)" :class="slots.quickBtn()">
              {{ t("today") || "Today" }}
            </button>
            <button type="button" @click="setQuickDate(1)" :class="slots.quickBtn()">
              {{ t("tomorrow") || "Tomorrow" }}
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
