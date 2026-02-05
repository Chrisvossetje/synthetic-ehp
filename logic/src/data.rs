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
            proof: Some("Geometric 9 - 13 Sphere needs a generator killed. This is the only viable differential.".to_string()),
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
            proof: Some("One geometric homotopy group needs to die on this sphere. Also on the 11 sphere there will be a diff from a 2-torsion element to this, meaning this differential must exist to make the Z[τ] structure work out".to_string()),
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

        // Stem 24/25, AF 7 -> 10
        // Sphere 3
        Differential {
            from: "2 3 5 7 3 3[2]".to_string(),
            to: "2 4 1 1 2 4 3 3 3[1]".to_string(),
            coeff: 2,
            d: 1,
            proof: Some("We need an Adams differential on the 3 sphere. The only possibilities are from this element or the element in AF 5. But the element in AF 5 will be killed by an AEHP, meaning it could not support a differential as the target survives to E_infinity.".to_string()),
            synthetic: Some(()),
        },

        // Stem 24/25, AF 6 -> 8
        // Sphere 6 - 10
        Differential {
            from: "3 5 7 3 3[4]".to_string(),
            to: "2 2 4 5 3 3 3[2]".to_string(),
            coeff: 1,
            d: 2,
            proof: Some("This differential both is needed for the geometric homotopy groups and is the only possible differential which can repair the possible torsion problem.".to_string()),
            synthetic: Some(()),
        },

        // Stem 24/25, AF 5 -> 7
        // Sphere 9 - 11
        Differential {
            from: "3 6 2 3 3[8]".to_string(),
            to: "2 4 5 3 3 3[4]".to_string(),
            coeff: 1,
            d: 4,
            proof: Some("We need a unstable differential which does not survive stably.".to_string()),
            synthetic: Some(()),
        },

        // Stem 24/25, AF 8 -> 10
        // Sphere 9
        Differential {
            from: "1 1 2 4 3 3 3[8]".to_string(),
            to: "2 4 1 1 2 4 3 3 3[1]".to_string(),
            coeff: 1,
            d: 7,
            proof: Some("Together with the other differential with this target it represents the stable differential in these stems.".to_string()),
            synthetic: Some(()),
        },

        // Stem 24/25, AF 5 -> 6
        // Sphere 12
        Differential {
            from: "6 2 3 3[11]".to_string(),
            to: "3 6 2 3 3[7]".to_string(),
            coeff: 0,
            d: 4,
            proof: Some("Algebraicly we need to this element to die from this sphere, this is the only possibility.".to_string()),
            synthetic: Some(()),
        },
        
        // Stem 24/25, AF 4 -> 5
        // Sphere 17
        Differential {
            from: "3 3 3[16]".to_string(),
            to: "2 4 3 3 3[9]".to_string(),
            coeff: 0,
            d: 7,
            proof: Some("Algebraicly we need to this element to die from this sphere, this is the only possibility.".to_string()),
            synthetic: Some(()),
        },



        // Stem 25/26, AF 9 -> 11
        // Sphere 3
        Differential {
            from: "2 2 2 4 5 3 3 3[2]".to_string(),
            to: "1 2 4 1 1 2 4 3 3 3[1]".to_string(),
            coeff: 1,
            d: 1,
            proof: Some("We need this differential to correct the stable adams differential in this range".to_string()),
            synthetic: Some(()),
        },
        // Stem 25/26, AF 8 -> 10
        // Sphere 5
        Differential {
            from: "2 2 4 5 3 3 3[4]".to_string(),
            to: "2 4 1 1 2 4 3 3 3[2]".to_string(),
            coeff: 1,
            d: 2,
            proof: Some("Only possible place to support this stable adams differential".to_string()),
            synthetic: Some(()),
        },
        // Stem 25/26, AF 7 -> 9
        // Sphere 7
        Differential {
            from: "3 5 7 3 3[5]".to_string(),
            to: "6 2 3 4 4 1 1 1[3]".to_string(),
            coeff: 2,
            d: 2,
            proof: Some("We need one more ".to_string()),
            synthetic: Some(()),
        },
        // Stem 25/26, AF 7 -> 9
        // Sphere 7
        Differential {
            from: "2 4 5 3 3 3[6]".to_string(),
            to: "6 2 3 4 4 1 1 1[3]".to_string(),
            coeff: 1,
            d: 3,
            proof: Some("We need the AF 9 element to be the correct torsion. And stem 26 needs this element in AF 6.".to_string()),
            synthetic: Some(()),
        },
        // Stem 25/26, AF 2 -> 3
        // Sphere 12
        Differential {
            from: "7 7[12]".to_string(),
            to: "11 7[7]".to_string(),
            coeff: 0,
            d: 5,
            proof: Some("Algebraicly we need an element in AF 3 to die. We cannot hit 7 7[11] as we already hit something else on the E1 page.".to_string()),
            synthetic: Some(()),
        },
        // Stem 25/26, AF 3 -> 6
        // Sphere 13
        Differential {
            from: "6 2 3 3[12]".to_string(),
            to: "2 4 3 3 3[10]".to_string(),
            coeff: 2,
            d: 2,
            proof: Some("Algebraicly we need an element in AF 5 to die. It also must come from this AF 4 + AF 3 element. The other AF 5 element 3 5 7 7[3] will get killed later and we must have zero surviving elements in AF 5.".to_string()),
            synthetic: Some(()),
        },
        // Stem 25/26, AF 4 -> 6
        // Sphere 13
        Differential {
            from: "6 5 3[12]".to_string(),
            to: "2 4 3 3 3[10]".to_string(),
            coeff: 1,
            d: 2,
            proof: Some("Algebraicly we need an element in AF 5 to die. It also must come from this AF 4 + AF 3 element. The other AF 5 element 3 5 7 7[3] will get killed later and we must have zero surviving elements in AF 5.".to_string()),
            synthetic: Some(()),
        },

        // Stem 26/27, 7 -> 10
        // Sphere 5
        Differential {
            from: "2 3 5 7 3 3[4]".to_string(),
            to: "2 4 1 1 2 4 3 3 3[3]".to_string(),
            coeff: 2,
            d: 1,
            proof: Some("The target of this differential is the only one which could represent an unstable differential, as it should also be killed on the 6 sphere. Only the element in degree 7 could support this differential.".to_string()),
            synthetic: Some(()),
        },
        // Stem 26/27, 5 -> 6
        // Sphere 7
        Differential {
            from: "7 6 2 3 3[6]".to_string(),
            to: "5 5 7 3 3[3]".to_string(),
            coeff: 0,
            d: 3,
            proof: Some("Algebraicly we need an element in AF 6 to die. We only need to kill this from the 7 sphere so the other AF 5 element which could support this diff is ruled out.".to_string()),
            synthetic: Some(()),
        },
        // Stem 26/27, 5 -> 6
        // Sphere 11
        Differential {
            from: "3 6 2 3 3[10]".to_string(),
            to: "2 4 5 3 3 3[6]".to_string(),
            coeff: 0,
            d: 4,
            proof: Some("We need an AF 6 element to die. This is the only element which can support one on this sphere. TODO: Why not target the AF6 element on the seven sphere?".to_string()),
            synthetic: Some(()),
        },

        // Stem 26/27, 5 -> 6
        // Sphere 13
        Differential {
            from: "2 4 3 3 3[12]".to_string(),
            to: "3 6 2 3 3[9]".to_string(),
            coeff: 0,
            d: 3,
            proof: Some("We need an AF 6 element to die. This is the only element which can support one on this sphere. TODO: Why not target the AF6 element in y=3?".to_string()),
            synthetic: Some(()),
        },

        // Stem 26/27, 4 -> 5
        // Sphere 19
        Differential {
            from: "2 4 1 1 1[18]".to_string(),
            to: "12 1 1 1[11]".to_string(),
            coeff: 0,
            d: 7,
            proof: Some("We need an AF 5 element to die. The tau multiplication also implies that this element should be the one to kill it. Also the other AF 4 element here cannot support the differential as it is killed later.".to_string()),
            synthetic: Some(()),
        },


        // Stem 27/28, 10 -> 12
        // Sphere 6
        Differential {
            from: "8 4 1 1 2 4 1 1 1[5]".to_string(),
            to: "2 3 4 4 1 1 2 4 1 1 1[3]".to_string(),
            coeff: 1,
            d: 2,
            proof: Some("There is an induced differential from AEHP, which is now from torsion to torsion free. This implies that we need to make its target torsion as well! There is only one element which can do this, and this also gives us a unstable differential!".to_string()),
            synthetic: Some(()),
        },
        // Stem 27/28, 10 -> 12
        // Sphere 9
        Differential {
            from: "2 4 5 3 3 3[8]".to_string(),
            to: "2 4 1 1 2 4 3 3 3[4]".to_string(),
            coeff: 0,
            d: 4,
            proof: Some("Need to fix the algebraic convergence and delete an AF 8 element. There is only one AF 7 element which can do this.".to_string()),
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

        // Stem 24
        (
            "3 6 2 3 3[7]".to_string(),
            vec![(9, "4 5 3 3 3[6]".to_string())],
        ),
        (
            "2 4 3 3 3[9]".to_string(),
            vec![(10, "8 3 3 3[7]".to_string())],
        ),
        
        // Stem 25
        (
            "2 4 3 3 3[10]".to_string(),
            vec![(11, "5 7 3 3[7]".to_string())],
        ),
        
        // Stem 26
        (
            "5 5 7 3 3[3]".to_string(),
            vec![(6, "3 5 7 3 3[5]".to_string())],
        ),
        (
            "2 4 5 3 3 3[6]".to_string(),
            vec![(11, "3 5 7 3 3[5]".to_string())],
        ),
        
        // Stem 27
        (
            "6 2 3 4 4 1 1 1[5]".to_string(),
            vec![(6, "2 3 5 7 3 3[4]".to_string())],
        ),
        (
            "2 4 1 1 2 4 3 3 3[4]".to_string(),
            vec![(5, "2 2 3 5 7 3 3[2]".to_string())],
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
        TauMult {
            from: "6 5 3[8]".to_string(),
            to: "6 2 3 3[8]".to_string(),
        },

        // Stem 23
        TauMult {
            from: "4 1 1 2 4 3 3 3[2]".to_string(),
            to: "2 2 4 5 3 3 3[1]".to_string(),
        },
        
        // Stem 24
        TauMult {
            from: "1 1 2 4 3 3 3[8]".to_string(),
            to: "2 3 5 7 3 3[2]".to_string(),
        },
        TauMult {
            from: "6 5 3[10]".to_string(),
            to: "6 2 3 3[10]".to_string(),
        },

        // Stem 25        
        TauMult {
            from: "6 5 3[9]".to_string(),
            to: "13 3[7]".to_string(),
        },
        TauMult {
            from: "6 5 3[11]".to_string(),
            to: "11 7[7]".to_string(),
        },

        // Stem 26
        TauMult {
            from: "6 5 3[12]".to_string(),
            to: "6 2 3 3[12]".to_string(),
        },
        TauMult {
            from: "6 2 4 3 3 3[5]".to_string(),
            to: "7 6 2 3 3[5]".to_string(),
        },
        TauMult {
            from: "2 2 4 3 3 3[9]".to_string(),
            to: "3 6 2 3 3[9]".to_string(),
        },
        TauMult {
            from: "1 2 4 3 3 3[10]".to_string(),
            to: "3 6 2 3 3[9]".to_string(),
        },
        TauMult {
            from: "3 4 4 1 1 1[12]".to_string(),
            to: "12 1 1 1[11]".to_string(),
        },
        
        // Stem 27
        TauMult {
            from: "1 1 2 4 3 3 3[10]".to_string(),
            to: "6 2 3 4 4 1 1 1[5]".to_string(),
        },
        TauMult {
            from: "1 2 4 3 3 3[11]".to_string(),
            to: "3 5 7 3 3[6]".to_string(),
        },

    ]
}