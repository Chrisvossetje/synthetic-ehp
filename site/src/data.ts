import { data } from "./logic";
import { Differential } from "./types";


export function add_homotoptic_information() {
    unstable_diffs();
}


const stable_diffs: Differential[] = [
    {
        from: "[15]",
        to: "5 3[6]",
        coeff: 1,
        d: 9
    },
    {
        from: "1[14]",
        to: "1 2 3 3[5]",
        coeff: 2,
        d: 8
    },
    {
        from: "1 1[13]",
        to: "4 4 1 1 1[3]",
        coeff: 2,
        d: 8
    },


    {
        from: "3 3 3[8]",
        to: "2 4 3 3 3[1]",
        coeff: 1,
        d: 7
    },


    {
        from: "5 3 3 3[4]",
        to: "1 2 4 3 3 3[1]",
        coeff: 1,
        d: 3
    },
    {
        from: "7 3 3[5]",
        to: "2 4 3 3 3[2]",
        coeff: 1,
        d: 3
    },
];

function unstable_diffs() {
    data.differentials.push(...stable_diffs);
}