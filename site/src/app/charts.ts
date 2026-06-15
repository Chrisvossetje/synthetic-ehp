import { Chart } from "../chart/chart";
import { ChartMode } from "../chart/chartMode";

/**
 * The two chart instances, created once and shared across the app. The EHP
 * chart shows class names and Adams filtrations; the ASS chart is a plain dot
 * plot, so labels are switched off for it.
 */
export const ehpChart = new Chart("svgchart-ehp", ChartMode.EHP);
export const assChart = new Chart("svgchart-ass", ChartMode.ASS);
ehpChart.set_label_display(true, true);
assChart.set_label_display(false, false);
