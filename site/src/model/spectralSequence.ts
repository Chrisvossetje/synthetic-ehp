import { Differential, Generators, InternalTauMult, SyntheticEHP } from "../types";
import { MAX_STEM } from "../data";
import { getGeneratorIndex } from "./generatorIndex";
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
    /** First page on which a generator is absent after dying. */
    deathPages: ReadonlyMap<string, number>;
}

type GeneratorState = {
    readonly af: number;
    readonly torsion?: number;
};

type GeneratorPageState = { page: number; state: GeneratorState };
const EXTERNAL_TAU_PAGE = 500;

/**
 * The forward run of the synthetic SS, computed once and reused: for each
 * surviving generator, the list of states keyed by the page on which they take
 * effect, plus the complete list of differentials (tagged with their page).
 */
type SyntheticTimeline = {
    generators: ReadonlyMap<string, Generators>;
    changesByGenerator: Map<string, GeneratorPageState[]>;
    allDiffs: Differential[];
    deathPages: Map<string, number>;
    external?: { states: Map<string, GeneratorState>; deaths: string[] };
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
 * Apply Rust's τ-tower extension rule, retaining any overlap in the target.
 * Returns `null` when the extension does not apply.
 */
function applyTauMultiplication(
    from: GeneratorState,
    to: GeneratorState,
    af: number = 0
): { from: GeneratorState; to: GeneratorState } | null {
    if (!torsionAlive(from.torsion) || !torsionAlive(to.torsion)) {
        return null;
    }
    if (from.torsion === undefined || from.af < af) {
        return null;
    }
    const afDiff = from.af - to.af;
    if (afDiff < 0 || from.torsion < afDiff) {
        return null;
    }
    // A target already contained in the source's tower cannot extend it.
    if (to.torsion !== undefined && from.torsion >= to.torsion + afDiff) {
        return null;
    }

    const newFromTorsion = to.torsion !== undefined ? to.torsion + afDiff : undefined;
    return {
        from: { af: from.af, torsion: newFromTorsion },
        to: { af: to.af, torsion: from.torsion - afDiff },
    };
}

type PageData = {
    sortedDifferentials: { diff: Differential; page: number }[];
    internalTauByPage: InternalTauMult[][];
};

const pageDataCache = new WeakMap<SyntheticEHP, PageData>();

function getPageData(data: SyntheticEHP): PageData {
    const cached = pageDataCache.get(data);
    if (cached) return cached;
    const { byName } = getGeneratorIndex(data.generators);
    const af = (name: string) => byName.get(name)?.af ?? Infinity;
    const sortedDifferentials: PageData["sortedDifferentials"] = [];
    for (const diff of data.differentials) {
        const page = getDiffPage(diff, byName);
        if (page !== undefined && Number.isFinite(page)) sortedDifferentials.push({ diff, page });
    }
    // Within each page, process lower source AF first, then lower target AF.
    sortedDifferentials.sort((a, b) =>
        a.page - b.page || af(a.diff.from) - af(b.diff.from) || af(a.diff.to) - af(b.diff.to)
    );
    const internalTauByPage: InternalTauMult[][] = Array.from({ length: MAX_STEM + 1 }, () => []);
    for (const tm of data.internal_tau_mults) {
        if (tm.kind === "Real" && Number.isInteger(tm.page) && tm.page >= 0 && tm.page <= MAX_STEM) {
            internalTauByPage[tm.page].push(tm);
        }
    }
    const pageData = { sortedDifferentials, internalTauByPage };
    pageDataCache.set(data, pageData);
    return pageData;
}

/** The page a differential fires on: explicit `d`, else inferred from `y`s. */
function getDiffPage(diff: Differential, byName: ReadonlyMap<string, Generators>): number | undefined {
    if (Number.isFinite(diff.d)) {
        return diff.d;
    }
    const fromY = byName.get(diff.from)?.y;
    const toY = byName.get(diff.to)?.y;
    if (fromY === undefined || toY === undefined || !Number.isFinite(fromY) || !Number.isFinite(toY)) {
        return undefined;
    }
    return fromY - toY;
}

export function getDifferentialPage(data: SyntheticEHP, diff: Differential): number | undefined {
    return Number.isFinite(diff.d) ? diff.d : getDiffPage(diff, getGeneratorIndex(data.generators).byName);
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
function getStateAtPage(
    timeline: SyntheticTimeline, generator: Generators, page: number, externalState?: GeneratorState
): GeneratorState {
    const changes = timeline.changesByGenerator.get(generator.name);
    let state: GeneratorState = generator;
    if (changes) {
        for (const change of changes) {
            if (!(change.page <= page)) return state;
            state = change.state;
        }
    }
    return externalState ?? state;
}

function getLatestState(timeline: SyntheticTimeline, name: string): GeneratorState | undefined {
    const changes = timeline.changesByGenerator.get(name);
    if (changes) return changes[changes.length - 1].state;
    const generator = timeline.generators.get(name);
    // Older generated data can explicitly use null for a free module.
    return generator?.torsion === null ? { af: generator.af, torsion: undefined } : generator;
}

function selectTimelineGenerators(
    data: SyntheticEHP, truncation: number | undefined, bottomTruncation: number | undefined, limitX: number | undefined
): ReadonlyMap<string, Generators> {
    if (!truncation && bottomTruncation === undefined && !limitX && !data.generators.some((g) => g.torsion === 0)) {
        return getGeneratorIndex(data.generators).byName;
    }
    const generators = new Map<string, Generators>();
    for (const generator of data.generators) {
        if (generator.torsion !== 0 && passesFilters(generator, truncation, bottomTruncation, limitX)) {
            generators.set(generator.name, generator);
        }
    }
    return generators;
}

function buildSyntheticTimeline(
    data: SyntheticEHP,
    truncation: number | undefined,
    bottomTruncation: number | undefined,
    limitX: number | undefined
): SyntheticTimeline {
    const { sortedDifferentials, internalTauByPage } = getPageData(data);
    // The original generator already stores the E1 state. Record only later changes.
    const timeline: SyntheticTimeline = {
        generators: selectTimelineGenerators(data, truncation, bottomTruncation, limitX),
        changesByGenerator: new Map(), allDiffs: [], deathPages: new Map(),
    };
    const recordState = (name: string, page: number, state: GeneratorState) => {
        const changes = timeline.changesByGenerator.get(name);
        const change = { page, state };
        if (changes) changes.push(change);
        else timeline.changesByGenerator.set(name, [change]);
        if (state.torsion === 0 && !timeline.deathPages.has(name)) timeline.deathPages.set(name, page);
    };

    // Walk the pages in order, applying τ-multiplications then differentials.
    let diffIndex = 0;
    for (let p = 0; p <= MAX_STEM; p++) {
        for (const tm of internalTauByPage[p]) {
            const fromState = getLatestState(timeline, tm.from);
            const toState = getLatestState(timeline, tm.to);
            if (!fromState || !toState) continue;
            const updated = applyTauMultiplication(fromState, toState);
            if (!updated) continue;
            recordState(tm.from, p, updated.from);
            recordState(tm.to, p, updated.to);
        }

        while (diffIndex < sortedDifferentials.length && sortedDifferentials[diffIndex].page <= p) {
            const { diff, page } = sortedDifferentials[diffIndex++];
            if (page !== p || !shouldIncludeKind(diff.kind)) continue;
            const fromState = getLatestState(timeline, diff.from);
            const toState = getLatestState(timeline, diff.to);
            if (!fromState || !toState) continue;

            const mapped = mapBetweenGenerators(fromState, toState);
            if (!mapped) continue;

            // Only real/algebraic differentials actually mutate the page state;
            // others are recorded for display but don't kill classes.
            if (diff.kind == "Real" || diff.kind == "Algebraic") {
                recordState(diff.from, p + 1, mapped.from);
                recordState(diff.to, p + 1, mapped.to);
            }
            timeline.allDiffs.push({ ...diff, d: p });
        }
    }
    return timeline;
}

/** External τ extensions share the base timeline and store only their endpoint changes. */
function getExternalStates(data: SyntheticEHP, timeline: SyntheticTimeline) {
    if (timeline.external) return timeline.external;
    const states = new Map<string, GeneratorState>();
    for (const tm of data.external_tau_mults) {
        if (tm.kind !== "Real") continue;
        const fromState = states.get(tm.from) ?? getLatestState(timeline, tm.from);
        const toState = states.get(tm.to) ?? getLatestState(timeline, tm.to);
        if (!fromState || !toState) continue;
        const updated = applyTauMultiplication(fromState, toState, tm.af);
        if (!updated) continue;
        states.set(tm.from, updated.from);
        states.set(tm.to, updated.to);
    }
    const deaths: string[] = [];
    for (const [name, state] of states) {
        if (state.torsion === 0) deaths.push(name);
    }
    timeline.external = { states, deaths };
    return timeline.external;
}

/**
 * Keep one timeline per dataset. A new cutoff or Fakes setting replaces it.
 */
type CachedTimeline = { key: string; timeline: SyntheticTimeline };
const timelinesByDataset = new WeakMap<SyntheticEHP, CachedTimeline>();

function getSyntheticTimeline(
    data: SyntheticEHP,
    truncation: number | undefined,
    bottomTruncation: number | undefined,
    limitX: number | undefined
): SyntheticTimeline {
    const key = `${truncation}|${bottomTruncation}|${limitX}|${viewSettings.fakeDataMode}`;
    const cached = timelinesByDataset.get(data);
    if (cached?.key === key) return cached.timeline;
    const timeline = buildSyntheticTimeline(data, truncation, bottomTruncation, limitX);
    timelinesByDataset.set(data, { key, timeline });
    return timeline;
}

function computeSyntheticPage(data: SyntheticEHP, query: PageQuery): PageView {
    const timeline = getSyntheticTimeline(
        data,
        query.truncation,
        query.bottomTruncation,
        query.limitX
    );
    const external = query.applyTauMults ? getExternalStates(data, timeline) : undefined;
    const externalStates = query.page >= EXTERNAL_TAU_PAGE ? external?.states : undefined;
    const generators: Record<string, TorsionFiltration> = {};
    for (const [name, generator] of timeline.generators) {
        const state = getStateAtPage(timeline, generator, query.page, externalStates?.get(name));
        generators[name] = [normalizeTorsion(state.torsion), state.af];
    }

    // The recorded differentials are in page order. Copy only the visible prefix.
    let low = 0;
    let high = timeline.allDiffs.length;
    while (low < high) {
        const mid = Math.floor((low + high) / 2);
        if (timeline.allDiffs[mid].d! < query.page) low = mid + 1;
        else high = mid;
    }
    const differentials = timeline.allDiffs.slice(0, low);
    let deathPages = timeline.deathPages;
    if (external?.deaths.length) {
        deathPages = new Map(deathPages);
        for (const name of external.deaths) deathPages.set(name, EXTERNAL_TAU_PAGE);
    }
    return { generators, differentials, deathPages };
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
    const { sortedDifferentials } = getPageData(data);
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
    const deathPages = new Map<string, number>();
    for (const { diff, page: diffPage } of sortedDifferentials) {
        if (diffPage >= page) break;
        if (diff.kind !== "Algebraic" && (category !== Category.Geometric || diff.kind !== "Real")) continue;
        if (!generators[diff.from] || !generators[diff.to]) continue;

        const fromDeath = deathPages.get(diff.from) ?? Infinity;
        const toDeath = deathPages.get(diff.to) ?? Infinity;
        if (fromDeath <= diffPage || toDeath <= diffPage) continue;
        // The geometric view keeps a single hit for each target on a page.
        if (category === Category.Geometric && toDeath === diffPage + 1) continue;

        generators[diff.from][0] = 0;
        generators[diff.to][0] = 0;
        deathPages.set(diff.from, diffPage + 1);
        deathPages.set(diff.to, diffPage + 1);
        differentials.push({ ...diff, d: diffPage, coeff: diff.coeff ?? 0 });
    }

    return { generators, differentials, deathPages };
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
