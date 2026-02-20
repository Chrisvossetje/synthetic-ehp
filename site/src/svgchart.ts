import { select } from "d3-selection";
import { zoom, zoomIdentity, zoomTransform } from "d3-zoom";
import { ChartMode } from "./chartMode";
import { ASS_CHART_CSS, EHP_CHART_CSS } from "./svg_chart_styles";

export const svgNS = 'http://www.w3.org/2000/svg';

// Code originated from: https://github.com/SpectralSequences/sseq/
// exact file at https://github.com/SpectralSequences/sseq/tree/master/svg-chart/chart.js


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

    public mode: ChartMode;

    public minX: number;
    public minY: number;
    public maxX: number;
    public maxY: number;
    public chartMinX: number;
    public chartMinY: number;
    public chartMaxX: number;
    public chartMaxY: number;

    public height: number;
    public width: number;

    public animationId: number;

    public svg: SVGSVGElement;

    public zoom: any;
    public select: any;
    public zoomTimeout: number;
    private xAxisLabelNodes: Map<number, SVGTextElement>;
    private yAxisLabelNodes: Map<number, SVGTextElement>;

    public inner: HTMLElement;
    public axis: HTMLElement;
    public axisLabels: HTMLElement;
    public grid: HTMLElement;
    public invalidCells: HTMLElement;
    public contents: HTMLElement;
    public xBlock: HTMLElement;
    public yBlock: HTMLElement;
    public topBorder: HTMLElement;
    public leftBorder: HTMLElement;
    public bottomBorder: HTMLElement;

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
        this.chartMinX = minx;
        this.chartMaxX = maxx;
        this.chartMinY = miny;
        this.chartMaxY = maxy;

        // ASS can use a small negative minX; EHP stays clamped at x >= 0.
        if (this.mode === ChartMode.ASS) {
            this.minX = minx;
        } else {
            this.minX = Math.max(0, minx - SvgChart.GRID_MARGIN);
        }
        this.maxX = maxx + SvgChart.GRID_MARGIN;
        this.minY = Math.max(0, miny - SvgChart.GRID_MARGIN);
        this.maxY = this.mode === ChartMode.ASS ? maxy + 0.5 : maxy + SvgChart.GRID_MARGIN;

        // Resize/re-anchor is handled centrally in onResize.
        this.onResize();
    }

    constructor(chartMode: ChartMode = ChartMode.EHP) {
        super();

        this.mode = chartMode;
        this.attachShadow({ mode: 'open' });

        this.animationId = null;
        this.xAxisLabelNodes = new Map();
        this.yAxisLabelNodes = new Map();

        // Default to x >= 0 and y >= 0
        this.chartMinX = 0;
        this.chartMinY = 0;
        this.chartMaxX = 20;
        this.chartMaxY = 20;
        this.minX = 0;
        this.minY = 0;
        this.maxX = 20 + SvgChart.GRID_MARGIN;
        this.maxY = 20 + SvgChart.GRID_MARGIN;

        this.svg = document.createElementNS(svgNS, 'svg');
        this.svg.setAttribute('xmlns', svgNS);

        // Use different CSS based on mode
        const node = document.createElement('style');
        node.textContent = this.mode === ChartMode.ASS ? ASS_CHART_CSS : EHP_CHART_CSS;

        this.line_style = document.createElement('style');
        this.node_style = document.createElement('style');

        this.shadowRoot.appendChild(node);
        this.shadowRoot.appendChild(this.line_style);
        this.shadowRoot.appendChild(this.node_style);

        this.shadowRoot.appendChild(this.svg);

        if (chartMode == ChartMode.EHP) {
        this.svg.innerHTML = `
<defs>
<pattern id="gridPattern" width="1" height="1" patternUnits="userSpaceOnUse">
    <rect width="1" height="1" fill="white" stroke="black" stroke-width="0.01" />
</pattern>
<pattern id="crossPattern" width="1" height="1" patternUnits="userSpaceOnUse">
    <rect width="1" height="1" fill="white" stroke="black" stroke-width="0.01" />
    <line x1="0" y1="0" x2="1" y2="1" stroke="#ccc" stroke-width="0.02" />
    <line x1="1" y1="0" x2="0" y2="1" stroke="#ccc" stroke-width="0.02" />
</pattern>
</defs>
<g id="inner">
<rect id="grid" fill="url(#gridPattern)" />
<g id="invalidCells"></g>
<g id="contents"></g>
</g>
<rect id="xBlock" x="${-SvgChart.MARGIN}" height="${SvgChart.MARGIN + 0.1}" y="${-SvgChart.MARGIN}" fill="white"/>
<rect id="yBlock" x="${-SvgChart.MARGIN}" width="${SvgChart.MARGIN + 0.1}" y="${-SvgChart.MARGIN}" fill="white"/>
<line id="topBorder" x1="0" y1="0" x2="1000" y2="0" stroke="black" stroke-width="2" vector-effect="non-scaling-stroke"/>
<line id="leftBorder" x1="0" y1="0" x2="0" y2="1000" stroke="black" stroke-width="2" vector-effect="non-scaling-stroke"/>
<g id="axisLabels"></g>
`;
        } else {
this.svg.innerHTML = 
`
<defs>
  <pattern id="smallGrid" width="1" height="1" patternUnits="userSpaceOnUse">
    <path d="M 1 1 L 0 1 0 0" fill="none" stroke="black" stroke-width="0.01" />
  </pattern>
  <pattern id="bigGrid" width="4" height="4" patternUnits="userSpaceOnUse">
    <rect width="4" height="4" fill="url(#smallGrid)" />
    <path d="M 4 4 L 0 4 0 0" fill="none" stroke="black" stroke-width="0.03" />
  </pattern>
</defs>
<g id="inner">
  <rect id="grid" fill="url(#bigGrid)" />
  <g id="contents"></g>
</g>
<rect id="xBlock" x="${-SvgChart.MARGIN}" height="${
            SvgChart.MARGIN
        }" y="0" fill="white"/>
<rect id="yBlock" x="${-SvgChart.MARGIN}" width="${SvgChart.MARGIN}" fill="white"/>
<line id="leftBorder" x1="0" y1="0" x2="0" y2="1000" stroke="black" stroke-width="2" vector-effect="non-scaling-stroke"/>
<line id="bottomBorder" x1="0" y1="1000" x2="1000" y2="1000" stroke="black" stroke-width="2" vector-effect="non-scaling-stroke"/>
<g id="axisLabels"></g>
`
        }

        for (const item of [
            'inner',
            'axis',
            'axisLabels',
            'grid',
            'invalidCells',
            'contents',
            'xBlock',
            'yBlock',
            'topBorder',
            'leftBorder',
            'bottomBorder',
        ]) {
            this[item] = this.shadowRoot.getElementById(`${item}`);
        }

        this.select = select(this.svg);
        this.zoom = zoom().on('zoom', this._zoomFunc.bind(this));

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
        this.updateGridCoverage(transform);

        const sep = 1;
        const isASS = this.mode === ChartMode.ASS;
        const visibleX = new Set<number>();
        const visibleY = new Set<number>();

        // X-axis labels
        const minX = Math.max(0, Math.ceil(transform.invertX(0)));
        const maxX = Math.floor(transform.invertX(this.width));

        for (let x = minX; x <= maxX; x += sep) {
            visibleX.add(x);
            const textNode = this.getOrCreateAxisLabelNode(this.xAxisLabelNodes, x);
            textNode.style.display = "";
            textNode.textContent = x.toString();
            // ASS: on grid lines (x), EHP: centered in boxes (x + 0.5)
            const xPos = isASS ? x : x + 0.5;
            textNode.setAttribute('x', transform.applyX(xPos).toString());
            // ASS labels sit at the bottom margin and remain fixed while panning/zooming.
            const xLabelY = isASS ? this.height + SvgChart.LABEL_MARGIN : -SvgChart.LABEL_MARGIN;
            textNode.setAttribute('y', xLabelY.toString());
            textNode.setAttribute('text-anchor', 'middle');
            textNode.setAttribute('dominant-baseline', isASS ? 'hanging' : 'text-after-edge');
        }

        // Y-axis labels
        const minY = isASS ? Math.ceil(transform.invertY(0)) : Math.max(0, Math.ceil(transform.invertY(0)));
        const maxY = Math.floor(transform.invertY(this.height));

        for (let y = minY; y <= maxY; y += sep) {
            visibleY.add(y);
            const textNode = this.getOrCreateAxisLabelNode(this.yAxisLabelNodes, y);
            // ASS: show flipped value (0 at bottom), EHP: normal value
            const displayY = isASS ? Math.round(this.chartMaxY - y) : y;
            if (isASS && displayY < 0) {
                textNode.style.display = "none";
                continue;
            }

            textNode.style.display = "";
            textNode.textContent = displayY.toString();
            // ASS: on grid lines (y), EHP: centered in boxes (y + 0.5)
            const yPos = isASS ? y : y + 0.5;
            textNode.setAttribute('y', transform.applyY(yPos).toString());
            textNode.setAttribute('x', (-SvgChart.LABEL_MARGIN).toString());
            textNode.setAttribute('text-anchor', 'end');
            textNode.setAttribute('dominant-baseline', 'middle');
        }

        this.hideUnusedAxisLabels(this.xAxisLabelNodes, visibleX);
        this.hideUnusedAxisLabels(this.yAxisLabelNodes, visibleY);
    }

    private updateGridCoverage(transform: any) {
        const visibleMinX = transform.invertX(0);
        const visibleMaxX = transform.invertX(this.width);
        const visibleMinY = transform.invertY(0);
        const visibleMaxY = transform.invertY(this.height);

        const grid_min_x = Math.floor(Math.min(this.minX, visibleMinX));
        const grid_max_x = Math.max(Math.ceil(this.maxX), Math.ceil(visibleMaxX));
        const grid_min_y = this.mode === ChartMode.ASS ? Math.floor(Math.min(0, visibleMinY)) : 0;
        const grid_max_y = Math.max(Math.ceil(this.maxY), Math.ceil(visibleMaxY));

        this.grid.setAttribute('x', grid_min_x.toString());
        this.grid.setAttribute('y', grid_min_y.toString());
        this.grid.setAttribute('width', (grid_max_x - grid_min_x).toString());
        this.grid.setAttribute('height', (grid_max_y - grid_min_y).toString());
    }

    private constrainTransform(transform: any, min_k: number) {
        let x = transform.x;
        let y = transform.y;
        let k = transform.k;

        k = Math.max(k, min_k);

        const minXTranslate = -this.maxX * k + this.width;
        const maxXTranslate = -this.minX * k;
        if (minXTranslate <= maxXTranslate) {
            x = Math.max(x, minXTranslate);
            x = Math.min(x, maxXTranslate);
        } else {
            // Viewport is wider than content; prefer left alignment.
            x = maxXTranslate;
        }

        const minYTranslate = -this.maxY * k + this.height;
        const maxYTranslate = -this.minY * k;
        if (minYTranslate <= maxYTranslate) {
            y = Math.max(y, minYTranslate);
            y = Math.min(y, maxYTranslate);
        } else {
            // Viewport is taller than content.
            // ASS: bottom-align; EHP: top-align.
            y = this.mode === ChartMode.ASS ? minYTranslate : maxYTranslate;
        }

        return zoomIdentity.translate(x, y).scale(k);
    }

    private getDefaultAnchoredTransform(min_k: number) {
        // Left edge is fixed for both modes.
        const x = -this.minX * min_k;
        // EHP anchors at top; ASS anchors at bottom so extra space appears above.
        const y = this.mode === ChartMode.ASS
            ? -this.maxY * min_k + this.height
            : -this.minY * min_k;
        return zoomIdentity.translate(x, y).scale(min_k);
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

        // Skip if element is not visible (display: none) - size will be 0
        if (size.width === 0 || size.height === 0) {
            return;
        }

        this.height = size.height - SvgChart.MARGIN;
        this.width = size.width - SvgChart.MARGIN;

        const min_k = Math.min(
            this.width / (this.maxX - this.minX),
            this.height / (this.maxY - this.minY),
        );

        if (this.mode === ChartMode.ASS) {
            // ASS reserves margin on the left and bottom (not top).
            this.svg.setAttribute(
                'viewBox',
                `${-SvgChart.MARGIN} 0 ${size.width} ${size.height}`,
            );
        } else {
            this.svg.setAttribute(
                'viewBox',
                `${-SvgChart.MARGIN} ${-SvgChart.MARGIN} ${size.width} ${size.height}`,
            );
        }

        this.zoom.constrain(transform => this.constrainTransform(transform, min_k));

        // xBlock covers the top margin area (where x-axis labels go)
        this.xBlock.setAttribute('width', size.width.toString());
        if (this.mode === ChartMode.ASS) {
            this.xBlock.setAttribute('y', this.height.toString());
        }

        // yBlock covers the left margin area (where y-axis labels go)
        this.yBlock.setAttribute('height', size.height.toString());

        // Update border lines for EHP mode (these are in screen coordinates, not chart coordinates)
        if (this.mode === ChartMode.EHP && this.topBorder && this.leftBorder) {
            this.topBorder.setAttribute('x2', this.width.toString());
            this.leftBorder.setAttribute('y2', this.height.toString());
        }
        if (this.mode === ChartMode.ASS && this.leftBorder && this.bottomBorder) {
            this.leftBorder.setAttribute('y2', this.height.toString());
            this.bottomBorder.setAttribute('x2', this.width.toString());
            this.bottomBorder.setAttribute('y1', this.height.toString());
            this.bottomBorder.setAttribute('y2', this.height.toString());
        }

        // Reapply a valid constrained transform after resize.
        // ASS always refits to the bottom-left anchor; EHP preserves user location when possible.
        const currentTransform = zoomTransform(this.svg);
        const targetTransform = this.mode === ChartMode.ASS
            ? this.getDefaultAnchoredTransform(min_k)
            : this.constrainTransform(currentTransform, min_k);
        this.zoom.transform(this.select, this.constrainTransform(targetTransform, min_k));
    }

    private getOrCreateAxisLabelNode(cache: Map<number, SVGTextElement>, key: number): SVGTextElement {
        let textNode = cache.get(key);
        if (!textNode) {
            textNode = document.createElementNS(svgNS, 'text');
            cache.set(key, textNode);
            this.axisLabels.appendChild(textNode);
        }
        return textNode;
    }

    private hideUnusedAxisLabels(cache: Map<number, SVGTextElement>, visibleValues: Set<number>) {
        cache.forEach((node, key) => {
            if (!visibleValues.has(key)) {
                node.style.display = "none";
            }
        });
    }
}
customElements.define('svg-chart', SvgChart);
