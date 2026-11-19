import { Kind } from "../types";

/**
 * View / selection state.
 *
 * This module holds the *mutable* state that describes "what the user is
 * currently looking at" — independent of any particular chart or DOM control.
 * The UI controls (see `app/controls.ts`) write into `viewSettings`; the
 * charts read from it when they redraw. Keeping it in one small module means
 * there is a single source of truth and no chart needs to know how another
 * chart is configured.
 */

/** The three flavours of the spectral sequence the viewer can show. */
export enum Category {
    Synthetic,
    Algebraic,
    Geometric,
}

export type FakeDataMode = "none" | "all" | "unnecessary-and-invisible" | "unnecessary" | "invisible";

/** Everything that influences how the active chart is rendered. */
export const viewSettings = {
    /** Show every differential at once instead of only the current page's. */
    allDiffs: true,
    /** Current page; `1000` is used throughout as a stand-in for E∞. */
    page: 1,
    category: Category.Synthetic,
    tagNotation: "adams" as "adams" | "curtis",
    /** Top truncation (inclusive max `y`); `undefined` means "no top cutoff". */
    truncation: undefined as number | undefined,
    /** Bottom truncation (inclusive min `y`); `undefined` means "no bottom cutoff". */
    bottomTruncation: undefined as number | undefined,
    fakeDataMode: "none" as FakeDataMode,
    /** Display overlays only; never passed to the page computation. */
    /** Number of consecutive operators enabled (0=none, 1=h_0, …). */
    multiplicationCounts: { ehp: 1 },
};

/** Include baseline data and any provisional kinds enabled by the Fakes filter. */
export function shouldIncludeKind(kind: Kind): boolean {
    const { fakeDataMode } = viewSettings;
    if (fakeDataMode === "all" || kind === "Real" || kind === "Unknown" || kind === "Algebraic") {
        return true;
    }
    if (kind === "Unnecessary" || kind === "Unneccessary") {
        return fakeDataMode === "unnecessary" || fakeDataMode === "unnecessary-and-invisible";
    }
    return kind === "Invisible" && (fakeDataMode === "invisible" || fakeDataMode === "unnecessary-and-invisible");
}

// Selection is shared across modes and data sources: clicking a class in the
// EHP chart keeps it highlighted when you flip to the ASS chart, and vice versa.
let selectedGeneratorName: string | null = null;
export function setSelectedGenerator(name: string | null) { selectedGeneratorName = name; }
export function getSelectedGenerator() { return selectedGeneratorName; }
