import { Chart } from "./chart";
import { Category, find, get_filtered_data, generating_name, generates, isUsingStableData } from "./logic";
import { data as mainData } from "./data";
import { data_stable as stableData } from "./data_stable";
import { assChart, ehpChart } from "./main";
import { Differential, Generators } from "./types";

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

    // Get generating name and what it generates
    const genName = generating_name(gen);
    const gensList = generates(gen);

    // Build the info display
    const floatingBox = document.getElementById('floatingBox');
    if (!floatingBox) return;

    let content = `<span class="close-btn" onclick="document.getElementById('floatingBox').style.display='none'">x</span>`;
    content += `<h4>Generator: ${gen.name}</h4>`;
    content += `<pre style="background-color: #00000000; margin: 0;">`;
    content += `x: ${gen.x}\n`;
    content += `y: ${gen.y}\n`;
    content += `Adams filtration: ${gen.adams_filtration}\n`;
    content += `Torsion: ${gen.torsion !== undefined ? 'F2[τ]/τ^' + gen.torsion : 'F2[τ]'}\n`;

    if (gen.alg_name) {
        content += `Algebraic name: ${gen.alg_name}\n`;
    }
    if (gen.hom_name) {
        content += `Homotopy name: ${gen.hom_name}\n`;
    }
    const filteredInducedNames = gen.induced_name.filter(([num, _]) => num !== 0);
    if (filteredInducedNames.length > 0) {
        const namesList = filteredInducedNames.map(([sphere, name]) => `${name} (sphere ${sphere})`).join(', ');
        content += `Induced name: ${namesList}\n`;
    }

    content += `\n<b>Generating name:</b> ${genName}\n`;

    if (gensList.length > 0) {
        content += `\n<b>Generates:</b>\n`;
        gensList.forEach(g => {
            content += `  • ${g.name}\n`;
        });
    }

    content += `</pre>`;

    floatingBox.innerHTML = content;
    floatingBox.style.display = 'block';
}

export function handleAssLineClick(from: string, to: string) {
    const key = `${from}->${to}`;
    const inferred = inferredAssDifferentials.get(key);
    if (!inferred) return;

    const floatingBox = document.getElementById('floatingBox');
    if (!floatingBox) return;

    let content = `<span class="close-btn" onclick="document.getElementById('floatingBox').style.display='none'">x</span>`;
    content += `<h4>ASS Differential</h4>`;
    content += `<pre style="background-color: #00000000; margin: 0;">`;
    content += `From: ${inferred.sourceBase}\n`;
    content += `To: ${inferred.targetBase}\n`;
    content += `Page: d_${inferred.page}\n`;
    content += `Coefficient: ${inferred.torsion === 0 ? "1" : "τ^" + inferred.torsion}\n`;

    if (inferred.linked) {
        content += `\nLinked EHP differential:\n`;
        content += `  ${inferred.linked.from} -> ${inferred.linked.to}\n`;
        if (inferred.linked.proof) {
            content += `  Proof: ${inferred.linked.proof}\n`;
        }
    }
    if (inferred.reason) {
        content += `\nReason: ${inferred.reason}\n`;
    }

    content += `</pre>`;

    floatingBox.innerHTML = content;
    floatingBox.style.display = 'block';
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
    truncation: number | undefined
) {
    assChart.clear();
    inferredAssDifferentials.clear();
    const activeData = isUsingStableData() ? stableData : mainData;

    // Bind click handlers for ASS chart
    assChart.dotCallback = handleAssDotClick;
    assChart.lineCallback = handleAssLineClick;

    // Compare synthetic and algebraic E_infinity states.
    const [syntheticClasses, _syntheticDiffs] = get_filtered_data(
        activeData,
        Category.Synthetic,
        truncation,
        1000,
        true
    );
    const [algebraicClasses, _algebraicDiffs] = get_filtered_data(
        activeData,
        Category.Algebraic,
        truncation,
        1000,
        true
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
                x: g.x,
                y: filtration,
                adams_filtration: filtration,
                induced_name: [[0, g.name]]
            });
            return;
        }

        const sourceName = `${g.name}__ass_src`;
        const targetName = `${g.name}__ass_tgt`;
        const targetFiltration = filtration - (syntheticTorsion + 1);

        gens.push({
            name: sourceName,
            x: g.x,
            y: filtration,
            adams_filtration: filtration,
            induced_name: [[0, g.name]]
        });
        gens.push({
            name: targetName,
            x: g.x + 1,
            y: targetFiltration,
            adams_filtration: targetFiltration,
            induced_name: [[0, g.name]]
        });
        diffs.push({
            from: sourceName,
            to: targetName,
            coeff: syntheticTorsion,
            d: syntheticTorsion + 1,
            fake: true,
            proof: "Inferred from ASS torsion."
        });

        let linked: Differential | undefined;
        const exactMatch = activeData.differentials.find(d => d.to === g.name && d.coeff === syntheticTorsion && d.d === syntheticTorsion + 1);
        if (exactMatch) {
            linked = exactMatch;
        } else {
            linked = activeData.differentials.find(d => d.to === g.name && d.coeff === syntheticTorsion);
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
        assChart.display_dot(gen.name, true, true, 0, gen.adams_filtration);
    });

    // Differential color encodes the torsion exponent.
    diffs.forEach((d) => {
        assChart.display_diff(d.from, d.to, true, d.coeff);
    });
}
