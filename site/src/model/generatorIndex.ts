import type { Generators } from "../types";

type GeneratorIndex = {
    byName: ReadonlyMap<string, Generators>;
    firstDuplicates?: ReadonlyMap<string, Generators>;
};

const indices = new WeakMap<readonly Generators[], GeneratorIndex>();

/** Share the name lookup across page computation, chart rendering and products. */
export function getGeneratorIndex(generators: readonly Generators[]): GeneratorIndex {
    const cached = indices.get(generators);
    if (cached) return cached;

    const byName = new Map<string, Generators>();
    let firstDuplicates: Map<string, Generators> | undefined;
    for (const generator of generators) {
        const previous = byName.get(generator.name);
        if (previous) {
            firstDuplicates ??= new Map();
            if (!firstDuplicates.has(generator.name)) firstDuplicates.set(generator.name, previous);
        }
        byName.set(generator.name, generator);
    }
    const index = { byName, firstDuplicates };
    indices.set(generators, index);
    return index;
}
