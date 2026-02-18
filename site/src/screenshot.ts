import { Chart } from "./chart";
import { ehpChart } from "./main";
import { isUsingStableData, viewSettings, get_filtered_data, Category } from "./logic";

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

/**
 * Convert torsion/color to TikZ color
 */
function getTikzColor(torsion: number | undefined): string {
    if (torsion === undefined || torsion === 0) return "black";
    if (torsion === 1) return "blue";
    if (torsion === 2) return "red";
    if (torsion === 3) return "green";
    return "black";
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
    const svgPt = pt.matrixTransform(chart.svgchart.inner.getScreenCTM()?.inverse());

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
    const svgPt = pt.matrixTransform(chart.svgchart.inner.getScreenCTM()?.inverse());

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

/**
 * Format generator name for LaTeX display
 */
function formatNameForLatex(name: string): string {
    // Escape square brackets for LaTeX by wrapping in braces
    return name.replace(/\[/g, '{[}').replace(/\]/g, '{]}');
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

    let tikz = "\\begin{figure}\n\\centering\n\\begin{tikzpicture}[scale=1.2]\n";

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
    for (let y = yMin; y < yMax; y++) {
        for (let x = xMin; x < xMax; x++) {
            if ((y === 0 && x !== 0) || x < y) {
                const yFlip = flipY(y);
                const yFlipNext = flipY(y + 1);
                tikz += `\\draw[gray!30] (${x},${yFlip}) -- (${x + 1},${yFlipNext});\n`;
                tikz += `\\draw[gray!30] (${x + 1},${yFlip}) -- (${x},${yFlipNext});\n`;
            }
        }
    }

    // Draw differentials first (so they're behind dots)
    const differentials = chart.differentials.filter(d => {
        const fromLoc = chart.name_to_location.get(d.from);
        const toLoc = chart.name_to_location.get(d.to);
        if (!fromLoc || !toLoc) return false;

        return (fromLoc[0] >= xMin && fromLoc[0] <= xMax && fromLoc[1] >= yMin && fromLoc[1] <= yMax) ||
               (toLoc[0] >= xMin && toLoc[0] <= xMax && toLoc[1] >= yMin && toLoc[1] <= yMax);
    });

    for (const diff of differentials) {
        const fromLoc = chart.name_to_location.get(diff.from);
        const toLoc = chart.name_to_location.get(diff.to);
        if (!fromLoc || !toLoc) continue;

        const color = getTikzColor(diff.coeff);
        const fromYFlip = flipY(fromLoc[1]);
        const toYFlip = flipY(toLoc[1]);
        tikz += `\\draw[${color},line width=0.4pt] (${roundCoord(fromLoc[0])},${roundCoord(fromYFlip)}) -- (${roundCoord(toLoc[0])},${roundCoord(toYFlip)});\n`;
    }

    // Draw generators (dots)
    const generators = chart.generators.filter(g =>
        g.x >= xMin && g.x <= xMax && g.y >= yMin && g.y <= yMax
    );

    for (const gen of generators) {
        const location = chart.name_to_location.get(gen.name);
        if (!location) continue;

        const [cx, cy] = location;
        const cyFlip = flipY(cy);
        const color = getTikzColor(gen.torsion);
        const radius = 0.022; // Match SVG radius of 0.022 in chart units

        // Check if the dot is filled (permanent) by looking at the actual SVG element
        const dotElement = chart.svgchart.shadowRoot?.getElementById(`dot-${gen.name}`) as unknown as SVGCircleElement;
        const isFilled = dotElement && dotElement.style.fill && dotElement.style.fill !== 'white' && dotElement.style.fill !== '';

        if (isFilled) {
            // Permanent cycle - filled dot
            tikz += `\\fill[${color}] (${roundCoord(cx)},${roundCoord(cyFlip)}) circle (${radius});\n`;
        } else {
            // Non-permanent cycle - hollow dot with colored border
            tikz += `\\fill[white] (${roundCoord(cx)},${roundCoord(cyFlip)}) circle (${radius});\n`;
            tikz += `\\draw[${color},line width=0.4] (${roundCoord(cx)},${roundCoord(cyFlip)}) circle (${radius});\n`;
        }

        // Add generator name label (to the left of the dot, very small)
        const latexName = formatNameForLatex(gen.name);
        if (latexName && latexName.length > 0 && latexName.length < 30) {
            tikz += `\\node[anchor=east,scale=0.15,inner sep=0pt] at (${roundCoord(cx - 0.04)},${roundCoord(cyFlip)}) {$${latexName}$};\n`;
        }

        // Add adams filtration (to the right of the dot, very small)
        tikz += `\\node[anchor=west,scale=0.15,gray,inner sep=0pt] at (${roundCoord(cx + 0.04)},${roundCoord(cyFlip)}) {${gen.adams_filtration}};\n`;
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
