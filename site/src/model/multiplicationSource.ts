import type { MultiplicationOperator, MultiplicationTable } from "./multiplicationTable";

export type MultiplicationTables = Readonly<Record<MultiplicationOperator, MultiplicationTable>>;
type MultiplicationDataset = "ehp" | "ahss";

const importTables = {
    ehp: () => import("./multiplicationData.js"),
    ahss: () => import("./ahssMultiplicationData.js"),
};
const loadPromises: Partial<Record<MultiplicationDataset, Promise<MultiplicationTables>>> = {};
const loadedTables: Partial<Record<MultiplicationDataset, MultiplicationTables>> = {};

export function getMultiplicationTables(dataset: MultiplicationDataset): MultiplicationTables | undefined {
    return loadedTables[dataset];
}

/** Lazily load the three tables for the requested dataset. */
export function loadMultiplicationTables(dataset: MultiplicationDataset = "ehp"): Promise<MultiplicationTables> {
    return loadPromises[dataset] ??= importTables[dataset]()
        .then(({ multiplicationTables }) => {
            loadedTables[dataset] = multiplicationTables;
            return multiplicationTables;
        })
        .catch((error) => {
            delete loadPromises[dataset];
            throw error;
        });
}
