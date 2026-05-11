import { ref } from "vue";

/**
 * Returns a chart-theme handle for `vue-echarts`.
 *
 * We intentionally always return `undefined` — ECharts' built-in `"dark"`
 * theme paints a solid `#100C2A` background which clashes with our card
 * surfaces in both light and dark mode. All charts in this app set their own
 * text/axis/tooltip colors in `option` using mid-gray rgba values that read
 * fine on top of either theme, so we want the canvas background to stay
 * transparent.
 *
 * Kept as a composable (rather than removed entirely) so call sites don't
 * have to change and we can swap in a custom registered theme later without
 * touching every widget.
 */
export function useChartTheme() {
  return ref<"dark" | undefined>(undefined);
}
