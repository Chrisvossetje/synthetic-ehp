import { Generators, SyntheticEHP } from "../types";
import { setStableTagNames, withAlgebraicNames } from "./tagNames";

/**
 * Data source management.
 *
 * The site ships two generated datasets:
 *   - `data.ts`         — the synthetic EHP spectral sequence (the default)
 *   - `data_stable.ts`  — the stable AHSS
 *
 * Both are large, so the stable dataset is loaded lazily (and only once) the
 * first time it is needed. Everything else in the app asks this module for
 * "the active data" rather than importing the datasets directly, which keeps
 * the EHP/AHSS switch a one-line state change.
 */

let useStableData = false;
export function isUsingStableData() { return useStableData; }
export function setUseStableData(value: boolean) {
    useStableData = value;
    setStableTagNames(value);
}

let mainData: SyntheticEHP | null = null;
let stableData: SyntheticEHP | null = null;
let stableDataLoadPromise: Promise<void> | null = null;
const generatorsByDataset = new WeakMap<SyntheticEHP, Map<string, Generators>>();

/** Load the primary dataset. */
export async function initializeData() {
    const mainModule = await import("../data.js");
    mainData = withAlgebraicNames(mainModule.data);
}

/**
 * Begin loading the stable dataset if it isn't already loaded/loading.
 * Returns a promise that resolves once `stableData` is available. Safe to call
 * repeatedly — the import only happens once.
 */
export function ensureStableDataLoading(): Promise<void> {
    if (stableData) {
        return Promise.resolve();
    }
    if (!stableDataLoadPromise) {
        stableDataLoadPromise = import("../data_stable.js")
            .then((stableModule) => {
                stableData = withAlgebraicNames(stableModule.data_stable, true);
            })
            .catch((error) => {
                // Allow a later retry if the import failed.
                stableDataLoadPromise = null;
                throw error;
            });
    }
    return stableDataLoadPromise;
}

export function isStableDataReady(): boolean {
    return stableData !== null;
}

/** The dataset currently selected by the EHP/AHSS switch. */
export function getActiveData(): SyntheticEHP | null {
    return useStableData ? stableData : mainData;
}

export function getMainData(): SyntheticEHP | null {
    return mainData;
}

/** Look up a generator by name in the active dataset. */
export function find(name: string): Generators | undefined {
    const data = getActiveData();
    if (!data) return undefined;
    let generators = generatorsByDataset.get(data);
    if (!generators) {
        generators = new Map();
        for (const generator of data.generators) {
            if (!generators.has(generator.name)) generators.set(generator.name, generator);
        }
        generatorsByDataset.set(data, generators);
    }
    return generators.get(name);
}
