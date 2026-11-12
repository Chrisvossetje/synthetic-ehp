import type { Chart } from "./chart/chart";
import { initializeData, ensureStableDataLoading } from "./model/dataSource";
import { loadMultiplicationTables } from "./model/multiplicationSource";
import { ehpChart } from "./app/charts";
import { setupUIControls } from "./app/controls";
import { setupKeyboardControls } from "./app/keyboard";
import {
    cancelScreenshot,
    handleScreenshotPointerDown,
    handleScreenshotPointerMove,
    handleScreenshotPointerUp,
    isScreenshotMode,
} from "./app/screenshot";

/**
 * Application entry point.
 *
 * Responsibilities are deliberately thin: load the data,
 * draw the initial frame, and hand control off to the UI/keyboard wiring. All
 * the per-control behaviour lives in `app/controls.ts` and `app/keyboard.ts`.
 */

declare global {
    interface Window {
        // Set so the SVG `onclick` attributes generated in chart.ts can reach
        // the active chart's click handlers.
        chartInstance: Chart;
    }
}

bootstrap().catch((error) => {
    console.error("Failed to initialize data", error);
});

async function bootstrap() {
    await Promise.all([initializeData(), loadMultiplicationTables()]);

    window.chartInstance = ehpChart;
    setupKeyboardControls();
    setupUIControls();
    setupScreenshotHandlers();

    ensureStableDataLoading().catch((error) => {
        console.error("Failed to preload stable data", error);
    });
}

/** Forward pointer events on the EHP chart to the screenshot selection logic. */
function setupScreenshotHandlers() {
    const svg = ehpChart.svgchart.svg;

    const forward = (handler: (e: PointerEvent) => void) => (e: PointerEvent) => {
        if (!isScreenshotMode()) return;
        handler(e);
        e.stopPropagation();
        e.preventDefault();
    };

    svg.addEventListener("pointerdown", forward((e) => handleScreenshotPointerDown(e, ehpChart)));
    svg.addEventListener("pointermove", forward((e) => handleScreenshotPointerMove(e, ehpChart)));
    svg.addEventListener("pointerup", forward((e) => handleScreenshotPointerUp(e, ehpChart)));
}
