import type { Chart } from "./chart";
import { viewSettings, Category, isUsingStableData, initializeData, ensureStableDataLoading, isStableDataReady } from "./logic";
import { fill_ehp_chart, switchDataSource, update_ehp_chart } from "./ehp_chart";
import { update_ass_chart } from "./ass_logic";
import { ehpChart, assChart } from "./charts";
import { applyTruncationFromControls, syncSphereControlsFromState } from "./truncation";
import type { TruncationControls } from "./truncation";
import { startScreenshot, handleScreenshotPointerDown, handleScreenshotPointerMove, handleScreenshotPointerUp, cancelScreenshot, isScreenshotMode } from "./screenshot";

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
        update_ass_chart(viewSettings.truncation, viewSettings.bottomTruncation);
    }
}

function getTruncationControls(): TruncationControls | null {
    const truncationCheckbox = document.getElementById('truncation-checkbox') as HTMLInputElement | null;
    const truncationInput = document.querySelector('input[name="Truncation"]') as HTMLInputElement | null;
    const bottomTruncationCheckbox = document.getElementById('bottom-truncation-checkbox') as HTMLInputElement | null;
    const bottomTruncationInput = document.querySelector('input[name="BottomTruncation"]') as HTMLInputElement | null;
    const sphereCheckbox = document.getElementById('sphere-truncation-checkbox') as HTMLInputElement | null;
    const sphereInput = document.querySelector('input[name="Sphere"]') as HTMLInputElement | null;

    if (!truncationCheckbox || !truncationInput || !bottomTruncationCheckbox || !bottomTruncationInput || !sphereCheckbox || !sphereInput) {
        return null;
    }

    return {
        truncationCheckbox,
        truncationInput,
        bottomTruncationCheckbox,
        bottomTruncationInput,
        sphereCheckbox,
        sphereInput,
    };
}

function syncViewControlsForDataSource(updateChart = true) {
    const dataSourceSwitch = document.getElementById('data-source-switch') as HTMLInputElement | null;
    const truncationControls = document.getElementById('truncation-controls') as HTMLDivElement | null;
    const sphereControls = document.getElementById('sphere-controls') as HTMLDivElement | null;
    const fakeControls = document.getElementById('fake-controls') as HTMLDivElement | null;
    const fakeControlsSpacer = document.getElementById('fake-controls-spacer') as HTMLDivElement | null;
    const showFakeCheckbox = document.getElementById('show-fake-checkbox') as HTMLInputElement | null;
    const controls = getTruncationControls();

    if (!dataSourceSwitch || !truncationControls || !sphereControls || !fakeControls || !fakeControlsSpacer || !showFakeCheckbox || !controls) {
        return;
    }

    fakeControls.style.display = 'flex';
    fakeControlsSpacer.style.display = 'block';
    fakeControls.style.opacity = '1';
    fakeControls.style.pointerEvents = 'auto';
    showFakeCheckbox.disabled = false;

    if (dataSourceSwitch.checked) {
        truncationControls.style.display = 'flex';
        sphereControls.style.display = 'none';
        syncSphereControlsFromState(controls, viewSettings, true);
        applyTruncationFromControls("ahss", controls, viewSettings);
    } else {
        truncationControls.style.display = 'none';
        sphereControls.style.display = 'flex';
        syncSphereControlsFromState(controls, viewSettings, false);
        applyTruncationFromControls("sphere", controls, viewSettings);
    }

    if (updateChart) {
        updateActiveChart();
    }
}

function setupUIControls() {
    // Keyboard controls
    setupKeyboardControls();

    // Data source switch (data.ts vs data_stable.ts)
    const dataSourceSwitch = document.getElementById('data-source-switch') as HTMLInputElement;

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
        syncViewControlsForDataSource(false);
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

    const showFakeCheckbox = document.getElementById('show-fake-checkbox') as HTMLInputElement | null;
    showFakeCheckbox?.addEventListener('change', () => {
        viewSettings.showFakeData = showFakeCheckbox.checked;
        updateActiveChart();
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
    const truncationControls = getTruncationControls();
    if (!truncationControls) {
        return;
    }

    const usingAhssViewSettings = () => !!dataSourceSwitch?.checked;

    truncationControls.truncationCheckbox.addEventListener('change', () => {
        if (!usingAhssViewSettings()) {
            return;
        }
        applyTruncationFromControls("ahss", truncationControls, viewSettings);
        updateActiveChart();
    });

    let truncationDebounceTimeout: number | null = null;
    truncationControls.truncationInput.addEventListener('input', () => {
        if (!usingAhssViewSettings()) {
            return;
        }
        if (truncationControls.truncationCheckbox.checked) {
            if (truncationDebounceTimeout !== null) {
                window.clearTimeout(truncationDebounceTimeout);
            }
            truncationDebounceTimeout = window.setTimeout(() => {
                applyTruncationFromControls("ahss", truncationControls, viewSettings);
                updateActiveChart();
                truncationDebounceTimeout = null;
            }, 120);
        }
    });

    truncationControls.bottomTruncationCheckbox.addEventListener('change', () => {
        if (!usingAhssViewSettings()) {
            return;
        }
        applyTruncationFromControls("ahss", truncationControls, viewSettings);
        updateActiveChart();
    });

    let bottomTruncationDebounceTimeout: number | null = null;
    truncationControls.bottomTruncationInput.addEventListener('input', () => {
        if (!usingAhssViewSettings()) {
            return;
        }
        if (truncationControls.bottomTruncationCheckbox.checked) {
            if (bottomTruncationDebounceTimeout !== null) {
                window.clearTimeout(bottomTruncationDebounceTimeout);
            }
            bottomTruncationDebounceTimeout = window.setTimeout(() => {
                applyTruncationFromControls("ahss", truncationControls, viewSettings);
                updateActiveChart();
                bottomTruncationDebounceTimeout = null;
            }, 120);
        }
    });

    let sphereDebounceTimeout: number | null = null;
    truncationControls.sphereInput.addEventListener('input', () => {
        if (usingAhssViewSettings()) {
            return;
        }
        if (!truncationControls.sphereCheckbox.checked) {
            return;
        }
        if (sphereDebounceTimeout !== null) {
            window.clearTimeout(sphereDebounceTimeout);
        }
        sphereDebounceTimeout = window.setTimeout(() => {
            applyTruncationFromControls("sphere", truncationControls, viewSettings);
            updateActiveChart();
            sphereDebounceTimeout = null;
        }, 120);
    });

    truncationControls.sphereCheckbox.addEventListener('change', () => {
        if (usingAhssViewSettings()) {
            return;
        }
        applyTruncationFromControls("sphere", truncationControls, viewSettings);
        updateActiveChart();
    });

    if (dataSourceSwitch) {
        dataSourceSwitch.addEventListener('change', async () => {
            dataSourceSwitch.disabled = true;
            await switchDataSource();
            dataSourceSwitch.checked = isUsingStableData();
            dataSourceSwitch.disabled = false;
            syncViewControlsForDataSource();
        });
    }

    syncViewControlsForDataSource();
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
        const truncationControls = getTruncationControls();
        if (!truncationControls) {
            return;
        }
        const {
            truncationCheckbox,
            truncationInput,
            bottomTruncationCheckbox,
            bottomTruncationInput,
            sphereCheckbox,
            sphereInput
        } = truncationControls;
        const ehpAssSwitch = document.getElementById('ehp-ass-switch') as HTMLInputElement;
        const allDiffCheckbox = document.getElementById('all-diff-checkbox') as HTMLInputElement;
        const showFakeCheckbox = document.getElementById('show-fake-checkbox') as HTMLInputElement | null;
        const dataSourceSwitch = document.getElementById('data-source-switch') as HTMLInputElement;
        const usingAhssViewSettings = () => !!dataSourceSwitch?.checked;

        switch(e.key) {
            // Data source switch
            case 's':
            case 'S':
                if (dataSourceSwitch) {
                    dataSourceSwitch.checked = !dataSourceSwitch.checked;
                    dataSourceSwitch.dispatchEvent(new Event('change'));
                }
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

            case 'f':
            case 'F':
                if (!showFakeCheckbox) {
                    break;
                }
                viewSettings.showFakeData = !viewSettings.showFakeData;
                showFakeCheckbox.checked = viewSettings.showFakeData;
                needsUpdate = true;
                break;

            // Truncation controls
            case 'j':
            case 'J': {
                if (!usingAhssViewSettings()) {
                    if (!sphereCheckbox.checked) {
                        sphereCheckbox.checked = true;
                    }
                    const current = parseInt(sphereInput.value);
                    const base = isNaN(current) ? 7 : current;
                    const newValue = Math.max(2, base - 1);
                    sphereInput.value = newValue.toString();
                    applyTruncationFromControls("sphere", truncationControls, viewSettings);
                    needsUpdate = true;
                    break;
                }
                // Lower truncation (enable if disabled)
                if (!truncationCheckbox.checked) {
                    truncationCheckbox.checked = true;
                    const value = parseInt(truncationInput.value);
                    if (isNaN(value)) {
                        truncationInput.value = "5";
                    }
                }
                const currentTop = parseInt(truncationInput.value);
                const topBase = isNaN(currentTop) ? 5 : currentTop;
                const newValue = Math.max(2, topBase - 1);
                truncationInput.value = newValue.toString();
                applyTruncationFromControls("ahss", truncationControls, viewSettings);
                needsUpdate = true;
                break;
            }

            case 'k':
            case 'K': {
                if (!usingAhssViewSettings()) {
                    if (!sphereCheckbox.checked) {
                        sphereCheckbox.checked = true;
                    }
                    const current = parseInt(sphereInput.value);
                    const base = isNaN(current) ? 7 : current;
                    const newValue = Math.min(50, base + 1);
                    sphereInput.value = newValue.toString();
                    applyTruncationFromControls("sphere", truncationControls, viewSettings);
                    needsUpdate = true;
                    break;
                }
                // Higher truncation (enable if disabled)
                if (!truncationCheckbox.checked) {
                    truncationCheckbox.checked = true;
                    const value = parseInt(truncationInput.value);
                    if (isNaN(value)) {
                        truncationInput.value = "5";
                    }
                }
                const currentTopInc = parseInt(truncationInput.value);
                const topBaseInc = isNaN(currentTopInc) ? 5 : currentTopInc;
                const newValue = Math.min(50, topBaseInc + 1);
                truncationInput.value = newValue.toString();
                applyTruncationFromControls("ahss", truncationControls, viewSettings);
                needsUpdate = true;
                break;
            }

            case 'l':
            case 'L': {
                if (!usingAhssViewSettings()) {
                    sphereCheckbox.checked = !sphereCheckbox.checked;
                    if (sphereCheckbox.checked) {
                        const value = parseInt(sphereInput.value);
                        const sphereValue = isNaN(value) ? 7 : value;
                        sphereInput.value = sphereValue.toString();
                    }
                    applyTruncationFromControls("sphere", truncationControls, viewSettings);
                    needsUpdate = true;
                    break;
                }
                // Toggle truncation
                truncationCheckbox.checked = !truncationCheckbox.checked;
                if (truncationCheckbox.checked) {
                    const value = parseInt(truncationInput.value);
                    if (isNaN(value)) {
                        truncationInput.value = "5";
                    }
                }
                applyTruncationFromControls("ahss", truncationControls, viewSettings);
                needsUpdate = true;
                break;
            }

            case 'u':
            case 'U': {
                if (!usingAhssViewSettings()) {
                    break;
                }
                // Lower bottom truncation (enable if disabled)
                if (!bottomTruncationCheckbox.checked) {
                    bottomTruncationCheckbox.checked = true;
                    const value = parseInt(bottomTruncationInput.value);
                    if (isNaN(value)) {
                        bottomTruncationInput.value = "0";
                    }
                }
                const currentBottom = parseInt(bottomTruncationInput.value);
                const bottomBase = isNaN(currentBottom) ? 0 : currentBottom;
                const newBottom = Math.max(0, bottomBase - 1);
                bottomTruncationInput.value = newBottom.toString();
                applyTruncationFromControls("ahss", truncationControls, viewSettings);
                needsUpdate = true;
                break;
            }

            case 'i':
            case 'I': {
                if (!usingAhssViewSettings()) {
                    break;
                }
                // Higher bottom truncation (enable if disabled)
                if (!bottomTruncationCheckbox.checked) {
                    bottomTruncationCheckbox.checked = true;
                    const value = parseInt(bottomTruncationInput.value);
                    if (isNaN(value)) {
                        bottomTruncationInput.value = "0";
                    }
                }
                const currentBottomInc = parseInt(bottomTruncationInput.value);
                const bottomBaseInc = isNaN(currentBottomInc) ? 0 : currentBottomInc;
                const maxTop = viewSettings.truncation ?? 50;
                const newValue = Math.min(maxTop - 1, bottomBaseInc + 1);
                bottomTruncationInput.value = Math.max(0, newValue).toString();
                applyTruncationFromControls("ahss", truncationControls, viewSettings);
                needsUpdate = true;
                break;
            }

            case 'o':
            case 'O': {
                if (!usingAhssViewSettings()) {
                    break;
                }
                // Toggle bottom truncation
                bottomTruncationCheckbox.checked = !bottomTruncationCheckbox.checked;
                if (bottomTruncationCheckbox.checked) {
                    const value = parseInt(bottomTruncationInput.value);
                    if (isNaN(value)) {
                        bottomTruncationInput.value = "0";
                    }
                }
                applyTruncationFromControls("ahss", truncationControls, viewSettings);
                needsUpdate = true;
                break;
            }

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
    update_ass_chart(viewSettings.truncation, viewSettings.bottomTruncation);

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
