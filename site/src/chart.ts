import { ToStringMap } from "./stringmap";
import { SvgChart } from "./svgchart";
import { Differential, Generators, Multiplication } from "./types";

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

    // Cached element references
    private dotElements: Map<string, SVGCircleElement> = new Map();
    private labelElements: Map<string, SVGTextElement> = new Map();
    private filtrationElements: Map<string, SVGTextElement> = new Map();
    private diffElements: Map<string, SVGLineElement> = new Map();
    private multElements: Map<string, SVGLineElement> = new Map();

    // Track current bounds to avoid unnecessary zoom resets
    private currentBounds: [number, number, number, number] | null = null;

    public dotCallback: Function;
    public lineCallback: Function;

    constructor() {
        this.svgchart = new SvgChart();
        document.getElementById("svgchart").append(this.svgchart);

        // Make click handlers available globally so SVG onclick can access them
        (window as any).chartInstance = this;
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
        return `<circle class="generator-dot" id="dot-${name}" cx="${x}" cy="${y}" r="0.022" style="${style}" onclick="window.chartInstance.handleDotClickEvent('${name}')"/>`;
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

    generate_label(x: number, y: number, name: string) {
        // Position the label to the left of the dot using SVG text
        const labelX = x - 0.05; // Position to the left
        const labelY = y; // Vertically centered with dot

        return `<text class="generator-label" id="label-${name}" x="${labelX}" y="${labelY}" text-anchor="end" dominant-baseline="middle">${name}</text>`;
    }

    generate_filtration_label(x: number, y: number, name: string, filtration: number, filtrationName: string = "adams filtration") {
        // Position the label to the right of the dot using SVG text
        const labelX = x + 0.05; // Position to the right
        const labelY = y; // Vertically centered with dot

        return `<text class="generator-filtration-label" id="filtration-${name}" x="${labelX}" y="${labelY}" text-anchor="start" dominant-baseline="middle">${filtration}</text>`;
    }

    init() {
        let dots = this.init_dots();
        let lines = this.init_diffs();
        let mults = this.init_multiplications();

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

        this.svgchart.replace_inner(lines + mults + dots);

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

        // Generate SVG for each group, offsetting if multiple generators at same location
        // Dots are centered in grid squares (at integer + 0.5)
        let dots = "";
        let labels = "";
        let filtrationLabels = "";

        Object.values(temp.map).forEach((gens) => {
            gens.forEach((gen, index) => {
                const step = 0.08;
                const offset = -((gens.length - 1) / 2) * step;
                const x = gen.x + 0.5 + offset + index * step;
                const yOffset = offset + index * step ;
                const y = gen.y + 0.5 - yOffset;
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

}

