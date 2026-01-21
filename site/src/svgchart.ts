import * as d3 from "d3";

export const svgNS = 'http://www.w3.org/2000/svg';

// Code originated from: https://github.com/SpectralSequences/sseq/
// exact file at https://github.com/SpectralSequences/sseq/tree/master/svg-chart/chart.js

const CHART_CSS = `
:host { display: block; }
.struct { stroke: black; fill: none; stroke-width: 0.03; }

/* Generator dot styles */    
.generator-dot {
    r: 0.022;
    cursor: pointer;
    transition: all 0.15s ease;
    stroke-width: 0.01;
}

.generator-dot:hover {
    fill: #555;
    r: 0.03;
    filter: drop-shadow(0 0 0.08 rgba(0,0,0,0.4));
}

.generator-dot:active {
    fill: #777;
}

/* Differential line styles */
.differential-line {
    stroke: #555;
    stroke-width: 0.008;
    cursor: pointer;
    transition: all 0.15s ease;
}

.differential-line:hover {
    stroke: #666 !important;
    stroke-width: 0.016 !important;
    filter: drop-shadow(0 0 0.08 rgba(0,0,0,0.4));
}

.differential-line:active {
    stroke: #777 !important;
}

/* Generator label styles */
.generator-label {
    pointer-events: none;
    font-size: 0.04px;
    fill: #000;
    font-family: monospace;
}

/* Filtration label styles */
.generator-filtration-label {
    pointer-events: none;
    font-size: 0.035px;
    fill: #666;
    font-family: monospace;
}

/* Multiplication line styles - internal (softer color) */
.multiplication-line-internal {
    stroke: #8888;
    stroke-width: 0.008;
    pointer-events: none;
    opacity: 0.6;
}

/* Multiplication line styles - external (more prominent) */
.multiplication-line-external {
    stroke: #444;
    stroke-width: 0.01;
    pointer-events: none;
    opacity: 0.8;
}
`;


/**
 * A Web Component for a chart.
 *
 * @property {SVGGElement} contents The group containing the actual chart, as
 * opposed to e.g. the axes. Users should add their chart into this group.
 */
export class SvgChart extends HTMLElement {
    /**
     * The amount of space reserved for the axes and axes labels
     */
    static MARGIN = 20;
    /**
     * The amount of space between the axes and the axes labels
     */
    static LABEL_MARGIN = 3;
    /**
     * The amount of extra space from the edge of the chart. For example, if
     * minX = 0, then we allow users to pan to up to x = -GRID_MARGIN. This
     * allows us to fully display the class, instead of cutting it in half
     * along the grid lines.
     */
    static GRID_MARGIN = 0.5;

    public minX: number;
    public minY: number; 
    public maxX: number;
    public maxY: number;

    public height: number;
    public width: number;

    public animationId: number;

    public svg: SVGSVGElement;

    public zoom: any;
    public select: any;
    public zoomTimeout: number;

    public inner: HTMLElement;
    public axisLabels: HTMLElement;
    public grid: HTMLElement;
    public contents: HTMLElement;
    public xBlock: HTMLElement;
    public yBlock: HTMLElement;

    public node_style: HTMLElement;
    public line_style: HTMLElement;

    static get observedAttributes() {
        return ['minx', 'miny', 'maxx', 'maxy'];
    }

    attributeChangedCallback(name, _oldValue, newValue) {
        if (name == 'minx') {
            this.minX = Math.max(0, parseInt(newValue) - SvgChart.GRID_MARGIN);
        } else if (name == 'miny') {
            this.minY = Math.max(0, parseInt(newValue) - SvgChart.GRID_MARGIN);
        } else if (name == 'maxx') {
            this.maxX = parseInt(newValue) + SvgChart.GRID_MARGIN;
        } else if (name == 'maxy') {
            this.maxY = parseInt(newValue) + SvgChart.GRID_MARGIN;
        }
        this.onResize();
    }

    connectedCallback() {
        this.onResize();
    }

    public set_line_style(style: string) {
        this.line_style.textContent = style;
    }

    public set_node_style(style: string) {
        this.node_style.textContent = style;
    }

    public set_size(minx, maxx, miny, maxy) {
        // Enforce x >= 0 and y >= 0
        this.minX = Math.max(0, minx - SvgChart.GRID_MARGIN);
        this.maxX = maxx + SvgChart.GRID_MARGIN;
        this.minY = Math.max(0, miny - SvgChart.GRID_MARGIN);
        this.maxY = maxy + SvgChart.GRID_MARGIN;
        
        // Zoom out to show the entire chart
        const scaleX = this.width / (this.maxX - this.minX);
        const scaleY = this.height / (this.maxY - this.minY);
        const scale = Math.min(scaleX, scaleY); // Choose the smaller scale to ensure full visibility

        // Reset zoom to fit the new size (y is now positive, going down)
        this.zoom.transform(this.select, d3.zoomIdentity.scale(scale).translate(-this.minX, -this.minY));
        this.onResize();
    }

    constructor() {
        super();

        this.attachShadow({ mode: 'open' });

        this.animationId = null;

        // Default to x >= 0 and y >= 0
        this.minX = 0;
        this.minY = 0;
        this.maxX = 20 + SvgChart.GRID_MARGIN;
        this.maxY = 20 + SvgChart.GRID_MARGIN;

        this.svg = document.createElementNS(svgNS, 'svg');
        this.svg.setAttribute('xmlns', svgNS);


        const node = document.createElement('style');
        node.textContent = CHART_CSS;

        this.line_style = document.createElement('style');
        this.node_style = document.createElement('style');

        this.shadowRoot.appendChild(node);
        this.shadowRoot.appendChild(this.line_style);
        this.shadowRoot.appendChild(this.node_style);

        this.shadowRoot.appendChild(this.svg);

        this.svg.innerHTML = `
<defs>
  <pattern id="gridPattern" width="1" height="1" patternUnits="userSpaceOnUse">
    <rect width="1" height="1" fill="white" stroke="black" stroke-width="0.01" />
  </pattern>
</defs>
<rect id="xBlock" x="0" height="${SvgChart.MARGIN}" y="${-SvgChart.MARGIN}" fill="white"/>
<rect id="yBlock" x="${-SvgChart.MARGIN}" width="${SvgChart.MARGIN}" y="0" fill="white"/>
<g id="inner">
  <rect id="grid" fill="url(#gridPattern)" />
  <g id="contents"></g>
</g>
<g id="axisLabels"></g>
`;

        for (const item of [
            'inner',
            'axisLabels',
            'grid',
            'contents',
            'xBlock',
            'yBlock',
        ]) {
            this[item] = this.shadowRoot.getElementById(`${item}`);
        }

        this.select = d3.select(this.svg);
        this.zoom = d3.zoom().on('zoom', this._zoomFunc.bind(this));

        if (navigator.userAgent.includes('Firefox')) {
            this.zoom.on('zoom', e => {
                this._zoomFunc(e);
                clearTimeout(this.zoomTimeout);
                this.zoomTimeout = setTimeout(() => this._zoomFunc(e), 500);
            });
        }
        window.addEventListener('resize', this.onResize.bind(this));

        this.onResize();
        this.select.call(this.zoom).on('dblclick.zoom', null);
    }


    replace_inner(inner: string) {
        this["contents"].innerHTML = inner;
    }

    /**
     * Add a stylesheet to the SVG.
     *
     * @return {HTMLStyleElement} The node containing the stylesheet
     */
    addStyle(style) {
        const node = document.createElementNS(svgNS, 'style');
        node.textContent = style;
        this["contents"].appendChild(node);
        return node;
    }

    /**
     * Pan the chart so that the given coordinates (x, y) are at the center of the chart.
     * @param {number} x
     * @param {number} y
     */
    goto(x, y) {
        this.zoom.translateTo(this.select, x, y);
    }

    _zoomFunc(e) {
        window.cancelAnimationFrame(this.animationId);
        this.animationId = requestAnimationFrame(() => this._zoomFuncInner(e));
    }

    _zoomFuncInner({ transform }) {
        this.inner.setAttribute('transform', transform);
        while (this.axisLabels.firstChild) {
            this.axisLabels.removeChild(this.axisLabels.firstChild);
        }

        // Always show all row/column numbers (sep = 1)
        const sep = 1;

        // X-axis labels: only show x >= 0, positioned at center of boxes (x + 0.5)
        const minX = Math.max(0, Math.ceil(transform.invertX(0)));
        const maxX = Math.floor(transform.invertX(this.width));

        for (let x = minX; x <= maxX; x += sep) {
            const textNode = document.createElementNS(svgNS, 'text');
            textNode.textContent = x.toString();
            textNode.setAttribute('x', transform.applyX(x + 0.5).toString());
            textNode.setAttribute('y', (-SvgChart.LABEL_MARGIN).toString());
            textNode.setAttribute('text-anchor', 'middle');
            textNode.setAttribute('dominant-baseline', 'text-after-edge');
            this.axisLabels.appendChild(textNode);
        }

        // Y-axis labels: only show y >= 0, positioned at center of boxes (y + 0.5)
        const minY = Math.max(0, Math.ceil(transform.invertY(0)));
        const maxY = Math.floor(transform.invertY(this.height));

        for (let y = minY; y <= maxY; y += sep) {
            const textNode = document.createElementNS(svgNS, 'text');
            textNode.textContent = y.toString();
            textNode.setAttribute('y', transform.applyY(y + 0.5).toString());
            textNode.setAttribute('x', (-SvgChart.LABEL_MARGIN).toString());
            textNode.setAttribute('text-anchor', 'end');
            textNode.setAttribute('dominant-baseline', 'middle');
            this.axisLabels.appendChild(textNode);
        }
    }

    /**
     * This function should be called whenever the component's size changes.
     * This is automatically triggered when window#resize is fired, but
     * otherwise the user should call this function when the dimensions change.
     */
    onResize() {
        if (!this.isConnected) {
            return;
        }

        const size = this.getBoundingClientRect();

        this.height = size.height - SvgChart.MARGIN;
        this.width = size.width - SvgChart.MARGIN;

        const min_k = Math.min(
            this.width / (this.maxX - this.minX),
            this.height / (this.maxY - this.minY),
        );

        this.svg.setAttribute(
            'viewBox',
            `${-SvgChart.MARGIN} ${-SvgChart.MARGIN} ${size.width} ${size.height}`,
        );

        this.zoom.constrain(transform => {
            let x = transform.x;
            let y = transform.y;
            let k = transform.k;

            k = Math.max(k, min_k);

            x = Math.max(x, -this.maxX * k + this.width);
            x = Math.min(x, -this.minX * k);

            y = Math.max(y, -this.maxY * k + this.height);
            y = Math.min(y, -this.minY * k);

            return d3.zoomIdentity.translate(x, y).scale(k);
        });

        // xBlock covers the top margin area (where x-axis labels go)
        this.xBlock.setAttribute('width', size.width.toString());

        // yBlock covers the left margin area (where y-axis labels go)
        this.yBlock.setAttribute('height', size.height.toString());

        // Grid should fill the entire visible viewport area
        // Calculate visible area in chart coordinates based on current scale
        const currentScale = this.zoom.scale ? this.zoom.scale() : 1;
        const visibleWidth = this.width / currentScale;
        const visibleHeight = this.height / currentScale;

        const grid_min_x = 0;
        const grid_max_x = Math.max(Math.ceil(this.maxX), Math.ceil(this.minX + visibleWidth));
        const grid_min_y = 0;
        const grid_max_y = Math.max(Math.ceil(this.maxY), Math.ceil(this.minY + visibleHeight));

        this.grid.setAttribute('x', grid_min_x.toString());
        this.grid.setAttribute('y', grid_min_y.toString());
        this.grid.setAttribute('width', (grid_max_x - grid_min_x).toString());
        this.grid.setAttribute('height', (grid_max_y - grid_min_y).toString());

        this.zoom.scaleBy(this.select, 1);
    }
}
customElements.define('svg-chart', SvgChart);
