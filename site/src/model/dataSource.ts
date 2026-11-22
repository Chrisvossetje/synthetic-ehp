import { Generators, SyntheticEHP } from "../types";
import { getGeneratorIndex } from "./generatorIndex";
import { loadMultiplicationTables } from "./multiplicationSource";
import { populateAlgebraicNames, setStableTagNames } from "./tagNames";

/**
 * Data source management.
 *
 * The site ships two generated datasets:
 *   - `data.ts`         — the synthetic EHP spectral sequence (the default)
 *   - `data_stable.ts`  — the stable AHSS
 *
 * The stable dataset loads once, after the initial chart renders or when the
 * user asks for it. Everything else in the app asks this module for
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
let stableDataLoadPromise: Promise<SyntheticEHP> | null = null;

/** Load the primary dataset. */
export async function initializeData() {
    if (mainData) return;
    const mainModule = await import("../data.js");
    if (!mainData) {
        populateAlgebraicNames(mainModule.data);
        mainData = mainModule.data;
    }
}

/**
 * Begin loading the stable dataset if it isn't already loaded/loading.
 * Returns a promise that resolves once the stable data and multiplication
 * tables are available. Safe to call repeatedly.
 */
export function ensureStableDataLoading(): Promise<SyntheticEHP> {
    if (stableData) {
        return Promise.resolve(stableData);
    }
    if (!stableDataLoadPromise) {
        stableDataLoadPromise = Promise.all([import("../data_stable.js"), loadMultiplicationTables("ahss")])
            .then(([stableModule]) => {
                populateAlgebraicNames(stableModule.data_stable, true);
                stableData = stableModule.data_stable;
                return stableData;
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
    const { byName, firstDuplicates } = getGeneratorIndex(data.generators);
    return firstDuplicates?.get(name) ?? byName.get(name);
}
