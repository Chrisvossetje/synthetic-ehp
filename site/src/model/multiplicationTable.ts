/** Display metadata only: these tables do not participate in page computation. */
export type MultiplicationOperator = 0 | 1 | 2;

export type MultiplicationResult =
    | { readonly kind: "sum"; readonly targets: readonly string[] }
    | { readonly kind: "zero" }
    | { readonly kind: "unknown" };

export interface MultiplicationCheckpoint {
    readonly index: number;
    readonly result: MultiplicationResult | undefined;
}

export interface MultiplicationTable {
    readonly operator: MultiplicationOperator;
    readonly maxIndex: number;
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
 * Parse multiplication files without resolving their tags to chart generators.
 * A result applies until the next checkpoint or the last covered index.
 * Malformed input fails with a filename and line number rather than being lost.
 */
export function parseMultiplicationTable(
    text: string,
    operator: MultiplicationOperator,
    sourceName = `h_${operator}mult.txt`,
    axis: "sphere" | "cell" = "sphere"
): MultiplicationTable {
    const headerName = `max_${axis}`;
    const headerPattern = new RegExp(`^#\\s*${headerName}\\s*=\\s*(\\d+)\\s*$`);
    const minIndex = axis === "cell" ? 0 : 1;
    let maxIndex: number | undefined;
    const checkpoints = new Map<string, MultiplicationCheckpoint[]>();
    const lineNumbers = new Map<string, number>();

    text.replace(/^\uFEFF/, "").split(/\r?\n/).forEach((rawLine, index) => {
        const line = rawLine.trim();
        if (!line) return;
        try {
            const header = headerPattern.exec(line);
            if (header) {
                if (maxIndex !== undefined) throw new Error(`Duplicate ${headerName} header`);
                maxIndex = Number(header[1]);
                if (!Number.isSafeInteger(maxIndex) || maxIndex < minIndex) {
                    throw new Error(`${headerName} must be a safe integer at least ${minIndex}`);
                }
                return;
            }
            if (line.startsWith("#") && !line.startsWith("#(")) {
                if (/^#\s*max_(sphere|cell)\b/.test(line)) throw new Error(`Expected a valid ${headerName} header`);
                return;
            }
            const row = /^#\(([^()]*)\)\s*:\s*(.+)$/.exec(line);
            if (!row) throw new Error(`Expected #(source): ${axis}=result; …`);
            const source = normalizeMultiplicationTag(row[1]);
            if (checkpoints.has(source)) throw new Error(`Duplicate source: ${source}`);
            const entries: MultiplicationCheckpoint[] = [];
            for (const part of row[2].split(";")) {
                const checkpoint = /^\s*(\d+)\s*=\s*(.*?)\s*$/.exec(part);
                if (!checkpoint) throw new Error(`Invalid checkpoint: ${part}`);
                const checkpointIndex = Number(checkpoint[1]);
                if (!Number.isSafeInteger(checkpointIndex) || checkpointIndex < minIndex) {
                    throw new Error(`Checkpoint ${axis} must be a safe integer at least ${minIndex}`);
                }
                if (entries.length && checkpointIndex <= entries[entries.length - 1].index) {
                    throw new Error(`Checkpoint ${axis}s must be strictly increasing`);
                }
                const rhs = checkpoint[2];
                let result: MultiplicationResult | undefined;
                if (rhs === (axis === "cell" ? "0" : "")) result = { kind: "zero" };
                else if (rhs === "?") result = { kind: "unknown" };
                else if (axis === "cell" && rhs === "-") result = undefined;
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
                entries.push({ index: checkpointIndex, result });
            }
            checkpoints.set(source, entries);
            lineNumbers.set(source, index + 1);
        } catch (error) {
            throw new Error(`${sourceName}:${index + 1}: ${(error as Error).message}`);
        }
    });

    if (maxIndex === undefined) throw new Error(`${sourceName}: Missing ${headerName} header`);
    for (const [source, entries] of checkpoints) {
        if (entries[entries.length - 1].index > maxIndex) {
            throw new Error(`${sourceName}:${lineNumbers.get(source)}: Checkpoint exceeds ${headerName}=${maxIndex}`);
        }
    }
    return { operator, maxIndex, checkpoints };
}

/**
 * Look up an EHP sphere or AHSS top cell. Unavailable products return undefined.
 */
export function lookupMultiplication(
    table: MultiplicationTable,
    sourceTag: string,
    index: number
): MultiplicationResult | undefined {
    if (!Number.isSafeInteger(index) || index < 0) {
        throw new Error("Lookup index must be a nonnegative safe integer");
    }
    if (index > table.maxIndex) return undefined;
    return lookupMultiplicationCheckpoint(table.checkpoints.get(normalizeMultiplicationTag(sourceTag)), index);
}

/** Read an already selected series of checkpoints within the table's coverage. */
export function lookupMultiplicationCheckpoint(
    entries: readonly MultiplicationCheckpoint[] | undefined,
    index: number
): MultiplicationResult | undefined {
    return findMultiplicationCheckpoint(entries, index)?.result;
}

export function findMultiplicationCheckpoint(
    entries: readonly MultiplicationCheckpoint[] | undefined,
    index: number
): MultiplicationCheckpoint | undefined {
    if (!entries) return undefined;
    let low = 0;
    let high = entries.length;
    while (low < high) {
        const mid = Math.floor((low + high) / 2);
        if (entries[mid].index <= index) low = mid + 1;
        else high = mid;
    }
    return entries[low - 1];
}
