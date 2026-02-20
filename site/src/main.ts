import { Chart } from "./chart";
import { viewSettings, update_ehp_chart, fill_ehp_chart, Category, switchDataSource, isUsingStableData, initializeData, ensureStableDataLoading, isStableDataReady } from "./logic";
import { update_ass_chart } from "./ass_logic";
import { ChartMode } from "./chartMode";
import { startScreenshot, handleScreenshotPointerDown, handleScreenshotPointerMove, handleScreenshotPointerUp, cancelScreenshot, isScreenshotMode } from "./screenshot";

// Create two separate chart instances
export const ehpChart = new Chart('svgchart-ehp', ChartMode.EHP);
export const assChart = new Chart('svgchart-ass', ChartMode.ASS);
ehpChart.set_label_display(true, true);
assChart.set_label_display(false, false);

// Track which chart is currently active
let isEHPActive = true;

// Set up global reference for onclick handlers
declare global {
    interface Window {
        chartInstance: Chart;
    }
}

bootstrap().catch((error) => {
    console.error("Failed to initialize data", error);
});

function updateActiveChart() {
    if (isEHPActive) {
        update_ehp_chart();
    } else {
        update_ass_chart(viewSettings.truncation);
    }
}

function setupUIControls() {
    // Keyboard controls
    setupKeyboardControls();

    // Data source switch (data.ts vs data_stable.ts)
    const dataSourceSwitch = document.getElementById('data-source-switch') as HTMLInputElement;
    if (dataSourceSwitch) {
        dataSourceSwitch.addEventListener('change', async () => {
            dataSourceSwitch.disabled = true;
            await switchDataSource();
            dataSourceSwitch.checked = isUsingStableData();
            dataSourceSwitch.disabled = false;
            updateActiveChart();
        });
    }

    // EHP/ASS mode switch
    const ehpAssSwitch = document.getElementById('ehp-ass-switch') as HTMLInputElement;
    ehpAssSwitch.addEventListener('change', () => {
        if (ehpAssSwitch.checked) {
            // Switch to ASS mode
            ehpChart.hide();
            assChart.show();
            isEHPActive = false;
            window.chartInstance = assChart;
        } else {
            // Switch to EHP mode
            assChart.hide();
            ehpChart.show();
            isEHPActive = true;
            window.chartInstance = ehpChart;
        }
        updateActiveChart();
    });

    // All Diffs checkbox (only affects EHP chart)
    const allDiffCheckbox = document.getElementById('all-diff-checkbox') as HTMLInputElement;
    allDiffCheckbox.addEventListener('change', () => {
        viewSettings.allDiffs = allDiffCheckbox.checked;
        if (isEHPActive) {
            update_ehp_chart();
        }
    });

    // Page selector (only affects EHP chart)
    const pageSelect = document.getElementById('ss-page') as HTMLSelectElement;
    pageSelect.addEventListener('change', () => {
        viewSettings.page = parseInt(pageSelect.value);
        if (isEHPActive) {
            update_ehp_chart();
        }
    });

    // Category selector (only affects EHP chart)
    const categorySelect = document.getElementById('ss-category') as HTMLSelectElement;
    categorySelect.addEventListener('change', () => {
        viewSettings.category = parseInt(categorySelect.value);
        if (isEHPActive) {
            update_ehp_chart();
        }
    });

    // Truncation checkbox and input (affects both charts)
    const truncationCheckbox = document.getElementById('truncation-checkbox') as HTMLInputElement;
    const truncationInput = document.querySelector('input[name="Truncation"]') as HTMLInputElement;

    truncationCheckbox.addEventListener('change', () => {
        if (truncationCheckbox.checked) {
            const value = parseInt(truncationInput.value);
            viewSettings.truncation = isNaN(value) ? null : value;
        } else {
            viewSettings.truncation = null;
        }
        updateActiveChart();
    });

    let truncationDebounceTimeout: number | null = null;
    truncationInput.addEventListener('input', () => {
        if (truncationCheckbox.checked) {
            if (truncationDebounceTimeout !== null) {
                window.clearTimeout(truncationDebounceTimeout);
            }
            truncationDebounceTimeout = window.setTimeout(() => {
                const value = parseInt(truncationInput.value);
                viewSettings.truncation = isNaN(value) ? null : value;
                updateActiveChart();
                truncationDebounceTimeout = null;
            }, 120);
        }
    });
}

function setupScreenshotHandlers() {
    const svg = ehpChart.svgchart.svg;

    svg.addEventListener('pointerdown', (e) => {
        if (isScreenshotMode()) {
            handleScreenshotPointerDown(e, ehpChart);
            e.stopPropagation();
            e.preventDefault();
        }
    });

    svg.addEventListener('pointermove', (e) => {
        if (isScreenshotMode()) {
            handleScreenshotPointerMove(e, ehpChart);
            e.stopPropagation();
            e.preventDefault();
        }
    });

    svg.addEventListener('pointerup', (e) => {
        if (isScreenshotMode()) {
            handleScreenshotPointerUp(e, ehpChart);
            e.stopPropagation();
            e.preventDefault();
        }
    });
}

function setupKeyboardControls() {
    document.addEventListener('keydown', async (e) => {
        // Escape closes the info popup first.
        if (e.key === 'Escape') {
            const floatingBox = document.getElementById('floatingBox');
            if (floatingBox && floatingBox.style.display !== 'none') {
                floatingBox.style.display = 'none';
                return;
            }
        }

        // Escape key cancels screenshot mode
        if (e.key === 'Escape' && isScreenshotMode()) {
            cancelScreenshot();
            return;
        }

        // Ignore if user is typing in an input field
        if (e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement) {
            return;
        }

        let needsUpdate = false;
        const pageSelect = document.getElementById('ss-page') as HTMLSelectElement;
        const categorySelect = document.getElementById('ss-category') as HTMLSelectElement;
        const truncationCheckbox = document.getElementById('truncation-checkbox') as HTMLInputElement;
        const truncationInput = document.querySelector('input[name="Truncation"]') as HTMLInputElement;
        const ehpAssSwitch = document.getElementById('ehp-ass-switch') as HTMLInputElement;
        const allDiffCheckbox = document.getElementById('all-diff-checkbox') as HTMLInputElement;
        const dataSourceSwitch = document.getElementById('data-source-switch') as HTMLInputElement;

        switch(e.key) {
            // Data source switch
            case 's':
            case 'S':
                if (dataSourceSwitch) {
                    dataSourceSwitch.disabled = true;
                }
                await switchDataSource();
                if (dataSourceSwitch) {
                    dataSourceSwitch.checked = isUsingStableData();
                    dataSourceSwitch.disabled = false;
                }
                updateActiveChart();
                return;

            // EHP/ASS mode switch
            case 'a':
            case 'A':
                ehpAssSwitch.checked = !ehpAssSwitch.checked;
                if (ehpAssSwitch.checked) {
                    // Switch to ASS mode
                    ehpChart.hide();
                    assChart.show();
                    isEHPActive = false;
                    window.chartInstance = assChart;
                } else {
                    // Switch to EHP mode
                    assChart.hide();
                    ehpChart.show();
                    isEHPActive = true;
                    window.chartInstance = ehpChart;
                }
                needsUpdate = true;
                break;
            // Pages 1-9
            case '1':
            case '2':
            case '3':
            case '4':
            case '5':
            case '6':
            case '7':
            case '8':
            case '9':
                viewSettings.page = parseInt(e.key);
                pageSelect.value = e.key;
                needsUpdate = true;
                break;

            // Page 0 = E∞
            case '0':
                viewSettings.page = 1000;
                pageSelect.value = '1000';
                needsUpdate = true;
                break;

            // Categories
            case 'q':
            case 'Q':
                viewSettings.category = Category.Synthetic;
                categorySelect.value = '0';
                needsUpdate = true;
                break;

            case 'w':
            case 'W':
                viewSettings.category = Category.Algebraic;
                categorySelect.value = '1';
                needsUpdate = true;
                break;

            case 'e':
            case 'E':
                viewSettings.category = Category.Geometric;
                categorySelect.value = '2';
                needsUpdate = true;
                break;

            // Toggle all diffs (only affects EHP mode)
            case 'd':
            case 'D':
                viewSettings.allDiffs = !viewSettings.allDiffs;
                allDiffCheckbox.checked = viewSettings.allDiffs;
                if (isEHPActive) {
                    needsUpdate = true;
                }
                break;

            // Truncation controls
            case 'j':
            case 'J':
                // Lower truncation (enable if disabled)
                if (!truncationCheckbox.checked) {
                    truncationCheckbox.checked = true;
                    const value = parseInt(truncationInput.value);
                    viewSettings.truncation = isNaN(value) ? 5 : value;
                }
                if (viewSettings.truncation !== null && viewSettings.truncation !== undefined) {
                    const newValue = Math.max(2, viewSettings.truncation - 1);
                    viewSettings.truncation = newValue;
                    truncationInput.value = newValue.toString();
                }
                needsUpdate = true;
                break;

            case 'k':
            case 'K':
                // Higher truncation (enable if disabled)
                if (!truncationCheckbox.checked) {
                    truncationCheckbox.checked = true;
                    const value = parseInt(truncationInput.value);
                    viewSettings.truncation = isNaN(value) ? 5 : value;
                }
                if (viewSettings.truncation !== null && viewSettings.truncation !== undefined) {
                    const newValue = Math.min(50, viewSettings.truncation + 1);
                    viewSettings.truncation = newValue;
                    truncationInput.value = newValue.toString();
                }
                needsUpdate = true;
                break;

            case 'l':
            case 'L':
                // Toggle truncation
                truncationCheckbox.checked = !truncationCheckbox.checked;
                if (truncationCheckbox.checked) {
                    const value = parseInt(truncationInput.value);
                    viewSettings.truncation = isNaN(value) ? null : value;
                } else {
                    viewSettings.truncation = null;
                }
                needsUpdate = true;
                break;

            // Screenshot mode
            case 'p':
            case 'P':
                if (isEHPActive) {
                    startScreenshot();
                }
                return; // Don't trigger update
        }

        if (needsUpdate) {
            updateActiveChart();
        }
    });
}

async function bootstrap() {
    await initializeData();
    ensureStableDataLoading();

    // Initialize both charts
    fill_ehp_chart();

    // Update charts with initial data
    update_ehp_chart();
    update_ass_chart(viewSettings.truncation);

    // Set initial global chart reference
    window.chartInstance = ehpChart;

    setupUIControls();
    setupScreenshotHandlers();

    const dataSourceSwitch = document.getElementById('data-source-switch') as HTMLInputElement;
    if (dataSourceSwitch && !isStableDataReady()) {
        dataSourceSwitch.disabled = true;
        ensureStableDataLoading()
            .then(() => {
                dataSourceSwitch.disabled = false;
            })
            .catch((error) => {
                console.error("Failed to preload stable data", error);
                dataSourceSwitch.disabled = false;
            });
    }
}
