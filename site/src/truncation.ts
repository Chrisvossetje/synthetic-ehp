export type TruncationMode = "ahss" | "sphere";

export type TruncationControls = {
    truncationCheckbox: HTMLInputElement;
    truncationInput: HTMLInputElement;
    bottomTruncationCheckbox: HTMLInputElement;
    bottomTruncationInput: HTMLInputElement;
    sphereCheckbox: HTMLInputElement;
    sphereInput: HTMLInputElement;
};

export type TruncationState = {
    truncation: number | null | undefined;
    bottomTruncation: number | undefined;
};

function parseIntOrNull(value: string): number | null {
    const parsed = parseInt(value, 10);
    return Number.isNaN(parsed) ? null : parsed;
}

function ensureSphereValue(input: HTMLInputElement): number {
    const parsed = parseIntOrNull(input.value);
    if (parsed === null) {
        input.value = "7";
        return 7;
    }
    return parsed;
}

export function applyTruncationFromControls(
    mode: TruncationMode,
    controls: TruncationControls,
    state: TruncationState
) {
    if (mode === "ahss") {
        if (controls.truncationCheckbox.checked) {
            const value = parseIntOrNull(controls.truncationInput.value);
            state.truncation = value === null ? null : value;
        } else {
            state.truncation = null;
        }

        if (controls.bottomTruncationCheckbox.checked) {
            const value = parseIntOrNull(controls.bottomTruncationInput.value);
            state.bottomTruncation = value === null ? 0 : value;
        } else {
            state.bottomTruncation = undefined;
        }
        return;
    }

    const sphereEnabled = controls.sphereCheckbox.checked;
    const sphereValue = ensureSphereValue(controls.sphereInput);
    state.truncation = sphereEnabled ? (sphereValue - 1) : null;
    state.bottomTruncation = undefined;

}

export function syncSphereControlsFromState(
    controls: TruncationControls,
    state: TruncationState,
    ahss: boolean,
) {
    if (ahss) {
        controls.truncationCheckbox.checked = state.truncation !== null && state.truncation !== undefined;
        if (controls.truncationCheckbox.checked) {
            const value = Math.max(0, (state.truncation ?? 0));
            controls.truncationInput.value = value.toString();
        }
    } else {
        controls.sphereCheckbox.checked = state.truncation !== null && state.truncation !== undefined;
        if (controls.sphereCheckbox.checked) {
            const value = Math.max(0, (state.truncation ?? 0) + 1);
            controls.sphereInput.value = value.toString();
        }
    }
}
