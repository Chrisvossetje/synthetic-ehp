import { ehpChart } from "./charts";
import { Differential, Kind, SyntheticEHP } from "../types";
import { Category, getSelectedGenerator, setSelectedGenerator, shouldIncludeKind, viewSettings } from "../model/settings";
import { find, getActiveData } from "../model/dataSource";
import { generated_by_name, generates, getSphereLifecycleInfo } from "../model/names";
import { computePage, survivesFilteredGenerator, TorsionFiltration } from "../model/spectralSequence";
import { buildGeneratorInfoLines, showInfoPanel } from "../chart/infoPanel";
import { multiplicationInfoLines, updateMultiplicationOverlay } from "./multiplications";
import { displayName, displayText } from "../model/tagNames";

/**
 * EHP chart controller.
 *
 * Owns everything specific to the EHP chart: the click handlers (which open the
 * info panel and drive selection highlighting), filling the chart with the full
 * dataset once, and re-deriving what is visible whenever the view settings
 * change. The actual page maths lives in `model/spectralSequence.ts`; this file
 * only decides what to *show* given the result.
 */

// Resolved differentials for the line-click handler, keyed `from->to`.
const computedDiffsByKey = new Map<string, Differential>();
let differentialCoefficients: ReadonlyMap<Differential, number> = new Map();
// The page-filtered [torsion, filtration] of every class currently drawn.
let currentFilteredGenerators: Record<string, TorsionFiltration> = {};
let chartData: SyntheticEHP | null = null;

function cacheComputedDiffs(diffs: Differential[]) {
    computedDiffsByKey.clear();
    diffs.forEach((d) => {
        computedDiffsByKey.set(`${d.from}->${d.to}`, d);
    });
}

function getAllTauMults(data: SyntheticEHP) {
    return [...data.internal_tau_mults, ...data.external_tau_mults];
}

/** Keep the earliest eligible hit per target, even when showing future pages. */
function geometricDifferentials(diffs: Differential[], page: number, allDiffs: boolean): Differential[] {
    const incoming = new Map<string, { diff: Differential; page: number }>();
    for (const diff of diffs) {
        let diffPage = diff.d;
        if (diffPage === undefined) {
            const from = find(diff.from);
            const to = find(diff.to);
            if (!from || !to) continue;
            diffPage = from.y - to.y;
        }
        if (diffPage < page || (!allDiffs && diffPage !== page)) continue;

        const current = incoming.get(diff.to);
        if (!current || diffPage < current.page ||
            (diffPage === current.page && diff.kind === "Real" && current.diff.kind !== "Real")) {
            incoming.set(diff.to, { diff, page: diffPage });
        }
    }
    return Array.from(incoming.values(), ({ diff }) => diff);
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
 * The [torsion, filtration] computed for a generator on the selected page.
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
    const label = ehpChart.generator_label(dot);
    navigator.clipboard.writeText(label).then(() => {
        console.log('Copied to clipboard:', label);
    }).catch(err => {
        console.error('Failed to copy to clipboard:', err);
    });

    const sphereInfo = getSphereLifecycleInfo(gen);
    const lines = buildGeneratorInfoLines(gen, sphereInfo, {
        xLabel: "stem",
        yLabel: "y",
        moduleLabel: "Module",
    }, displayName, getDisplayedGenerator(dot)?.[1]);

    showInfoPanel("Generator", lines, multiplicationInfoLines(ehpChart, gen.name), label);
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

    const coeff = rawDiff ? differentialCoefficients.get(rawDiff) ?? 0 : 0;
    const page = rawDiff?.d ?? computedDiff?.d ?? 0;

    const lines = [
        `From: ${ehpChart.generator_label(from)}`,
        `To: ${ehpChart.generator_label(to)}`,
        `Kind: ${rawDiff?.kind}`,
        `Page: E${page}`,
        `Coefficient: ${coeff === 0 ? '1' : 'τ^' + coeff}`,
    ];

    const extraLines: string[] = [];
    if (rawDiff && "proof" in rawDiff) {
        extraLines.push(`Proof: ${displayText(rawDiff.proof ?? "")}`);
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
        `From: ${ehpChart.generator_label(tauMult.from)}`,
        `To: ${ehpChart.generator_label(tauMult.to)}`,
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
        extraLines.push(`Proof: ${displayText(tauMult.proof ?? "")}`);
    }

    showInfoPanel("τ Multiplication", lines, extraLines);
}

/** Load the dataset into the chart and wire up click handlers. */
function fill_ehp_chart(activeData: SyntheticEHP, visibleNames: ReadonlySet<string>) {
    ehpChart.dotCallback = handleDotClick;
    ehpChart.lineCallback = handleLineClick;
    ehpChart.tauMultCallback = handleTauMultClick;

    ehpChart.set_all_generators(activeData.generators);
    ehpChart.set_all_differentials(activeData.differentials);
    ehpChart.set_all_multiplications(activeData.multiplications);
    ehpChart.set_all_tau_mults(getAllTauMults(activeData));

    ehpChart.init(visibleNames);
    chartData = activeData;
}

/**
 * Re-derive what the EHP chart shows from the current view settings.
 *
 * Update the classes and relations for the selected page.
 * Dot borders use torsion on the selected page, and fills use E∞ torsion.
 */
export function update_ehp_chart() {
    const activeData = getActiveData();
    if (!activeData) {
        return;
    }
    const query = {
        category: viewSettings.category,
        truncation: viewSettings.truncation,
        bottomTruncation: viewSettings.bottomTruncation,
        page: viewSettings.page,
    };
    const currentView = computePage(activeData, query);
    const gens = currentView.generators;
    currentFilteredGenerators = gens;
    const visibleNames = new Set(Object.keys(gens).filter((name) => survivesFilteredGenerator(gens[name])));
    if (chartData !== activeData) {
        fill_ehp_chart(activeData, visibleNames);
    }

    // Hide all relations first, then turn the relevant ones back on.
    activeData.differentials.forEach((d) => {
        ehpChart.display_diff(d.from, d.to, false);
    });
    activeData.multiplications.forEach((m) => {
        ehpChart.display_mult(m.from, m.to, false);
    });
    ehpChart.tau_mults.forEach((t) => {
        ehpChart.display_tau_mult(t.from, t.to, false);
    });

    const permView = query.page === 1000 ? currentView : computePage(activeData, { ...query, page: 1000 });
    const perm_classes = permView.generators;
    cacheComputedDiffs(permView.differentials);
    differentialCoefficients = currentView.differentialCoefficients;

    const eligibleDiffs = activeData.differentials.filter((d) => {
        if (!shouldIncludeKind(d.kind)) {
            return false;
        }
        if (!gens[d.from] || gens[d.from][0] === 0 || !gens[d.to] || gens[d.to][0] === 0) {
            return false;
        }
        return true;
    });
    const displayedDiffs = query.category === Category.Geometric
        ? geometricDifferentials(eligibleDiffs, query.page, viewSettings.allDiffs)
        : eligibleDiffs.filter((d) => {
            if (!viewSettings.allDiffs && d.d != viewSettings.page) {
                return false;
            }
            if (d.d && d.d < viewSettings.page) {
                return false;
            }
            return true;
        });

    activeData.generators.forEach(({ name }) => {
        const entry = gens[name];
        if (!entry || !survivesFilteredGenerator(entry)) {
            ehpChart.display_dot(name, false);
            return;
        }
        const finalEntry = perm_classes[name];
        ehpChart.display_dot(name, true, {
            currentTorsion: entry[0],
            finalTorsion: finalEntry ? finalEntry[0] : 0,
            filtration: entry[1],
        });
    });

    displayedDiffs.forEach((d) => {
        if (viewSettings.category === Category.Algebraic && d.kind !== "Algebraic") {
            return;
        }
        ehpChart.display_diff(d.from, d.to, true, differentialCoefficients.get(d));
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

    updateMultiplicationOverlay(ehpChart, activeData.generators, visibleNames, "ehp", undefined,
        new Set(Object.keys(perm_classes).filter((name) => survivesFilteredGenerator(perm_classes[name]))));
    applyEhpSelectionHighlight();
}
