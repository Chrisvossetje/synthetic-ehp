import type { Chart } from "../chart/chart";
import { viewSettings } from "../model/settings";

function clipboardLabel(chart: Chart, name: string): string {
    if (viewSettings.tagNotation === "curtis") return chart.generator_label(name);
    const power = chart.generator_tau_power(name);
    return (power ? `\\tau^{${power}} ` : "") + chart.generator_latex_tag(name);
}

function copyText(text: string): void {
    navigator.clipboard?.writeText(text).catch(err => {
        console.error("Failed to copy to clipboard:", err);
    });
}

export function copyGenerator(chart: Chart, name: string): void {
    copyText(clipboardLabel(chart, name));
}

export function copyDifferential(chart: Chart, from: string, to: string, tauExponent: number): void {
    const source = clipboardLabel(chart, from);
    const target = clipboardLabel(chart, to);
    const tau = tauExponent === 1 ? "\\tau" : `\\tau^{${tauExponent}}`;
    const coefficient = tauExponent === 0 ? "" : `${tau} \\cdot `;
    copyText(`${source} \\to ${coefficient}${target}`);
}
