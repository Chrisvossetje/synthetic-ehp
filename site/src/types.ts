export type Generators = {
    "name": string;
    "x": number;
    "y": number;
    "adams_filtration": number,
    "induced_name": [number, string][],
    "torsion"?: number,
    "alg_name"?: string;
    "hom_name"?: string;
}

export type Differential = {
    "from": string,
    "to": string,
    "coeff": number,
    "d": number,
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
    "to": string
}

export type SyntheticEHP = {
    "generators": Generators[],
    "differentials": Differential[],
    "multiplications": Multiplication[],
    "tau_mults": TauMult[]
}