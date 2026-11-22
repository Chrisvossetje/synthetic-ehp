import h0Text from "../../../ALG_DATA/h_0mult_stable.txt";
import h1Text from "../../../ALG_DATA/h_1mult_stable.txt";
import h2Text from "../../../ALG_DATA/h_2mult_stable.txt";
import { parseMultiplicationTable } from "./multiplicationTable";

export const multiplicationTables = {
    0: parseMultiplicationTable(h0Text, 0, "h_0mult_stable.txt", "cell"),
    1: parseMultiplicationTable(h1Text, 1, "h_1mult_stable.txt", "cell"),
    2: parseMultiplicationTable(h2Text, 2, "h_2mult_stable.txt", "cell"),
} as const;
