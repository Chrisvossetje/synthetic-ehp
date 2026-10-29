import h0Text from "../../../h_0mult.txt";
import h1Text from "../../../h_1mult.txt";
import h2Text from "../../../h_2mult.txt";
import { parseMultiplicationTable } from "./multiplicationTable";

/** Parsed once when the display first requests multiplication metadata. */
export const multiplicationTables = {
    0: parseMultiplicationTable(h0Text, 0),
    1: parseMultiplicationTable(h1Text, 1),
    2: parseMultiplicationTable(h2Text, 2),
};
