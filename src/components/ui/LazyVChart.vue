<script setup lang="ts">
/**
 * Drop-in replacement for `vue-echarts` <VChart>. Defers mounting of the real
 * chart until the wrapper element has a non-zero size, so that ECharts doesn't
 * crash with "Can't get DOM width or height. Please check dom.clientWidth and
 * dom.clientHeight. They should not be 0."
 *
 * Layout contract: any `class`/`style` you'd put on <VChart> is applied to the
 * wrapper element instead — including size, max-width, min-height, margins,
 * etc. The real <VChart> always fills the wrapper at 100%×100%. This is what
 * call sites want: e.g. `class="max-w-[220px] mx-auto h-44"` sizes the chart
 * area itself, not a nested div inside it.
 *
 * Other props/listeners (`option`, `theme`, `autoresize`, …) are forwarded to
 * <VChart> transparently.
 */
import { computed, onBeforeUnmount, onMounted, ref, useAttrs } from "vue";
import VChart from "vue-echarts";

defineOptions({ inheritAttrs: false });

const attrs = useAttrs();

/** class/style live on the wrapper (size, spacing, position). */
const wrapperClass = computed(() => attrs.class as unknown);
const wrapperStyle = computed(() => attrs.style as unknown);

/** Everything else (option, theme, autoresize, listeners, refs) goes to <VChart>. */
const chartAttrs = computed(() => {
  const out: Record<string, unknown> = {};
  for (const key of Object.keys(attrs)) {
    if (key === "class" || key === "style") continue;
    out[key] = (attrs as Record<string, unknown>)[key];
  }
  return out;
});

const wrapper = ref<HTMLElement | null>(null);
const ready = ref(false);
let observer: ResizeObserver | null = null;
let rafId = 0;

function check(el: HTMLElement): boolean {
  if (el.clientWidth > 0 && el.clientHeight > 0) {
    ready.value = true;
    observer?.disconnect();
    observer = null;
    if (rafId) cancelAnimationFrame(rafId);
    return true;
  }
  return false;
}

onMounted(() => {
  const el = wrapper.value;
  if (!el) return;
  if (check(el)) return;

  if (typeof ResizeObserver !== "undefined") {
    observer = new ResizeObserver(() => {
      const node = wrapper.value;
      if (node) check(node);
    });
    observer.observe(el);
  }

  const tick = () => {
    if (ready.value) return;
    const node = wrapper.value;
    if (node && check(node)) return;
    rafId = requestAnimationFrame(tick);
  };
  rafId = requestAnimationFrame(tick);
});

onBeforeUnmount(() => {
  observer?.disconnect();
  observer = null;
  if (rafId) cancelAnimationFrame(rafId);
});
</script>

<template>
  <div
    ref="wrapper"
    :class="wrapperClass as never"
    :style="wrapperStyle as never"
  >
    <VChart
      v-if="ready"
      v-bind="chartAttrs"
      style="width: 100%; height: 100%"
    >
      <template v-for="(_, name) in $slots" #[name]="slotData">
        <slot :name="name" v-bind="slotData ?? {}" />
      </template>
    </VChart>
  </div>
</template>
