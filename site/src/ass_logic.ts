import { Chart } from "./chart";
import { Category, get_filtered_data } from "./logic";
import { data } from "./data";
import { assChart } from "./main";

/**
 * ASS (Adams Spectral Sequence) Chart Logic
 *
 * The ASS chart displays permanent classes (survivors) from the synthetic EHP spectral sequence.
 * For each truncation level, the ASS chart shows what survives to E_infinity in the synthetic category.
 *
 * ASS chart always shows E_infinity page with all differentials resolved.
 */

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
    // Hide all generators and differentials first
    data.generators.forEach((g) => {
        assChart.display_dot(g.name, false, false, undefined, g.adams_filtration);
    });
    data.differentials.forEach((d) => {
        assChart.display_diff(d.from, d.to, false);
    });
    data.multiplications.forEach((m) => {
        assChart.display_mult(m.from, m.to, false);
    });

    // Get permanent classes from synthetic category at current truncation
    // Use page=1000 (E_infinity) and allDiffs=true for ASS
    const [perm_classes, _] = get_filtered_data(
        data,
        true,  // perm_classes = true
        Category.Synthetic,  // Always use synthetic category for ASS
        truncation,
        1000,  // E_infinity page
        true   // All differentials
    );

    // Display only permanent classes (survivors)
    Object.entries(perm_classes).forEach(([name, [torsion, filtration]]) => {
        if (torsion == undefined || torsion > 0) {
            // In ASS, all displayed classes are permanent (survived to E_infinity)
            assChart.display_dot(name, true, true, torsion, filtration);
        }
    });


    // Display multiplications only between surviving classes
    data.multiplications.forEach((m) => {
        const fromAlive = perm_classes[m.from] && (perm_classes[m.from][0] == undefined || perm_classes[m.from][0] > 0);
        const toAlive = perm_classes[m.to] && (perm_classes[m.to][0] == undefined || perm_classes[m.to][0] > 0);
        if (fromAlive && toAlive) {
            assChart.display_mult(m.from, m.to, true);
        }
    });
}
