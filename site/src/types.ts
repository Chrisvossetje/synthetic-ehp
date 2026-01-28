export type Generators = {
    "name": string;
    "x": number;
    "y": number;
    "adams_filtration": number,
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
}

export type Multiplication = {
    "from": string,
    "to": string,
    "internal": boolean
}

export type SyntheticEHP = {
    "generators": Generators[],
    "differentials": Differential[],
    "multiplications": Multiplication[]
}