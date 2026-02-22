<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue";

const THUMB_W = 16;

const props = withDefaults(defineProps<{
  modelValue: number;
  min?: number;
  max?: number;
  step?: number;
}>(), {
  min: 0,
  max: 100,
  step: 0.1,
});

const emit = defineEmits<{
  "update:modelValue": [value: number];
}>();

const dragging = ref(false);
const dragMax = ref(props.max);
const trackRef = ref<HTMLElement | null>(null);
const trackWidth = ref(200);

const effectiveMax = computed(() => dragging.value ? dragMax.value : props.max);

const ratio = computed(() => {
  const range = effectiveMax.value - props.min;
  if (range <= 0) return 0;
  return Math.min(1, Math.max(0, (props.modelValue - props.min) / range));
});

const fillWidth = computed(() => {
  const usable = trackWidth.value - THUMB_W;
  if (usable <= 0) return THUMB_W / 2;
  return THUMB_W / 2 + ratio.value * usable;
});

watch(() => props.max, (v) => {
  if (!dragging.value) dragMax.value = v;
});

function measure() {
  if (trackRef.value) trackWidth.value = trackRef.value.offsetWidth;
}

let ro: ResizeObserver | null = null;
onMounted(() => {
  measure();
  if (trackRef.value) {
    ro = new ResizeObserver(measure);
    ro.observe(trackRef.value);
  }
});
onBeforeUnmount(() => ro?.disconnect());

function onStart() {
  dragging.value = true;
  dragMax.value = props.max;
}

function onEnd() {
  dragging.value = false;
}

function onInput(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  if (!isNaN(val)) emit("update:modelValue", val);
}
</script>

<template>
  <div ref="trackRef" class="slider-wrap">
    <div class="track">
      <div class="track-fill" :style="{ width: fillWidth + 'px' }" />
    </div>
    <input
      type="range"
      :min="props.min"
      :max="effectiveMax"
      :step="props.step"
      :value="props.modelValue"
      @input="onInput"
      @mousedown="onStart"
      @touchstart.passive="onStart"
      @mouseup="onEnd"
      @touchend.passive="onEnd"
      @pointercancel="onEnd"
      class="range-input"
    />
  </div>
</template>

<style scoped>
.slider-wrap {
  position: relative;
  height: 24px;
  display: flex;
  align-items: center;
}

.track {
  position: absolute;
  left: 0;
  right: 0;
  height: 4px;
  border-radius: 9999px;
  background: var(--color-border);
  overflow: hidden;
  pointer-events: none;
}

.track-fill {
  height: 100%;
  border-radius: 9999px;
  background: var(--color-primary);
  min-width: 0;
}

.range-input {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 24px;
  background: transparent;
  outline: none;
  cursor: pointer;
  margin: 0;
  position: relative;
  z-index: 1;
}

.range-input::-webkit-slider-runnable-track {
  height: 4px;
  background: transparent;
  border-radius: 9999px;
}

.range-input::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--color-primary);
  border: 2px solid white;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.25);
  margin-top: -6px;
  transition: transform 0.1s ease;
}
.range-input::-webkit-slider-thumb:hover {
  transform: scale(1.2);
}
.range-input:active::-webkit-slider-thumb {
  transform: scale(1.3);
}

.range-input::-moz-range-track {
  height: 4px;
  background: transparent;
  border-radius: 9999px;
}

.range-input::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--color-primary);
  border: 2px solid white;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.25);
  cursor: pointer;
}
</style>
