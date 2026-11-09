import type { DisplayMultiplication, ExternalTauMult, Generators, InternalTauMult } from "../types";
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
    tauExtensions?: DisplayMultiplication["tauExtensions"];
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
 * Keep accessible same-row terms from every applicable checkpoint, even when
 * later checkpoints change the product. Recorded real tau extensions can also
 * retain earlier cross-row h_0 products on their visible representatives.
 * Otherwise cross-row products require two E-infinity survivors; h_1/h_2
 * additionally require the complete sum to resolve.
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
        /** Real extensions support products hidden by later algebraic zeros. */
        tauMults?: readonly (InternalTauMult | ExternalTauMult)[];
    } = {}
): { lines: DisplayMultiplication[]; info: DisplayMultiplicationInfo[] } {
    const lines: DisplayMultiplication[] = [];
    const info: DisplayMultiplicationInfo[] = [];
    const generatorsByName = new Map(generators.map((gen) => [gen.name, gen]));
    const survivors = options.survivingNames ?? visibleNames;
    const rowOf = (gen: Generators) => options.rows?.get(gen.name) ?? (mode === "ehp" ? gen.y : gen.af);
    type TauMult = InternalTauMult | ExternalTauMult;
    const tauFrom = new Map<string, TauMult[]>();
    const tauNodes = new Set<string>();
    const tauRelations: TauMult[] = [];
    for (const relation of options.tauMults ?? []) {
        const from = generatorsByName.get(relation.from);
        const to = generatorsByName.get(relation.to);
        if (relation.kind !== "Real" || !from || !to) continue;
        if (sphere !== undefined && (from.y >= sphere || to.y >= sphere)) continue;
        const outgoing = tauFrom.get(from.name) ?? [];
        outgoing.push(relation);
        tauFrom.set(from.name, outgoing);
        tauNodes.add(from.name);
        tauNodes.add(to.name);
        tauRelations.push(relation);
    }
    const tauPaths = new Map<string, Map<string, TauMult[]>>();
    const tauPath = (from: string, to: string): TauMult[] | undefined => {
        let paths = tauPaths.get(from);
        if (!paths) {
            paths = new Map([[from, []]]);
            const queue = [from];
            for (let i = 0; i < queue.length; i++) {
                for (const relation of tauFrom.get(queue[i]) ?? []) {
                    if (paths.has(relation.to)) continue;
                    paths.set(relation.to, [...paths.get(queue[i])!, relation]);
                    queue.push(relation.to);
                }
            }
            tauPaths.set(from, paths);
        }
        return paths.get(to);
    };
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
            // A later zero or changed representative does not remove the
            // multiplication on the original row while its dots remain alive.
            const original = { ...source, induced_name: [] };
            const originalTag = multiplicationTag(original, n);
            const checkpoints = originalTag === undefined ? undefined : table.checkpoints.get(originalTag);
            const rowProducts: {
                sphere: number;
                result: MultiplicationResult | undefined;
                inheritedFrom?: { tag: string; sphere: number };
            }[] = checkpoints
                ? checkpoints.filter((entry) => entry.sphere <= n)
                : [{ sphere: n, ...productAtSphere(table, original, n) }];
            for (const rowProduct of rowProducts) {
                const { result } = rowProduct;
                if (result?.kind !== "sum") continue;
                const rowTargets = result.targets.map((tag) => {
                    const [bracket, ...word] = tag.split(" ");
                    return generatorsByName.get(`${word.join(" ")}[${bracket}]`);
                });
                const targets = rowTargets.filter((target): target is Generators => !!target
                    && visibleNames.has(target.name) && target.stem === expectedStem
                    && rowOf(target) === rowOf(source));
                const proofs = new Map<string, TauMult[]>();
                if (operator === 0 && tauRelations.length) {
                    for (let i = 0; i < result.targets.length; i++) {
                        const originalTarget = rowTargets[i];
                        if (!originalTarget || rowOf(originalTarget) === rowOf(source)
                            || !tauNodes.has(originalTarget.name)) continue;
                        const candidates = (indexAt(n).get(result.targets[i]) ?? []).filter((target) =>
                            target.stem === expectedStem && rowOf(target) !== rowOf(source)
                            && tauPath(target.name, originalTarget.name) !== undefined);
                        // Prefer the representative above the other matches in
                        // the recorded tau tower, rather than the older dot.
                        const preferred = candidates.filter((candidate) => !candidates.some((other) =>
                            other !== candidate && tauPath(other.name, candidate.name) !== undefined));
                        if (preferred.length !== 1) continue;
                        const target = preferred[0];
                        const path = tauPath(target.name, originalTarget.name)!;
                        proofs.set(target.name, path.length ? path : tauRelations.filter((relation) =>
                            relation.from === target.name || relation.to === target.name));
                        targets.push(target);
                    }
                }
                if (!targets.length) continue;
                const support = checkpoints
                    ? Math.max(rowProduct.sphere, source.born, ...targets.map((target) => target.born))
                    : firstSupportingSphere(table, original, n, result, targets);
                const inheritedFrom = rowProduct.inheritedFrom;
                const proofRelations: TauMult[] = [];
                for (const path of proofs.values()) proofRelations.push(...path);
                const tauExtensions = [...new Map(proofRelations.map((relation) =>
                    [JSON.stringify([relation.from, relation.to]), relation])).values()];
                info.push({ source: source.name, operator, sphere: support, result,
                    unresolved: result.targets.filter((tag, i) => !targets.includes(rowTargets[i])
                        && !targets.some((target) => proofs.has(target.name) && multiplicationTag(target, n) === tag)),
                    ...(tauExtensions.length ? { tauExtensions } : {}),
                    ...(inheritedFrom ? { inheritedFrom } : {}) });
                for (const target of targets) {
                    lines.push({ from: source.name, to: target.name, operator, sphere: support,
                        targets: result.targets,
                        ...(proofs.has(target.name) ? { tauExtensions: proofs.get(target.name) } : {}),
                        ...(inheritedFrom ? { inheritedFrom } : {}) });
                }
            }
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
    // Prefer the current table metadata when it describes an edge already
    // retained from an earlier checkpoint; draw each edge only once.
    const uniqueLines = new Map(lines.map((line) => [JSON.stringify([line.from, line.to, line.operator]), line]));
    const uniqueInfo = new Map(info.map((entry) => [JSON.stringify([
        entry.source, entry.operator, entry.sphere, resultKey(entry.result), entry.inheritedFrom, entry.tauExtensions,
        [...entry.unresolved].sort(),
    ]), entry]));
    return { lines: [...uniqueLines.values()], info: [...uniqueInfo.values()] };
}

export function formatMultiplicationResult(result: MultiplicationResult | undefined): string {
    if (!result) return "no data";
    if (result.kind === "zero") return "0";
    if (result.kind === "unknown") return "?";
    return result.targets.map(algebraicMultiplicationTag).join(" + ");
}
