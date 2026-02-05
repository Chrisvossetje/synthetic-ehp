import { Chart } from "./chart";
import { viewSettings, update_ehp_chart, fill_ehp_chart, Category } from "./logic";
import { update_ass_chart } from "./ass_logic";
import { ChartMode } from "./chartMode";

// Create two separate chart instances
export const ehpChart = new Chart('svgchart-ehp', ChartMode.EHP);
export const assChart = new Chart('svgchart-ass', ChartMode.ASS);

// Track which chart is currently active
let isEHPActive = true;

// Set up global reference for onclick handlers
declare global {
    interface Window {
        chartInstance: Chart;
    }
}

// Initialize both charts
fill_ehp_chart();

// Update charts with initial data
update_ehp_chart();
update_ass_chart(viewSettings.truncation);

// Set initial global chart reference
window.chartInstance = ehpChart;

setupUIControls();

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

    truncationInput.addEventListener('input', () => {
        if (truncationCheckbox.checked) {
            const value = parseInt(truncationInput.value);
            viewSettings.truncation = isNaN(value) ? null : value;
            updateActiveChart();
        }
    });
}

function setupKeyboardControls() {
    document.addEventListener('keydown', (e) => {
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

        switch(e.key) {
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
                    const newValue = Math.max(1, viewSettings.truncation - 1);
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
        }

        if (needsUpdate) {
            updateActiveChart();
        }
    });
}
