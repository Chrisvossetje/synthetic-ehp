/** Escape text before inserting it into HTML/SVG markup (tags can contain <...>). */
export function escapeHtml(text: string): string {
    return text.replace(/[&<>"']/g, (character) => ({
        "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
    })[character]!);
}
