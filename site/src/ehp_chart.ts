import { ehpChart } from "./charts";
import {
    Category,
    find,
    generated_by_name,
    generates,
    getActiveData,
    get_filtered_data,
    getSelectedGenerator,
    getSphereLifecycleInfo,
    isUsingStableData,
    setSelectedGenerator,
    setUseStableData,
    survivesFilteredGenerator,
    viewSettings,
    ensureStableDataLoading
} from "./logic";
import { Differential, SyntheticEHP } from "./types";
import { buildGeneratorInfoLines, showInfoPanel } from "./ui/info_panel";

let computedDiffsByKey: Map<string, Differential> = new Map();
let displayedDiffsByKey: Map<string, Differential> = new Map();
let currentFilteredGenerators: Record<string, [number | undefined, number]> = {};

function cacheComputedDiffs(diffs: Differential[]) {
    computedDiffsByKey.clear();
    diffs.forEach((d) => {
        computedDiffsByKey.set(`${d.from}->${d.to}`, d);
    });
}

function getAllTauMults(data: SyntheticEHP) {
    return [...data.internal_tau_mults, ...data.external_tau_mults];
}

export function getComputedDiff(from: string, to: string): Differential | undefined {
    return displayedDiffsByKey.get(`${from}->${to}`) ?? computedDiffsByKey.get(`${from}->${to}`);
}

function getDisplayedDiffCoeff(from: string, to: string): number | undefined {
    const fromEntry = currentFilteredGenerators[from];
    const toEntry = currentFilteredGenerators[to];
    if (!fromEntry || !toEntry) {
        return undefined;
    }
    return toEntry[1] - fromEntry[1] - 1;
}

export function getComputedDiffCoeff(from: string, to: string): number | undefined {
    return getDisplayedDiffCoeff(from, to);
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

    const coeff = getDisplayedDiffCoeff(from, to) ?? rawDiff?.coeff ?? 0;
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
    } else {
        extraLines.push("AEHP differential");
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

export function fill_ehp_chart() {
    const activeData = getActiveData();
    if (!activeData) {
        return;
    }

    // Bind click handlers
    ehpChart.dotCallback = handleDotClick;
    ehpChart.lineCallback = handleLineClick;
    ehpChart.tauMultCallback = handleTauMultClick;

    // Set all generators and differentials (complete data set)
    ehpChart.set_all_generators(activeData.generators);
    ehpChart.set_all_differentials(activeData.differentials);
    ehpChart.set_all_multiplications(activeData.multiplications);
    ehpChart.set_all_tau_mults(getAllTauMults(activeData));

    ehpChart.init();
}

/**
 * Switch between data and data_stable
 */
export async function switchDataSource() {
    const nextUseStableData = !isUsingStableData();
    if (nextUseStableData) {
        await ensureStableDataLoading();
    }
    setUseStableData(nextUseStableData);

    // Clear the chart
    ehpChart.clear();

    // Refill with the new data
    fill_ehp_chart();

    // Update the chart with current view settings
    update_ehp_chart();
}

/**
 * Update the EHP chart with current filter settings
 */
export function update_ehp_chart() {
    const activeData = getActiveData();
    if (!activeData) {
        return;
    }

    // Hide all generators and differentials first
    activeData.generators.forEach((g) => {
        ehpChart.display_dot(g.name, false, false, undefined, g.af);
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
    const [gens, _] = get_filtered_data(
        activeData,
        viewSettings.category,
        viewSettings.truncation,
        viewSettings.page,
        viewSettings.allDiffs,
        undefined,
        false,
        viewSettings.bottomTruncation
    );
    currentFilteredGenerators = gens as Record<string, [number | undefined, number]>;
    const [perm_classes, diffs] = get_filtered_data(
        activeData,
        viewSettings.category,
        viewSettings.truncation,
        1000,
        viewSettings.allDiffs,
        undefined,
        false,
        viewSettings.bottomTruncation
    );
    cacheComputedDiffs(diffs);

    const real_diffs = diffs.filter((d) => {
        if (!gens[d.from] || !gens[d.to]) {
            return false;
        }
        if (!viewSettings.allDiffs && d.d != viewSettings.page) {
            return false;
        }
        return true;
    });
    displayedDiffsByKey.clear();
    real_diffs.forEach((d) => {
        if (viewSettings.allDiffs && d.d < viewSettings.page) {
            return;
        }
        const key = `${d.from}->${d.to}`;
        const existing = displayedDiffsByKey.get(key);
        if (!existing) {
            displayedDiffsByKey.set(key, d);
            return;
        }
        if (d.d < existing.d) {
            displayedDiffsByKey.set(key, d);
        }
    });

    Object.entries(gens).forEach(([name, [torsion, filtration]]) => {
        if (torsion == undefined || torsion > 0) {
            let perm = perm_classes[name] != undefined && (perm_classes[name][0] == undefined || perm_classes[name][0] > 0);
            ehpChart.display_dot(name, true, perm, torsion, filtration);
        }
    });
    displayedDiffsByKey.forEach((d) => {
        let torsion = getDisplayedDiffCoeff(d.from, d.to);
        if (viewSettings.category != Category.Synthetic) {
            torsion = 0;
        }
        ehpChart.display_diff(d.from, d.to, true, torsion);
    });

    // Display multiplications only when both generators are alive
    activeData.multiplications.forEach((m) => {
        const fromAlive = survivesFilteredGenerator(gens[m.from]);
        const toAlive = survivesFilteredGenerator(gens[m.to]);
        if (fromAlive && toAlive) {
            ehpChart.display_mult(m.from, m.to, true);
        }
    });

    // Display tau multiplications only when both generators are alive
    if (viewSettings.category == Category.Synthetic) {
        activeData.internal_tau_mults.forEach((t) => {
            if (!viewSettings.allDiffs && t.page !== viewSettings.page) {
                return;
            }
            if (survivesFilteredGenerator(gens[t.from]) && survivesFilteredGenerator(gens[t.to])) {
                ehpChart.display_tau_mult(t.from, t.to, true);
            }
        });

        if (viewSettings.allDiffs || viewSettings.page > 999) {
            activeData.external_tau_mults.forEach((t) => {
                if (survivesFilteredGenerator(gens[t.from]) && survivesFilteredGenerator(gens[t.to])) {
                    ehpChart.display_tau_mult(t.from, t.to, true);
                }
            });
        }
    }

    applyEhpSelectionHighlight();
}
