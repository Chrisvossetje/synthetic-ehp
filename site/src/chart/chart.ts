import { ToStringMap } from "./stringMap";
import { SvgChart } from "./svgChart";
import { Differential, DisplayMultiplication, ExternalTauMult, Generators, InternalTauMult, Multiplication } from "../types";
import { displayName } from "../model/tagNames";
import { viewSettings } from "../model/settings";
import { ChartMode } from "./chartMode";
import { svgNS } from "./svgChart";
import { escapeHtml } from "./text";
import { renderAdamsTag } from "./tagMath";

type Point = [number, number];

export const MULTIPLICATION_COLOR = "#b0b0b0";

export function tagLabelDirection(row: number): -1 | 1 {
    return row % 2 === 0 ? 1 : -1;
}

/** Bend grows gently with distance, approaching a cap on long products. */
export function h0MultiplicationControls(from: Point, to: Point, sameRow: boolean): [Point, Point] {
    const dx = to[0] - from[0];
    const dy = to[1] - from[1];
    const distance = Math.hypot(dx, dy);
    if (distance === 0) return [from, to];
    const bend =sameRow ? 0.025 : 0;
    const controlAt = (t: number): Point => [
        from[0] + t * dx - bend * dy / distance,
        from[1] + t * dy + bend * dx / distance,
    ];
    return [controlAt(1 / 3), controlAt(2 / 3)];
}

const TORSION_COLORS = ["#0080ff", "#ff0000", "#3cb371", "#00ffff", "#800080"];

function torsionColor(order: number | undefined): string {
    return order && order > 0 ? TORSION_COLORS[(order - 1) % TORSION_COLORS.length] : "black";
}

type DotDisplay = {
    currentTorsion?: number;
    /** Zero means the class dies. Undefined means it survives as a free module. */
    finalTorsion?: number;
    filtration?: number;
    initialFiltration?: number;
};

export class Chart {
    public svgchart: SvgChart;
    private maxStem: number = 32;

    // Use generator names as unique identifiers
    public name_to_location: Map<string, Point> = new Map();

    // All generators and differentials (complete data set)
    public generators: Generators[] = [];
    private generatorsByName = new Map<string, Generators>();
    private tauPowers = new Map<string, number>();
    public differentials: Differential[] = [];
    public multiplications: Multiplication[] = [];
    public tau_mults: (InternalTauMult | ExternalTauMult)[] = [];
    public displayMultiplications: DisplayMultiplication[] = [];

    // Cached element references
    private dotElements: Map<string, SVGCircleElement | SVGRectElement> = new Map();
    private labelElements: Map<string, SVGForeignObjectElement> = new Map();
    private filtrationElements: Map<string, SVGTextElement> = new Map();
    private diffElements: Map<string, SVGLineElement> = new Map();
    private multElements: Map<string, SVGLineElement> = new Map();
    private tauMultElements: Map<string, SVGLineElement> = new Map();

    // Track current bounds to avoid unnecessary zoom resets
    private currentBounds: [number, number, number, number] | null = null;

    // Chart display mode (EHP vs ASS)
    public readonly mode: ChartMode;
    private showNames: boolean;
    private showFiltration: boolean;

    public dotCallback?: (name: string) => void;
    public lineCallback?: (from: string, to: string) => void;
    public tauMultCallback?: (from: string, to: string) => void;
    public multiplicationCallback?: (relation: DisplayMultiplication) => void;

    constructor(containerId: string, mode: ChartMode) {
        this.mode = mode;
        this.showNames = mode !== ChartMode.ASS;
        this.showFiltration = mode !== ChartMode.ASS;
        // Pass mode to SvgChart for appropriate styling
        this.svgchart = new SvgChart(mode);
        const container = document.getElementById(containerId);
        if (container) {
            container.append(this.svgchart);
        }
    }

    set_label_display(showNames: boolean, showFiltration: boolean) {
        this.showNames = showNames;
        this.showFiltration = showFiltration;
    }

    clear() {
        this.displayMultiplications = [];
        this.svgchart.replace_inner("");
    }

    /** Replace only the overlay so changing pages/toggles preserves zoom. */
    set_display_multiplications(relations: DisplayMultiplication[]) {
        this.displayMultiplications = relations;
        const contents = this.svgchart.shadowRoot?.getElementById("contents");
        if (!contents) return;
        contents.querySelector(".h-multiplications")?.remove();
        const layer = document.createElementNS(svgNS, "g");
        layer.setAttribute("class", "h-multiplications");
        for (const relation of relations) {
            const from = this.name_to_location.get(relation.from);
            const to = this.name_to_location.get(relation.to);
            if (!from || !to) continue;
            const group = document.createElementNS(svgNS, "g");
            group.setAttribute("class", `h-multiplication h-multiplication-${relation.operator}`);
            group.setAttribute("data-from", relation.from);
            group.setAttribute("data-to", relation.to);
            group.setAttribute("data-sphere", String(relation.sphere));
            const title = document.createElementNS(svgNS, "title");
            title.textContent = `h_${relation.operator} · ${this.generator_label(relation.from)} = ${(relation.displayTargets ?? [this.generator_label(relation.to)]).join(" + ")} (sphere ${relation.sphere})`;
            group.append(title);
            const line = document.createElementNS(svgNS, relation.operator === 0 ? "path" : "line");
            if (relation.operator === 0) {
                const [c1, c2] = h0MultiplicationControls(from, to,
                    this.generator_row(relation.from) === this.generator_row(relation.to));
                line.setAttribute("d", `M ${from[0]} ${from[1]} C ${c1[0]} ${c1[1]} ${c2[0]} ${c2[1]} ${to[0]} ${to[1]}`);
                line.setAttribute("fill", "none");
            } else {
                line.setAttribute("x1", String(from[0]));
                line.setAttribute("y1", String(from[1]));
                line.setAttribute("x2", String(to[0]));
                line.setAttribute("y2", String(to[1]));
            }
            const color = this.mode === ChartMode.ASS ? "black" : MULTIPLICATION_COLOR;
            line.setAttribute("stroke", color);
            const width = this.mode === ChartMode.ASS ? 0.02 : 0.01;
            line.setAttribute("stroke-width", String(width));
            line.setAttribute("pointer-events", "none");
            const hit = line.cloneNode(true) as SVGPathElement | SVGLineElement;
            hit.setAttribute("class", "h-multiplication-hit");
            hit.setAttribute("stroke", "transparent");
            // Keep the clickable stroke usable at every zoom level.
            hit.setAttribute("stroke-width", "8");
            hit.setAttribute("vector-effect", "non-scaling-stroke");
            hit.setAttribute("stroke-linecap", "round");
            hit.setAttribute("pointer-events", "stroke");
            hit.style.cursor = "pointer";
            group.append(line, hit);
            group.addEventListener("click", (event) => {
                event.stopPropagation();
                this.multiplicationCallback?.(relation);
            });
            group.addEventListener("pointerenter", () => {
                line.setAttribute("stroke", this.mode === ChartMode.ASS ? "black" : "#555");
                line.setAttribute("stroke-width", String(width * 2.5));
            });
            group.addEventListener("pointerleave", () => {
                line.setAttribute("stroke", color);
                line.setAttribute("stroke-width", String(width));
            });
            layer.append(group);
        }
        contents.prepend(layer);
    }

    /**
     * Show this chart
     */
    show() {
        const container = this.svgchart.parentElement;
        if (container) {
            container.style.display = 'block';
            // Trigger resize and reapply zoom after showing to ensure proper dimensions
            setTimeout(() => this.svgchart.onResize(), 0);
        }
    }

    /**
     * Hide this chart
     */
    hide() {
        const container = this.svgchart.parentElement;
        if (container) {
            container.style.display = 'none';
        }
    }

    // Set all generators (complete data set)
    set_all_generators(generators: Generators[]) {
        this.generators = generators;
        this.generatorsByName = new Map(generators.map((gen) => [gen.name, gen]));
        this.tauPowers.clear();
        this.maxStem = this.computeMaxStem();
    }

    generator_tag(name: string): string {
        const gen = this.generatorsByName.get(name);
        const representative = this.mode === ChartMode.ASS ? gen?.induced_name?.[0]?.[1] : undefined;
        return displayName(representative ?? name, gen?.alg_name);
    }

    generator_tau_power(name: string): number {
        return this.tauPowers.get(name) ?? 0;
    }

    generator_label(name: string): string {
        const power = this.generator_tau_power(name);
        return (power ? `τ^${power} ` : "") + this.generator_tag(name);
    }

    generator_row(name: string): number | undefined {
        return this.generatorsByName.get(name)?.y;
    }

    // Set all differentials (complete data set)
    set_all_differentials(differentials: Differential[]) {
        this.differentials = differentials;
    }

    // Set all multiplications (complete data set)
    set_all_multiplications(multiplications: Multiplication[]) {
        this.multiplications = multiplications;
    }

    // Set all tau multiplications (complete data set)
    set_all_tau_mults(tau_mults: (InternalTauMult | ExternalTauMult)[]) {
        this.tau_mults = tau_mults;
    }

    get_dot_radius(): number {
        return this.mode === ChartMode.ASS ? 0.064 : 0.02;
    }

    clear_selection_highlights() {
        const contents = this.svgchart.shadowRoot?.getElementById("contents");
        if (!contents) return;
        contents.querySelectorAll(".selection-highlight-ring").forEach((el) => el.remove());
    }

    add_selection_highlight(
        name: string,
        color: string,
        scale: number = 1.8,
        strokeWidthScale: number = 0.14,
        fillOpacity: number = 0.35
    ) {
        const loc = this.name_to_location.get(name);
        const contents = this.svgchart.shadowRoot?.getElementById("contents");
        if (!loc || !contents) return;

        const ring = document.createElementNS(svgNS, "circle");
        ring.setAttribute("class", "selection-highlight-ring");
        ring.setAttribute("cx", loc[0].toString());
        ring.setAttribute("cy", loc[1].toString());
        ring.setAttribute("r", (this.get_dot_radius() * scale).toString());
        ring.setAttribute("fill", color);
        ring.setAttribute("fill-opacity", fillOpacity.toString());
        ring.setAttribute("stroke", color);
        ring.setAttribute("stroke-width", (this.get_dot_radius() * strokeWidthScale).toString());
        ring.setAttribute("pointer-events", "none");
        contents.prepend(ring);
    }

    display_dot(
        gen_name: string,
        display: boolean,
        { currentTorsion, finalTorsion, filtration, initialFiltration = this.generatorsByName.get(gen_name)?.af }: DotDisplay = {}
    ) {
        const el = this.dotElements.get(gen_name);
        const labelEl = this.labelElements.get(gen_name);
        const filtrationEl = this.filtrationElements.get(gen_name);

        if (!el) return;

        if (filtrationEl && filtration !== undefined) {
            const value = filtration.toString();
            if (filtrationEl.textContent !== value) filtrationEl.textContent = value;
        }

        if (display) {
            el.style.fill = finalTorsion === 0 ? "white" : torsionColor(finalTorsion);
            if (this.mode !== ChartMode.ASS) {
                el.style.stroke = torsionColor(currentTorsion);
            }
            if (filtration !== undefined && initialFiltration !== undefined && filtration !== initialFiltration) {
                // Each factor of τ lowers Adams filtration by one.
                this.tauPowers.set(gen_name, initialFiltration - filtration);
            } else {
                this.tauPowers.delete(gen_name);
            }
            el.style.visibility = "";
            if (labelEl) {
                const label = this.generator_label(gen_name);
                const content = labelEl.firstElementChild!;
                const hasMath = content.firstElementChild?.localName === "math";
                if (labelEl.getAttribute("aria-label") !== label || hasMath !== (viewSettings.tagNotation === "adams")) {
                    content.innerHTML = this.generator_label_markup(gen_name, label);
                    labelEl.setAttribute("aria-label", label);
                }
                labelEl.style.display = "";
            }
            if (filtrationEl) filtrationEl.style.display = "";
        } else {
            this.tauPowers.delete(gen_name);
            el.style.visibility = "hidden";
            if (labelEl) labelEl.style.display = "none";
            if (filtrationEl) filtrationEl.style.display = "none";
        }

    }

    display_diff(diff_from: string, diff_to: string, display: boolean, coeff?: number) {
        const key = `${diff_from}-${diff_to}`;
        const el = this.diffElements.get(key);

        if (!el) return;

        if (display) {
            el.style.stroke = torsionColor(coeff);
            el.style.visibility = "";
        } else {
            el.style.visibility = "hidden";
        }
    }

    display_mult(mult_from: string, mult_to: string, display: boolean) {
        const key = `${mult_from}-${mult_to}`;
        const el = this.multElements.get(key);

        if (!el) return;

        if (display) {
            el.style.visibility = "";
        } else {
            el.style.visibility = "hidden";
        }
    }

    display_tau_mult(tau_mult_from: string, tau_mult_to: string, display: boolean) {
        const key = `${tau_mult_from}-${tau_mult_to}`;
        const el = this.tauMultElements.get(key);

        if (!el) return;

        if (display) {
            el.style.visibility = "";
        } else {
            el.style.visibility = "hidden";
        }
    }

    // Methods callable from onclick handlers
    handleDotClickEvent(name: string) {
        if (this.dotCallback) {
            this.dotCallback(name);
        }
    }

    handleLineClickEvent(from: string, to: string) {
        if (this.lineCallback) {
            this.lineCallback(from, to);
        }
    }

    handleTauMultClickEvent(from: string, to: string) {
        if (this.tauMultCallback) {
            this.tauMultCallback(from, to);
        }
    }

    // Sanitize names for use in HTML IDs (remove special characters)
    sanitizeId(name: string): string {
        return name.replace(/[^a-zA-Z0-9_-]/g, '_');
    }

    generate_dot(x: number, y: number, name: string, style: string = "") {
        const radius = this.get_dot_radius();
        const label = /^(1|h_0)\[(\d+)\]$/.exec(this.generatorsByName.get(name)?.alg_name ?? name);
        const square = label && Number(label[2]) % 2 === (label[1] === "1" ? 0 : 1);
        const shape = square
            ? `rect x="${x - radius}" y="${y - radius}" width="${2 * radius}" height="${2 * radius}"`
            : `circle cx="${x}" cy="${y}" r="${radius}"`;
        return `<${shape} class="generator-dot" id="dot-${name}" style="${style}" onclick="window.chartInstance.handleDotClickEvent('${name}')"/>`;
    }

    generate_diff(x1: number, y1: number, x2: number, y2: number, from: string, to: string, style: string = "") {
        return `<line class="differential-line" id="diff-${from}-${to}" x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" style="${style}" onclick="window.chartInstance.handleLineClickEvent('${from}', '${to}')"/>`;
    }

    generate_mult(x1: number, y1: number, x2: number, y2: number, from: string, to: string, internal: boolean, style: string = "") {
        const className = internal ? "multiplication-line-internal" : "multiplication-line-external";
        return `<line class="${className}" id="mult-${from}-${to}" x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" style="${style}"/>`;
    }

    generate_tau_mult(x1: number, y1: number, x2: number, y2: number, from: string, to: string, style: string = "") {
        return `<line class="tau-mult-line" id="tau-mult-${from}-${to}" x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" style="${style}" onclick="window.chartInstance.handleTauMultClickEvent('${from}', '${to}')"/>`;
    }

    private generator_label_markup(name: string, label: string): string {
        if (viewSettings.tagNotation === "adams") return renderAdamsTag(label);
        const power = this.generator_tau_power(name);
        const prefix = power ? `τ<sup>${power}</sup> ` : "";
        return `<span>${prefix}${escapeHtml(this.generator_tag(name))}</span>`;
    }

    generate_label(x: number, y: number, name: string, direction: -1 | 1, visible: boolean) {
        const labelX = x + 0.05 * direction;
        const width = 288;
        const left = direction === -1;
        const label = visible ? this.generator_label(name) : "";
        const labelAttribute = visible ? `aria-label="${escapeHtml(label)}"` : 'style="display:none"';
        const markup = visible ? this.generator_label_markup(name, label) : "";
        // Lay out the math at normal CSS font sizes, then scale it into chart units.
        return `<foreignObject class="generator-label" id="label-${name}" x="${left ? -width : 0}" y="-14" width="${width}" height="28" transform="translate(${labelX},${y}) scale(0.0033333333)" ${labelAttribute} onclick="window.chartInstance.handleDotClickEvent('${name}')"><div xmlns="http://www.w3.org/1999/xhtml" class="generator-label-content${left ? " align-left" : ""}">${markup}</div></foreignObject>`;
    }

    generate_filtration_label(x: number, y: number, name: string, filtration: number, tagDirection: -1 | 1) {
        const labelX = x - 0.05 * tagDirection;
        const anchor = tagDirection === -1 ? "start" : "end";
        return `<text class="generator-filtration-label" id="filtration-${name}" x="${labelX}" y="${y}" text-anchor="${anchor}" dominant-baseline="middle" onclick="window.chartInstance.handleDotClickEvent('${name}')">${filtration}</text>`;
    }

    generate_stable_line(): string {
        // Only generate for EHP mode
        if (this.mode !== ChartMode.EHP) {
            return "";
        }

        // Use MAX_STEM for bounds
        let maxX = this.maxStem;

        // Build the path: starts at y=1, goes right 3 units, down 1 unit, repeat
        let pathData = "M 0 1"; // Start at (0, 1)
        let currentX = 0;
        let currentY = 1;

        while (currentX < maxX + 3) {
            // Go right 3 units
            currentX += 3;
            pathData += ` L ${currentX} ${currentY}`;

            // Go down 1 unit
            currentY += 1;
            pathData += ` L ${currentX} ${currentY}`;
        }

        return `<path class="stable-line" d="${pathData}" stroke="#ff6b00" stroke-width="0.02" fill="none" stroke-dasharray="0.1,0.05"/>`;
    }

    generate_invalid_cells(): string {
        // Only generate for EHP mode
        if (this.mode !== ChartMode.EHP) {
            return "";
        }
        // Use MAX_STEM + extra padding for bounds
        let maxX = this.maxStem + 3;
        let maxY = this.maxStem + 3;

        let cells = "";

        // Generate crossed-out cells for:
        // 1. First row (y = 0), except for (0, 0)
        // 2. Below diagonal (x < y)
        for (let x = 1; x <= maxX*2 + 10; x++) {
            cells += `<rect x="${x}" y="${0}" width="1" height="1" fill="url(#crossPattern)" pointer-events="none"/>\n`;
        }
        for (let y = 1; y <= maxY; y++) {
            for (let x = 0; x <= maxX; x++) {
                // Skip the (0, 0) cell
                if (x === 0 && y === 0) {
                    continue;
                }
                // Check if this cell should be crossed out
                if (x < y) {
                    cells += `<rect x="${x}" y="${y}" width="1" height="1" fill="url(#crossPattern)" pointer-events="none"/>\n`;
                }
            }
        }

        return cells;
    }

    init(visibleNames?: ReadonlySet<string>) {
        let dots = this.init_dots(visibleNames);
        let lines = this.init_diffs();
        let mults = this.init_multiplications();
        let tauMults = this.init_tau_mults();
        let stableLine = this.generate_stable_line();

        // Use fixed bounds for predictable zoom/pan behavior.
        // ASS has much smaller y-range (Adams filtration), while EHP uses full stem range.
        const isASS = this.mode === ChartMode.ASS;
        let minx = isASS ? -0.5 : 0;
        let maxx = this.maxStem + 2;
        let miny = 0;
        let maxy = isASS ? Math.ceil(this.maxStem / 2) + 2 : this.maxStem + 2;

        // Only update size if bounds have changed
        const newBounds: [number, number, number, number] = [minx, maxx, miny, maxy];
        if (!this.currentBounds ||
            this.currentBounds[0] !== newBounds[0] ||
            this.currentBounds[1] !== newBounds[1] ||
            this.currentBounds[2] !== newBounds[2] ||
            this.currentBounds[3] !== newBounds[3]) {
            this.svgchart.set_size(minx, maxx, miny, maxy);
            this.currentBounds = newBounds;
            if (this.svgchart.invalidCells) {
                this.svgchart.invalidCells.innerHTML = this.generate_invalid_cells();
            }
        }

        // Render in order: stable line (background), lines, mults, tau mults, dots (foreground)
        this.svgchart.replace_inner(stableLine + lines + mults + tauMults + dots);

        // Cache element references after SVG is populated
        this.cacheElementReferences();
    }

    private cacheElementReferences() {
        const shadowRoot = this.svgchart.shadowRoot;
        if (!shadowRoot) return;

        // Clear existing caches
        this.dotElements.clear();
        this.labelElements.clear();
        this.filtrationElements.clear();
        this.diffElements.clear();
        this.multElements.clear();
        this.tauMultElements.clear();

        // Cache dot, label, and filtration elements for each generator
        this.generators.forEach(gen => {
            const dotEl = shadowRoot.getElementById(`dot-${gen.name}`) as unknown as SVGCircleElement | SVGRectElement | null;
            const labelEl = shadowRoot.getElementById(`label-${gen.name}`) as unknown as SVGForeignObjectElement | null;
            const filtrationEl = shadowRoot.getElementById(`filtration-${gen.name}`) as unknown as SVGTextElement | null;

            if (dotEl) this.dotElements.set(gen.name, dotEl);
            if (labelEl) this.labelElements.set(gen.name, labelEl);
            if (filtrationEl) this.filtrationElements.set(gen.name, filtrationEl);
        });

        // Cache differential line elements
        this.differentials.forEach(diff => {
            if (!diff.from || !diff.to) return;

            const key = `${diff.from}-${diff.to}`;
            const diffEl = shadowRoot.getElementById(`diff-${diff.from}-${diff.to}`) as unknown as SVGLineElement | null;

            if (diffEl) this.diffElements.set(key, diffEl);
        });

        // Cache multiplication line elements
        this.multiplications.forEach(mult => {
            if (!mult.from || !mult.to) return;

            const key = `${mult.from}-${mult.to}`;
            const multEl = shadowRoot.getElementById(`mult-${mult.from}-${mult.to}`) as unknown as SVGLineElement | null;

            if (multEl) this.multElements.set(key, multEl);
        });

        // Cache tau multiplication line elements
        this.tau_mults.forEach(tauMult => {
            if (!tauMult.from || !tauMult.to) return;

            const key = `${tauMult.from}-${tauMult.to}`;
            const tauMultEl = shadowRoot.getElementById(`tau-mult-${tauMult.from}-${tauMult.to}`) as unknown as SVGLineElement | null;

            if (tauMultEl) this.tauMultElements.set(key, tauMultEl);
        });
    }

    init_dots(visibleNames?: ReadonlySet<string>): string {
        this.name_to_location.clear();

        // Group ALL generators by their (x,y) coordinates
        let temp: ToStringMap<Point, Generators[]> = new ToStringMap();

        this.generators.forEach(gen => {
            let xy: Point = [gen.stem, gen.y];
            if (temp.has(xy)) {
                temp.get(xy)?.push(gen);
            } else {
                temp.set(xy, [gen]);
            }
        });

        // Determine positioning based on mode
        const isASS = this.mode === ChartMode.ASS;
        const xOffset = isASS ? 0 : 0.5;  // ASS: on grid lines; EHP: centered
        const yOffset = isASS ? 0 : 0.5;

        // Generate SVG for each group, offsetting if multiple generators at same location
        let dots = "";
        let labels = "";
        let filtrationLabels = "";

        // For ASS, use a fixed vertical range so AF=0 is consistently at the bottom.
        const maxY = isASS ? Math.ceil(this.maxStem / 2) + 2 : 0;

        Object.values(temp.map).forEach((gens) => {
            gens.forEach((gen, index) => {
                const step = isASS ? 0.12 : 0.08;
                const offset = -((gens.length - 1) / 2) * step;

                // Apply mode-specific offsets
                let x = gen.stem + xOffset + offset + index * step;
                const yOffsetAdjust = offset + index * step * 0.85;

                let y: number;
                if (isASS) {
                    // ASS mode: flip Y so y=0 appears at bottom
                    // Map gen.y=0 -> maxY, gen.y=maxY -> 0
                    y = (maxY - gen.y) + yOffset - yOffsetAdjust;
                } else {
                    // EHP mode: normal Y axis (y=0 at top)
                    y = gen.y + yOffset - yOffsetAdjust;
                }

                this.name_to_location.set(gen.name, [x, y]);

                dots += this.generate_dot(x, y, gen.name, "") + "\n";
                const tagDirection = tagLabelDirection(gen.y);
                if (this.showNames) {
                    labels += this.generate_label(x, y, gen.name, tagDirection, visibleNames?.has(gen.name) ?? true) + "\n";
                }
                if (this.showFiltration) {
                    filtrationLabels += this.generate_filtration_label(x, y, gen.name, gen.af, tagDirection) + "\n";
                }
            });
        });

        // Labels should be rendered after dots so they appear on top
        return dots + labels + filtrationLabels;
    }

    private computeMaxStem(): number {
        let maxStem = 0;
        this.generators.forEach((gen) => {
            maxStem = Math.max(maxStem, gen.stem, gen.y);
        });
        return maxStem || 32;
    }

    init_diffs(): string {
        // Draw ALL differentials
        return this.differentials.map(diff => {
            if (!diff.from || !diff.to) return "";

            let from_loc = this.name_to_location.get(diff.from);
            let to_loc = this.name_to_location.get(diff.to);

            if (!from_loc || !to_loc) {
                console.warn(`Differential ${diff.from} -> ${diff.to} missing location`);
                return "";
            }

            const style = (diff.kind !== "Real" && diff.kind !== "Algebraic") && this.mode !== ChartMode.ASS ? "stroke-dasharray: 0.025,0.02;" : "";
            return this.generate_diff(from_loc[0], from_loc[1], to_loc[0], to_loc[1], diff.from, diff.to, style);
        }).join("\n");
    }

    init_multiplications(): string {
        // Draw ALL multiplications
        return this.multiplications.map(mult => {
            if (!mult.from || !mult.to) return "";

            let from_loc = this.name_to_location.get(mult.from);
            let to_loc = this.name_to_location.get(mult.to);

            if (!from_loc || !to_loc) {
                console.warn(`Multiplication ${mult.from} -> ${mult.to} missing location`);
                return "";
            }

            const internal = "internal" in mult ? !!(mult as { internal?: boolean }).internal : false;
            return this.generate_mult(from_loc[0], from_loc[1], to_loc[0], to_loc[1], mult.from, mult.to, internal, "");
        }).join("\n");
    }

    init_tau_mults(): string {
        // Draw ALL tau multiplications
        return this.tau_mults.map(tauMult => {
            if (!tauMult.from || !tauMult.to) return "";

            let from_loc = this.name_to_location.get(tauMult.from);
            let to_loc = this.name_to_location.get(tauMult.to);

            if (!from_loc || !to_loc) {
                console.warn(`TauMult ${tauMult.from} -> ${tauMult.to} missing location`);
                return "";
            }

            const style = tauMult.kind !== "Real" && this.mode !== ChartMode.ASS ? "stroke-dasharray: 0.025,0.02;" : "";
            return this.generate_tau_mult(from_loc[0], from_loc[1], to_loc[0], to_loc[1], tauMult.from, tauMult.to, style);
        }).join("\n");
    }

}
