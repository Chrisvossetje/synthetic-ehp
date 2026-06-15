import { ehpChart } from "./charts";
import { Differential, Kind, SyntheticEHP } from "../types";
import { Category, getSelectedGenerator, setSelectedGenerator, shouldIncludeKind, viewSettings } from "../model/settings";
import { ensureStableDataLoading, find, getActiveData, isUsingStableData, setUseStableData } from "../model/dataSource";
import { generated_by_name, generates, getSphereLifecycleInfo } from "../model/names";
import { computePage, survivesFilteredGenerator, TorsionFiltration } from "../model/spectralSequence";
import { buildGeneratorInfoLines, showInfoPanel } from "../chart/infoPanel";

/**
 * EHP chart controller.
 *
 * Owns everything specific to the EHP chart: the click handlers (which open the
 * info panel and drive selection highlighting), filling the chart with the full
 * dataset once, and re-deriving what is visible whenever the view settings
 * change. The actual page maths lives in `model/spectralSequence.ts`; this file
 * only decides what to *show* given the result.
 */

// Differentials as computed at E∞ for the current view, keyed `from->to`.
// Used by the line-click handler and the screenshot exporter.
let computedDiffsByKey: Map<string, Differential> = new Map();
// The page-filtered [torsion, filtration] of every class currently drawn.
let currentFilteredGenerators: Record<string, TorsionFiltration> = {};

function cacheComputedDiffs(diffs: Differential[]) {
    computedDiffsByKey.clear();
    diffs.forEach((d) => {
        computedDiffsByKey.set(`${d.from}->${d.to}`, d);
    });
}

function getAllTauMults(data: SyntheticEHP) {
    return [...data.internal_tau_mults, ...data.external_tau_mults];
}

/** A τ-multiplication is only drawn when both endpoints survive on this page. */
function shouldDisplayTauMult(
    kind: Kind,
    gens: Record<string, TorsionFiltration>,
    from: string,
    to: string
): boolean {
    const fromEntry = gens[from];
    const toEntry = gens[to];
    if (!fromEntry || !toEntry) {
        return false;
    }
    if (kind === "Real") {
        return survivesFilteredGenerator(fromEntry) && survivesFilteredGenerator(toEntry);
    }
    return true;
}

export function getComputedDiff(from: string, to: string): Differential | undefined {
    return computedDiffsByKey.get(`${from}->${to}`);
}

/**
 * The τ-coefficient of a differential as displayed: the gap in Adams filtration
 * between its endpoints. Returns undefined if either endpoint is not drawn.
 */
export function getComputedDiffCoeff(from: string, to: string): number | undefined {
    const fromEntry = currentFilteredGenerators[from];
    const toEntry = currentFilteredGenerators[to];
    if (!fromEntry || !toEntry) {
        return undefined;
    }
    return toEntry[1] - fromEntry[1] - 1;
}

/**
 * The page-filtered [torsion, filtration] for a generator as currently drawn.
 * Returns undefined if the generator is not alive on the current page. Used by
 * the screenshot export so colours/AF match the chart.
 */
export function getDisplayedGenerator(name: string): TorsionFiltration | undefined {
    return currentFilteredGenerators[name];
}

export function handleDotClick(dot: string) {
    console.log('Dot clicked:', dot);
    const gen = find(dot);
    console.log(gen);

    if (!gen) return;

    setSelectedGenerator(dot);
    applyEhpSelectionHighlight();

    // Copy generator name to clipboard
    navigator.clipboard.writeText(gen.name).then(() => {
        console.log('Copied to clipboard:', gen.name);
    }).catch(err => {
        console.error('Failed to copy to clipboard:', err);
    });

    const sphereInfo = getSphereLifecycleInfo(gen);
    const lines = buildGeneratorInfoLines(gen, sphereInfo, {
        xLabel: "stem",
        yLabel: "y",
        moduleLabel: "Module",
    });

    showInfoPanel(`Generator: ${gen.name}`, lines);
}

/**
 * Highlight the selected class (orange), the class that generates it (cyan), and
 * the family it generates (green). No-op if nothing is selected / on screen.
 */
export function applyEhpSelectionHighlight() {
    ehpChart.clear_selection_highlights();

    const selected = getSelectedGenerator();
    if (!selected) return;
    const gen = find(selected);
    if (!gen) return;

    if (ehpChart.name_to_location.has(selected)) {
        ehpChart.add_selection_highlight(selected, "#ff6a00", 2.2, 0.18, 0.55);
    }

    const genName = generated_by_name(gen);
    const gensList = generates(gen);
    if (ehpChart.name_to_location.has(genName)) {
        ehpChart.add_selection_highlight(genName, "#00bcd4", 2.0, 0.14, 0.42);
    }
    gensList.forEach((g) => {
        if (ehpChart.name_to_location.has(g.name)) {
            ehpChart.add_selection_highlight(g.name, "#66bb00", 1.9, 0.12, 0.35);
        }
    });
}

export function handleLineClick(from: string, to: string) {
    console.log('Line clicked:', from, '->', to);
    const activeData = getActiveData();
    if (!activeData) return;
    const rawDiff = activeData.differentials.find(d => d.from === from && d.to === to);
    const computedDiff = getComputedDiff(from, to);
    console.log(rawDiff ?? computedDiff);

    if (!rawDiff && !computedDiff) return;

    const coeff = getComputedDiffCoeff(from, to) ?? rawDiff?.coeff ?? 0;
    const page = rawDiff?.d ?? computedDiff?.d ?? 0;

    const lines = [
        `From: ${rawDiff?.from ?? from}`,
        `To: ${rawDiff?.to ?? to}`,
        `Kind: ${rawDiff?.kind}`,
        `Page: E${page}`,
        `Coefficient: ${coeff === 0 ? '1' : 'τ^' + coeff}`,
    ];

    const extraLines: string[] = [];
    if (rawDiff && "proof" in rawDiff) {
        extraLines.push(`Proof: ${rawDiff.proof ?? ""}`);
    }

    showInfoPanel("Differential", lines, extraLines);
}

export function handleTauMultClick(from: string, to: string) {
    const activeData = getActiveData();
    if (!activeData) return;

    const internalTauMult = activeData.internal_tau_mults.find((t) => t.from === from && t.to === to);
    const externalTauMult = activeData.external_tau_mults.find((t) => t.from === from && t.to === to);
    const tauMult = internalTauMult ?? externalTauMult;
    if (!tauMult) return;

    const lines = [
        `From: ${tauMult.from}`,
        `To: ${tauMult.to}`,
        `Kind: ${tauMult.kind}`,
    ];
    if (internalTauMult) {
        lines.push(`Page: E${internalTauMult.page}`);
        lines.push("Type: Internal");
    } else {
        lines.push("Type: External");
    }

    const extraLines: string[] = [];
    if ("proof" in tauMult) {
        extraLines.push(`Proof: ${tauMult.proof ?? ""}`);
    }

    showInfoPanel("τ Multiplication", lines, extraLines);
}

/** Load the full dataset into the chart once and wire up click handlers. */
export function fill_ehp_chart() {
    const activeData = getActiveData();
    if (!activeData) {
        return;
    }

    ehpChart.dotCallback = handleDotClick;
    ehpChart.lineCallback = handleLineClick;
    ehpChart.tauMultCallback = handleTauMultClick;

    ehpChart.set_all_generators(activeData.generators);
    ehpChart.set_all_differentials(activeData.differentials);
    ehpChart.set_all_multiplications(activeData.multiplications);
    ehpChart.set_all_tau_mults(getAllTauMults(activeData));

    ehpChart.init();
}

/** Flip between `data` and `data_stable`, then rebuild the EHP chart. */
export async function switchDataSource() {
    const nextUseStableData = !isUsingStableData();
    if (nextUseStableData) {
        await ensureStableDataLoading();
    }
    setUseStableData(nextUseStableData);

    ehpChart.clear();
    fill_ehp_chart();
    update_ehp_chart();
}

/**
 * Re-derive what the EHP chart shows from the current view settings.
 *
 * Strategy: hide everything, then compute the live classes for the current page
 * (and, separately, for E∞ to decide which dots are "permanent") and turn the
 * relevant dots / differentials / multiplications back on.
 */
export function update_ehp_chart() {
    const activeData = getActiveData();
    if (!activeData) {
        return;
    }

    // Hide all generators, differentials, multiplications and τ-mults first.
    activeData.generators.forEach((g) => {
        ehpChart.display_dot(g.name, false, false, null, g.af);
    });
    activeData.differentials.forEach((d) => {
        ehpChart.display_diff(d.from, d.to, false);
    });
    activeData.multiplications.forEach((m) => {
        ehpChart.display_mult(m.from, m.to, false);
    });
    getAllTauMults(activeData).forEach((t) => {
        ehpChart.display_tau_mult(t.from, t.to, false);
    });

    // Classes alive on the current page...
    const gens = computePage(activeData, {
        category: viewSettings.category,
        truncation: viewSettings.truncation,
        bottomTruncation: viewSettings.bottomTruncation,
        page: viewSettings.page,
    }).generators;
    currentFilteredGenerators = gens;

    // ...and the E∞ view, which tells us which classes are permanent and gives
    // us the resolved differentials to remember for click/screenshot lookups.
    const permView = computePage(activeData, {
        category: viewSettings.category,
        truncation: viewSettings.truncation,
        bottomTruncation: viewSettings.bottomTruncation,
        page: 1000,
    });
    const perm_classes = permView.generators;
    cacheComputedDiffs(permView.differentials);

    const real_diffs = activeData.differentials.filter((d) => {
        if (!shouldIncludeKind(d.kind)) {
            return false;
        }
        if (!gens[d.from] || gens[d.from][0] === 0 || !gens[d.to] || gens[d.to][0] === 0) {
            return false;
        }
        if (!viewSettings.allDiffs && d.d != viewSettings.page) {
            return false;
        }
        if (d.d && d.d < viewSettings.page) {
            return false;
        }
        return true;
    });

    // Draw the live dots; a filled dot is a permanent cycle (survives to E∞).
    Object.entries(gens).forEach(([name, [torsion, filtration]]) => {
        if (torsion == undefined || torsion > 0) {
            const permanentEntry = perm_classes[name];
            const perm = permanentEntry != undefined && (permanentEntry[0] == undefined || permanentEntry[0] > 0);
            ehpChart.display_dot(name, true, perm, torsion ?? null, filtration);
        }
    });

    real_diffs.forEach((d) => {
        let torsion = getComputedDiffCoeff(d.from, d.to);
        if (viewSettings.category != Category.Synthetic) {
            torsion = 0;
        }
        if (viewSettings.category === Category.Algebraic && d.kind !== "Algebraic") {
            return;
        }
        ehpChart.display_diff(d.from, d.to, true, torsion);
    });

    // Multiplications: only when both endpoints are alive.
    activeData.multiplications.forEach((m) => {
        if (survivesFilteredGenerator(gens[m.from]) && survivesFilteredGenerator(gens[m.to])) {
            ehpChart.display_mult(m.from, m.to, true);
        }
    });

    // τ-multiplications: synthetic category only.
    if (viewSettings.category == Category.Synthetic) {
        activeData.internal_tau_mults.forEach((t) => {
            if (!shouldIncludeKind(t.kind)) return;
            if (!viewSettings.allDiffs && t.page !== viewSettings.page) return;
            if (shouldDisplayTauMult(t.kind, gens, t.from, t.to)) {
                ehpChart.display_tau_mult(t.from, t.to, true);
            }
        });

        // External τ-mults resolve at E∞, so only show them with All Diffs or E∞.
        if (viewSettings.allDiffs || viewSettings.page > 999) {
            activeData.external_tau_mults.forEach((t) => {
                if (!shouldIncludeKind(t.kind)) return;
                if (shouldDisplayTauMult(t.kind, gens, t.from, t.to)) {
                    ehpChart.display_tau_mult(t.from, t.to, true);
                }
            });
        }
    }

    applyEhpSelectionHighlight();
}
