import { Category, viewSettings } from "../model/settings";
import { isUsingStableData } from "../model/dataSource";
import { assChart, ehpChart } from "./charts";
import { switchDataSource, update_ehp_chart } from "./ehpChart";
import { update_ass_chart } from "./assChart";
import { applyTruncationFromControls, syncSphereControlsFromState, TruncationControls } from "./truncation";

/**
 * UI control layer.
 *
 * This module is the single home for "a control changed → update the view".
 * Both the DOM event listeners (`setupUIControls`) and the keyboard shortcuts
 * (`keyboard.ts`) route through the same small set of operations here, so there
 * is exactly one implementation of "switch mode", "step the truncation", etc.
 * — no more parallel copies in the change handlers and the keydown switch.
 */

// ---------------------------------------------------------------------------
// Active chart
// ---------------------------------------------------------------------------

let _isEHPActive = true;
export const isEHPActive = () => _isEHPActive;

/** Redraw whichever chart is currently on screen. */
export function updateActiveChart() {
    if (_isEHPActive) {
        update_ehp_chart();
    } else {
        update_ass_chart(viewSettings.truncation, viewSettings.bottomTruncation);
    }
}

// ---------------------------------------------------------------------------
// DOM element access
// ---------------------------------------------------------------------------

const byId = <T extends HTMLElement>(id: string) => document.getElementById(id) as T | null;

export function getTruncationControls(): TruncationControls | null {
    const truncationCheckbox = byId<HTMLInputElement>("truncation-checkbox");
    const truncationInput = document.querySelector('input[name="Truncation"]') as HTMLInputElement | null;
    const bottomTruncationCheckbox = byId<HTMLInputElement>("bottom-truncation-checkbox");
    const bottomTruncationInput = document.querySelector('input[name="BottomTruncation"]') as HTMLInputElement | null;
    const sphereCheckbox = byId<HTMLInputElement>("sphere-truncation-checkbox");
    const sphereInput = document.querySelector('input[name="Sphere"]') as HTMLInputElement | null;

    if (!truncationCheckbox || !truncationInput || !bottomTruncationCheckbox || !bottomTruncationInput || !sphereCheckbox || !sphereInput) {
        return null;
    }
    return { truncationCheckbox, truncationInput, bottomTruncationCheckbox, bottomTruncationInput, sphereCheckbox, sphereInput };
}

/** Whether the AHSS data source is selected (vs. the sphere/EHP view). */
export function usingAhss(): boolean {
    return !!byId<HTMLInputElement>("data-source-switch")?.checked;
}

// ---------------------------------------------------------------------------
// High-level operations (shared by mouse + keyboard)
// ---------------------------------------------------------------------------

/** Switch between the EHP chart and the ASS chart. */
export function setEhpAssMode(toAss: boolean) {
    if (toAss) {
        ehpChart.hide();
        assChart.show();
        _isEHPActive = false;
        window.chartInstance = assChart;
    } else {
        assChart.hide();
        ehpChart.show();
        _isEHPActive = true;
        window.chartInstance = ehpChart;
    }
    syncViewControlsForDataSource(false);
    updateActiveChart();
}

export function setPage(page: number) {
    viewSettings.page = page;
    const select = byId<HTMLSelectElement>("ss-page");
    if (select) select.value = String(page);
    if (_isEHPActive) update_ehp_chart();
}

export function setCategory(category: Category) {
    viewSettings.category = category;
    const select = byId<HTMLSelectElement>("ss-category");
    if (select) select.value = String(category);
    if (_isEHPActive) update_ehp_chart();
}

export function setAllDiffs(value: boolean) {
    viewSettings.allDiffs = value;
    const checkbox = byId<HTMLInputElement>("all-diff-checkbox");
    if (checkbox) checkbox.checked = value;
    if (_isEHPActive) update_ehp_chart();
}
export function toggleAllDiffs() { setAllDiffs(!viewSettings.allDiffs); }

export function setShowFakeData(value: boolean) {
    viewSettings.showFakeData = value;
    const checkbox = byId<HTMLInputElement>("show-fake-checkbox");
    if (checkbox) checkbox.checked = value;
    updateActiveChart();
}
export function toggleShowFakeData() { setShowFakeData(!viewSettings.showFakeData); }

/** Flip the data-source switch and run its change handler (keyboard `s`). */
export function toggleDataSource() {
    const sw = byId<HTMLInputElement>("data-source-switch");
    if (sw) {
        sw.checked = !sw.checked;
        sw.dispatchEvent(new Event("change"));
    }
}

// --- Truncation steppers / toggles ----------------------------------------
//
// Each reads the current control value (substituting a sensible default if the
// field is empty), nudges it, re-applies the truncation to `viewSettings`, and
// redraws. These replace the per-key copies that used to live in main.ts.

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));

function applyAndUpdate(mode: "ahss" | "sphere") {
    const controls = getTruncationControls();
    if (!controls) return;
    applyTruncationFromControls(mode, controls, viewSettings);
    updateActiveChart();
}

export function stepSphere(delta: number) {
    const c = getTruncationControls();
    if (!c) return;
    if (!c.sphereCheckbox.checked) c.sphereCheckbox.checked = true;
    const current = parseInt(c.sphereInput.value);
    const base = isNaN(current) ? 7 : current;
    c.sphereInput.value = clamp(base + delta, 2, 50).toString();
    applyAndUpdate("sphere");
}

export function toggleSphere() {
    const c = getTruncationControls();
    if (!c) return;
    c.sphereCheckbox.checked = !c.sphereCheckbox.checked;
    if (c.sphereCheckbox.checked) {
        const value = parseInt(c.sphereInput.value);
        c.sphereInput.value = (isNaN(value) ? 7 : value).toString();
    }
    applyAndUpdate("sphere");
}

export function stepTopTruncation(delta: number) {
    const c = getTruncationControls();
    if (!c) return;
    if (!c.truncationCheckbox.checked) {
        c.truncationCheckbox.checked = true;
        if (isNaN(parseInt(c.truncationInput.value))) c.truncationInput.value = "5";
    }
    const current = parseInt(c.truncationInput.value);
    const base = isNaN(current) ? 5 : current;
    c.truncationInput.value = clamp(base + delta, 2, 50).toString();
    applyAndUpdate("ahss");
}

export function toggleTopTruncation() {
    const c = getTruncationControls();
    if (!c) return;
    c.truncationCheckbox.checked = !c.truncationCheckbox.checked;
    if (c.truncationCheckbox.checked && isNaN(parseInt(c.truncationInput.value))) {
        c.truncationInput.value = "5";
    }
    applyAndUpdate("ahss");
}

export function stepBottomTruncation(delta: number) {
    const c = getTruncationControls();
    if (!c) return;
    if (!c.bottomTruncationCheckbox.checked) {
        c.bottomTruncationCheckbox.checked = true;
        if (isNaN(parseInt(c.bottomTruncationInput.value))) c.bottomTruncationInput.value = "0";
    }
    const current = parseInt(c.bottomTruncationInput.value);
    const base = isNaN(current) ? 0 : current;
    // Decreasing only floors at 0; increasing also caps just below the top.
    const next = delta < 0
        ? Math.max(0, base + delta)
        : Math.max(0, Math.min((viewSettings.truncation ?? 50) - 1, base + delta));
    c.bottomTruncationInput.value = next.toString();
    applyAndUpdate("ahss");
}

export function toggleBottomTruncation() {
    const c = getTruncationControls();
    if (!c) return;
    c.bottomTruncationCheckbox.checked = !c.bottomTruncationCheckbox.checked;
    if (c.bottomTruncationCheckbox.checked && isNaN(parseInt(c.bottomTruncationInput.value))) {
        c.bottomTruncationInput.value = "0";
    }
    applyAndUpdate("ahss");
}

// ---------------------------------------------------------------------------
// Data-source-dependent control visibility
// ---------------------------------------------------------------------------

/**
 * Show the truncation controls that match the current data source (AHSS shows
 * top/bottom truncation; the synthetic/EHP view shows the sphere selector) and
 * sync their values from `viewSettings`. Optionally redraw afterwards.
 */
export function syncViewControlsForDataSource(updateChart = true) {
    const truncationControls = byId<HTMLDivElement>("truncation-controls");
    const sphereControls = byId<HTMLDivElement>("sphere-controls");
    const fakeControls = byId<HTMLDivElement>("fake-controls");
    const showFakeCheckbox = byId<HTMLInputElement>("show-fake-checkbox");
    const dataSourceSwitch = byId<HTMLInputElement>("data-source-switch");
    const controls = getTruncationControls();

    if (!dataSourceSwitch || !truncationControls || !sphereControls || !fakeControls || !showFakeCheckbox || !controls) {
        return;
    }

    fakeControls.style.display = "flex";
    fakeControls.style.opacity = "1";
    fakeControls.style.pointerEvents = "auto";
    showFakeCheckbox.disabled = false;

    if (dataSourceSwitch.checked) {
        truncationControls.style.display = "flex";
        sphereControls.style.display = "none";
        syncSphereControlsFromState(controls, viewSettings, true);
        applyTruncationFromControls("ahss", controls, viewSettings);
    } else {
        truncationControls.style.display = "none";
        sphereControls.style.display = "flex";
        syncSphereControlsFromState(controls, viewSettings, false);
        applyTruncationFromControls("sphere", controls, viewSettings);
    }

    if (updateChart) {
        updateActiveChart();
    }
}

// ---------------------------------------------------------------------------
// Wiring
// ---------------------------------------------------------------------------

/** Debounce a zero-arg callback (used for the free-typed number inputs). */
function debounce(fn: () => void, ms: number): () => void {
    let timeout: number | null = null;
    return () => {
        if (timeout !== null) window.clearTimeout(timeout);
        timeout = window.setTimeout(() => {
            timeout = null;
            fn();
        }, ms);
    };
}

/** Attach all DOM event listeners. Call once after the data is loaded. */
export function setupUIControls() {
    const ehpAssSwitch = byId<HTMLInputElement>("ehp-ass-switch");
    ehpAssSwitch?.addEventListener("change", () => setEhpAssMode(ehpAssSwitch.checked));

    const allDiffCheckbox = byId<HTMLInputElement>("all-diff-checkbox");
    allDiffCheckbox?.addEventListener("change", () => setAllDiffs(allDiffCheckbox.checked));

    const showFakeCheckbox = byId<HTMLInputElement>("show-fake-checkbox");
    showFakeCheckbox?.addEventListener("change", () => setShowFakeData(showFakeCheckbox.checked));

    const pageSelect = byId<HTMLSelectElement>("ss-page");
    pageSelect?.addEventListener("change", () => setPage(parseInt(pageSelect.value)));

    const categorySelect = byId<HTMLSelectElement>("ss-category");
    categorySelect?.addEventListener("change", () => setCategory(parseInt(categorySelect.value)));

    const controls = getTruncationControls();
    if (controls) {
        // AHSS top/bottom truncation: act on change, debounce while typing.
        const applyAhss = () => { applyAndUpdate("ahss"); };
        const debouncedAhss = debounce(applyAhss, 120);

        controls.truncationCheckbox.addEventListener("change", () => { if (usingAhss()) applyAhss(); });
        controls.truncationInput.addEventListener("input", () => {
            if (usingAhss() && controls.truncationCheckbox.checked) debouncedAhss();
        });
        controls.bottomTruncationCheckbox.addEventListener("change", () => { if (usingAhss()) applyAhss(); });
        controls.bottomTruncationInput.addEventListener("input", () => {
            if (usingAhss() && controls.bottomTruncationCheckbox.checked) debouncedAhss();
        });

        // Sphere selector (synthetic/EHP view).
        const applySphere = () => { applyAndUpdate("sphere"); };
        const debouncedSphere = debounce(applySphere, 120);
        controls.sphereCheckbox.addEventListener("change", () => { if (!usingAhss()) applySphere(); });
        controls.sphereInput.addEventListener("input", () => {
            if (!usingAhss() && controls.sphereCheckbox.checked) debouncedSphere();
        });
    }

    const dataSourceSwitch = byId<HTMLInputElement>("data-source-switch");
    dataSourceSwitch?.addEventListener("change", async () => {
        dataSourceSwitch.disabled = true;
        await switchDataSource();
        dataSourceSwitch.checked = isUsingStableData();
        dataSourceSwitch.disabled = false;
        syncViewControlsForDataSource();
    });

    syncViewControlsForDataSource();
}
