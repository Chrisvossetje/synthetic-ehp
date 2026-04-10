import { Differential, ExternalTauMult, Generators, InternalTauMult, SyntheticEHP } from "./types";
import { MAX_STEM } from "./data";

// Track which data is active
let useStableData = false;
export function isUsingStableData() { return useStableData; }
export function setUseStableData(value: boolean) { useStableData = value; }

let mainData: SyntheticEHP | null = null;
let stableData: SyntheticEHP | null = null;
let stableDataLoadPromise: Promise<void> | null = null;

export async function initializeData() {
    const mainModule = await import("./data");
    mainData = mainModule.data;
    ensureStableDataLoading();
}

export function ensureStableDataLoading(): Promise<void> {
    if (stableData) {
        return Promise.resolve();
    }
    if (!stableDataLoadPromise) {
        stableDataLoadPromise = import("./data_stable")
            .then((stableModule) => {
                stableData = stableModule.data_stable;
            })
            .catch((error) => {
                stableDataLoadPromise = null;
                throw error;
            });
    }
    return stableDataLoadPromise;
}

export function isStableDataReady(): boolean {
    return stableData !== null;
}

export function getActiveData(): SyntheticEHP | null {
    if (useStableData) {
        return stableData;
    }
    return mainData;
}

export function getMainData(): SyntheticEHP | null {
    return mainData;
}

/*
 * SPECTRAL SEQUENCE OVER F2[t]
 *
 * We are computing a spectral sequence over the polynomial ring F2[t], not just F2.
 * This means generators can be:
 *   - Free F2[t]-modules (torsion = undefined)
 *   - Torsion F2[t]/(t^n) modules (torsion = n)
 *
 * DIFFERENTIAL BEHAVIOR:
 * When a differential d_r(x) = coeff * y occurs:
 *
 * 1. The SOURCE (x) DIES on the E_{r+1} page if y is torsion-free, else its adams filtration jumps
 *      (We assume that our source is always a free F2[t]-module, but this is not checked yet)
 *
 * 2. The TARGET (y) behavior depends on the coefficient:
 *    - If coeff involves t (e.g., t, t^2, etc.) and y was FREE:
 *      → y SURVIVES but becomes TORSION
 *      → y becomes an F2[t]/(t) module (killed by multiplication by t)
 *    - If y was already torsion F2[t]/(t^n), it may die or change torsion
 *
 * Example: If d(x) = t*y where x,y are free modules:
 *   - Page r+1: x is dead, y survives but is now torsion (killed by t)
 *
 * This tracks how algebraic torsion propagates through the spectral sequence.
 */

export enum Category {
    Synthetic,
    Algebraic,
    Geometric
}

type GeneratorState = {
    af: number;
    torsion: number | undefined;
};

type TorsionFiltration = [number | undefined, number];
type GeneratorPageState = { page: number; state: GeneratorState };
type SyntheticCache = {
    data: SyntheticEHP;
    truncation: number | undefined;
    bottomTruncation: number | undefined;
    limit_x: number | undefined;
    showFakeData: boolean;
    pagesByGenerator: Record<string, GeneratorPageState[]>;
    generatorNames: string[];
    allDiffs: Differential[];
};

let syntheticCache: SyntheticCache | null = null;

function torsionAlive(torsion: number | undefined): boolean {
    return torsion !== 0;
}

function normalizeTorsion(torsion: number | null | undefined): number | undefined {
    return torsion === null ? undefined : torsion;
}

function mapBetweenGenerators(from: GeneratorState, to: GeneratorState): { from: GeneratorState; to: GeneratorState; coeff: number } | null {
    if (!torsionAlive(from.torsion)) {
        return null;
    }
    if (!torsionAlive(to.torsion)) {
        return null;
    }

    const coeff = to.af - from.af - 1;
    if (coeff < 0) {
        return null;
    }

    if (to.torsion !== undefined) {
        if (from.torsion !== undefined) {
            const delta = to.torsion - coeff;
            if (delta > from.torsion) {
                return null;
            }
            const newFromAf = from.af - delta;
            const newFromTorsion = from.torsion - delta;
            return {
                from: { af: newFromAf, torsion: newFromTorsion },
                to: { af: to.af, torsion: coeff },
                coeff,
            };
        }

        const newFromAf = from.af - to.torsion + coeff;
        return {
            from: { af: newFromAf, torsion: undefined },
            to: { af: to.af, torsion: coeff },
            coeff,
        };
    }

    if (from.torsion !== undefined) {
        return null;
    }

    return {
        from: { af: from.af, torsion: 0 },
        to: { af: to.af, torsion: coeff },
        coeff,
    };
}

function generatorSurvives(entry: TorsionFiltration | undefined): boolean {
    if (!entry) {
        return false;
    }
    return entry[0] === undefined || entry[0] > 0;
}

function applyTauMultiplication(
    from: GeneratorState,
    to: GeneratorState,
    matchesFiltration: (fromAf: number, fromTorsion: number, toAf: number) => boolean
): { from: GeneratorState; to: GeneratorState } | null {
    if (!torsionAlive(from.torsion) || !torsionAlive(to.torsion)) {
        return null;
    }

    if (from.torsion === undefined) {
        return null;
    }

    if (!matchesFiltration(from.af, from.torsion, to.af)) {
        return null;
    }

    const newFromTorsion = to.torsion !== undefined ? from.torsion + to.torsion : undefined;
    return {
        from: { af: from.af, torsion: newFromTorsion },
        to: { af: to.af, torsion: 0 },
    };
}

function applyInternalTauMultiplication(from: GeneratorState, to: GeneratorState): { from: GeneratorState; to: GeneratorState } | null {
    return applyTauMultiplication(from, to, (fromAf, fromTorsion, toAf) => fromAf - fromTorsion === toAf);
}

function applyExternalTauMultiplication(from: GeneratorState, to: GeneratorState): { from: GeneratorState; to: GeneratorState } | null {
    return applyTauMultiplication(from, to, (fromAf, fromTorsion, toAf) => fromAf - fromTorsion === toAf);
}

function buildYByName(data: SyntheticEHP): Record<string, number> {
    const yByName: Record<string, number> = {};
    data.generators.forEach((g) => {
        yByName[g.name] = g.y;
    });
    return yByName;
}

function shouldIncludeKind(kind: "Real" | "Fake" | "Unknown", data: SyntheticEHP): boolean {
    return viewSettings.showFakeData || kind !== "Fake";
}

function getDiffPage(diff: Differential, yByName: Record<string, number>): number | undefined {
    if (Number.isFinite(diff.d)) {
        return diff.d;
    }
    const fromY = yByName[diff.from];
    const toY = yByName[diff.to];
    if (!Number.isFinite(fromY) || !Number.isFinite(toY)) {
        return undefined;
    }
    return fromY - toY;
}

function getStateAtPage(states: GeneratorPageState[], page: number): GeneratorState {
    let idx = 0;

    while (idx + 1 < states.length && states[idx + 1].page <= page) {
        idx += 1;
    }

    return states[idx].state;
}

function buildSyntheticCache(
    data: SyntheticEHP,
    truncation: number | undefined,
    bottomTruncation: number | undefined,
    applyTauMults: boolean,
    limit_x: number | undefined
): SyntheticCache {
    const yByName = buildYByName(data);
    const pagesByGenerator: Record<string, GeneratorPageState[]> = {};
    const currentState: Record<string, GeneratorState> = {};
    const generatorNames: string[] = [];

    data.generators.forEach((g) => {
        const passesTop = !truncation || g.y <= truncation;
        const passesBottom = bottomTruncation === undefined || g.y >= bottomTruncation;
        const passesStem = !limit_x || (limit_x - 1 <= g.stem && g.stem <= limit_x + 1);
        if (!passesTop || !passesBottom || !passesStem) {
            return;
        }
        const initialTorsion = normalizeTorsion(g.torsion);
        if (initialTorsion === 0) {
            return;
        }
        const initialState = { af: g.af, torsion: initialTorsion };
        pagesByGenerator[g.name] = [{ page: 1, state: initialState }];
        currentState[g.name] = { ...initialState };
        generatorNames.push(g.name);
    });

    const diffsByPage: Differential[][] = Array.from({ length: MAX_STEM + 1 }, () => []);
    data.differentials.forEach((diff) => {
        if (!shouldIncludeKind(diff.kind, data)) return;
        // if (diff.kind !== "Real") return;
        const diffPage = getDiffPage(diff, yByName);
        if (!Number.isFinite(diffPage)) return;
        if (diffPage < 0 || diffPage > MAX_STEM) return;
        diffsByPage[diffPage].push(diff);
    });
    const internalTauByPage: InternalTauMult[][] = Array.from({ length: MAX_STEM + 1 }, () => []);
    data.internal_tau_mults.forEach((tm) => {
        if (!shouldIncludeKind(tm.kind, data)) return;
        if (tm.kind !== "Real") return;
        if (!Number.isFinite(tm.page)) return;
        if (tm.page < 0 || tm.page > MAX_STEM) return;
        internalTauByPage[tm.page].push(tm);
    });

    const externalTaus: ExternalTauMult[] = [];
    data.external_tau_mults.forEach((tm) => {
        if (!shouldIncludeKind(tm.kind, data)) return;
        if (tm.kind !== "Real") return;
        externalTaus.push(tm);
    });

    const allDiffs: Differential[] = [];

    for (let p = 0; p <= MAX_STEM; p++) {
        const pageTauMults = internalTauByPage[p];
        for (const tm of pageTauMults) {
            const fromState = currentState[tm.from];
            const toState = currentState[tm.to];
            if (!fromState || !toState) {
                continue;
            }
            const updated = applyInternalTauMultiplication(fromState, toState);
            if (!updated) {
                continue;
            }
            currentState[tm.from] = updated.from;
            currentState[tm.to] = updated.to;
            pagesByGenerator[tm.from]?.push({ page: p, state: updated.from });
            pagesByGenerator[tm.to]?.push({ page: p, state: updated.to });
        }

        const pageDiffs = diffsByPage[p];
        for (const diff of pageDiffs) {
            const fromState = currentState[diff.from];
            const toState = currentState[diff.to];
            if (!fromState || !toState) {
                continue;
            }

            const mapped = mapBetweenGenerators(fromState, toState);
            if (!mapped) {
                continue;
            }

            if (diff.kind == "Real") {
                currentState[diff.from] = mapped.from;
                currentState[diff.to] = mapped.to;
                pagesByGenerator[diff.from]?.push({ page: p + 1, state: mapped.from });
                pagesByGenerator[diff.to]?.push({ page: p + 1, state: mapped.to });
            }
            allDiffs.push({ ...diff, d: p });
        }
    }

    if (applyTauMults) {
        for (const tm of externalTaus) {
            const fromState = currentState[tm.from];
            const toState = currentState[tm.to];
            if (!fromState || !toState) {
                continue;
            }

            const updated = applyInternalTauMultiplication(fromState, toState);
            if (!updated) {
                continue;
            }

            currentState[tm.from] = updated.from;
            currentState[tm.to] = updated.to;
            pagesByGenerator[tm.from]?.push({ page: 500, state: updated.from });
            pagesByGenerator[tm.to]?.push({ page: 500, state: updated.to });
        }
    }

    return {
        data,
        truncation,
        bottomTruncation,
        limit_x,
        showFakeData: viewSettings.showFakeData,
        pagesByGenerator,
        generatorNames,
        allDiffs,
    };
}

function getSyntheticCache(
    data: SyntheticEHP,
    truncation: number | undefined,
    bottomTruncation: number | undefined,
    applyTauMults: boolean, 
    limit_x: number | undefined
): SyntheticCache {
    if (
        syntheticCache &&
        syntheticCache.data === data &&
        syntheticCache.truncation === truncation &&
        syntheticCache.bottomTruncation === bottomTruncation &&
        syntheticCache.limit_x === limit_x &&
        syntheticCache.showFakeData === viewSettings.showFakeData
    ) {
        return syntheticCache;
    }

    syntheticCache = buildSyntheticCache(data, truncation, bottomTruncation, applyTauMults, limit_x);
    return syntheticCache;
}

function computeSyntheticPages(
    data: SyntheticEHP,
    truncation: number | undefined,
    page: number,
    limit_x: number | undefined,
    applyTauMults: boolean,
    bottomTruncation: number | undefined
): [Record<string, TorsionFiltration>, Differential[]] {
    const cache = getSyntheticCache(data, truncation, bottomTruncation, applyTauMults, limit_x);
    const torsion: Record<string, TorsionFiltration> = {};
    cache.generatorNames.forEach((name) => {
        const states = cache.pagesByGenerator[name];
        if (!states) {
            return;
        }
        const st = getStateAtPage(states, page);
        torsion[name] = [st.torsion, st.af];
    });

    const diffs = cache.allDiffs.filter((d) => d.d < page);
    return [torsion, diffs];
}

// Current view settings
export let viewSettings = {
    allDiffs: true,
    page: 1,
    category: Category.Synthetic, // 0: Synthetic, 1: Algebraic, 2: Geometric
    truncation: undefined as number | undefined,
    bottomTruncation: undefined as number | undefined,
    showFakeData: true
};

// Shared selection across all modes/data sources.
let selectedGeneratorName: string | null = null;
export function setSelectedGenerator(name: string | null) { selectedGeneratorName = name; }
export function getSelectedGenerator() { return selectedGeneratorName; }

export function find(name: string): Generators | undefined {
    const activeData = getActiveData();
    if (!activeData) return undefined;
    return activeData.generators.find(g => g.name === name);
}

export function generated_by_name(gen: Generators): string {
    const initial = gen.name.split("[")[0];

    const split_first = initial.split(' ');
    const end = "[" + String(split_first[0]) + "]";
    if (split_first.length == 1) {
        return end;
    } else {
        return split_first.slice(1).join(" ") + end;
    }
}

export function generating_name(gen: Generators): string {
    const [initial, last] = gen.name.split("[");
    const real_last = last.split("]")[0];
    if (initial == "") {
        return real_last;
    } else {
        return real_last + " " + initial;
    }
}

function parseSphereFromName(name: string): number | undefined {
    const match = name.match(/\[(\d+)\]$/);
    if (!match) return undefined;
    return parseInt(match[1], 10);
}

export function get_induced_name(gen: Generators, sphere: number): string {
    let l = gen.induced_name;
    let id = 0; 

    while (true) {
        if (id + 1 == l.length) {
            return l[id][1];
        }
        if (l[id+1][0] > sphere) {
            return l[id][1];
        } 
        id += 1;
    }
}

export function getSphereLifecycleInfo(gen: Generators): { bornSphere: string; diesOnAlgebraicSphere: string } {
    const explicitBorn = (gen as any).born;
    const explicitDies = (gen as any).dies;

    if (explicitDies === null || explicitDies === undefined) {
        return {
            bornSphere: explicitBorn !== undefined ? String(explicitBorn) : "Unknown",
            diesOnAlgebraicSphere: "survives"
        };
    }

    const inducedSpheres = gen.induced_name
        .map(([sphere]) => sphere)
        .filter((sphere) => sphere > 0);

    const parsedSphere = parseSphereFromName(gen.name);
    const born = explicitBorn !== undefined
        ? explicitBorn
        : inducedSpheres.length > 0
        ? Math.min(...inducedSpheres)
        : parsedSphere;
    const maxPresent = explicitDies !== undefined
        ? explicitDies - 1
        : inducedSpheres.length > 0
        ? Math.max(...inducedSpheres)
        : parsedSphere;
    const dies = maxPresent !== undefined ? maxPresent + 1 : undefined;

    return {
        bornSphere: born !== undefined ? born.toString() : "Unknown",
        diesOnAlgebraicSphere: dies !== undefined ? dies.toString() : "Unknown"
    };
}


export function generates(gen: Generators): Generators[] {
    let name = generating_name(gen);
    const activeData = getActiveData();
    if (!activeData) {
        return [];
    }

    const escapedName = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const re = new RegExp(`^${escapedName}\\[(\\d+)\\]$`);
    return activeData.generators
        .map((g) => {
            const match = re.exec(g.name);
            return match ? { g, idx: parseInt(match[1], 10) } : null;
        })
        .filter((entry): entry is { g: Generators; idx: number } => entry !== null)
        .sort((a, b) => a.idx - b.idx)
        .map((entry) => entry.g);
}

/**
 * Get the filtered view based on current settings
 */
export function get_filtered_data(
    data: SyntheticEHP,
    category: Category,
    truncation: number | undefined,
    page: number,
    allDiffs: boolean,
    limit_x?: number,
    applyTauMults: boolean = false,
    bottomTruncation: number | undefined = undefined
): [Object, Differential[]] {
    if (category == Category.Synthetic) {
        return computeSyntheticPages(data, truncation, page, limit_x, applyTauMults, bottomTruncation);
    }

    const yByName = buildYByName(data);

    // name -> torsion + adams filtration
    const torsion = new Object();
    const original_af = new Object();

    data.generators.forEach((g) => {
        original_af[g.name] = g.af;

        const passesTop = !truncation || g.y <= truncation;
        const passesBottom = bottomTruncation === undefined || g.y >= bottomTruncation;
        if (passesTop && passesBottom && ((limit_x - 1 <= g.stem && g.stem <= limit_x + 1) || !limit_x)) {
            if (category == Category.Algebraic) { // Special Algebraic
                torsion[g.name] = [undefined, g.af];
            }
            else if (category == Category.Geometric) { // Geometric
                if (g.torsion == undefined) {
                    torsion[g.name] = [undefined, g.af];
                }
            } else {
                torsion[g.name] = [g.torsion, g.af];
            }
        }
    });

    let diffs = [];

    // Find all generators killed by differentials before this page
    for (const diff of data.differentials) {
        if (!shouldIncludeKind(diff.kind, data)) {
            continue;
        }
        const diffPage = getDiffPage(diff, yByName);
        if (!Number.isFinite(diffPage)) {
            continue;
        }
        const coeff = diff.coeff ?? 0;

        // Make sure the elements exist
        if (torsion[diff.from] && torsion[diff.to]) {

            // Only calculate diffs which would have elemented before
            if (diffPage < page) {
                // Algebraic
                if (category == Category.Algebraic) { 
                    if (coeff == 0 && diff.proof === undefined) {
                        torsion[diff.from][0] = 0;
                        torsion[diff.to][0] = 0;
                        diffs.push({ ...diff, d: diffPage, coeff });              
                    }
                    
                    
                    
                    // Geometric
                } else { 
                    if (diff.kind == "Real") {
                        if (torsion[diff.to][0] || torsion[diff.to][0] != 0) {
                            torsion[diff.from][0] = 0;
                            torsion[diff.to][0] = 0;  
                            diffs.push({ ...diff, d: diffPage, coeff });                  
                        } else {
                            // Element had already been killed 
                            // This can occur in geometric !
                        }               
                    } else {
                        diffs.push({ ...diff, d: diffPage, coeff });                  
                    }
                }
            }
        }
    }

    return [torsion, diffs]
}

export function survivesFilteredGenerator(entry: TorsionFiltration | undefined): boolean {
    return generatorSurvives(entry);
}
