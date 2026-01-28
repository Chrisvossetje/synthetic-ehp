import { Chart } from "./chart";
import { fill_chart, viewSettings, update_ehp_chart } from "./logic";
import { update_ass_chart } from "./ass_logic";
import { ChartMode } from "./chartMode";

// Create two separate chart instances
export const ehpChart = new Chart('svgchart-ehp', ChartMode.EHP);
export const assChart = new Chart('svgchart-ass', ChartMode.ASS);

// Track which chart is currently active
let isEHPActive = true;

// Initialize both charts
fill_chart(ehpChart);
fill_chart(assChart);

// Update charts with initial data
update_ehp_chart();
update_ass_chart(viewSettings.truncation);

setupUIControls();

function updateActiveChart() {
    if (isEHPActive) {
        update_ehp_chart();
    } else {
        update_ass_chart(viewSettings.truncation);
    }
}

function setupUIControls() {
    // EHP/ASS mode switch
    const ehpAssSwitch = document.getElementById('ehp-ass-switch') as HTMLInputElement;
    ehpAssSwitch.addEventListener('change', () => {
        if (ehpAssSwitch.checked) {
            // Switch to ASS mode
            ehpChart.hide();
            assChart.show();
            isEHPActive = false;
        } else {
            // Switch to EHP mode
            assChart.hide();
            ehpChart.show();
            isEHPActive = true;
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
