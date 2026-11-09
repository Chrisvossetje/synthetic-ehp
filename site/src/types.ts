export type Generators = {
    "name": string;
    "stem": number;
    "y": number;
    "af": number,
    "born": number,
    "dies": number,
    "induced_name"?: [number, string][],
    "torsion"?: number,
    /** Algebraic display label; `name` remains the Curtis identifier. */
    "alg_name"?: string;
    "hom_name"?: string;
}

export type Kind = "Algebraic" | "Real" | "Fake" | "Unknown" | "Lifted" | "Invisible" | "Unneccessary" | "MinimalLength" | "AdditiveStructure" | "Periodicity";

export type Differential = {
    "from": string,
    "to": string,
    "kind": Kind;
    "d"?: number,
    "coeff"?: number,
    "proof"?: string,
}

export type Multiplication = {
    "from": string,
    "to": string,
}

/** A resolved display overlay, independent of spectral-sequence computation. */
export type DisplayMultiplication = {
    from: string;
    to: string;
    operator: 0 | 1 | 2;
    sphere: number;
    targets: readonly string[];
    displayTargets?: readonly string[];
    inheritedFrom?: { tag: string; sphere: number };
    /** Recorded real tau extensions supporting a historical cross-row product. */
    tauExtensions?: readonly { from: string; to: string }[];
};

export type InternalTauMult = {
    "from": string,
    "to": string,
    "page": number,
    "kind": "Real" | "Fake" | "Unknown";
    "proof"?: string,
}

export type ExternalTauMult = {
    "from": string,
    "to": string,
    "af": number,
    "kind": "Real" | "Fake" | "Unknown";
    "proof"?: string,
}

export type SyntheticEHP = {
    "generators": Generators[],
    "differentials": Differential[],
    "multiplications": Multiplication[],
    "internal_tau_mults": InternalTauMult[],
    "external_tau_mults": ExternalTauMult[]
}
