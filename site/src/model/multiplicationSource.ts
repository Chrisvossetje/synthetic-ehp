import type { MultiplicationOperator, MultiplicationTable } from "./multiplicationTable";

export type MultiplicationTables = Readonly<Record<MultiplicationOperator, MultiplicationTable>>;
let loadPromise: Promise<MultiplicationTables> | undefined;
let loadedTables: MultiplicationTables | undefined;

export function getMultiplicationTables(): MultiplicationTables | undefined {
    return loadedTables;
}

/** Lazily load all three files through esbuild's text loader; no runtime fetch. */
export function loadMultiplicationTables(): Promise<MultiplicationTables> {
    if (!loadPromise) {
        loadPromise = import("./multiplicationData.js")
            .then((module) => {
                loadedTables = module.multiplicationTables;
                return loadedTables;
            })
            .catch((error) => {
                loadPromise = undefined;
                throw error;
            });
    }
    return loadPromise;
}
