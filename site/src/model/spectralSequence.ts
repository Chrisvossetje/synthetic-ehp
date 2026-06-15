import { Differential, ExternalTauMult, InternalTauMult, SyntheticEHP } from "../types";
import { MAX_STEM } from "../data";
import { Category, shouldIncludeKind, viewSettings } from "./settings";

/**
 * SPECTRAL-SEQUENCE PAGE COMPUTATION
 *
 * This module is the single place that answers the question:
 *   "Given a dataset, a category, a page, and truncations, which classes are
 *    alive and what are their (torsion, Adams-filtration) values — and which
 *    differentials have fired?"
 *
 * Previously this logic was split across two code paths (a forward state
 * machine for the synthetic category, and a separate inline loop for the
 * algebraic/geometric categories) with an awkward 8-positional-argument entry
 * point. It is now consolidated behind one function, `computePage`, that takes
 * a single `PageQuery` object and dispatches internally.
 *
 *
 * THE F2[t] STORY (synthetic category)
 *
 * We compute a spectral sequence over the polynomial ring F2[t], not just F2.
 * Generators are either free F2[t]-modules (torsion `undefined`) or torsion
 * F2[t]/(t^n) modules (torsion `n`). A differential d_r(x) = t^c * y then:
 *
 *   1. kills the source x (or raises its Adams filtration), and
 *   2. depending on the coefficient, can turn a free target y into a torsion
 *      module or change an existing torsion order.
 *
 * `mapBetweenGenerators` encodes exactly how a differential updates the
 * (af, torsion) pair of its two endpoints; `applyTauMultiplication` does the
 * same for τ-multiplications. The synthetic path runs these forward page by
 * page to build a per-generator timeline.
 */

/** `[torsion, adamsFiltration]`; `torsion === undefined` means a free module. */
export type TorsionFiltration = [number | undefined, number];

/** A fully-specified request for one page of one category. */
export interface PageQuery {
    category: Category;
    truncation: number | undefined;
    bottomTruncation: number | undefined;
    page: number;
    /** Apply external τ-multiplications at E∞ (synthetic only). Default false. */
    applyTauMults?: boolean;
    /** Restrict to generators within ±1 of this stem (used for narrow views). */
    limitX?: number | undefined;
}

/** What `computePage` returns: live classes plus the differentials so far. */
export interface PageView {
    generators: Record<string, TorsionFiltration>;
    differentials: Differential[];
}

type GeneratorState = {
    af: number;
    torsion: number | undefined;
};

type GeneratorPageState = { page: number; state: GeneratorState };

/**
 * The forward run of the synthetic SS, computed once and reused: for each
 * surviving generator, the list of states keyed by the page on which they take
 * effect, plus the complete list of differentials (tagged with their page).
 */
type SyntheticTimeline = {
    generatorNames: string[];
    pagesByGenerator: Record<string, GeneratorPageState[]>;
    allDiffs: Differential[];
};

// ---------------------------------------------------------------------------
// Torsion / filtration arithmetic (pure helpers)
// ---------------------------------------------------------------------------

function torsionAlive(torsion: number | undefined): boolean {
    return torsion !== 0;
}

function normalizeTorsion(torsion: number | null | undefined): number | undefined {
    return torsion === null ? undefined : torsion;
}

/** A class survives iff it is free, or still has positive torsion order. */
export function survivesFilteredGenerator(entry: TorsionFiltration | undefined): boolean {
    if (!entry) return false;
    return entry[0] === undefined || entry[0] > 0;
}

/**
 * Update both endpoints of a differential `from -> to`. Returns the new states,
 * or `null` if the differential cannot fire (an endpoint is dead, the
 * filtration jump is invalid, etc.). See the module header for the F2[t] rules.
 */
function mapBetweenGenerators(
    from: GeneratorState,
    to: GeneratorState
): { from: GeneratorState; to: GeneratorState; coeff: number } | null {
    if (!torsionAlive(from.torsion) || !torsionAlive(to.torsion)) {
        return null;
    }

    const coeff = to.af - from.af - 1;
    if (coeff < 0) {
        return null;
    }

    if (to.torsion !== undefined) {
        const delta = to.torsion - coeff;
        if (delta <= 0) {
            return null;
        }

        if (from.torsion !== undefined) {
            if (delta > from.torsion) {
                return null;
            }
            return {
                from: { af: from.af - delta, torsion: from.torsion - delta },
                to: { af: to.af, torsion: coeff },
                coeff,
            };
        }

        return {
            from: { af: from.af - delta, torsion: undefined },
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

/**
 * Apply a τ-multiplication `from -> to`. A τ-multiplication merges the torsion
 * orders of the two classes (the target is consumed). Returns `null` when it
 * does not apply.
 */
function applyTauMultiplication(
    from: GeneratorState,
    to: GeneratorState
): { from: GeneratorState; to: GeneratorState } | null {
    if (!torsionAlive(from.torsion) || !torsionAlive(to.torsion)) {
        return null;
    }
    if (from.torsion === undefined) {
        return null;
    }
    // The source must sit exactly one τ-tower step below the target.
    if (from.af - from.torsion !== to.af) {
        return null;
    }

    const newFromTorsion = to.torsion !== undefined ? from.torsion + to.torsion : undefined;
    return {
        from: { af: from.af, torsion: newFromTorsion },
        to: { af: to.af, torsion: 0 },
    };
}

function buildYByName(data: SyntheticEHP): Record<string, number> {
    const yByName: Record<string, number> = {};
    data.generators.forEach((g) => {
        yByName[g.name] = g.y;
    });
    return yByName;
}

/** The page a differential fires on: explicit `d`, else inferred from `y`s. */
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

/** Whether a generator passes the current truncation / stem window. */
function passesFilters(
    g: { y: number; stem: number },
    truncation: number | undefined,
    bottomTruncation: number | undefined,
    limitX: number | undefined
): boolean {
    const passesTop = !truncation || g.y <= truncation;
    const passesBottom = bottomTruncation === undefined || g.y >= bottomTruncation;
    const passesStem = !limitX || (limitX - 1 <= g.stem && g.stem <= limitX + 1);
    return passesTop && passesBottom && passesStem;
}

// ---------------------------------------------------------------------------
// Synthetic category: forward timeline + memoised cache
// ---------------------------------------------------------------------------

/** The recorded state of a generator on the requested page. */
function getStateAtPage(states: GeneratorPageState[], page: number): GeneratorState {
    let idx = 0;
    while (idx + 1 < states.length && states[idx + 1].page <= page) {
        idx += 1;
    }
    return states[idx].state;
}

function buildSyntheticTimeline(
    data: SyntheticEHP,
    truncation: number | undefined,
    bottomTruncation: number | undefined,
    limitX: number | undefined,
    applyTauMults: boolean
): SyntheticTimeline {
    const yByName = buildYByName(data);
    const pagesByGenerator: Record<string, GeneratorPageState[]> = {};
    const currentState: Record<string, GeneratorState> = {};
    const generatorNames: string[] = [];

    // Seed every surviving generator with its E1 state.
    data.generators.forEach((g) => {
        if (!passesFilters(g, truncation, bottomTruncation, limitX)) {
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

    // Bucket differentials and internal τ-multiplications by the page they act.
    const diffsByPage: Differential[][] = Array.from({ length: MAX_STEM + 1 }, () => []);
    data.differentials.forEach((diff) => {
        if (!shouldIncludeKind(diff.kind)) return;
        const diffPage = getDiffPage(diff, yByName);
        if (diffPage === undefined || !Number.isFinite(diffPage)) return;
        if (diffPage < 0 || diffPage > MAX_STEM) return;
        diffsByPage[diffPage].push(diff);
    });

    const internalTauByPage: InternalTauMult[][] = Array.from({ length: MAX_STEM + 1 }, () => []);
    data.internal_tau_mults.forEach((tm) => {
        if (!shouldIncludeKind(tm.kind)) return;
        if (tm.kind !== "Real") return;
        if (!Number.isFinite(tm.page)) return;
        if (tm.page < 0 || tm.page > MAX_STEM) return;
        internalTauByPage[tm.page].push(tm);
    });

    const externalTaus: ExternalTauMult[] = [];
    data.external_tau_mults.forEach((tm) => {
        if (!shouldIncludeKind(tm.kind)) return;
        if (tm.kind !== "Real") return;
        externalTaus.push(tm);
    });

    const allDiffs: Differential[] = [];

    // Walk the pages in order, applying τ-multiplications then differentials.
    for (let p = 0; p <= MAX_STEM; p++) {
        for (const tm of internalTauByPage[p]) {
            const fromState = currentState[tm.from];
            const toState = currentState[tm.to];
            if (!fromState || !toState) continue;
            const updated = applyTauMultiplication(fromState, toState);
            if (!updated) continue;
            currentState[tm.from] = updated.from;
            currentState[tm.to] = updated.to;
            pagesByGenerator[tm.from]?.push({ page: p, state: updated.from });
            pagesByGenerator[tm.to]?.push({ page: p, state: updated.to });
        }

        for (const diff of diffsByPage[p]) {
            const fromState = currentState[diff.from];
            const toState = currentState[diff.to];
            if (!fromState || !toState) continue;

            const mapped = mapBetweenGenerators(fromState, toState);
            if (!mapped) continue;

            // Only real/algebraic differentials actually mutate the page state;
            // others are recorded for display but don't kill classes.
            if (diff.kind == "Real" || diff.kind == "Algebraic") {
                currentState[diff.from] = mapped.from;
                currentState[diff.to] = mapped.to;
                pagesByGenerator[diff.from]?.push({ page: p + 1, state: mapped.from });
                pagesByGenerator[diff.to]?.push({ page: p + 1, state: mapped.to });
            }
            allDiffs.push({ ...diff, d: p });
        }
    }

    // External τ-multiplications resolve "at the very end" (E∞), if requested.
    if (applyTauMults) {
        for (const tm of externalTaus) {
            const fromState = currentState[tm.from];
            const toState = currentState[tm.to];
            if (!fromState || !toState) continue;
            const updated = applyTauMultiplication(fromState, toState);
            if (!updated) continue;
            currentState[tm.from] = updated.from;
            currentState[tm.to] = updated.to;
            pagesByGenerator[tm.from]?.push({ page: 500, state: updated.from });
            pagesByGenerator[tm.to]?.push({ page: 500, state: updated.to });
        }
    }

    return { generatorNames, pagesByGenerator, allDiffs };
}

/**
 * Single-slot memo for the synthetic timeline. Building it walks every page, so
 * we cache the most recent result and reuse it while the inputs are unchanged.
 * The key includes everything that affects the result — crucially including
 * `applyTauMults` and `showFakeData`, which the previous cache omitted.
 */
let timelineCache: { data: SyntheticEHP; key: string; timeline: SyntheticTimeline } | null = null;

function getSyntheticTimeline(
    data: SyntheticEHP,
    truncation: number | undefined,
    bottomTruncation: number | undefined,
    limitX: number | undefined,
    applyTauMults: boolean
): SyntheticTimeline {
    const key = `${truncation}|${bottomTruncation}|${limitX}|${applyTauMults}|${viewSettings.showFakeData}`;
    if (timelineCache && timelineCache.data === data && timelineCache.key === key) {
        return timelineCache.timeline;
    }
    const timeline = buildSyntheticTimeline(data, truncation, bottomTruncation, limitX, applyTauMults);
    timelineCache = { data, key, timeline };
    return timeline;
}

function computeSyntheticPage(data: SyntheticEHP, query: PageQuery): PageView {
    const timeline = getSyntheticTimeline(
        data,
        query.truncation,
        query.bottomTruncation,
        query.limitX,
        query.applyTauMults ?? false
    );

    const generators: Record<string, TorsionFiltration> = {};
    timeline.generatorNames.forEach((name) => {
        const states = timeline.pagesByGenerator[name];
        if (!states) return;
        const st = getStateAtPage(states, query.page);
        generators[name] = [st.torsion, st.af];
    });

    const differentials = timeline.allDiffs.filter((d) => d.d !== undefined && d.d < query.page);
    return { generators, differentials };
}

// ---------------------------------------------------------------------------
// Algebraic / geometric categories
// ---------------------------------------------------------------------------

/**
 * The algebraic and geometric views don't track F2[t] torsion towers; they just
 * start from the E1 generators (filtered per category) and let real/algebraic
 * differentials kill pairs of classes before the requested page.
 */
function computeClassicalPage(data: SyntheticEHP, query: PageQuery): PageView {
    const { category, truncation, bottomTruncation, page, limitX } = query;
    const yByName = buildYByName(data);
    const generators: Record<string, TorsionFiltration> = {};

    data.generators.forEach((g) => {
        if (!passesFilters(g, truncation, bottomTruncation, limitX)) {
            return;
        }
        if (category == Category.Algebraic) {
            // Algebraic view treats everything as free.
            generators[g.name] = [undefined, g.af];
        } else if (category == Category.Geometric) {
            // Geometric view only shows classes that are free to begin with.
            if (g.torsion == undefined) {
                generators[g.name] = [undefined, g.af];
            }
        } else {
            generators[g.name] = [g.torsion, g.af];
        }
    });

    const differentials: Differential[] = [];
    for (const diff of data.differentials) {
        if (diff.kind !== "Real" && diff.kind !== "Algebraic") {
            continue;
        }
        const diffPage = getDiffPage(diff, yByName);
        if (diffPage === undefined || !Number.isFinite(diffPage)) {
            continue;
        }
        const coeff = diff.coeff ?? 0;

        // Both endpoints must exist and the differential must fire before `page`.
        if (!generators[diff.from] || !generators[diff.to] || diffPage >= page) {
            continue;
        }

        if (category == Category.Algebraic) {
            if (diff.kind === "Algebraic") {
                generators[diff.from][0] = 0;
                generators[diff.to][0] = 0;
                differentials.push({ ...diff, d: diffPage, coeff });
            }
        } else {
            // Geometric: a class can already be dead (it happens here), so only
            // record the differential when the target is still alive.
            if (generators[diff.to][0] || generators[diff.to][0] != 0) {
                generators[diff.from][0] = 0;
                generators[diff.to][0] = 0;
                differentials.push({ ...diff, d: diffPage, coeff });
            }
        }
    }

    return { generators, differentials };
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/**
 * Compute one page of the spectral sequence. The single dispatch point for all
 * three categories — callers describe what they want with a `PageQuery` and
 * never need to know which internal algorithm runs.
 */
export function computePage(data: SyntheticEHP, query: PageQuery): PageView {
    return query.category == Category.Synthetic
        ? computeSyntheticPage(data, query)
        : computeClassicalPage(data, query);
}
