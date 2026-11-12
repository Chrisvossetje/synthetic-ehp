import { STABLE_TAG_TRANSLATIONS, TAG_TRANSLATIONS } from "../tag_translation";
import type { SyntheticEHP } from "../types";
import { viewSettings } from "./settings";

type TagTranslations = Readonly<Record<string, string>>;
let activeTranslations: TagTranslations = TAG_TRANSLATIONS;

/** Keep names in popups, products and exports aligned with the selected dataset. */
export function setStableTagNames(stable: boolean): void {
    activeTranslations = stable ? STABLE_TAG_TRANSLATIONS : TAG_TRANSLATIONS;
}

/** Translate the tag independently of the final sphere, so families keep their names. */
export function algebraicTag(tag: string): string {
    return translateTag(tag, activeTranslations);
}

function translateTag(tag: string, translations: TagTranslations): string {
    const normalized = tag.trim().replace(/\s+/g, " ");
    if (Object.prototype.hasOwnProperty.call(translations, normalized)) {
        return translations[normalized];
    }
    if (/^\d+(?: \d+)*$/.test(normalized)) {
        const indices = normalized.split(" ").map(Number);
        // The remaining manual basis choices have no verified algebraic name.
        // Show their degree rather than presenting a Curtis word as a name.
        return `?_{${indices.reduce((sum, index) => sum + index, 0)},${indices.length}}`;
    }
    return normalized;
}

/** Internal Curtis identifiers stay intact; only user-facing names pass through here. */
export function algebraicName(name: string): string {
    return translateName(name, activeTranslations);
}

export function displayName(name: string, translated?: string): string {
    return viewSettings.tagNotation === "curtis"
        ? name.replace(/__ass_(src|tgt)$/, "")
        : translated ?? algebraicName(name);
}

function translateName(name: string, translations: TagTranslations): string {
    const base = name.replace(/__ass_(src|tgt)$/, "");
    const match = /^(.*?)\[(\d+)\]$/.exec(base);
    return match ? `${translateTag(match[1], translations)}[${match[2]}]` : base;
}

/** Multiplication tables put the sphere first: #(7 5 3) means 5 3[7]. */
export function displayMultiplicationTag(tag: string): string {
    const normalized = tag.replace(/^#\(|\)$/g, "").trim();
    const [sphere, ...indices] = normalized.split(/\s+/);
    return displayName(`${indices.join(" ")}[${sphere}]`);
}

/** Format names embedded in a raw proof, without interpreting it as HTML. */
export function displayText(text: string): string {
    if (viewSettings.tagNotation === "curtis") return text;
    return text.replace(
        /#\((\d+(?:\s+\d+)*)\)|(^|[\s("'=,:;+>−-])((?:\d+(?:[ \t]+\d+)*)?\[\d+\])/g,
        (_match, tableTag: string | undefined, prefix: string, name: string) =>
            tableTag !== undefined ? displayMultiplicationTag(tableTag) : prefix + algebraicName(name)
    );
}

export function withAlgebraicNames(data: SyntheticEHP, stable: boolean = false): SyntheticEHP {
    const translations = stable ? STABLE_TAG_TRANSLATIONS : TAG_TRANSLATIONS;
    return {
        ...data,
        generators: data.generators.map((gen) => ({ ...gen, alg_name: translateName(gen.name, translations) })),
    };
}
