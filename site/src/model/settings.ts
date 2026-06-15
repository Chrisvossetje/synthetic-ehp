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

/** Everything that influences how the active chart is rendered. */
export const viewSettings = {
    /** Show every differential at once instead of only the current page's. */
    allDiffs: true,
    /** Current page; `1000` is used throughout as a stand-in for E∞. */
    page: 1,
    category: Category.Synthetic,
    /** Top truncation (inclusive max `y`); `undefined` means "no top cutoff". */
    truncation: undefined as number | undefined,
    /** Bottom truncation (inclusive min `y`); `undefined` means "no bottom cutoff". */
    bottomTruncation: undefined as number | undefined,
    /** Include classes/differentials whose `kind` is provisional ("Fake", etc.). */
    showFakeData: false,
};

/**
 * Whether a differential/class of the given `kind` should be considered at all.
 * "Real" / "Unknown" / "Algebraic" are always real data; everything else is
 * only included when the user opts in via the "Fakes" toggle.
 */
export function shouldIncludeKind(kind: Kind): boolean {
    return viewSettings.showFakeData || kind === "Real" || kind === "Unknown" || kind === "Algebraic";
}

// Selection is shared across modes and data sources: clicking a class in the
// EHP chart keeps it highlighted when you flip to the ASS chart, and vice versa.
let selectedGeneratorName: string | null = null;
export function setSelectedGenerator(name: string | null) { selectedGeneratorName = name; }
export function getSelectedGenerator() { return selectedGeneratorName; }
