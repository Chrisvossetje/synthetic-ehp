import type { DisplayMultiplication, Generators } from "../types";
import type { MultiplicationTables } from "./multiplicationSource";
import { lookupMultiplication, MultiplicationOperator, MultiplicationResult, MultiplicationTable, normalizeMultiplicationTag } from "./multiplicationTable";
import { algebraicMultiplicationTag } from "./tagNames";

export interface DisplayMultiplicationInfo {
    source: string;
    operator: MultiplicationOperator;
    sphere: number;
    result: MultiplicationResult | undefined;
    unresolved: string[];
    inheritedFrom?: { tag: string; sphere: number };
}

/** The leading Curtis index is stored in the EHP name's brackets. */
export function multiplicationTag(gen: Generators, sphere: number): string | undefined {
    let name = gen.name;
    let latestSphere = -Infinity;
    for (const [fromSphere, inducedName] of gen.induced_name ?? []) {
        if (fromSphere <= sphere && fromSphere >= latestSphere) {
            name = inducedName;
            latestSphere = fromSphere;
        }
    }
    const match = /^(.*?)\[(\d+)\]$/.exec(name);
    if (!match) return undefined;
    // The exporter also includes formal infinity classes such as
    // "1(∞)[2]"; these are not Curtis representatives in the tables.
    if (match[1].trim() && !/^\d+(?:\s+\d+)*$/.test(match[1].trim())) return undefined;
    return normalizeMultiplicationTag(`${match[2]} ${match[1]}`);
}

function resultKey(result: MultiplicationResult | undefined): string {
    return result?.kind === "sum" ? [...result.targets].sort().join(";") : result?.kind ?? "missing";
}

function firstTableSphere(table: MultiplicationTable, tag: string, result: MultiplicationResult): number {
    return table.checkpoints.get(tag)?.find((entry) => resultKey(entry.result) === resultKey(result))?.sphere ?? table.maxSphere;
}

/** Query at a sphere; the file's coverage limit is only an internal lookup bound. */
function productAtSphere(table: MultiplicationTable, source: Generators, sphere: number): {
    result: MultiplicationResult | undefined;
    inheritedFrom?: { tag: string; sphere: number };
} {
    const tag = multiplicationTag(source, sphere);
    const result = tag === undefined ? undefined : lookupMultiplication(table, tag, sphere);
    if (tag !== undefined && sphere <= table.maxSphere && !table.checkpoints.has(tag)) {
        // Missing full tags inherit stable tail products, keeping outer indices.
        // Explicit zero/unknown entries and uncovered checkpoints always win.
        const tokens = tag.split(" ");
        for (let start = 1; start < tokens.length; start++) {
            const tail = tokens.slice(start).join(" ");
            const stable = lookupMultiplication(table, tail, table.maxSphere);
            if (stable?.kind !== "sum") continue;
            const prefix = tokens.slice(0, start).join(" ");
            return {
                result: { kind: "sum", targets: stable.targets.map((target) => `${prefix} ${target}`) },
                inheritedFrom: { tag: tail, sphere: firstTableSphere(table, tail, stable) },
            };
        }
    }
    return { result };
}

/** Earliest support for this product with the current endpoint representatives. */
function firstSupportingSphere(
    table: MultiplicationTable, source: Generators, querySphere: number,
    result: MultiplicationResult | undefined, targets: readonly Generators[]
): number {
    if (!result) return querySphere;
    const sourceTag = multiplicationTag(source, querySphere);
    const targetTags = targets.map((target) => multiplicationTag(target, querySphere));
    const lowerBound = Math.max(1, source.born, ...targets.map((target) => target.born));
    for (let sphere = lowerBound; sphere <= Math.min(querySphere, table.maxSphere); sphere++) {
        if (multiplicationTag(source, sphere) !== sourceTag) continue;
        if (targets.some((target, index) => multiplicationTag(target, sphere) !== targetTags[index])) continue;
        if (resultKey(productAtSphere(table, source, sphere).result) === resultKey(result)) return sphere;
    }
    return querySphere;
}

/**
 * Resolve metadata against the actual visible nodes. h_0 keeps every accessible
 * summand; other operators require the complete sum. Cross-row h_0 products
 * additionally require both endpoints to survive to E-infinity. h_1/h_2
 * require two survivors even on the same row.
 * With no sphere cutoff, both charts use the final covered sphere, so later
 * checkpoints (including stable products) are visible. No tau coefficients
 * are inferred.
 */
export function resolveDisplayMultiplications(
    tables: MultiplicationTables,
    generators: readonly Generators[],
    visibleNames: ReadonlySet<string>,
    operators: readonly MultiplicationOperator[],
    sphere: number | undefined,
    mode: "ehp" | "ass",
    options: {
        survivingNames?: ReadonlySet<string>;
        /** Logical chart rows, before dot offsets (ASS uses Adams filtration). */
        rows?: ReadonlyMap<string, number>;
    } = {}
): { lines: DisplayMultiplication[]; info: DisplayMultiplicationInfo[] } {
    const lines: DisplayMultiplication[] = [];
    const info: DisplayMultiplicationInfo[] = [];
    const generatorsByName = new Map(generators.map((gen) => [gen.name, gen]));
    const survivors = options.survivingNames ?? visibleNames;
    const rowOf = (gen: Generators) => options.rows?.get(gen.name) ?? (mode === "ehp" ? gen.y : gen.af);
    const indices = new Map<number, Map<string, Generators[]>>();
    const aliasesBySphere = new Map<number, Map<string, Set<string>>>();
    const indexAt = (n: number) => {
        let index = indices.get(n);
        if (!index) {
            index = new Map();
            const aliases = new Map<string, Set<string>>();
            for (const gen of generators) {
                if (!visibleNames.has(gen.name)) continue;
                const effectiveTag = multiplicationTag(gen, n);
                if (effectiveTag === undefined) continue;
                const candidates = index.get(effectiveTag) ?? [];
                candidates.push(gen);
                index.set(effectiveTag, candidates);
                // Original names are a fallback only: an old name can now be
                // the current representative of a different visible class.
                const originalTag = multiplicationTag({ ...gen, induced_name: [] }, n);
                if (originalTag !== undefined) {
                    const names = aliases.get(originalTag) ?? new Set<string>();
                    names.add(effectiveTag);
                    aliases.set(originalTag, names);
                }
            }
            indices.set(n, index);
            aliasesBySphere.set(n, aliases);
        }
        return index;
    };

    for (const source of generators) {
        if (!visibleNames.has(source.name)) continue;
        for (const operator of operators) {
            const table = tables[operator];
            const n = sphere ?? table.maxSphere;
            // Multiplication by h_i shifts stem by 2^i - 1.
            const expectedStem = source.stem + 2 ** operator - 1;
            const eligible = (gen: Generators) => gen.stem === expectedStem
                && (mode === "ass" || (operator === 0 && rowOf(source) === rowOf(gen))
                    || (survivors.has(source.name) && survivors.has(gen.name)));
            // The added diagonal h_0 classes have no Curtis-table tag.
            // Add their products alongside, rather than replacing, table data.
            const bottomCell = operator === 0 ? /^\[(\d*[13579])\]$/.exec(source.name) : null;
            const diagonalTarget = bottomCell ? generatorsByName.get(`h_0[${bottomCell[1]}]`) : undefined;
            if (diagonalTarget) {
                const targets = [`${bottomCell![1]} h_0`];
                const support = Math.max(1, source.born, diagonalTarget.born);
                const drawn = visibleNames.has(diagonalTarget.name) && eligible(diagonalTarget);
                info.push({ source: source.name, operator, sphere: support,
                    result: { kind: "sum", targets }, unresolved: drawn ? [] : targets });
                if (drawn) {
                    lines.push({ from: source.name, to: diagonalTarget.name, operator,
                        sphere: support, targets });
                }
            }
            const product = productAtSphere(table, source, n);
            const { result } = product;
            const entry: DisplayMultiplicationInfo = { source: source.name, operator,
                sphere: firstSupportingSphere(table, source, n, result, []), ...product, unresolved: [] };
            info.push(entry);
            if (result?.kind !== "sum") continue;
            const targets: Generators[] = [];
            for (const targetTag of result.targets) {
                const index = indexAt(n);
                // Resolve among eligible endpoints first. On an early page,
                // both an old algebraic representative and the synthetic
                // survivor carrying its induced name can still be visible.
                let matches = (index.get(targetTag) ?? []).filter(eligible);
                if (!matches.length) {
                    const aliasedMatches: Generators[] = [];
                    for (const alias of aliasesBySphere.get(n)?.get(targetTag) ?? []) {
                        aliasedMatches.push(...(index.get(alias) ?? []).filter(eligible));
                    }
                    matches = [...new Set(aliasedMatches)];
                }
                if (operator === 0 && matches.length > 1) {
                    const permanentMatches = matches.filter((gen) => survivors.has(gen.name));
                    if (permanentMatches.length === 1) matches = permanentMatches;
                }
                if (matches.length !== 1) entry.unresolved.push(targetTag);
                else targets.push(matches[0]);
            }
            entry.sphere = firstSupportingSphere(table, source, n, result, targets);
            if (operator !== 0 && entry.unresolved.length) continue;
            for (const target of new Set(targets)) {
                lines.push({ from: source.name, to: target.name, operator, sphere: entry.sphere, targets: result.targets,
                    ...(entry.inheritedFrom ? { inheritedFrom: entry.inheritedFrom } : {}) });
            }
        }
    }
    return { lines, info };
}

export function formatMultiplicationResult(result: MultiplicationResult | undefined): string {
    if (!result) return "no data";
    if (result.kind === "zero") return "0";
    if (result.kind === "unknown") return "?";
    return result.targets.map(algebraicMultiplicationTag).join(" + ");
}
