/**
 * Chart display modes
 */
export enum ChartMode {
    EHP = "ehp",  // Extended Hopf fibration: dots centered in grid squares (x+0.5, y+0.5)
    ASS = "ass"   // Adams Spectral Sequence: dots on grid intersections (x, y)
}

/**
 * Configuration for chart display based on mode
 */
export interface ChartModeConfig {
    mode: ChartMode;

    /** X offset for dot positioning (0.5 for EHP, 0 for ASS) */
    xOffset: number;

    /** Y offset for dot positioning (0.5 for EHP, 0 for ASS) */
    yOffset: number;

    /** Whether to flip Y axis (false for EHP, true for ASS to have y=0 at bottom) */
    flipY: boolean;

    /** Display name for the mode */
    displayName: string;
}

export const CHART_MODE_CONFIGS: Record<ChartMode, ChartModeConfig> = {
    [ChartMode.EHP]: {
        mode: ChartMode.EHP,
        xOffset: 0.5,
        yOffset: 0.5,
        flipY: false,
        displayName: "EHP View"
    },
    [ChartMode.ASS]: {
        mode: ChartMode.ASS,
        xOffset: 0,
        yOffset: 0,
        flipY: true,
        displayName: "Adams SS View"
    }
};

/**
 * Get configuration for a specific chart mode
 */
export function getChartModeConfig(mode: ChartMode): ChartModeConfig {
    return CHART_MODE_CONFIGS[mode];
}
