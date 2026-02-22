<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(defineProps<{
  data: number[];
  width?: number;
  height?: number;
  color?: string;
}>(), {
  width: 60,
  height: 20,
  color: "var(--color-primary)",
});

const path = computed(() => {
  const d = props.data;
  if (d.length === 0) return "";
  if (d.length === 1) {
    const cx = props.width / 2;
    const cy = props.height / 2;
    return `M${cx - 0.5},${cy} L${cx + 0.5},${cy}`;
  }
  const min = Math.min(...d);
  const max = Math.max(...d);
  const range = max - min || 1;
  const pad = 1;
  const w = props.width - pad * 2;
  const h = props.height - pad * 2;
  const stepX = w / (d.length - 1);
  return d
    .map((v, i) => {
      const x = pad + i * stepX;
      const y = pad + h - ((v - min) / range) * h;
      return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
});

const trend = computed(() => {
  const d = props.data;
  if (d.length < 2) return 0;
  return d[d.length - 1] - d[0];
});
</script>

<template>
  <svg
    v-if="data.length >= 1"
    :width="width"
    :height="height"
    :viewBox="`0 0 ${width} ${height}`"
    class="inline-block"
  >
    <path
      :d="path"
      fill="none"
      :stroke="trend >= 0 ? 'var(--color-sparkline-up, #22c55e)' : 'var(--color-sparkline-down, #ef4444)'"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
  <span v-else class="text-[10px] text-[var(--color-text-muted)] opacity-40">—</span>
</template>
