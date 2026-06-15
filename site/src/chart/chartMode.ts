/**
 * Chart display modes.
 *
 * The two modes differ in how dots are placed on the grid and which axes are
 * decorated; the per-mode behaviour lives in `chart.ts` / `svgChart.ts`, keyed
 * off this enum.
 */
export enum ChartMode {
    /** Extended Hopf fibration: dots centred in grid squares (x+0.5, y+0.5). */
    EHP = "ehp",
    /** Adams Spectral Sequence: dots on grid intersections (x, y). */
    ASS = "ass",
}
