const GENERATOR_LABEL_CSS = `
.generator-label {
    pointer-events: none;
    overflow: visible;
}

.generator-label-content {
    content-visibility: auto;
    display: flex;
    align-items: center;
    height: 100%;
    color: black;
    font-size: 12px;
    white-space: nowrap;
}

.generator-label-content.align-left {
    justify-content: flex-end;
}

.generator-label-content > * {
    pointer-events: auto;
    cursor: pointer;
}

.generator-label-content > span {
    font-family: monospace;
}
`;

export const EHP_CHART_CSS = `
${GENERATOR_LABEL_CSS}
:host { display: block; }
.struct { stroke: black; fill: none; stroke-width: 0.03; }

/* Generator dot styles */
.generator-dot {
    cursor: pointer;
    transition: all 0.15s ease;
    stroke-width: 0.01;
}

.generator-dot:hover {
    fill: #555;
    r: 0.027;
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

/* Filtration label styles */
.generator-filtration-label {
    pointer-events: auto;
    cursor: pointer;
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

/* Tau multiplication line styles */
.tau-mult-line {
    stroke: #FFA500;
    stroke-width: 0.008;
    pointer-events: auto;
    cursor: pointer;
    opacity: 0.7;
    transition: all 0.15s ease;
}

.tau-mult-line:hover {
    stroke: #ffbc3e	;
    stroke-width: 0.016;
}

.tau-mult-line:active {
    stroke: #ffcd70;
}
`;

export const ASS_CHART_CSS = `
${GENERATOR_LABEL_CSS}
:host { display: block; }
.struct { stroke: black; fill: none; stroke-width: 0.04; }

/* Generator dot styles - LARGER */
.generator-dot {
    cursor: pointer;
    transition: all 0.15s ease;
    stroke-width: 0.015;
}

.generator-dot:hover {
    fill: #555;
    r: 0.077;
    filter: drop-shadow(0 0 0.1 rgba(0,0,0,0.4));
}

.generator-dot:active {
    fill: #777;
}

/* Differential line styles - LARGER */
.differential-line {
    stroke: #555;
    stroke-width: 0.014;
    cursor: pointer;
    transition: all 0.15s ease;
}

.differential-line:hover {
    stroke: #666 !important;
    stroke-width: 0.028 !important;
    filter: drop-shadow(0 0 0.1 rgba(0,0,0,0.4));
}

.differential-line:active {
    stroke: #777 !important;
}

/* Filtration label styles - LARGER */
.generator-filtration-label {
    pointer-events: auto;
    cursor: pointer;
    font-size: 0.05px;
    fill: #666;
    font-family: monospace;
}

/* Multiplication line styles - internal - LARGER */
.multiplication-line-internal {
    stroke: #8888;
    stroke-width: 0.012;
    pointer-events: none;
    opacity: 0.6;
}

/* Multiplication line styles - external - LARGER */
.multiplication-line-external {
    stroke: #444;
    stroke-width: 0.015;
    pointer-events: none;
    opacity: 0.8;
}

/* Tau multiplication line styles - LARGER */
.tau-mult-line {
    stroke: orange;
    stroke-width: 0.012;
    pointer-events: auto;
    cursor: pointer;
    opacity: 0.7;
}
`;
