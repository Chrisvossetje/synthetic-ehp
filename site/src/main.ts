import type { Chart } from "./chart/chart";
import { initializeData, ensureStableDataLoading, isStableDataReady } from "./model/dataSource";
import { viewSettings } from "./model/settings";
import { ehpChart } from "./app/charts";
import { fill_ehp_chart, update_ehp_chart } from "./app/ehpChart";
import { update_ass_chart } from "./app/assChart";
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
 * Responsibilities are deliberately thin: load the data, build both charts,
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
    await initializeData();
    ensureStableDataLoading();

    // Build and draw both charts with the initial view settings.
    fill_ehp_chart();
    update_ehp_chart();
    update_ass_chart(viewSettings.truncation, viewSettings.bottomTruncation);

    window.chartInstance = ehpChart;

    setupKeyboardControls();
    setupUIControls();
    setupScreenshotHandlers();

    // The stable dataset loads lazily; keep the data-source switch disabled
    // until it is ready so we never switch to half-loaded data.
    const dataSourceSwitch = document.getElementById("data-source-switch") as HTMLInputElement | null;
    if (dataSourceSwitch && !isStableDataReady()) {
        dataSourceSwitch.disabled = true;
        ensureStableDataLoading()
            .then(() => { dataSourceSwitch.disabled = false; })
            .catch((error) => {
                console.error("Failed to preload stable data", error);
                dataSourceSwitch.disabled = false;
            });
    }
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
