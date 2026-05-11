/**
 * Treeshaken ECharts — подключаем только то, что нужно для дашборда.
 *
 * `LegacyGridContainLabel` нужен для echarts 6+: без него старый
 * `grid.containLabel: true` молча игнорируется и печатает варнинг
 * "Specified `grid.containLabel` but no `use(LegacyGridContainLabel)`".
 * Альтернатива — переписывать опции на `grid.outerBounds`, но проще
 * подключить feature — она копеечная по размеру.
 */
import { use } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import { BarChart, LineChart, PieChart } from "echarts/charts";
import {
  DatasetComponent,
  GridComponent,
  LegendComponent,
  TooltipComponent,
} from "echarts/components";
import { LegacyGridContainLabel } from "echarts/features";

use([
  CanvasRenderer,
  BarChart,
  LineChart,
  PieChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  DatasetComponent,
  LegacyGridContainLabel,
]);
