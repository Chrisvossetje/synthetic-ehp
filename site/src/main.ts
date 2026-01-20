import { Chart } from "./chart";
import { initdata, fill_chart, viewSettings, update_chart } from "./logic";

let chart = new Chart();
initdata(chart);
setupUIControls();

function setupUIControls() {
    // All Diffs checkbox
    const allDiffCheckbox = document.getElementById('all-diff-checkbox') as HTMLInputElement;
    allDiffCheckbox.addEventListener('change', () => {
        viewSettings.allDiffs = allDiffCheckbox.checked;
        update_chart();
    });

    // Page selector
    const pageSelect = document.getElementById('ss-page') as HTMLSelectElement;
    pageSelect.addEventListener('change', () => {
        viewSettings.page = parseInt(pageSelect.value);
        update_chart();
    });

    // Category selector
    const categorySelect = document.getElementById('ss-category') as HTMLSelectElement;
    categorySelect.addEventListener('change', () => {
        viewSettings.category = parseInt(categorySelect.value);
        update_chart();
    });

    // Truncation checkbox and input
    const truncationCheckbox = document.getElementById('truncation-checkbox') as HTMLInputElement;
    const truncationInput = document.querySelector('input[name="Truncation"]') as HTMLInputElement;

    truncationCheckbox.addEventListener('change', () => {
        if (truncationCheckbox.checked) {
            const value = parseInt(truncationInput.value);
            viewSettings.truncation = isNaN(value) ? null : value;
        } else {
            viewSettings.truncation = null;
        }
        update_chart();
    });

    truncationInput.addEventListener('input', () => {
        if (truncationCheckbox.checked) {
            const value = parseInt(truncationInput.value);
            viewSettings.truncation = isNaN(value) ? null : value;
            update_chart();
        }
    });
}