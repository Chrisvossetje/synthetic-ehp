import { xml } from "d3";
import { Chart } from "./chart";
import { Differential, Generators, SyntheticEHP } from "./types";
import { data, MAX_STEM } from "./data";

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
 * 1. The SOURCE (x) DIES on the E_{r+1} page if y is torsion-free, else its adams filtration jumps
 *      (We assume that our source is always a free F2[t]-module, but this is not checked yet)
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

export enum Category {
    Synthetic,
    Algebraic,
    Classical
}

// Current view settings
export let viewSettings = {
    allDiffs: true,
    page: 1,
    category: Category.Synthetic, // 0: Synthetic, 1: Algebraic, 2: Classical
    truncation: undefined as number | undefined
};

export function find(name: string): Generators {
    return data.generators.find(g => g.name === name);
}

export function initdata(chart: Chart) {
    fill_chart(chart);
}

export function generated_by_name(gen: Generators): string {
    const initial = gen.name.split("[")[0];

    const split_first = initial.split(' ');
    const end = "[" + String(split_first[0]) + "]";
    if (split_first.length == 1) {
        return end;
    } else {
        return split_first.slice(1).join(" ") + end;
    }
}

export function generating_name(gen: Generators): string {
    const [initial, last] = gen.name.split("[");
    const real_last = last.split("]")[0];
    if (initial == "") {
        return real_last;
    } else {
        return real_last + " " + initial;
    }
}


export function generates(gen: Generators): Generators[] {
    let name = generating_name(gen);

    let gens = [];
    for (let index = 1; index <= MAX_STEM; index++) {
        const element = name + "[" + String(index) + "]";
        const g = find(element);
        if (g) {
            gens.push(g);
        }
    }
    return gens;
}

/**
 * Get the filtered view based on current settings
 */
export function get_filtered_data(data: SyntheticEHP, perm_classes: boolean, category: Category, truncation: number | undefined, page: number, allDiffs: boolean, limit_x?: number): [Object, Differential[]] {
    // name -> torsion + adams filtration
    const torsion = new Object();

    data.generators.forEach((g) => {
        if ((!truncation || g.y < truncation) && ((limit_x - 1 <= g.x && g.x <= limit_x + 1) || !limit_x)) {
            if (category == Category.Algebraic) { // Special Algebraic
                torsion[g.name] = [undefined, g.adams_filtration];
            }
            else if (category == Category.Classical) { // Classical
                if (g.torsion == undefined) {
                    torsion[g.name] = [undefined, g.adams_filtration];
                } 
            } else { 
                torsion[g.name] = [g.torsion, g.adams_filtration];
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
                if (category == Category.Synthetic) {
                    if (torsion[diff.to][0] == undefined) {
                        if (torsion[diff.from][0] == 0) {
                            continue;
                        } 

                        torsion[diff.from][0] = 0;
                        torsion[diff.to][0] = diff.coeff;
                        diffs.push(diff);              
                    } else {
                        if (torsion[diff.from][0] > 0 && torsion[diff.from][0] < torsion[diff.to][0]) {
                            console.error(`For ${diff.from} -> ${diff.to}, ${torsion[diff.from]} | ${torsion[diff.to]}.  Mapping from lower torsion to another higher torsion element, This cannot happen !`);
                        }
                            
                        // This is where we have a diff mapping into a torsion module 
                        if (torsion[diff.to][0] != 0) {
                            if (torsion[diff.from][0] > 0) {
                                torsion[diff.from][0] -= torsion[diff.to][0];
                            }
                            torsion[diff.from][1] -= torsion[diff.to][0];
                            torsion[diff.to][0] = 0

                            diffs.push(diff);              
                        }
                    }
                    

                    
                } else if (category == Category.Algebraic) { // Algebraic
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




                } else { // Classical
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
    const gen = find(dot);
    console.log(gen);
}

export function handleLineClick(from: string, to: string) {
    console.log('Line clicked:', from, '->', to);
    const diff = data.differentials.find(d => d.from === from && d.to === to);
    console.log(diff);
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

    const [gens, _] = get_filtered_data(data, false, viewSettings.category, viewSettings.truncation, viewSettings.page, viewSettings.allDiffs);
    const [perm_classes, diffs] = get_filtered_data(data, true, viewSettings.category, viewSettings.truncation, viewSettings.page, viewSettings.allDiffs);

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
        if (viewSettings.category != Category.Synthetic) {
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

