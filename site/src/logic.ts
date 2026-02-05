import { xml } from "d3";
import { Chart } from "./chart";
import { Differential, Generators, SyntheticEHP } from "./types";
import { data, MAX_STEM } from "./data";
import { ehpChart } from "./main";

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
    Geometric
}

// Current view settings
export let viewSettings = {
    allDiffs: true,
    page: 1,
    category: Category.Synthetic, // 0: Synthetic, 1: Algebraic, 2: Geometric
    truncation: undefined as number | undefined
};

export function find(name: string): Generators {
    return data.generators.find(g => g.name === name);
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
export function get_filtered_data(data: SyntheticEHP, category: Category, truncation: number | undefined, page: number, allDiffs: boolean, limit_x?: number): [Object, Differential[]] {
    // name -> torsion + adams filtration
    const torsion = new Object();

    data.generators.forEach((g) => {
        if ((!truncation || g.y < truncation) && ((limit_x - 1 <= g.x && g.x <= limit_x + 1) || !limit_x)) {
            if (category == Category.Algebraic) { // Special Algebraic
                torsion[g.name] = [undefined, g.adams_filtration];
            }
            else if (category == Category.Geometric) { // Geometric
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
            if (diff.d < page) {
                
                // Synthetic
                if (category == Category.Synthetic) { 
                    if (torsion[diff.to][0] == 0) {
                        continue;
                    }
                    if (torsion[diff.from][0] == 0) {
                        continue;
                    }

                    if (torsion[diff.to][0] == undefined) {
                        torsion[diff.from][0] = 0;
                        torsion[diff.to][0] = diff.coeff;
                        diffs.push(diff);              
                    } else {                         
                        // This is where we have a diff mapping into a torsion module 
                        if (torsion[diff.from][0] > 0) {
                            torsion[diff.from][0] = torsion[diff.from][0] - torsion[diff.to][0] + diff.coeff;
                        }
                        torsion[diff.from][1] = torsion[diff.from][1] - torsion[diff.to][0] + diff.coeff;
                        torsion[diff.to][0] = diff.coeff;
                        diffs.push(diff);              
                    }    




                // Algebraic
                } else if (category == Category.Algebraic) { 
                    if (diff.coeff == 0 && diff.synthetic === undefined) {
                        torsion[diff.from][0] = 0;
                        torsion[diff.to][0] = 0;
                        diffs.push(diff);              
                    }



                // Geometric
                } else { 
                    if (torsion[diff.to][0] || torsion[diff.to][0] != 0) {
                        torsion[diff.from][0] = 0;
                        torsion[diff.to][0] = 0;  
                        diffs.push(diff);                  
                    } else {
                        // Element had already been killed 
                        // This can occur in geometric !
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

    if (!gen) return;

    // Get generating name and what it generates
    const genName = generated_by_name(gen);
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
    content += `Module: ${gen.torsion !== undefined ? 'F2[τ]/τ^' + gen.torsion : 'F2[τ]'}\n`;

    if (gen.alg_name) {
        content += `Algebraic name: ${gen.alg_name}\n`;
    }
    if (gen.hom_name) {
        content += `Homotopy name: ${gen.hom_name}\n`;
    }
    const filteredInducedNames = gen.induced_name.filter(([num, _]) => num !== 0);
    if (filteredInducedNames.length > 0) {
        const namesList = filteredInducedNames.map(([_, name]) => name).join(', ');
        content += `Induced name: ${namesList}\n`;
    }

    content += `\n<b>Generated by:</b> ${genName}\n`;

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

export function handleLineClick(from: string, to: string) {
    console.log('Line clicked:', from, '->', to);
    const diff = data.differentials.find(d => d.from === from && d.to === to);
    console.log(diff);

    if (!diff) return;

    // Build the info display
    const floatingBox = document.getElementById('floatingBox');
    if (!floatingBox) return;

    let content = `<span class="close-btn" onclick="document.getElementById('floatingBox').style.display='none'">x</span>`;
    content += `<h4>Differential</h4>`;
    content += `<pre style="background-color: #00000000; margin: 0;">`;
    content += `From: ${diff.from}\n`;
    content += `To: ${diff.to}\n`;
    content += `Page: E${diff.d}\n`;
    content += `Coefficient: ${diff.coeff === 0 ? '1' : 'τ^' + diff.coeff}\n`;
    
    if (diff.synthetic !== undefined) {
        content += `\nSynthetic Differential\n`;
    }
    if (diff.proof) {
        content += `\nProof: ${diff.proof}\n`;
    }
    
    content += `</pre>`;

    floatingBox.innerHTML = content;
    floatingBox.style.display = 'block';
}

export function fill_ehp_chart() {
    // Bind click handlers
    ehpChart.dotCallback = handleDotClick;
    ehpChart.lineCallback = handleLineClick;

    // Set all generators and differentials (complete data set)
    ehpChart.set_all_generators(data.generators);
    ehpChart.set_all_differentials(data.differentials);
    ehpChart.set_all_multiplications(data.multiplications);
    ehpChart.set_all_tau_mults(data.tau_mults);

    ehpChart.init();
}

/**
 * Update the EHP chart with current filter settings
 */
export function update_ehp_chart() {
    // Hide all generators and differentials first
    data.generators.forEach((g) => {
        ehpChart.display_dot(g.name, false, false, undefined, g.adams_filtration);
    });
    data.differentials.forEach((d) => {
        ehpChart.display_diff(d.from, d.to, false);
    });
    data.multiplications.forEach((m) => {
        ehpChart.display_mult(m.from, m.to, false);
    });
    data.tau_mults.forEach((t) => {
        ehpChart.display_tau_mult(t.from, t.to, false);
    });

    const [gens, _] = get_filtered_data(data, viewSettings.category, viewSettings.truncation, viewSettings.page, viewSettings.allDiffs);
    const [perm_classes, diffs] = get_filtered_data(data, viewSettings.category, viewSettings.truncation, 1000, viewSettings.allDiffs);

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
            ehpChart.display_dot(name, true, perm, torsion, filtration);
        }
    });
    real_diffs.forEach((d) => {
        let torsion = d.coeff;
        if (viewSettings.category != Category.Synthetic) {
            torsion = 0;
        }
        if (d.d >= viewSettings.page) {
            ehpChart.display_diff(d.from, d.to, true, torsion);
        }
    });

    // Display multiplications only when both generators are alive
    data.multiplications.forEach((m) => {
        const fromAlive = gens[m.from] && (gens[m.from][0] == undefined || gens[m.from][0] > 0);
        const toAlive = gens[m.to] && (gens[m.to][0] == undefined || gens[m.to][0] > 0);
        if (fromAlive && toAlive) {
            ehpChart.display_mult(m.from, m.to, true);
        }
    });

    // Display tau multiplications only when both generators are alive
    data.tau_mults.forEach((t) => {
        if (viewSettings.category == Category.Synthetic) {
            const fromAlive = gens[t.from] && (gens[t.from][0] == undefined || gens[t.from][0] > 0);
            const toAlive = gens[t.to] && (gens[t.to][0] == undefined || gens[t.to][0] > 0);
            if (fromAlive && toAlive) {
                ehpChart.display_tau_mult(t.from, t.to, true);
            }
        }
    });
}

