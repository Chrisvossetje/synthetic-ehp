use crate::types::{Differential, TauMult};


pub fn get_diffs() -> Vec<Differential> {
    vec![
        // Stable diffs

        // Stem 14/15, AF 1 -> 3
        Differential {
            from: "[15]".to_string(),
            to: "5 3[6]".to_string(),
            coeff: 1,
            d: 9,
            proof: Some("Stable Diff".to_string()),
            synthetic: Some(()),
        },
        // Stem 14/15, AF 2 -> 5
        Differential {
            from: "1[14]".to_string(),
            to: "1 2 3 3[5]".to_string(),
            coeff: 2,
            d: 9,
            proof: Some("Stable Diff".to_string()),
            synthetic: Some(()),
        },
        // Stem 14/15, AF 3 -> 6
        Differential {
            from: "1 1[13]".to_string(),
            to: "4 4 1 1 1[3]".to_string(),
            coeff: 2,
            d: 10,
            proof: Some("Stable Diff".to_string()),
            synthetic: Some(()),
        },
        // Stem 16/17, AF 4 -> 6
        Differential {
            from: "3 3 3[8]".to_string(),
            to: "2 4 3 3 3[1]".to_string(),
            coeff: 1,
            d: 7,
            proof: Some("Stable Diff".to_string()),
            synthetic: Some(()),
        },
        // Stem 17/18, AF 5 -> 7
        Differential {
            from: "5 3 3 3[4]".to_string(),
            to: "1 2 4 3 3 3[1]".to_string(),
            coeff: 1,
            d: 3,
            proof: Some("Stable Diff".to_string()),
            synthetic: Some(()),
        },
        // Stem 17/18, AF 4 -> 6
        Differential {
            from: "7 3 3[5]".to_string(),
            to: "2 4 3 3 3[2]".to_string(),
            coeff: 1,
            d: 3,
            proof: Some("Stable Diff".to_string()),
            synthetic: Some(()),
        },
        // Stem 22/23, AF 7 -> 9
        Differential {
            from: "1 2 4 1 1 1[13]".to_string(),
            to: "1 2 3 4 4 1 1 1[5]".to_string(),
            coeff: 1,
            d: 8,
            proof: Some("Stable Diff".to_string()),
            synthetic: Some(()),
        },
        // Stem 22/23, AF 8 -> 10
        Differential {
            from: "1 1 2 4 1 1 1[12]".to_string(),
            to: "4 4 1 1 2 4 1 1 1[3]".to_string(),
            coeff: 1,
            d: 9,
            proof: Some("Stable Diff".to_string()),
            synthetic: Some(()),
        },


        
        // // Unstable diffs

        // Stem 19/20, AF 5 -> 7
        // Sphere 5
        Differential {
            from: "7 3 3 3[4]".to_string(),
            to: "1 2 4 3 3 3[3]".to_string(),
            coeff: 1,
            d: 1,
            proof: None,
            synthetic: Some(()),
        },

        // Stem 21/22, AF 5 -> 7
        // Sphere 9
        Differential {
            from: "6 5 3[8]".to_string(),
            to: "5 1 2 3 3[7]".to_string(),
            coeff: 1,
            d: 1,
            proof: Some("If this diff does not exists, we would have an element with AF 3 on the algebraic 9 sphere.".to_string()),
            synthetic: Some(()),
        },

        // Stem 21/22, AF 5 -> 7
        // Sphere 9 -> 13
        Differential {
            from: "6 2 3 3[8]".to_string(),
            to: "2 4 3 3 3[6]".to_string(),
            coeff: 1,
            d: 2,
            proof: Some("Classical 9 - 13 Sphere needs a generator killed. This is the only viable differential.".to_string()),
            synthetic: Some(()),
        },



        // Stem 22/23, AF 5 -> 6
        // Sphere 9
        Differential {
            from: "2 4 3 3 3[8]".to_string(),
            to: "3 6 2 3 3[5]".to_string(),
            coeff: 0,
            d: 3,
            proof: Some("We need the differential here such that the convergence of the synthetic SS is coherent with the algebraic sphere. No other differential could satisfy this.".to_string()),
            synthetic: Some(()),
        },


        // Stem 22/23, AF 5 -> 6
        // Sphere 17
        Differential {
            from: "4 1 1 1[16]".to_string(),
            to: "3 4 4 1 1 1[8]".to_string(),
            coeff: 0,
            d: 8,
            proof: Some("We need the differential here such that the convergence of the synthetic SS is coherent with the algebraic sphere. No other differential could satisfy this.".to_string()),
            synthetic: Some(()),
        },

        // Stem 22/23, AF 4 -> 5
        // Sphere 17
        Differential {
            from: "4 1 1 1[16]".to_string(),
            to: "12 1 1 1[7]".to_string(),
            coeff: 0,
            d: 9,
            proof: Some("We need the differential here such that the convergence of the synthetic SS is coherent with the algebraic sphere. No other differential could satisfy this. This one 'represents the same' as the other originating from this differential.".to_string()),
            synthetic: Some(()),
        },

        // Stem 22/23, AF 4 -> 5
        // Sphere 17
        Differential {
            from: "1 1 1[20]".to_string(),
            to: "4 4 1 1 1[11]".to_string(),
            coeff: 0,
            d: 9,
            proof: Some("We need the differential here such that the convergence of the synthetic SS is coherent with the algebraic sphere. No other differential could satisfy this.".to_string()),
            synthetic: Some(()),
        },

        // Stem 22/23, AF 3 -> 4
        // Sphere 17
        Differential {
            from: "1 1[21]".to_string(),
            to: "1 2 3 3[13]".to_string(),
            coeff: 0,
            d: 8,
            proof: Some("We need the differential here such that the convergence of the synthetic SS is coherent with the algebraic sphere. No other differential could satisfy this.".to_string()),
            synthetic: Some(()),
        },
        // Stem 22/23, AF 2 -> 3
        // Sphere 17
        Differential {
            from: "7 7[10]".to_string(),
            to: "13 3[7]".to_string(),
            coeff: 0,
            d: 3,
            proof: Some("We need the differential here such that the convergence of the synthetic SS is coherent with the algebraic sphere. No other differential could satisfy this.".to_string()),
            synthetic: Some(()),
        },




        // Stem 23/24, AF 5 -> 8
        // Sphere 10
        Differential {
            from: "12 1 1 1[9]".to_string(),
            to: "2 3 4 4 1 1 1[7]".to_string(),
            coeff: 2,
            d: 2,
            proof: Some("One classical homotopy group needs to die on this sphere. Also on the 11 sphere there will be a diff from a 2-torsion element to this, meaning this differential must exist to make the Z[τ] structure work out".to_string()),
            synthetic: Some(()),
        },

        // Stem 23/24, AF 3 -> 5
        // Sphere 11
        Differential {
            from: "6 2 3 3[10]".to_string(),
            to: "3 6 2 3 3[6]".to_string(),
            coeff: 1,
            d: 4,
            proof: Some("We need the differential here such that the convergence of the synthetic SS is coherent with the algebraic sphere. No other differential could satisfy this. Also this differential should exist together with the originating from this cell.".to_string()),
            synthetic: Some(()),
        },

        // Stem 23/24, AF 3 -> 5
        // Sphere 11
        Differential {
            from: "6 5 3[10]".to_string(),
            to: "3 6 2 3 3[6]".to_string(),
            coeff: 0,
            d: 4,
            proof: Some("We need the differential here such that the convergence of the synthetic SS is coherent with the algebraic sphere. No other differential could satisfy this. Also this differential should exist together with the originating from this cell.".to_string()),
            synthetic: Some(()),
        },
    ]
}

pub fn get_induced_names() -> Vec<(String, Vec<(i32, String)>)> {
    // (Original name , induced name)
    vec![
        // Stem 20
        (
            "2 4 3 3 3[5]".to_string(),
            vec![(6, "7 3 3 3[4]".to_string())],
        ),
        (
            "3 6 2 3 3[3]".to_string(),
            vec![(5, "4 5 3 3 3[2]".to_string())],
        ),


        // Stem 21
        (
            "3 6 2 3 3[4]".to_string(),
            vec![(5, "5 7 3 3[3]".to_string())],
        ),


        // Stem 22
        (
            "3 4 4 1 1 1[8]".to_string(),
            vec![(9, "4 5 3 3 3[4]".to_string())],
        ),
        (
            "4 4 1 1 1[11]".to_string(),
            vec![(12, "8 3 3 3[5]".to_string())],
        ),
        (
            "1 2 3 3[13]".to_string(),
            vec![(14, "6 5 3[8]".to_string())],
        ),
        
        
        // Stem 23
        (
            "3 6 2 3 3[6]".to_string(),
            vec![(7, "5 7 3 3[5]".to_string())],
        ),
    ]
}


pub fn get_tau_mults() -> Vec<TauMult> {
    vec![
        // Stem 22
        TauMult {
            from: "1 2 4 3 3 3[6]".to_string(),
            to: "3 6 2 3 3[5]".to_string(),
        },
        TauMult {
            from: "3 4 4 1 1 1[8]".to_string(),
            to: "12 1 1 1[7]".to_string(),
        },

        // Stem 23
        TauMult {
            from: "4 1 1 2 4 3 3 3[2]".to_string(),
            to: "2 2 4 5 3 3 3[1]".to_string(),
        },
        TauMult {
            from: "6 5 3[9]".to_string(),
            to: "13 3[7]".to_string(),
        },

    ]
}
