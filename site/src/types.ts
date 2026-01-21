export type Generators = {
    "name": string;
    "x": number;
    "y": number;
    "adams_filtration": number,
    "hom_name": string | undefined;
    "torsion": number | undefined,
    "purely_algebraic": boolean,
}

export type Differential = {
    "from": string,
    "to": string,
    "coeff": number,
    "d": number,
    "adams_d": number | undefined,
    "proof": string | undefined,
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