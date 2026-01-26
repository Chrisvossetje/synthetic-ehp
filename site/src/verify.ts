import { algebraic_data } from "./algebraic_data";
import { Category, data, get_filtered_data } from "./logic";
import { Differential, SyntheticEHP } from "./types";



export function verify_integrity(data: SyntheticEHP): boolean {
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

        let from_af = data.generators.find((p) => {return p.name == diff.from}).adams_filtration;
        let to_af = data.generators.find((p) => {return p.name == diff.to}).adams_filtration;

        if (to_af != from_af + diff.coeff + 1) {
            console.error()
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

const stable_gens = [
    1,1,1,3,
    0,0,1,4,
    2,3,1,3,
    0,0,2,6,
    2,4,4,4,
    3,2,2,8,
    2,2,2,3,
    1,0,1,8,
    4,5,5,5,
];

const alg_stable_gens = [
    1,1,1,3,
    0,0,1,4,
    2,3,1,3,
    0,0,5,9,
    3,7,6,4,
    3,2,4,10,
    3,6,5,3,
    3,3,13,22,
    8,10,11,7,
];



function generate_algebraic_stems() {

}



export function verify_stable() {
    const [gens, __] = get_filtered_data(data, true, Category.Synthetic, undefined, 1000, true);

    // const [alg_gens, _] = get_filtered_data(algebraic_data, true, Category.Synthetic, undefined, 1000, true);
    
    const MAX_STEM = 34;

    const count_gens = Array(MAX_STEM + 1).fill(0);
    const alg_count_gens = Array(MAX_STEM + 1).fill(0);

    Object.entries(gens).forEach(([name, [torsion, filtration]]) => {
        const real_gen = data.generators.find((p) => {return p.name == name;});
        if (torsion == undefined) {
            count_gens[real_gen.x] += 1;
            alg_count_gens[real_gen.x] += 1;
        } else if (torsion > 0) {
            // TODO: Check filtration ?
            alg_count_gens[real_gen.x] += 1;
            alg_count_gens[real_gen.x+1] += 1;
        }
    });

    for (let i = MAX_STEM; i >= 0; i--) {
        if (count_gens[i] !== stable_gens[i]) {
            console.error(`Classical stable stem in degree ${i} do not agree. We have ${count_gens[i]} and expect ${stable_gens[i]}`);
        }
        if (alg_count_gens[i] !== alg_stable_gens[i]) {
            console.error(`Algebraic stable stem in degree ${i} do not agree. We have ${alg_count_gens[i]} and expect ${alg_stable_gens[i]}`);
        }
    }   
}