import { Generators } from "../types";
import { viewSettings } from "../model/settings";
import { renderAdamsTag } from "./tagMath";
import { escapeHtml } from "./text";

type SphereInfo = {
    bornSphere: string;
    diesOnAlgebraicSphere: string;
};

type GeneratorLabels = {
    xLabel: string;
    yLabel: string;
    moduleLabel: string;
};

const defaultLabels: GeneratorLabels = {
    xLabel: "stem",
    yLabel: "y",
    moduleLabel: "Module",
};

function formatTorsion(gen: Generators): string {
    return gen.torsion !== undefined ? `F2[τ]/τ^${gen.torsion}` : "F2[τ]";
}

export function buildGeneratorInfoLines(
    gen: Generators,
    sphereInfo: SphereInfo,
    labels: Partial<GeneratorLabels> = {},
    formatName: (name: string) => string = (name) => name,
    filtration = gen.af
): string[] {
    const resolved = { ...defaultLabels, ...labels };
    const lines: string[] = [];

    lines.push(`${resolved.xLabel}: ${gen.stem}`);
    lines.push(`${resolved.yLabel}: ${gen.y}`);
    lines.push(`Adams filtration: ${filtration}`);
    lines.push(`${resolved.moduleLabel}: ${formatTorsion(gen)}`);

    if (gen.hom_name) {
        lines.push(`Homotopy name: ${gen.hom_name}`);
    }

    const filteredInducedNames = (gen.induced_name ?? []).filter(([num, _]) => num !== 0);
    if (filteredInducedNames.length > 0) {
        const namesList = filteredInducedNames
            .map(([sphere, name]) => `${formatName(name)} (sphere ${sphere})`)
            .join(', ');
        lines.push(`Induced name: ${namesList}`);
    }

    lines.push(`Born on sphere: ${sphereInfo.bornSphere}`);
    lines.push(`Dies on sphere: ${sphereInfo.diesOnAlgebraicSphere}`);

    return lines;
}

export function showInfoPanel(title: string, lines: string[], extraLines: string[] = [], tag?: string) {
    const floatingBox = document.getElementById('floatingBox');
    if (!floatingBox) return;

    let content = `<span class="close-btn" onclick="document.getElementById('floatingBox').style.display='none'">x</span>`;
    const tagMarkup = tag === undefined ? "" : ": " + (
        viewSettings.tagNotation === "adams" ? renderAdamsTag(tag) : escapeHtml(tag)
    );
    content += `<h4>${escapeHtml(title)}${tagMarkup}</h4>`;
    content += `<pre style="background-color: #00000000; margin: 0;">`;
    content += escapeHtml(lines.join("\n"));

    if (extraLines.length > 0) {
        content += `\n\n${escapeHtml(extraLines.join("\n"))}`;
    }

    content += `</pre>`;
    floatingBox.innerHTML = content;
    floatingBox.style.display = 'block';
}
