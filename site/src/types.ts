export type Generators = {
    "name": string;
    "stem": number;
    "y": number;
    "af": number,
    "born": number,
    "dies": number,
    "induced_name"?: [number, string][],
    "torsion"?: number,
    "alg_name"?: string;
    "hom_name"?: string;
}

export type Differential = {
    "from": string,
    "to": string,
    "d"?: number,
    "coeff"?: number,
    "kind": "Real" | "Fake" | "Unknown";
    "proof"?: string,
}

export type Multiplication = {
    "from": string,
    "to": string,
}

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
