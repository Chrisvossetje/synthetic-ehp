import { ehpChart } from "./charts";
import type { PreparedChartContent } from "../chart/chart";
import { copyDifferential, copyGenerator } from "./clipboard";
import { Differential, Kind, SyntheticEHP } from "../types";
import { Category, getSelectedGenerator, setSelectedGenerator, shouldIncludeKind, viewSettings } from "../model/settings";
import { ensureStableDataLoading, find, getActiveData } from "../model/dataSource";
import { generated_by_name, generates, getSphereLifecycleInfo } from "../model/names";
import { computePage, getDifferentialPage, survivesFilteredGenerator, TorsionFiltration } from "../model/spectralSequence";
import { buildGeneratorInfoLines, showInfoPanel } from "../chart/infoPanel";
import { multiplicationInfoLines, prepareAhssMultiplications, updateMultiplicationOverlay } from "./multiplications";
import { displayName, displayText } from "../model/tagNames";

/**
 * EHP chart controller.
 *
 * Owns everything specific to the EHP chart: the click handlers (which open the
 * info panel and drive selection highlighting), filling the chart with the full
 * dataset once, and re-deriving what is visible whenever the view settings
 * change. The actual page maths lives in `model/spectralSequence.ts`; this file
 * only decides what to *show* given the result.
 */

// Resolved differentials for the line-click handler, keyed `from->to`.
const computedDiffsByKey = new Map<string, Differential>();
// The page-filtered [torsion, filtration] of every class currently drawn.
let currentFilteredGenerators: Record<string, TorsionFiltration> = {};
let chartData: SyntheticEHP | null = null;

function cacheComputedDiffs(diffs: Differential[]) {
    computedDiffsByKey.clear();
    diffs.forEach((d) => {
        computedDiffsByKey.set(`${d.from}->${d.to}`, d);
    });
}

function getAllTauMults(data: SyntheticEHP) {
    return [...data.internal_tau_mults, ...data.external_tau_mults];
}

function getPageQuery() {
    return {
        category: viewSettings.category, page: viewSettings.page,
        truncation: viewSettings.truncation, bottomTruncation: viewSettings.bottomTruncation,
        allDiffs: viewSettings.allDiffs, fakeDataMode: viewSettings.fakeDataMode,
    };
}

type EhpPageQuery = ReturnType<typeof getPageQuery>;

function computeEhpPage(data: SyntheticEHP, query: EhpPageQuery) {
    const current = computePage(data, query);
    const permanent = query.page === 1000 ? current : computePage(data, { ...query, page: 1000 });
    const generators = current.generators;
    const visibleNames = new Set(Object.keys(generators).filter((name) => survivesFilteredGenerator(generators[name])));
    const eligibleDiffs: PagedDifferential[] = [];
    for (const diff of data.differentials) {
        if (!shouldIncludeKind(diff.kind)) continue;
        if (query.category === Category.Algebraic && diff.kind !== "Algebraic") continue;
        if (!visibleNames.has(diff.from) || !visibleNames.has(diff.to)) continue;

        const page = getDifferentialPage(data, diff);
        if (page === undefined || page < query.page || (!query.allDiffs && page !== query.page)) continue;
        if ((permanent.deathPages.get(diff.from) ?? Infinity) <= page ||
            (permanent.deathPages.get(diff.to) ?? Infinity) <= page) continue;

        eligibleDiffs.push({ diff, page });
    }
    const differentials = query.category === Category.Geometric
        ? geometricDifferentials(eligibleDiffs) : eligibleDiffs;
    return { generators, visibleNames, permanent, differentials };
}

type EhpPage = ReturnType<typeof computeEhpPage>;

function getDotDisplay(page: EhpPage, name: string) {
    const entry = page.generators[name];
    if (!survivesFilteredGenerator(entry)) return undefined;
    const finalEntry = page.permanent.generators[name];
    return { currentTorsion: entry[0], finalTorsion: finalEntry ? finalEntry[0] : 0, filtration: entry[1] };
}

type PreparedAhss = {
    data: SyntheticEHP;
    queryKey: string;
    page: EhpPage;
    notation: typeof viewSettings.tagNotation;
    content: PreparedChartContent;
    multiplications: ReturnType<typeof prepareAhssMultiplications>;
};

let preparedAhss: PreparedAhss | undefined;
let ahssPreparation: Promise<void> | "ready" | undefined;
let ahssRequested = false;

function* prepareAhss(data: SyntheticEHP): Generator<void, void> {
    // The first switch carries the current top cutoff into AHSS. Its bottom cutoff starts unchecked.
    const query = getPageQuery();
    query.truncation = query.truncation == null ? null : Math.max(0, query.truncation);
    query.bottomTruncation = undefined;
    const page = computeEhpPage(data, query);
    yield;
    const multiplications = prepareAhssMultiplications(data.generators, page.visibleNames, query.truncation ?? undefined);
    yield;
    const notation = viewSettings.tagNotation;
    const differentialStyles = new Map(page.differentials.map(({ diff: { from, to } }) => [
        `${from}-${to}`, { coeff: getDiffCoeff(page.generators, query.category, from, to) },
    ]));
    const content = yield* ehpChart.prepareContent({ ...data, tau_mults: getAllTauMults(data) }, page.visibleNames, {
        notation,
        dot: (name) => getDotDisplay(page, name),
        differential: (from, to) => differentialStyles.get(`${from}-${to}`),
    });
    preparedAhss = { data, queryKey: JSON.stringify(query), page, content, notation, multiplications };
}

function runPreparation(data: SyntheticEHP): Promise<void> {
    const work = prepareAhss(data);
    return new Promise((resolve, reject) => {
        const schedule = () => {
            if (!ahssRequested && typeof requestIdleCallback === "function") {
                requestIdleCallback((deadline) => advance(Math.min(8, deadline.timeRemaining())), { timeout: 120 });
            } else {
                setTimeout(() => advance(8), 0);
            }
        };
        const advance = (budget: number) => {
            try {
                if (chartData === data) {
                    work.return(undefined);
                    resolve();
                    return;
                }
                const end = performance.now() + budget;
                do {
                    if (work.next().done) {
                        resolve();
                        return;
                    }
                } while (performance.now() < end);
                schedule();
            } catch (error) {
                reject(error);
            }
        };
        schedule();
    });
}

/** Precompute the first AHSS chart once. An early switch can join the same work. */
export function prepareFirstAhssChart(switchRequested = false): Promise<void> | undefined {
    ahssRequested ||= switchRequested;
    if (ahssPreparation) return ahssPreparation === "ready" ? undefined : ahssPreparation;
    const pending = ensureStableDataLoading()
        .then(runPreparation)
        .then(() => { ahssPreparation = "ready"; })
        .catch((error) => {
            ahssPreparation = undefined;
            throw error;
        });
    ahssPreparation = pending;
    return pending;
}

type PagedDifferential = { diff: Differential; page: number };

/** Keep the earliest eligible hit per target, even when showing future pages. */
function geometricDifferentials(diffs: PagedDifferential[]): PagedDifferential[] {
    const incoming = new Map<string, PagedDifferential>();
    for (const candidate of diffs) {
        const { diff, page } = candidate;
        const current = incoming.get(diff.to);
        if (!current || page < current.page ||
            (page === current.page && diff.kind === "Real" && current.diff.kind !== "Real")) {
            incoming.set(diff.to, candidate);
        }
    }
    return [...incoming.values()];
}

/** A τ-multiplication is only drawn when both endpoints survive on this page. */
function shouldDisplayTauMult(
    kind: Kind,
    gens: Record<string, TorsionFiltration>,
    from: string,
    to: string
): boolean {
    const fromEntry = gens[from];
    const toEntry = gens[to];
    if (!fromEntry || !toEntry) {
        return false;
    }
    if (kind === "Real") {
        return survivesFilteredGenerator(fromEntry) && survivesFilteredGenerator(toEntry);
    }
    return true;
}

export function getComputedDiff(from: string, to: string): Differential | undefined {
    return computedDiffsByKey.get(`${from}->${to}`);
}

/**
 * The [torsion, filtration] computed for a generator on the selected page.
 */
export function getDisplayedGenerator(name: string): TorsionFiltration | undefined {
    return currentFilteredGenerators[name];
}

function getDiffCoeff(
    generators: Record<string, TorsionFiltration>, category: Category, from: string, to: string
): number | undefined {
    const fromState = generators[from];
    const toState = generators[to];
    if (!fromState || !toState) return undefined;
    if (category !== Category.Synthetic) return 0;
    return toState[1] - fromState[1] - 1;
}

export function handleDotClick(dot: string) {
    console.log('Dot clicked:', dot);
    const gen = find(dot);
    console.log(gen);

    if (!gen) return;

    setSelectedGenerator(dot);
    applyEhpSelectionHighlight();

    const label = ehpChart.generator_label(dot);
    copyGenerator(ehpChart, dot);

    const sphereInfo = getSphereLifecycleInfo(gen);
    const lines = buildGeneratorInfoLines(gen, sphereInfo, {
        xLabel: "stem",
        yLabel: "y",
        moduleLabel: "Module",
    }, displayName, getDisplayedGenerator(dot)?.[1]);

    showInfoPanel("Generator", lines, multiplicationInfoLines(ehpChart, gen.name), label);
}

/**
 * Highlight the selected class (orange), the class that generates it (cyan), and
 * the family it generates (green). No-op if nothing is selected / on screen.
 */
export function applyEhpSelectionHighlight() {
    ehpChart.clear_selection_highlights();

    const selected = getSelectedGenerator();
    if (!selected) return;
    const gen = find(selected);
    if (!gen) return;

    if (ehpChart.name_to_location.has(selected)) {
        ehpChart.add_selection_highlight(selected, "#ff6a00", 2.2, 0.18, 0.55);
    }

    const genName = generated_by_name(gen);
    const gensList = generates(gen);
    if (ehpChart.name_to_location.has(genName)) {
        ehpChart.add_selection_highlight(genName, "#00bcd4", 2.0, 0.14, 0.42);
    }
    gensList.forEach((g) => {
        if (ehpChart.name_to_location.has(g.name)) {
            ehpChart.add_selection_highlight(g.name, "#66bb00", 1.9, 0.12, 0.35);
        }
    });
}

export function handleLineClick(from: string, to: string) {
    console.log('Line clicked:', from, '->', to);
    const activeData = getActiveData();
    if (!activeData) return;
    const rawDiff = activeData.differentials.find(d => d.from === from && d.to === to);
    const computedDiff = getComputedDiff(from, to);
    console.log(rawDiff ?? computedDiff);

    if (!rawDiff && !computedDiff) return;

    const coeff = getDiffCoeff(currentFilteredGenerators, viewSettings.category, from, to);
    if (coeff === undefined) return;
    const page = (rawDiff && getDifferentialPage(activeData, rawDiff)) ?? computedDiff?.d ?? 0;

    const source = ehpChart.generator_label(from);
    const target = ehpChart.generator_label(to);
    copyDifferential(ehpChart, from, to, coeff);
    const lines = [
        `From: ${source}`,
        `To: ${target}`,
        `Kind: ${rawDiff?.kind}`,
        `Page: E${page}`,
        `Coefficient: ${coeff ? 'τ^' + coeff : '1'}`,
    ];

    const extraLines: string[] = [];
    if (rawDiff && "proof" in rawDiff) {
        extraLines.push(`Proof: ${displayText(rawDiff.proof ?? "")}`);
    }

    showInfoPanel("Differential", lines, extraLines);
}

export function handleTauMultClick(from: string, to: string) {
    const activeData = getActiveData();
    if (!activeData) return;

    const internalTauMult = activeData.internal_tau_mults.find((t) => t.from === from && t.to === to);
    const externalTauMult = activeData.external_tau_mults.find((t) => t.from === from && t.to === to);
    const tauMult = internalTauMult ?? externalTauMult;
    if (!tauMult) return;

    const lines = [
        `From: ${ehpChart.generator_label(tauMult.from)}`,
        `To: ${ehpChart.generator_label(tauMult.to)}`,
        `Kind: ${tauMult.kind}`,
    ];
    if (internalTauMult) {
        lines.push(`Page: E${internalTauMult.page}`);
        lines.push("Type: Internal");
    } else {
        lines.push("Type: External");
    }

    const extraLines: string[] = [];
    if ("proof" in tauMult) {
        extraLines.push(`Proof: ${displayText(tauMult.proof ?? "")}`);
    }

    showInfoPanel("τ Multiplication", lines, extraLines);
}

/** Load the dataset into the chart and wire up click handlers. */
function fill_ehp_chart(activeData: SyntheticEHP, visibleNames: ReadonlySet<string>, prepared?: PreparedChartContent) {
    ehpChart.dotCallback = handleDotClick;
    ehpChart.lineCallback = handleLineClick;
    ehpChart.tauMultCallback = handleTauMultClick;

    ehpChart.set_all_generators(activeData.generators);
    ehpChart.set_all_differentials(activeData.differentials);
    ehpChart.set_all_multiplications(activeData.multiplications);
    ehpChart.set_all_tau_mults(getAllTauMults(activeData));

    ehpChart.init(visibleNames, prepared);
    chartData = activeData;
}

/**
 * Re-derive what the EHP chart shows from the current view settings.
 *
 * Update the classes and relations for the selected page.
 * Dot borders use torsion on the selected page, and fills use E∞ torsion.
 */
export function update_ehp_chart() {
    const activeData = getActiveData();
    if (!activeData) {
        return;
    }
    const query = getPageQuery();
    const prepared = preparedAhss?.data === activeData ? preparedAhss : undefined;
    if (prepared) preparedAhss = undefined;
    const matchingPage = prepared?.queryKey === JSON.stringify(query) ? prepared : undefined;
    const page = matchingPage?.page ?? computeEhpPage(activeData, query);
    const { generators: gens, visibleNames, permanent: permView, differentials: displayedDiffs } = page;
    currentFilteredGenerators = gens;
    if (chartData !== activeData) {
        fill_ehp_chart(activeData, visibleNames, prepared?.content);
    }

    // When the view changed since preparation, apply the current differential visibility.
    if (!matchingPage?.content.styledDifferentials) {
        activeData.differentials.forEach((d) => ehpChart.display_diff(d.from, d.to, false));
        displayedDiffs.forEach(({ diff: { from, to } }) => {
            ehpChart.display_diff(from, to, true, getDiffCoeff(gens, query.category, from, to));
        });
    }
    activeData.multiplications.forEach((m) => {
        ehpChart.display_mult(m.from, m.to, false);
    });
    ehpChart.tau_mults.forEach((t) => {
        ehpChart.display_tau_mult(t.from, t.to, false);
    });

    const perm_classes = permView.generators;
    cacheComputedDiffs(permView.differentials);

    if (!matchingPage?.content.styledDots || matchingPage.notation !== viewSettings.tagNotation) {
        for (const { name } of activeData.generators) {
            const display = getDotDisplay(page, name);
            ehpChart.display_dot(name, display !== undefined, display);
        }
    }

    // Multiplications: only when both endpoints are alive.
    activeData.multiplications.forEach((m) => {
        if (survivesFilteredGenerator(gens[m.from]) && survivesFilteredGenerator(gens[m.to])) {
            ehpChart.display_mult(m.from, m.to, true);
        }
    });

    // τ-multiplications: synthetic category only.
    if (viewSettings.category == Category.Synthetic) {
        activeData.internal_tau_mults.forEach((t) => {
            if (!shouldIncludeKind(t.kind)) return;
            if (!viewSettings.allDiffs && t.page !== viewSettings.page) return;
            if (shouldDisplayTauMult(t.kind, gens, t.from, t.to)) {
                ehpChart.display_tau_mult(t.from, t.to, true);
            }
        });

        // External τ-mults resolve at E∞, so only show them with All Diffs or E∞.
        if (viewSettings.allDiffs || viewSettings.page > 999) {
            activeData.external_tau_mults.forEach((t) => {
                if (!shouldIncludeKind(t.kind)) return;
                if (shouldDisplayTauMult(t.kind, gens, t.from, t.to)) {
                    ehpChart.display_tau_mult(t.from, t.to, true);
                }
            });
        }
    }

    updateMultiplicationOverlay(ehpChart, activeData.generators, visibleNames, "ehp", undefined,
        new Set(Object.keys(perm_classes).filter((name) => survivesFilteredGenerator(perm_classes[name]))),
        matchingPage?.multiplications);
    applyEhpSelectionHighlight();
}
