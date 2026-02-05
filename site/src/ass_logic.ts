import { Chart } from "./chart";
import { Category, find, get_filtered_data, generating_name, generates } from "./logic";
import { data } from "./data";
import { assChart, ehpChart } from "./main";
import { Generators } from "./types";

/**
 * ASS (Adams Spectral Sequence) Chart Logic
 *
 * The ASS chart displays permanent classes (survivors) from the synthetic EHP spectral sequence.
 * For each truncation level, the ASS chart shows what survives to E_infinity in the synthetic category.
 *
 * ASS chart always shows E_infinity page with all differentials resolved.
 */

export function handleAssDotClick(dot: string) {
    console.log('ASS Dot clicked:', dot);
    const gen = find(dot);
    console.log(gen);

    if (!gen) return;

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
    content += `Torsion: ${gen.torsion !== undefined ? 'F2[τ]/τ^' + gen.torsion : 'F2[τ] (free)'}\n`;

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

    // Bind click handlers for ASS chart
    assChart.dotCallback = handleAssDotClick;
    assChart.lineCallback = () => {}; // ASS chart has no differentials

    // Get permanent classes from synthetic category at current truncation
    // Use page=1000 (E_infinity) and allDiffs=true for ASS
    const [perm_classes, _] = get_filtered_data(
        data,
        Category.Synthetic,  // Always use synthetic category for ASS
        truncation,
        1000,  // E_infinity page
        true   // All differentials
    );

    let gens = [];

    // Display only permanent classes (survivors)
    Object.entries(perm_classes).forEach(([name, [torsion, filtration]]) => {
        if (torsion == undefined || torsion > 0) {
            let g = find(name);
            let real_g: Generators = {
                name: g.name,
                x: g.x,
                y: filtration,
                adams_filtration: filtration,
                torsion: torsion,
            }

            gens.push(real_g);
        }
    });

    assChart.set_all_generators(gens);
    assChart.init();



    // // Display multiplications only between surviving classes
    // data.multiplications.forEach((m) => {
    //     const fromAlive = perm_classes[m.from] && (perm_classes[m.from][0] == undefined || perm_classes[m.from][0] > 0);
    //     const toAlive = perm_classes[m.to] && (perm_classes[m.to][0] == undefined || perm_classes[m.to][0] > 0);
    //     if (fromAlive && toAlive) {
    //         assChart.display_mult(m.from, m.to, true);
    //     }
    // });
}
