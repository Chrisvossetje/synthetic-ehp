import { Category, find, getActiveData, get_filtered_data, getSelectedGenerator, getSphereLifecycleInfo, setSelectedGenerator } from "./logic";
import { assChart } from "./charts";
import { Differential, Generators } from "./types";
import { buildGeneratorInfoLines, showInfoPanel } from "./ui/info_panel";

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
    navigator.clipboard.writeText(gen.name).then(() => {
        console.log('Copied to clipboard:', gen.name);
    }).catch(err => {
        console.error('Failed to copy to clipboard:', err);
    });

    const sphereInfo = getSphereLifecycleInfo(gen);
    const lines = buildGeneratorInfoLines(gen, sphereInfo, {
        xLabel: "x",
        yLabel: "y",
        moduleLabel: "Torsion",
    });

    showInfoPanel(`Generator: ${gen.name}`, lines);
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
        `From: ${inferred.sourceBase}`,
        `To: ${inferred.targetBase}`,
        `Page: d_${inferred.page}`,
        `Coefficient: ${inferred.torsion === 0 ? "1" : "τ^" + inferred.torsion}`,
    ];

    const extraLines: string[] = [];
    if (inferred.linked) {
        extraLines.push("Linked EHP differential:");
        extraLines.push(`  ${inferred.linked.from} -> ${inferred.linked.to}`);
        if ("proof" in inferred.linked) {
            extraLines.push(`  Proof: ${inferred.linked.proof ?? ""}`);
        } else {
            extraLines.push("  AEHP differential");
        }
    }
    if (inferred.reason) {
        extraLines.push(`Reason: ${inferred.reason}`);
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

    // Compare synthetic and algebraic E_infinity states.
    const [syntheticClasses, syntheticDiffs] = get_filtered_data(
        activeData,
        Category.Synthetic,
        truncation,
        1000,
        true,
        undefined,
        true,
        bottomTruncation
    );
    const [algebraicClasses] = get_filtered_data(
        activeData,
        Category.Algebraic,
        truncation,
        1000,
        true,
        undefined,
        false,
        bottomTruncation
    );

    // Build ASS nodes/differentials:
    // - Free classes: one black dot.
    // - Torsion classes: two black dots with a colored differential between them.
    const gens: Generators[] = [];
    const diffs: Differential[] = [];
    Object.entries(algebraicClasses).forEach(([name, [_torsion, filtration]]) => {
        const syntheticEntry = syntheticClasses[name];
        const syntheticTorsion = syntheticEntry ? syntheticEntry[0] : 0;
        if (!(syntheticTorsion == undefined || syntheticTorsion > 0)) {
            return;
        }

        const g = find(name);
        if (!g) return;

        if (syntheticTorsion == undefined) {
            gens.push({
                name: g.name,
                stem: g.stem,
                y: filtration,
                af: filtration,
                induced_name: [[0, g.name]],
                born: g.born,
                dies: g.dies,
            });
            return;
        }

        const sourceName = `${g.name}__ass_src`;
        const targetName = `${g.name}__ass_tgt`;
        const targetFiltration = filtration - (syntheticTorsion + 1);

        gens.push({
            name: sourceName,
            stem: g.stem,
            y: filtration,
            af: filtration,
            induced_name: [[0, g.name]],
            born: g.born,
            dies: g.dies,
        });
        gens.push({
            name: targetName,
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
            coeff: syntheticTorsion,
            d: syntheticTorsion + 1,
            kind: "Real",
            proof: "Inferred from ASS torsion."
        });

        let linked: Differential | undefined;
        const exactMatch = syntheticDiffs.find(d => d.to === g.name && d.coeff === syntheticTorsion && d.d === syntheticTorsion + 1);
        if (exactMatch) {
            linked = exactMatch;
        } else {
            linked = syntheticDiffs.find(d => d.to === g.name && d.coeff === syntheticTorsion);
        }

        const reason = (linked === undefined && g.torsion !== undefined && g.torsion > 0)
            ? "On E1 page the target of this differential was already torsion"
            : undefined;

        inferredAssDifferentials.set(`${sourceName}->${targetName}`, {
            sourceBase: g.name,
            targetBase: g.name,
            torsion: syntheticTorsion,
            page: syntheticTorsion + 1,
            linked,
            reason
        });
    });

    assChart.set_all_generators(gens);
    assChart.set_all_differentials(diffs);
    assChart.set_all_multiplications([]);
    assChart.set_all_tau_mults([]);
    assChart.init();

    // ASS dots are always black in this presentation.
    gens.forEach((gen) => {
        assChart.display_dot(gen.name, true, true, 0, gen.af);
    });

    // Differential color encodes the torsion exponent.
    diffs.forEach((d) => {
        assChart.display_diff(d.from, d.to, true, d.coeff);
    });

    applyAssSelectionHighlight();
}
