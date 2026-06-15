import { Generators } from "../types";

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
    labels: Partial<GeneratorLabels> = {}
): string[] {
    const resolved = { ...defaultLabels, ...labels };
    const lines: string[] = [];

    lines.push(`${resolved.xLabel}: ${gen.stem}`);
    lines.push(`${resolved.yLabel}: ${gen.y}`);
    lines.push(`Adams filtration: ${gen.af}`);
    lines.push(`${resolved.moduleLabel}: ${formatTorsion(gen)}`);

    if (gen.alg_name) {
        lines.push(`Algebraic name: ${gen.alg_name}`);
    }
    if (gen.hom_name) {
        lines.push(`Homotopy name: ${gen.hom_name}`);
    }

    const filteredInducedNames = (gen.induced_name ?? []).filter(([num, _]) => num !== 0);
    if (filteredInducedNames.length > 0) {
        const namesList = filteredInducedNames
            .map(([sphere, name]) => `${name} (sphere ${sphere})`)
            .join(', ');
        lines.push(`Induced name: ${namesList}`);
    }

    lines.push(`Born on sphere: ${sphereInfo.bornSphere}`);
    lines.push(`Dies on sphere: ${sphereInfo.diesOnAlgebraicSphere}`);

    return lines;
}

export function showInfoPanel(title: string, lines: string[], extraLines: string[] = []) {
    const floatingBox = document.getElementById('floatingBox');
    if (!floatingBox) return;

    let content = `<span class="close-btn" onclick="document.getElementById('floatingBox').style.display='none'">x</span>`;
    content += `<h4>${title}</h4>`;
    content += `<pre style="background-color: #00000000; margin: 0;">`;
    content += lines.join("\n");

    if (extraLines.length > 0) {
        content += `\n\n${extraLines.join("\n")}`;
    }

    content += `</pre>`;
    floatingBox.innerHTML = content;
    floatingBox.style.display = 'block';
}
