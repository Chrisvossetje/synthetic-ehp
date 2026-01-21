import { xml } from "d3";
import { Chart } from "./chart";
import { Differential, Generators, SyntheticEHP } from "./types";

/*
 * SPECTRAL SEQUENCE OVER F2[t]
 *
 * We are computing a spectral sequence over the polynomial ring F2[t], not just F2.
 * This means generators can be:
 *   - Free F2[t]-modules (torsion = undefined)
 *   - Torsion F2[t]/(t^n) modules (torsion = n)
 *
 * DIFFERENTIAL BEHAVIOR:
 * When a differential d_r(x) = coeff * y occurs:
 *
 * 1. The SOURCE (x) always DIES on the E_{r+1} page
 *
 * 2. The TARGET (y) behavior depends on the coefficient:
 *    - If coeff involves t (e.g., t, t^2, etc.) and y was FREE:
 *      → y SURVIVES but becomes TORSION
 *      → y becomes an F2[t]/(t) module (killed by multiplication by t)
 *    - If y was already torsion F2[t]/(t^n), it may die or change torsion
 *
 * Example: If d(x) = t*y where x,y are free modules:
 *   - Page r+1: x is dead, y survives but is now torsion (killed by t)
 *
 * This tracks how algebraic torsion propagates through the spectral sequence.
 */

export let data: SyntheticEHP = {
    generators: [],
    differentials: [],
    multiplications: []
};

// Current view settings
export let viewSettings = {
    allDiffs: true,
    page: 1,
    category: 0, // 0: Synthetic, 1: Algebraic, 2: Classical
    truncation: undefined as number | undefined
};

function sort_diffs() {
    data.differentials.sort((a,b) => {
        return a.d - b.d
    });
}

export async function initdata(chart: Chart) {
    fetch("./data.json").then(x => x.text()).then(x => {
        data = JSON.parse(x);
        sort_diffs();
        if (!verify_integrity()) {
            console.error("Data integrity has been compromised")
        }
        fill_chart(chart);
    });
}

function verify_integrity(): boolean {
    // Each generator has a unique name
    const names = new Set<string>();
    for (const gen of data.generators) {
        if (names.has(gen.name)) {
            console.error(`Duplicate generator name: ${gen.name}`);
            return false;
        }
        names.add(gen.name);
    }

    // Each differential maps to / from a generator
    for (const diff of data.differentials) {
        if (diff.from && !names.has(diff.from)) {
            console.error(`Differential references unknown 'from' generator: ${diff.from}`);
            return false;
        }
        if (diff.to && !names.has(diff.to)) {
            console.error(`Differential references unknown 'to' generator: ${diff.to}`);
            return false;
        }
    }

    // Each multiplication maps to / from a generator
    for (const mult of data.multiplications) {
        if (mult.from && !names.has(mult.from)) {
            console.error(`Multiplication references unknown 'from' generator: ${mult.from}`);
            return false;
        }
        if (mult.to && !names.has(mult.to)) {
            console.error(`Multiplication references unknown 'to' generator: ${mult.to}`);
            return false;
        }
    }

    return true;
}

/**
 * Get differentials to display based on page and allDiffs setting
 */
function get_differentials_for_page(page: number, allDiffs: boolean): Differential[] {
    if (page === 1000) { // E∞ page - no differentials
        return [];
    }

    if (allDiffs) {
        // Show all differentials with d >= page
        return data.differentials.filter(diff => diff.d >= page);
    } else {
        // Show only differentials for this page
        return data.differentials.filter(diff => diff.d === page);
    }
}

// /**
//  * Apply category filtering
//  * 0: Synthetic (normal)
//  * 1: Algebraic (remove tau-multiple diffs, keep all gens)
//  * 2: Classical (remove torsion gens, make tau-multiple diffs normal)
//  */
// function apply_category_filter(gens: Map<string, [number | undefined, number]>, diffs: Differential[], category: number): {gens: Map<string, [number | undefined, number]>, diffs: Differential[]} {
//     if (category === 0) {
//         // Synthetic: no changes
//         return {gens, diffs};
//     } else if (category === 1) {
//         // Algebraic: keep all generators, remove tau-multiple differentials
//         const filtered_diffs = diffs.filter(d => d.coeff === 0);
//         return {gens, diffs: filtered_diffs};
//     } else {
//         // Classical: remove torsion generators, keep all differentials (tau-multiple become normal)
//         const filtered_gens = gens.filter(g => g.torsion === undefined);
//         return {gens: filtered_gens, diffs};
//     }
// }


/**
 * Get the filtered view based on current settings
 */
export function get_filtered_data(perm_classes: boolean, category: number, truncation: number | undefined, page: number, allDiffs: boolean): [Object, Differential[]] {
    // name -> torsion + adams filtration
    const torsion = new Object();

    data.generators.forEach((g) => {
        if (!truncation || g.y < truncation) {
            if (g.purely_algebraic && category == 1) { // Special Algebraic
                torsion[g.name] = [undefined, g.adams_filtration];
            }
            if (category == 2) { // Classical
                if (g.torsion == undefined) {
                    torsion[g.name] = [undefined, g.adams_filtration];
                } 
            } else { 
                if (category == 0) { // Synthetic 
                    torsion[g.name] = [g.torsion, g.adams_filtration];
                } else { // Algebraic
                    torsion[g.name] = [undefined, g.adams_filtration];
                }
            }
        }
    });

    let diffs = [];

    // Find all generators killed by differentials before this page
    for (const diff of data.differentials) {

        // Make sure the elements exist
        if (torsion[diff.from] && torsion[diff.to]) {

            // Only calculate diffs which would have elemented before
            if (diff.d < page || perm_classes) {
                // Do it for synthetic
                if (category == 0) {
                    if (torsion[diff.to][0] == undefined) {
                        torsion[diff.from][0] = 0;
                        torsion[diff.to][0] = diff.coeff;
                        diffs.push(diff);              
                    } else {
                        if (torsion[diff.from][0] > 0) {
                            console.error(`For ${diff.from} -> ${diff.to}, ${torsion[diff.from]} | ${torsion[diff.to]}  Mapping from torsion to another torsion element, not yet supported`);
                        }

                        if (torsion[diff.to][0] != 0) {
                            console.log(torsion[diff.from])
                            console.log(torsion[diff.to])
                            torsion[diff.from][1] += torsion[diff.to][0];
                            torsion[diff.to][0] = 0
                            diffs.push(diff);              
                            console.log(torsion[diff.from])
                        }
                    }
                } else if (category == 1) { // Algebraic
                    if (diff.coeff == 0) {
                        if (torsion[diff.to][0] || torsion[diff.to][0] != 0) {
                            torsion[diff.from][0] = 0;
                            torsion[diff.to][0] = 0;
                            diffs.push(diff);              
                        } else {
                            // Element had already been killed ?
                            // This cannot occur in algebraic ?
                        }
                    }
                } else {
                    if (torsion[diff.to][0] || torsion[diff.to][0] != 0) {
                        torsion[diff.from][0] = 0;
                        torsion[diff.to][0] = 0;  
                        diffs.push(diff);                  
                    } else {
                        // Element had already been killed 
                        // This can occur in classical !
                    }               
                }
            }
        }
    }

    return [torsion, diffs]
}

export function handleDotClick(dot: string) {
    console.log('Dot clicked:', dot);
    const gen = data.generators.find(g => g.name === dot);
}

export function handleLineClick(from: string, to: string) {
    console.log('Line clicked:', from, '->', to);
    const diff = data.differentials.find(d => d.from === from && d.to === to);
}

let currentChart: Chart | undefined = undefined;

export function fill_chart(chart: Chart) {
    currentChart = chart;

    // Bind click handlers
    chart.dotCallback = handleDotClick;
    chart.lineCallback = handleLineClick;

    // Set all generators and differentials (complete data set)
    chart.set_all_generators(data.generators);
    chart.set_all_differentials(data.differentials);
    chart.set_all_multiplications(data.multiplications);

    chart.init();

    // Update with filtered data
    update_chart();
}

/**
 * Update the chart with current filter settings
 */
export function update_chart() {
    if (!currentChart) return;

    data.generators.forEach((g) => {
        currentChart.display_dot(g.name, false, false, undefined, g.adams_filtration);
    });
    data.differentials.forEach((d) => {
        currentChart.display_diff(d.from, d.to, false);
    });
    data.multiplications.forEach((m) => {
        currentChart.display_mult(m.from, m.to, false);
    });

    const [gens, _] = get_filtered_data(false, viewSettings.category, viewSettings.truncation, viewSettings.page, viewSettings.allDiffs);
    const [perm_classes, diffs] = get_filtered_data(true, viewSettings.category, viewSettings.truncation, viewSettings.page, viewSettings.allDiffs);

    const real_diffs = diffs.filter((d) => {
        if (!gens[d.from] || !gens[d.to]) {
            return false;
        }
        if (!viewSettings.allDiffs && d.d != viewSettings.page) {
            return false;
        }
        if (gens[d.from][0] == undefined || gens[d.from][0] > 0) {
            if (gens[d.to][0] == undefined || gens[d.to][0] > 0) {
                return true;
            }
        }
        return false;
    });

    Object.entries(gens).forEach(([name, [torsion, filtration]]) => {
        if (torsion == undefined || torsion > 0) {
            let perm = perm_classes[name] != undefined && (perm_classes[name][0] == undefined || perm_classes[name][0] > 0);
            currentChart.display_dot(name, true, perm, torsion, filtration);
        }
    });
    real_diffs.forEach((d) => {
        let torsion = d.coeff;
        if (viewSettings.category != 0) {
            torsion = 0;
        }
        currentChart.display_diff(d.from, d.to, true, torsion);
    });

    // Display multiplications only when both generators are alive
    data.multiplications.forEach((m) => {
        const fromAlive = gens[m.from] && (gens[m.from][0] == undefined || gens[m.from][0] > 0);
        const toAlive = gens[m.to] && (gens[m.to][0] == undefined || gens[m.to][0] > 0);
        if (fromAlive && toAlive) {
            currentChart.display_mult(m.from, m.to, true);
        }
    });
}

