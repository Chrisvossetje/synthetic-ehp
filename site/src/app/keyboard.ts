import { Category } from "../model/settings";
import { cancelScreenshot, isScreenshotMode, startScreenshot } from "./screenshot";
import {
    isEHPActive,
    setCategory,
    setEhpAssMode,
    setPage,
    stepBottomTruncation,
    stepSphere,
    stepTopTruncation,
    toggleAllDiffs,
    toggleBottomTruncation,
    toggleDataSource,
    toggleShowFakeData,
    toggleSphere,
    toggleTopTruncation,
    usingAhss,
} from "./controls";

/**
 * Keyboard shortcuts.
 *
 * Every key maps onto an operation in `controls.ts` — this file is purely the
 * key → action table, with no view logic of its own. The truncation keys act on
 * the sphere selector or the AHSS top/bottom truncation depending on the
 * current data source.
 *
 *   s            toggle data source (EHP ⇄ AHSS data)
 *   a            toggle EHP ⇄ ASS chart
 *   1–9 / 0      page E1–E9 / E∞
 *   q / w / e    category Synthetic / Algebraic / Geometric
 *   d            toggle "all differentials"
 *   f            toggle fake data
 *   j / k / l    decrease / increase / toggle  (sphere or top truncation)
 *   u / i / o    decrease / increase / toggle  bottom truncation (AHSS only)
 *   p            start screenshot/TikZ export (EHP only)
 *   Esc          close info popup, then cancel screenshot
 */
export function setupKeyboardControls() {
    document.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
            const floatingBox = document.getElementById("floatingBox");
            if (floatingBox && floatingBox.style.display !== "none") {
                floatingBox.style.display = "none";
                return;
            }
            if (isScreenshotMode()) {
                cancelScreenshot();
                return;
            }
        }

        // Ignore shortcuts while typing in a form field.
        if (e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement) {
            return;
        }

        // Pages: 1–9 select that page, 0 selects E∞.
        if (e.key >= "1" && e.key <= "9") {
            setPage(parseInt(e.key));
            return;
        }

        switch (e.key) {
            case "s": case "S": toggleDataSource(); return;
            case "a": case "A": {
                const sw = document.getElementById("ehp-ass-switch") as HTMLInputElement;
                sw.checked = !sw.checked;
                setEhpAssMode(sw.checked);
                return;
            }
            case "0": setPage(1000); return;

            case "q": case "Q": setCategory(Category.Synthetic); return;
            case "w": case "W": setCategory(Category.Algebraic); return;
            case "e": case "E": setCategory(Category.Geometric); return;

            case "d": case "D": toggleAllDiffs(); return;
            case "f": case "F": toggleShowFakeData(); return;

            case "j": case "J": usingAhss() ? stepTopTruncation(-1) : stepSphere(-1); return;
            case "k": case "K": usingAhss() ? stepTopTruncation(1) : stepSphere(1); return;
            case "l": case "L": usingAhss() ? toggleTopTruncation() : toggleSphere(); return;

            case "u": case "U": if (usingAhss()) stepBottomTruncation(-1); return;
            case "i": case "I": if (usingAhss()) stepBottomTruncation(1); return;
            case "o": case "O": if (usingAhss()) toggleBottomTruncation(); return;

            case "p": case "P": if (isEHPActive()) startScreenshot(); return;
        }
    });
}
