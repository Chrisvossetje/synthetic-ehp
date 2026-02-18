import { ToStringMap } from "./stringmap";
import { SvgChart } from "./svgchart";
import { Differential, Generators, Multiplication, TauMult } from "./types";
import { ChartMode } from "./chartMode";

type Point = [number, number];

const TorsionColor = ["black", "#0080ff", "red", "mediumseagreen", "cyan", "purple"];

export class Chart {
    public svgchart: SvgChart;

    // Use generator names as unique identifiers
    public name_to_location: Map<string, Point> = new Map();

    // All generators and differentials (complete data set)
    public generators: Generators[] = [];
    public differentials: Differential[] = [];
    public multiplications: Multiplication[] = [];
    public tau_mults: TauMult[] = [];

    // Cached element references
    private dotElements: Map<string, SVGCircleElement> = new Map();
    private labelElements: Map<string, SVGTextElement> = new Map();
    private filtrationElements: Map<string, SVGTextElement> = new Map();
    private diffElements: Map<string, SVGLineElement> = new Map();
    private multElements: Map<string, SVGLineElement> = new Map();
    private tauMultElements: Map<string, SVGLineElement> = new Map();

    // Track current bounds to avoid unnecessary zoom resets
    private currentBounds: [number, number, number, number] | null = null;

    // Chart display mode (EHP vs ASS)
    public readonly mode: ChartMode;

    public dotCallback: Function;
    public lineCallback: Function;

    constructor(containerId: string, mode: ChartMode) {
        this.mode = mode;
        // Pass mode to SvgChart for appropriate styling
        this.svgchart = new SvgChart(mode);
        const container = document.getElementById(containerId);
        if (container) {
            container.append(this.svgchart);
        }
    }

    clear() {
        this.svgchart.replace_inner("");
    }

    /**
     * Show this chart
     */
    show() {
        const container = this.svgchart.parentElement;
        if (container) {
            container.style.display = 'block';
            // Trigger resize and reapply zoom after showing to ensure proper dimensions
            setTimeout(() => {
                if (this.currentBounds) {
                    // Reapply size settings now that the element is visible
                    this.svgchart.set_size(
                        this.currentBounds[0],
                        this.currentBounds[1],
                        this.currentBounds[2],
                        this.currentBounds[3]
                    );
                } else {
                    this.svgchart.onResize();
                }
            }, 0);
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
    set_all_tau_mults(tau_mults: TauMult[]) {
        this.tau_mults = tau_mults;
    }

    display_dot(gen_name: string, display: boolean, permanent: boolean, torsion: number | null, filtration?: number) {
        const el = this.dotElements.get(gen_name);
        const labelEl = this.labelElements.get(gen_name);
        const filtrationEl = this.filtrationElements.get(gen_name);

        if (!el) return;

        if (filtrationEl && filtration !== undefined) {
            filtrationEl.textContent = filtration.toString();
        }

        if (display) {
            el.style.visibility = null;
            if (labelEl) labelEl.style.visibility = null;
            if (filtrationEl) filtrationEl.style.visibility = null;
        } else {
            el.style.visibility = "hidden";
            if (labelEl) labelEl.style.visibility = "hidden";
            if (filtrationEl) filtrationEl.style.visibility = "hidden";
        }

        if (torsion == null) {torsion = 0};
        if (permanent) {
            el.style.fill = TorsionColor[torsion];
            el.style.stroke = "";
        } else {
            el.style.fill = "white";
            el.style.stroke = TorsionColor[torsion];
        }
    }

    display_diff(diff_from: string, diff_to: string, display: boolean, torsion: number | null = null) {
        const key = `${diff_from}-${diff_to}`;
        const el = this.diffElements.get(key);

        if (!el) return;

        if (display) {
            el.style.visibility = null;
        } else {
            el.style.visibility = "hidden";
        }

        if (torsion == null) {torsion = 0};
        el.style.stroke = TorsionColor[torsion];
    }

    display_mult(mult_from: string, mult_to: string, display: boolean) {
        const key = `${mult_from}-${mult_to}`;
        const el = this.multElements.get(key);

        if (!el) return;

        if (display) {
            el.style.visibility = null;
        } else {
            el.style.visibility = "hidden";
        }
    }

    display_tau_mult(tau_mult_from: string, tau_mult_to: string, display: boolean) {
        const key = `${tau_mult_from}-${tau_mult_to}`;
        const el = this.tauMultElements.get(key);

        if (!el) return;

        if (display) {
            el.style.visibility = null;
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

    // Sanitize names for use in HTML IDs (remove special characters)
    sanitizeId(name: string): string {
        return name.replace(/[^a-zA-Z0-9_-]/g, '_');
    }

    // Escape string for use in HTML attributes
    escapeHtml(text: string): string {
        return text.replace(/'/g, '&apos;').replace(/"/g, '&quot;');
    }

    generate_dot(x: number, y: number, name: string, style: string = "") {
        const radius = this.mode === ChartMode.ASS ? "0.08" : "0.022";
        return `<circle class="generator-dot" id="dot-${name}" cx="${x}" cy="${y}" r="${radius}" style="${style}" onclick="window.chartInstance.handleDotClickEvent('${name}')"/>`;
    }

    generate_diff(x1: number, y1: number, x2: number, y2: number, from: string, to: string, style: string = "") {
        // const escapedFrom = this.escapeHtml(from);
        // const escapedTo = this.escapeHtml(to);
        return `<line class="differential-line" id="diff-${from}-${to}" x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" style="${style}" onclick="window.chartInstance.handleLineClickEvent('${from}', '${to}')"/>`;
    }

    generate_mult(x1: number, y1: number, x2: number, y2: number, from: string, to: string, internal: boolean, style: string = "") {
        const className = internal ? "multiplication-line-internal" : "multiplication-line-external";
        return `<line class="${className}" id="mult-${from}-${to}" x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" style="${style}"/>`;
    }

    generate_tau_mult(x1: number, y1: number, x2: number, y2: number, from: string, to: string, style: string = "") {
        return `<line class="tau-mult-line" id="tau-mult-${from}-${to}" x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" style="${style}"/>`;
    }

    generate_label(x: number, y: number, name: string) {
        // Position the label to the left of the dot using SVG text
        const labelX = x - 0.05; // Position to the left
        const labelY = y; // Vertically centered with dot

        return `<text class="generator-label" id="label-${name}" x="${labelX}" y="${labelY}" text-anchor="end" dominant-baseline="middle">${name}</text>`;
    }

    generate_filtration_label(x: number, y: number, name: string, filtration: number) {
        // Position the label to the right of the dot using SVG text
        const labelX = x + 0.05; // Position to the right
        const labelY = y; // Vertically centered with dot

        return `<text class="generator-filtration-label" id="filtration-${name}" x="${labelX}" y="${labelY}" text-anchor="start" dominant-baseline="middle">${filtration}</text>`;
    }

    generate_stable_line(): string {
        // Only generate for EHP mode
        if (this.mode !== ChartMode.EHP) {
            return "";
        }

        // Calculate bounds from name_to_location
        let locations = Array.from(this.name_to_location.values());
        if (locations.length === 0) {
            return "";
        }

        let x_col = locations.map(xy => xy[0]);
        let maxX = Math.max(...x_col);

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

        // Calculate bounds from name_to_location
        let locations = Array.from(this.name_to_location.values());
        if (locations.length === 0) {
            return "";
        }

        let x_col = locations.map(xy => xy[0]);
        let y_col = locations.map(xy => xy[1]);
        let maxX = Math.max(...x_col);
        let maxY = Math.max(...y_col);

        let cells = "";

        // Generate crossed-out cells for:
        // 1. First row (y = 0), except for (0, 0)
        // 2. Below diagonal (x < y)
        for (let y = 0; y <= maxY; y++) {
            for (let x = 0; x <= maxX; x++) {
                // Skip the (0, 0) cell
                if (x === 0 && y === 0) {
                    continue;
                }
                // Check if this cell should be crossed out
                if (y === 0 || x < y) {
                    cells += `<rect x="${x}" y="${y}" width="1" height="1" fill="url(#crossPattern)" pointer-events="none"/>\n`;
                }
            }
        }

        return cells;
    }

    init() {
        let dots = this.init_dots();
        let lines = this.init_diffs();
        let mults = this.init_multiplications();
        let tauMults = this.init_tau_mults();
        let stableLine = this.generate_stable_line();
        let invalidCells = this.generate_invalid_cells();

        // Calculate bounds from name_to_location
        let locations = Array.from(this.name_to_location.values());
        if (locations.length === 0) {
            return;
        }

        let x_col = locations.map(xy => xy[0]);
        let y_col = locations.map(xy => xy[1]);
        let minx = Math.min(...x_col);
        let maxx = Math.max(...x_col);
        let miny = Math.min(...y_col);
        let maxy = Math.max(...y_col);

        // Only update size if bounds have changed
        const newBounds: [number, number, number, number] = [minx, maxx, miny, maxy];
        if (!this.currentBounds ||
            this.currentBounds[0] !== newBounds[0] ||
            this.currentBounds[1] !== newBounds[1] ||
            this.currentBounds[2] !== newBounds[2] ||
            this.currentBounds[3] !== newBounds[3]) {
            this.svgchart.set_size(minx, maxx, miny, maxy);
            this.currentBounds = newBounds;
        }

        // Populate invalid cells group (for EHP mode)
        if (this.mode === ChartMode.EHP && this.svgchart.invalidCells) {
            this.svgchart.invalidCells.innerHTML = invalidCells;
        }

        // Render in order: stable line (background), lines, mults, tau mults, dots (foreground)
        this.svgchart.replace_inner(stableLine + lines + mults + tauMults + dots);

        // Cache element references after SVG is populated
        this.cacheElementReferences();
    }

    private cacheElementReferences() {
        // Clear existing caches
        this.dotElements.clear();
        this.labelElements.clear();
        this.filtrationElements.clear();
        this.diffElements.clear();
        this.multElements.clear();
        this.tauMultElements.clear();

        // Cache dot, label, and filtration elements for each generator
        this.generators.forEach(gen => {
            const dotEl = this.svgchart.shadowRoot.getElementById(`dot-${gen.name}`) as unknown as SVGCircleElement;
            const labelEl = this.svgchart.shadowRoot.getElementById(`label-${gen.name}`) as unknown as SVGTextElement;
            const filtrationEl = this.svgchart.shadowRoot.getElementById(`filtration-${gen.name}`) as unknown as SVGTextElement;

            if (dotEl) this.dotElements.set(gen.name, dotEl);
            if (labelEl) this.labelElements.set(gen.name, labelEl);
            if (filtrationEl) this.filtrationElements.set(gen.name, filtrationEl);
        });

        // Cache differential line elements
        this.differentials.forEach(diff => {
            if (!diff.from || !diff.to) return;

            const key = `${diff.from}-${diff.to}`;
            const diffEl = this.svgchart.shadowRoot.getElementById(`diff-${diff.from}-${diff.to}`) as unknown as SVGLineElement;

            if (diffEl) this.diffElements.set(key, diffEl);
        });

        // Cache multiplication line elements
        this.multiplications.forEach(mult => {
            if (!mult.from || !mult.to) return;

            const key = `${mult.from}-${mult.to}`;
            const multEl = this.svgchart.shadowRoot.getElementById(`mult-${mult.from}-${mult.to}`) as unknown as SVGLineElement;

            if (multEl) this.multElements.set(key, multEl);
        });

        // Cache tau multiplication line elements
        this.tau_mults.forEach(tauMult => {
            if (!tauMult.from || !tauMult.to) return;

            const key = `${tauMult.from}-${tauMult.to}`;
            const tauMultEl = this.svgchart.shadowRoot.getElementById(`tau-mult-${tauMult.from}-${tauMult.to}`) as unknown as SVGLineElement;

            if (tauMultEl) this.tauMultElements.set(key, tauMultEl);
        });
    }

    init_dots(): string {
        this.name_to_location.clear();

        // Group ALL generators by their (x,y) coordinates
        let temp: ToStringMap<Point, Generators[]> = new ToStringMap();

        this.generators.forEach(gen => {
            let xy: Point = [gen.x, gen.y];
            if (temp.has(xy)) {
                temp.get(xy).push(gen);
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

        // Find max Y for flipping in ASS mode
        const maxY = isASS ? Math.max(...this.generators.map(g => g.y)) : 0;

        Object.values(temp.map).forEach((gens) => {
            gens.forEach((gen, index) => {
                const step = 0.08;
                const offset = -((gens.length - 1) / 2) * step;

                // Apply mode-specific offsets
                let x = gen.x + xOffset + offset + index * step;
                const yOffsetAdjust = offset + index * step;

                let y: number;
                if (isASS) {
                    // ASS mode: flip Y so y=0 appears at bottom
                    // Map gen.y=0 -> maxY, gen.y=maxY -> 0
                    y = (maxY - gen.y) + yOffset + yOffsetAdjust;
                } else {
                    // EHP mode: normal Y axis (y=0 at top)
                    y = gen.y + yOffset - yOffsetAdjust;
                }

                this.name_to_location.set(gen.name, [x, y]);

                dots += this.generate_dot(x, y, gen.name, "") + "\n";
                labels += this.generate_label(x, y, gen.name) + "\n";
                filtrationLabels += this.generate_filtration_label(x, y, gen.name, gen.adams_filtration) + "\n";
            });
        });

        // Labels should be rendered after dots so they appear on top
        return dots + labels + filtrationLabels;
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

            return this.generate_diff(from_loc[0], from_loc[1], to_loc[0], to_loc[1], diff.from, diff.to, "");
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

            return this.generate_mult(from_loc[0], from_loc[1], to_loc[0], to_loc[1], mult.from, mult.to, mult.internal, "");
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

            return this.generate_tau_mult(from_loc[0], from_loc[1], to_loc[0], to_loc[1], tauMult.from, tauMult.to, "");
        }).join("\n");
    }

}

