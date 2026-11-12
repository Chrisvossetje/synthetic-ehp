import { Chart } from "../chart/chart";
import { showInfoPanel } from "../chart/infoPanel";
import type { DisplayMultiplication, Generators } from "../types";
import { formatMultiplicationResult, resolveDisplayMultiplications, DisplayMultiplicationInfo } from "../model/multiplicationDisplay";
import { getMultiplicationTables } from "../model/multiplicationSource";
import { MultiplicationOperator } from "../model/multiplicationTable";
import { Category, viewSettings } from "../model/settings";
import { displayMultiplicationTag, displayName } from "../model/tagNames";

const infoByChart = new WeakMap<Chart, DisplayMultiplicationInfo[]>();

export function updateMultiplicationOverlay(
    chart: Chart,
    generators: readonly Generators[],
    visibleNames: ReadonlySet<string>,
    mode: "ehp" | "ass",
    nodeNames?: ReadonlyMap<string, string>,
    survivingNames: ReadonlySet<string> = visibleNames
) {
    const tables = getMultiplicationTables();
    const enabled = mode === "ass" || viewSettings.category !== Category.Geometric;
    chart.multiplicationCallback = (relation) => handleMultiplicationClick(chart, relation);
    if (!tables) {
        infoByChart.set(chart, []);
        chart.set_display_multiplications([]);
        return;
    }
    // Popups always know all products; the keyboard setting only filters lines.
    const operators: MultiplicationOperator[] = [0, 1, 2];
    // The sphere control stores sphere - 1 in the truncation state.
    const sphere = viewSettings.truncation == null ? undefined : viewSettings.truncation + 1;
    const nodeRows = new Map(chart.generators.map((node) => [node.name, node.y]));
    const rows = new Map(generators.map((gen) => {
        const nodeName = nodeNames?.get(gen.name) ?? gen.name;
        const row = nodeRows.get(nodeName) ?? gen.y;
        return [gen.name, row] as [string, number];
    }));
    const resolved = resolveDisplayMultiplications(tables, generators, visibleNames, operators, sphere, mode,
        { survivingNames, rows,
            tauMults: viewSettings.category === Category.Synthetic ? chart.tau_mults : [] });
    infoByChart.set(chart, resolved.info);
    const visibleLines = resolved.lines.filter((line) => enabled
        && (mode === "ass" || line.operator < viewSettings.multiplicationCounts.ehp));
    chart.set_display_multiplications(visibleLines.map((line) => ({
        ...line,
        from: nodeNames?.get(line.from) ?? line.from,
        to: nodeNames?.get(line.to) ?? line.to,
        displayTargets: line.targets.map(displayMultiplicationTag),
    })));
}

export function multiplicationInfoLines(chart: Chart, name: string): string[] {
    return (infoByChart.get(chart) ?? []).filter((entry) => entry.source === name
        && entry.result?.kind === "sum" && entry.result.targets.length > 0).map((entry) =>
        `h_${entry.operator}${entry.result ? ` (sphere ${entry.sphere})` : ""}: ${formatMultiplicationResult(entry.result)}`
        + (entry.inheritedFrom ? `\n  Inherited from ${displayMultiplicationTag(entry.inheritedFrom.tag)} at sphere ${entry.inheritedFrom.sphere}` : "")
        + (entry.tauExtensions ? `\n  Via τ extensions: ${entry.tauExtensions.map((relation) =>
            `${displayName(relation.from)} → ${displayName(relation.to)}`).join(", ")}` : "")
        + (entry.unresolved.length ? `\n  Undrawn terms: ${entry.unresolved.map(displayMultiplicationTag).join(", ")}` : "")
    );
}

function handleMultiplicationClick(chart: Chart, relation: DisplayMultiplication) {
    showInfoPanel(`h_${relation.operator} multiplication`, [
        `From: ${chart.generator_label(relation.from)}`,
        `Line to: ${chart.generator_label(relation.to)}`,
        `Sphere: ${relation.sphere}`,
        `Product: ${(relation.displayTargets ?? relation.targets.map(displayMultiplicationTag)).join(" + ")}`,
        ...(relation.inheritedFrom ? [`Inherited from ${displayMultiplicationTag(relation.inheritedFrom.tag)} at sphere ${relation.inheritedFrom.sphere}`] : []),
        ...(relation.tauExtensions ? [`Via τ extensions: ${relation.tauExtensions.map((extension) =>
            `${displayName(extension.from)} → ${displayName(extension.to)}`).join(", ")}`] : []),
        ...(relation.targets.length > 1 ? ["Each line is a term of this sum."] : []),
    ], ["Table relation displayed on the current page."]);
}
