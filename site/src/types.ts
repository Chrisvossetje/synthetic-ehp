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
    "coeff": number,
    "d": number,
    "kind": "Real" | "Fake";
    "proof"?: string,
    "synthetic"?,
}

export type Multiplication = {
    "from": string,
    "to": string,
    "internal": boolean
}

export type TauMult = {
    "from": string,
    "to": string,
    "kind": "Real" | "Fake";
}

export type SyntheticEHP = {
    "generators": Generators[],
    "differentials": Differential[],
    "multiplications": Multiplication[],
    "tau_mults": TauMult[]
}