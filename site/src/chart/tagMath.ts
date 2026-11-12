import { escapeHtml } from "./text";

const atom = /\\[A-Za-z]+|\d+|[^\s_^{}]/g;
const term = /(\\[A-Za-z]+|\d+|[^\s_^{}])((?:[_^](?:\{[^{}]*\}|\\[A-Za-z]+|-?\d+|[^\s_^{}])){0,2})/g;
const script = /([_^])(?:\{([^{}]*)\}|(\\[A-Za-z]+|-?\d+|[^\s_^{}]))/g;

function mathAtom(value: string): string {
    if (/^\d+$/.test(value)) return "<mn>" + value + "</mn>";
    if (value === "\\Delta") return "<mi>Δ</mi>";
    if (value === "\\infty") return "<mo>∞</mo>";
    if (/^[A-Za-zτ]$/.test(value)) return "<mi>" + value + "</mi>";
    const symbol = value === "<" ? "⟨" : value === ">" ? "⟩" : value;
    return "<mo>" + escapeHtml(symbol) + "</mo>";
}

/** Render the generated Adams tag notation, including grouped scripts. */
export function renderAdamsTag(tag: string): string {
    let body = "";
    for (let match = term.exec(tag); match; match = term.exec(tag)) {
        const [, value, scripts] = match;
        const base = mathAtom(value);
        let subscript = "";
        let superscript = "";
        for (let suffix = script.exec(scripts); suffix; suffix = script.exec(scripts)) {
            const [, operator, group, single] = suffix;
            const tokens = (group ?? single).match(atom) ?? [];
            const content = tokens.map(mathAtom).join("");
            const argument = tokens.length === 1 ? content : "<mrow>" + content + "</mrow>";
            if (operator === "_") subscript = argument;
            else superscript = argument;
        }
        if (subscript && superscript) body += "<msubsup>" + base + subscript + superscript + "</msubsup>";
        else if (subscript) body += "<msub>" + base + subscript + "</msub>";
        else if (superscript) body += "<msup>" + base + superscript + "</msup>";
        else body += base;
    }
    return '<math xmlns="http://www.w3.org/1998/Math/MathML"><mrow>' + body + "</mrow></math>";
}
