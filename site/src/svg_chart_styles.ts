export const EHP_CHART_CSS = `
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

export const ASS_CHART_CSS = `
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
    r: 0.04;
    filter: drop-shadow(0 0 0.1 rgba(0,0,0,0.4));
}

.generator-dot:active {
    fill: #777;
}

/* Differential line styles - LARGER */
.differential-line {
    stroke: #555;
    stroke-width: 0.012;
    cursor: pointer;
    transition: all 0.15s ease;
}

.differential-line:hover {
    stroke: #666 !important;
    stroke-width: 0.024 !important;
    filter: drop-shadow(0 0 0.1 rgba(0,0,0,0.4));
}

.differential-line:active {
    stroke: #777 !important;
}

/* Generator label styles - LARGER */
.generator-label {
    pointer-events: none;
    font-size: 0.06px;
    fill: #000;
    font-family: monospace;
    font-weight: bold;
}

/* Filtration label styles - LARGER */
.generator-filtration-label {
    pointer-events: none;
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
`;