import { Chart } from "./chart";
import { ChartMode } from "./chartMode";

export const ehpChart = new Chart('svgchart-ehp', ChartMode.EHP);
export const assChart = new Chart('svgchart-ass', ChartMode.ASS);
ehpChart.set_label_display(true, true);
assChart.set_label_display(false, false);
