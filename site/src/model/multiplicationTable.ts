/** Display metadata only: these tables do not participate in page computation. */
export type MultiplicationOperator = 0 | 1 | 2;

export type MultiplicationResult =
    | { readonly kind: "sum"; readonly targets: readonly string[] }
    | { readonly kind: "zero" }
    | { readonly kind: "unknown" };

export interface MultiplicationCheckpoint {
    readonly sphere: number;
    readonly result: MultiplicationResult;
}

export interface MultiplicationTable {
    readonly operator: MultiplicationOperator;
    readonly maxSphere: number;
    /** Keys and target terms are normalized Curtis tags, e.g. "3 2 1". */
    readonly checkpoints: ReadonlyMap<string, readonly MultiplicationCheckpoint[]>;
}

/** Accept either a plain Curtis tag or the file's `#(…)` notation. */
export function normalizeMultiplicationTag(tag: string): string {
    const trimmed = tag.trim();
    const wrapped = /^#\(([^()]*)\)$/.exec(trimmed);
    const body = wrapped ? wrapped[1].trim() : trimmed;
    if (!/^\d+(?:\s+\d+)*$/.test(body)) {
        throw new Error(`Invalid Curtis tag: ${tag}`);
    }
    return body.split(/\s+/).map((token) => {
        const value = Number(token);
        if (!Number.isSafeInteger(value)) throw new Error(`Invalid Curtis index: ${token}`);
        return String(value);
    }).join(" ");
}

/**
 * Parse the root h_0mult/h_1mult/h_2mult files without resolving their tags to
 * chart generators. A result applies from its checkpoint through the sphere
 * preceding the next checkpoint, inclusive, and never beyond max_sphere.
 * Malformed input fails with a filename and line number rather than being lost.
 */
export function parseMultiplicationTable(
    text: string,
    operator: MultiplicationOperator,
    sourceName = `h_${operator}mult.txt`
): MultiplicationTable {
    let maxSphere: number | undefined;
    const checkpoints = new Map<string, MultiplicationCheckpoint[]>();
    const lineNumbers = new Map<string, number>();

    text.replace(/^\uFEFF/, "").split(/\r?\n/).forEach((rawLine, index) => {
        const line = rawLine.trim();
        if (!line) return;
        try {
            const header = /^#\s*max_sphere\s*=\s*(\d+)\s*$/.exec(line);
            if (header) {
                if (maxSphere !== undefined) throw new Error("Duplicate max_sphere header");
                maxSphere = Number(header[1]);
                if (!Number.isSafeInteger(maxSphere) || maxSphere < 1) {
                    throw new Error("max_sphere must be a positive safe integer");
                }
                return;
            }
            if (line.startsWith("#") && !line.startsWith("#(")) {
                if (/^#\s*max_sphere\b/.test(line)) throw new Error("Invalid max_sphere header");
                return;
            }
            const row = /^#\(([^()]*)\)\s*:\s*(.+)$/.exec(line);
            if (!row) throw new Error("Expected #(source): sphere=result; …");
            const source = normalizeMultiplicationTag(row[1]);
            if (checkpoints.has(source)) throw new Error(`Duplicate source: ${source}`);
            const entries: MultiplicationCheckpoint[] = [];
            for (const part of row[2].split(";")) {
                const checkpoint = /^\s*(\d+)\s*=\s*(.*?)\s*$/.exec(part);
                if (!checkpoint) throw new Error(`Invalid checkpoint: ${part}`);
                const sphere = Number(checkpoint[1]);
                if (!Number.isSafeInteger(sphere) || sphere < 1) {
                    throw new Error("Checkpoint sphere must be a positive safe integer");
                }
                if (entries.length && sphere <= entries[entries.length - 1].sphere) {
                    throw new Error("Checkpoint spheres must be strictly increasing");
                }
                const rhs = checkpoint[2];
                let result: MultiplicationResult;
                if (rhs === "") result = { kind: "zero" };
                else if (rhs === "?") result = { kind: "unknown" };
                else {
                    const targets = rhs.split("+").map((term) => {
                        if (!/^#\([^()]*\)$/.test(term.trim())) {
                            throw new Error(`Expected a target #(tag), got: ${term}`);
                        }
                        return normalizeMultiplicationTag(term);
                    });
                    // Preserve the full expression; resolving representatives is
                    // a separate display concern, as is any algebra on sums.
                    result = { kind: "sum", targets };
                }
                entries.push({ sphere, result });
            }
            checkpoints.set(source, entries);
            lineNumbers.set(source, index + 1);
        } catch (error) {
            throw new Error(`${sourceName}:${index + 1}: ${(error as Error).message}`);
        }
    });

    if (maxSphere === undefined) throw new Error(`${sourceName}: Missing max_sphere header`);
    for (const [source, entries] of checkpoints) {
        if (entries[entries.length - 1].sphere > maxSphere) {
            throw new Error(`${sourceName}:${lineNumbers.get(source)}: Checkpoint exceeds max_sphere=${maxSphere}`);
        }
    }
    return { operator, maxSphere, checkpoints };
}

/**
 * undefined means no data (missing source, before its first checkpoint, or
 * beyond the file's coverage). It is distinct from an explicit `unknown`.
 * Sphere is the table's actual sphere number, not the chart's bracket index.
 */
export function lookupMultiplication(
    table: MultiplicationTable,
    sourceTag: string,
    sphere: number
): MultiplicationResult | undefined {
    if (!Number.isSafeInteger(sphere) || sphere < 0) {
        throw new Error("Lookup sphere must be a nonnegative safe integer");
    }
    if (sphere > table.maxSphere) return undefined;
    return lookupMultiplicationCheckpoint(table.checkpoints.get(normalizeMultiplicationTag(sourceTag)), sphere);
}

/** Read an already selected series of checkpoints within the table's coverage. */
export function lookupMultiplicationCheckpoint(
    entries: readonly MultiplicationCheckpoint[] | undefined,
    sphere: number
): MultiplicationResult | undefined {
    if (!entries) return undefined;
    // Find the final checkpoint <= sphere.
    let low = 0;
    let high = entries.length;
    while (low < high) {
        const mid = Math.floor((low + high) / 2);
        if (entries[mid].sphere <= sphere) low = mid + 1;
        else high = mid;
    }
    return low === 0 ? undefined : entries[low - 1].result;
}
