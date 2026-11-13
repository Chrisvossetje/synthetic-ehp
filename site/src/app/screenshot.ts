import { Chart, h0MultiplicationControls, MULTIPLICATION_COLOR, tagLabelDirection } from "../chart/chart";
import { ehpChart } from "./charts";
import { getDisplayedGenerator } from "./ehpChart";
import { isUsingStableData } from "../model/dataSource";
import { shouldIncludeKind, viewSettings } from "../model/settings";

/**
 * Screenshot / TikZ export.
 *
 * Lets the user drag-select a rectangular region of the EHP chart and copies a
 * standalone TikZ `figure` reproducing that region to the clipboard. It reads
 * the displayed visibility, dot colors, and Adams filtration so the exported
 * figure matches the chart.
 */

// Screenshot state
let screenshotState: "idle" | "selecting" | "capturing" = "idle";
let firstPoint: { x: number; y: number } | null = null;
let selectionRect: SVGRectElement | null = null;

/**
 * Round coordinate to 3 decimal places
 */
function roundCoord(x: number): string {
    return (Math.round(x * 1000) / 1000).toString();
}

function getTikzColor(cssColor: string): string {
    const rgb = cssColor.startsWith("#")
        ? [1, 3, 5].map((offset) => parseInt(cssColor.slice(offset, offset + 2), 16))
        : /^rgb\((\d+),\s*(\d+),\s*(\d+)\)$/.exec(cssColor)?.slice(1);
    return rgb ? `{rgb,255:red,${rgb[0]};green,${rgb[1]};blue,${rgb[2]}}` : cssColor;
}

/**
 * Start screenshot mode
 */
export function startScreenshot() {
    screenshotState = "selecting";
    document.body.style.cursor = "crosshair";

    // Create selection rectangle if it doesn't exist
    if (!selectionRect) {
        selectionRect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
        selectionRect.setAttribute("fill", "blue");
        selectionRect.setAttribute("opacity", "0.2");
        selectionRect.setAttribute("stroke", "blue");
        selectionRect.setAttribute("stroke-width", "0.05");
        selectionRect.setAttribute("pointer-events", "none");
        selectionRect.style.display = "none";

        // Add to the chart's invalidCells group so it transforms with the chart
        if (ehpChart.svgchart.invalidCells) {
            ehpChart.svgchart.invalidCells.appendChild(selectionRect);
        }
    }

    console.log("Screenshot mode: Click and drag to select region");
}

/**
 * Handle pointer down during screenshot
 */
export function handleScreenshotPointerDown(event: PointerEvent, chart: Chart) {
    if (screenshotState !== "selecting") return false;

    const svg = chart.svgchart.svg;
    const pt = svg.createSVGPoint();
    pt.x = event.clientX;
    pt.y = event.clientY;

    // Transform to chart coordinates
    if (chart.svgchart.inner instanceof SVGGraphicsElement) {
        const svgPt = pt.matrixTransform(chart.svgchart.inner.getScreenCTM()!.inverse());
        
        firstPoint = { x: svgPt.x, y: svgPt.y };
    
        if (selectionRect) {
            selectionRect.style.display = "block";
            selectionRect.setAttribute("x", firstPoint.x.toString());
            selectionRect.setAttribute("y", firstPoint.y.toString());
            selectionRect.setAttribute("width", "0");
            selectionRect.setAttribute("height", "0");
        }
    
        return true;
    }
}

/**
 * Handle pointer move during screenshot
 */
export function handleScreenshotPointerMove(event: PointerEvent, chart: Chart) {
    if (screenshotState !== "selecting" || !firstPoint || !selectionRect) return false;

    const svg = chart.svgchart.svg;
    const pt = svg.createSVGPoint();
    pt.x = event.clientX;
    pt.y = event.clientY;

    // Transform to chart coordinates
    if (chart.svgchart.inner instanceof SVGGraphicsElement) {
        const svgPt = pt.matrixTransform(chart.svgchart.inner.getScreenCTM()?.inverse());

        const x = Math.min(firstPoint.x, svgPt.x);
        const y = Math.min(firstPoint.y, svgPt.y);
        const width = Math.abs(svgPt.x - firstPoint.x);
        const height = Math.abs(svgPt.y - firstPoint.y);

        selectionRect.setAttribute("x", x.toString());
        selectionRect.setAttribute("y", y.toString());
        selectionRect.setAttribute("width", width.toString());
        selectionRect.setAttribute("height", height.toString());

        return true;
    }
}

/**
 * Handle pointer up during screenshot - capture the region
 */
export function handleScreenshotPointerUp(event: PointerEvent, chart: Chart) {
    if (screenshotState !== "selecting" || !firstPoint || !selectionRect) return false;

    const svg = chart.svgchart.svg;
    const pt = svg.createSVGPoint();
    pt.x = event.clientX;
    pt.y = event.clientY;

    // Transform to chart coordinates
    if (chart.svgchart.inner instanceof SVGGraphicsElement) {
        const svgPt = pt.matrixTransform(chart.svgchart.inner.getScreenCTM()!.inverse());

        const x1 = Math.min(firstPoint.x, svgPt.x);
        const x2 = Math.max(firstPoint.x, svgPt.x);
        const y1 = Math.min(firstPoint.y, svgPt.y);
        const y2 = Math.max(firstPoint.y, svgPt.y);

        // Hide selection rectangle
        selectionRect.style.display = "none";

        // Reset state
        screenshotState = "idle";
        document.body.style.cursor = "default";
        firstPoint = null;

        // Generate TikZ code
        generateTikzCode(x1, x2, y1, y2, chart);

        return true;
    }
}

/**
 * Format generator name for LaTeX display
 */
function formatNameForLatex(name: string): string {
    // Spaces terminate the angle macros before adjacent letters or numbers.
    const withLatexSymbols = name.replace(/∞/g, '\\infty')
        .replace(/</g, '\\langle ').replace(/>/g, '\\rangle ');
    // Escape square brackets for LaTeX by wrapping in braces
    return withLatexSymbols.replace(/\[/g, '{[}').replace(/\]/g, '{]}');
}

function isVisibleSvgElement(element: SVGElement | null): boolean {
    return !!element && element.style.visibility !== "hidden" && element.style.display !== "none";
}

function getVisibleDotElement(chart: Chart, name: string): SVGCircleElement | SVGRectElement | null {
    const element = chart.svgchart.shadowRoot?.getElementById(`dot-${name}`) as unknown as SVGCircleElement | SVGRectElement | null;
    return isVisibleSvgElement(element) ? element : null;
}

function getVisibleDiffElement(chart: Chart, from: string, to: string): SVGLineElement | null {
    const element = chart.svgchart.shadowRoot?.getElementById(`diff-${from}-${to}`) as unknown as SVGLineElement | null;
    return isVisibleSvgElement(element) ? element : null;
}

function getVisibleTauMultElement(chart: Chart, from: string, to: string): SVGLineElement | null {
    const element = chart.svgchart.shadowRoot?.getElementById(`tau-mult-${from}-${to}`) as unknown as SVGLineElement | null;
    return isVisibleSvgElement(element) ? element : null;
}

/**
 * Generate TikZ code for the selected region
 */
function generateTikzCode(x1: number, x2: number, y1: number, y2: number, chart: Chart) {
    // Round to integer boundaries (inclusive range)
    const xMin = Math.floor(x1);
    const xMax = Math.floor(x2);
    const yMin = Math.floor(y1);
    const yMax = Math.floor(y2);

    // Flip Y coordinates for TikZ (TikZ has y increasing upward, chart has y increasing downward in display)
    const flipY = (y: number) => -y;

    const yMinFlipped = flipY(yMax);
    const yMaxFlipped = flipY(yMin);

    const isPointInSelectedRegion = ([x, y]: [number, number]): boolean =>
        x >= xMin && x <= xMax + 1 && y >= yMin && y <= yMax + 1;

    let tikz = "\\begin{figure}[H]\n\\centering\n\\begin{tikzpicture}\n";

    // Draw border (add 1 to max values to include the full cell)
    tikz += `\\draw (${xMin},${flipY(yMax + 1)}) rectangle (${xMax + 1},${flipY(yMin)});\n`;

    // Draw grid (at integer positions to define cells)
    tikz += `\\begin{scope}\n\\clip (${xMin},${flipY(yMax + 1)}) rectangle (${xMax + 1},${flipY(yMin)});\n`;
    for (let x = xMin; x <= xMax + 1; x++) {
        tikz += `\\draw[black!10] (${x},${flipY(yMax + 1)}) -- (${x},${flipY(yMin)});\n`;
    }
    for (let y = yMin; y <= yMax + 1; y++) {
        const yFlip = flipY(y);
        tikz += `\\draw[black!10] (${xMin},${yFlip}) -- (${xMax + 1},${yFlip});\n`;
    }

    // Draw crossed-out cells (invalid cells)
    for (let y = yMin; y <= yMax; y++) {
        for (let x = xMin; x <= xMax; x++) {
            if ((y === 0 && x !== 0) || x < y) {
                const yFlip = flipY(y);
                const yFlipNext = flipY(y + 1);
                tikz += `\\draw[gray!30] (${x},${yFlip}) -- (${x + 1},${yFlipNext});\n`;
                tikz += `\\draw[gray!30] (${x + 1},${yFlip}) -- (${x},${yFlipNext});\n`;
            }
        }
    }

    // Draw stable reference line (same staircase pattern as chart.ts), clipped to selection.
    let stablePath = `(0,${roundCoord(flipY(1))})`;
    let stableX = 0;
    let stableY = 1;
    while (stableX < xMax + 3) {
        stableX += 3;
        stablePath += ` -- (${roundCoord(stableX)},${roundCoord(flipY(stableY))})`;
        stableY += 1;
        stablePath += ` -- (${roundCoord(stableX)},${roundCoord(flipY(stableY))})`;
    }
    tikz += `\\draw[color={rgb,255:red,255;green,107;blue,0},line width=0.4pt,dash pattern=on 2pt off 1pt] ${stablePath};\n`;

    // Draw differentials first (so they're behind dots)
    for (const diff of chart.differentials) {
        if (!shouldIncludeKind(diff.kind)) continue;
        const line = getVisibleDiffElement(chart, diff.from, diff.to);
        if (!line) continue;
        const fromLoc = chart.name_to_location.get(diff.from);
        const toLoc = chart.name_to_location.get(diff.to);
        if (!fromLoc || !toLoc) continue;
        if (!isPointInSelectedRegion(fromLoc) && !isPointInSelectedRegion(toLoc)) continue;

        const color = getTikzColor(line.style.stroke);
        const lineStyle = (diff.kind === "Algebraic" || diff.kind === "Real") ?  "": ",dotted";
        const fromYFlip = flipY(fromLoc[1]);
        const toYFlip = flipY(toLoc[1]);
        tikz += `\\draw[color=${color},line width=0.4pt${lineStyle}] (${roundCoord(fromLoc[0])},${roundCoord(fromYFlip)}) -- (${roundCoord(toLoc[0])},${roundCoord(toYFlip)});\n`;
    }

    // Draw tau extensions with the same visibility/page/fake filtering as the chart.
    const tauMults = chart.tau_mults.filter((tauMult) => {
        if (!getVisibleTauMultElement(chart, tauMult.from, tauMult.to)) {
            return false;
        }

        const fromLoc = chart.name_to_location.get(tauMult.from);
        const toLoc = chart.name_to_location.get(tauMult.to);
        if (!fromLoc || !toLoc) return false;

        return isPointInSelectedRegion(fromLoc) || isPointInSelectedRegion(toLoc);
    });

    for (const tauMult of tauMults) {
        const fromLoc = chart.name_to_location.get(tauMult.from);
        const toLoc = chart.name_to_location.get(tauMult.to);
        if (!fromLoc || !toLoc) continue;

        const lineStyle = tauMult.kind !== "Real" ? ",dotted" : "";
        const fromYFlip = flipY(fromLoc[1]);
        const toYFlip = flipY(toLoc[1]);
        tikz += `\\draw[color={rgb,255:red,255;green,165;blue,0},line width=0.4pt${lineStyle}] (${roundCoord(fromLoc[0])},${roundCoord(fromYFlip)}) -- (${roundCoord(toLoc[0])},${roundCoord(toYFlip)});\n`;
    }

    // Multiplication overlays are already resolved/filtered exactly as drawn.
    for (const mult of chart.displayMultiplications) {
        const fromLoc = chart.name_to_location.get(mult.from);
        const toLoc = chart.name_to_location.get(mult.to);
        if (!fromLoc || !toLoc || !(isPointInSelectedRegion(fromLoc) || isPointInSelectedRegion(toLoc))) continue;
        const color = MULTIPLICATION_COLOR;
        const rgb = [1, 3, 5].map((offset) => parseInt(color.slice(offset, offset + 2), 16));
        let segment = " -- ";
        if (mult.operator === 0) {
            const [c1, c2] = h0MultiplicationControls(fromLoc, toLoc,
                chart.generator_row(mult.from) === chart.generator_row(mult.to));
            segment = ` .. controls (${roundCoord(c1[0])},${roundCoord(flipY(c1[1]))}) and (${roundCoord(c2[0])},${roundCoord(flipY(c2[1]))}) .. `;
        }
        tikz += `\\draw[color={rgb,255:red,${rgb[0]};green,${rgb[1]};blue,${rgb[2]}},line width=0.3pt] (${roundCoord(fromLoc[0])},${roundCoord(flipY(fromLoc[1]))})${segment}(${roundCoord(toLoc[0])},${roundCoord(flipY(toLoc[1]))});\n`;
    }

    // Draw generators (dots)
    const generators = chart.generators.filter(g =>
        g.stem >= xMin && g.stem <= xMax && g.y >= yMin && g.y <= yMax
    );

    for (const gen of generators) {
        const location = chart.name_to_location.get(gen.name);
        const dotElement = getVisibleDotElement(chart, gen.name);
        if (!location || !dotElement) continue;

        const [cx, cy] = location;
        const cyFlip = flipY(cy);
        const af = getDisplayedGenerator(gen.name)?.[1] ?? gen.af;
        const border = getTikzColor(dotElement.style.stroke);
        const fill = getTikzColor(dotElement.style.fill);
        const radius = chart.get_dot_radius();

        const shape = dotElement.localName === "rect"
            ? `(${roundCoord(cx - radius)},${roundCoord(cyFlip - radius)}) rectangle (${roundCoord(cx + radius)},${roundCoord(cyFlip + radius)})`
            : `(${roundCoord(cx)},${roundCoord(cyFlip)}) circle (${radius})`;

        tikz += `\\filldraw[draw=${border},fill=${fill},line width=0.4pt] ${shape};\n`;

        const tagDirection = tagLabelDirection(gen.y);
        const tagAnchor = tagDirection === -1 ? "east" : "west";
        const afAnchor = tagDirection === -1 ? "west" : "east";
        const power = chart.generator_tau_power(gen.name);
        const tag = formatNameForLatex(chart.generator_tag(gen.name));
        const latexName = (power ? `\\tau^{${power}}\\,` : "") + tag;
        if (latexName && latexName.length > 0) {
            tikz += `\\node[anchor=${tagAnchor},scale=0.15,inner sep=0pt] at (${roundCoord(cx + 0.04 * tagDirection)},${roundCoord(cyFlip)}) {$${latexName}$};\n`;
        }

        tikz += `\\node[anchor=${afAnchor},scale=0.15,gray,inner sep=0pt] at (${roundCoord(cx - 0.04 * tagDirection)},${roundCoord(cyFlip)}) {${af}};\n`;
    }

    tikz += "\\end{scope}\n";

    // Draw axis labels (outside the clip region)
    // X-axis labels at the top (above the chart)
    for (let x = xMin; x <= xMax; x++) {
        tikz += `\\node[font=\\small] at (${x + 0.5},${flipY(yMin) + 0.3}) {$${x}$};\n`;
    }
    // Y-axis labels on the left
    for (let y = yMin; y <= yMax; y++) {
        const yFlip = flipY(y);
        tikz += `\\node[font=\\small] at (${xMin - 0.3},${yFlip - 0.5}) {$${y}$};\n`;
    }

    tikz += "\\end{tikzpicture}\n";

    // Add caption with page information
    const pageInfo = viewSettings.page === 1000 ? "E∞" : `E${viewSettings.page}`;
    tikz += `\\caption{EHP Chart${isUsingStableData() ? " (Stable)" : ""} - ${pageInfo} - Region (${xMin},${yMin}) to (${xMax},${yMax})}\n`;
    tikz += "\\end{figure}\n";

    // Copy to clipboard
    navigator.clipboard.writeText(tikz).then(() => {
        alert(`TikZ code copied to clipboard!\nRegion: (${xMin},${yMin}) to (${xMax},${yMax})\nPage: ${pageInfo}`);
    }).catch(err => {
        console.error('Failed to copy to clipboard:', err);
        alert('Failed to copy to clipboard. Check console for TikZ code.');
        console.log(tikz);
    });
}

/**
 * Cancel screenshot mode
 */
export function cancelScreenshot() {
    screenshotState = "idle";
    document.body.style.cursor = "default";
    firstPoint = null;

    if (selectionRect) {
        selectionRect.style.display = "none";
    }
}

/**
 * Check if currently in screenshot mode
 */
export function isScreenshotMode(): boolean {
    return screenshotState !== "idle";
}
