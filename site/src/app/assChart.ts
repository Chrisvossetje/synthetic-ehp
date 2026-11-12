import { assChart } from "./charts";
import { Differential, Generators } from "../types";
import { Category, getSelectedGenerator, setSelectedGenerator } from "../model/settings";
import { find, getActiveData } from "../model/dataSource";
import { get_induced_name, getSphereLifecycleInfo } from "../model/names";
import { computePage } from "../model/spectralSequence";
import { buildGeneratorInfoLines, showInfoPanel } from "../chart/infoPanel";
import { multiplicationInfoLines, updateMultiplicationOverlay } from "./multiplications";
import { algebraicName, displayName, displayText } from "../model/tagNames";

type InferredAssDifferential = {
    sourceBase: string;
    targetBase: string;
    torsion: number;
    page: number;
    linked?: Differential;
    reason?: string;
};

const inferredAssDifferentials: Map<string, InferredAssDifferential> = new Map();

/**
 * ASS (Adams Spectral Sequence) Chart Logic
 *
 * The ASS chart displays permanent classes (survivors) from the synthetic EHP spectral sequence.
 * For each truncation level, the ASS chart shows what survives to E_infinity in the synthetic category.
 *
 * ASS chart always shows E_infinity page with all differentials resolved.
 */

export function handleAssDotClick(dot: string) {
    const baseName = dot.replace(/__ass_(src|tgt)$/, '');
    setSelectedGenerator(baseName);
    applyAssSelectionHighlight();

    console.log('ASS Dot clicked:', baseName);
    const gen = find(baseName);
    console.log(gen);

    if (!gen) return;

    // Copy generator name to clipboard
    const label = assChart.generator_label(dot);
    navigator.clipboard.writeText(label).then(() => {
        console.log('Copied to clipboard:', label);
    }).catch(err => {
        console.error('Failed to copy to clipboard:', err);
    });

    const sphereInfo = getSphereLifecycleInfo(gen);
    const lines = buildGeneratorInfoLines(gen, sphereInfo, {
        xLabel: "x",
        yLabel: "y",
        moduleLabel: "Torsion",
    }, displayName, assChart.generator_row(dot));

    showInfoPanel("Generator", lines, multiplicationInfoLines(assChart, gen.name), label);
}

function applyAssSelectionHighlight() {
    assChart.clear_selection_highlights();
    const selected = getSelectedGenerator();
    if (!selected) return;

    // ASS may display a base dot (free class) or synthetic source/target dots (torsion class).
    const preferredNames = [selected, `${selected}__ass_src`, `${selected}__ass_tgt`];
    for (const name of preferredNames) {
        if (assChart.name_to_location.has(name)) {
            assChart.add_selection_highlight(name, "#ff6a00", 2.35, 0.2, 0.6);
            return;
        }
    }
}

export function handleAssLineClick(from: string, to: string) {
    const key = `${from}->${to}`;
    const inferred = inferredAssDifferentials.get(key);
    if (!inferred) return;

    const lines = [
        `From: ${assChart.generator_label(from)}`,
        `To: ${assChart.generator_label(to)}`,
        `Page: d_${inferred.page}`,
        `Coefficient: ${inferred.torsion === 0 ? "1" : "τ^" + inferred.torsion}`,
    ];

    const extraLines: string[] = [];
    if (inferred.linked) {
        extraLines.push("Linked EHP differential:");
        extraLines.push(`  ${displayName(inferred.linked.from)} -> ${displayName(inferred.linked.to)}`);
        if ("proof" in inferred.linked) {
            extraLines.push(`  Proof: ${displayText(inferred.linked.proof ?? "")}`);
        } else {
            extraLines.push("  AEHP differential");
        }
    }
    if (inferred.reason) {
        extraLines.push(`Reason: ${displayText(inferred.reason)}`);
    }

    showInfoPanel("ASS Differential", lines, extraLines);
}

/**
 * Update the ASS chart based on current truncation
 *
 * Key differences from EHP chart:
 * - Displays only permanent classes from synthetic category
 * - Always shows E_infinity page (page and allDiffs settings don't apply)
 * - For each truncation, shows what survives at that level
 */
export function update_ass_chart(
    truncation: number | undefined,
    bottomTruncation: number | undefined
) {
    assChart.clear();
    inferredAssDifferentials.clear();
    const activeData = getActiveData();
    if (!activeData) {
        return;
    }

    // Bind click handlers for ASS chart
    assChart.dotCallback = handleAssDotClick;
    assChart.lineCallback = handleAssLineClick;

    // The ASS chart is the synthetic E∞ page with external τ-mults resolved.
    const syntheticClasses = computePage(activeData, {
        category: Category.Synthetic,
        truncation,
        bottomTruncation,
        page: 1000,
        applyTauMults: true,
    }).generators;

    // Build ASS nodes/differentials:
    // - Free classes: one black dot.
    // - Torsion classes: two black dots with a colored differential between them.
    const gens: Generators[] = [];
    const diffs: Differential[] = [];
    const initialFiltrations = new Map<string, number>();
    Object.entries(syntheticClasses).forEach(([name, [torsion, filtration]]) => {
        if (!(torsion == undefined || torsion > 0)) {
            return;
        }

        const g = find(name);
        if (!g) return;
        
        const algebraicSphere = truncation === undefined ? 0 : truncation - 1;
        const representative = get_induced_name(g, algebraicSphere);

        if (torsion == undefined) {
            initialFiltrations.set(g.name, g.af);
            gens.push({
                name: g.name,
                alg_name: algebraicName(representative),
                stem: g.stem,
                y: filtration,
                af: filtration,
                induced_name: [[0, representative]],
                born: g.born,
                dies: g.dies,
            });

        } else {
            const sourceName = `${g.name}__ass_src`;
            const targetName = `${g.name}__ass_tgt`;
            const targetFiltration = filtration - (torsion + 1);
            initialFiltrations.set(sourceName, g.af);
            // The inferred target already has the Bockstein's filtration offset.
            initialFiltrations.set(targetName, g.af - (torsion + 1));
    
            gens.push({
                name: sourceName,
                alg_name: algebraicName(representative),
                stem: g.stem,
                y: filtration,
                af: filtration,
                induced_name: [[0, representative]],
                born: g.born,
                dies: g.dies,
            });
            gens.push({
                name: targetName,
                alg_name: g.alg_name ?? algebraicName(g.name),
                stem: g.stem + 1,
                y: targetFiltration,
                af: targetFiltration,
                induced_name: [[0, g.name]],
                born: -1,
                dies: -1,
            });
            diffs.push({
                from: sourceName,
                to: targetName,
                coeff: torsion,
                d: torsion + 1,
                kind: "Real",
                proof: "Inferred from ASS torsion."
            });

            inferredAssDifferentials.set(`${sourceName}->${targetName}`, {
                sourceBase: g.name,
                targetBase: g.name,
                torsion: torsion,
                page: torsion + 1,
            });
        }
    });

    assChart.set_all_generators(gens);
    assChart.set_all_differentials(diffs);
    assChart.set_all_multiplications([]);
    assChart.set_all_tau_mults([]);
    assChart.init();

    // ASS dots are always black in this presentation.
    gens.forEach((gen) => {
        assChart.display_dot(gen.name, true, {
            filtration: gen.af,
            initialFiltration: initialFiltrations.get(gen.name),
        });
    });

    // Differential color encodes the torsion exponent.
    diffs.forEach((d) => {
        assChart.display_diff(d.from, d.to, true, d.coeff);
    });

    const nodeNames = new Map(gens.filter((g) => !g.name.endsWith("__ass_tgt"))
        .map((g) => [g.name.replace(/__ass_src$/, ""), g.name] as [string, string]));
    updateMultiplicationOverlay(assChart, activeData.generators, new Set(nodeNames.keys()), "ass", nodeNames);
    applyAssSelectionHighlight();
}
