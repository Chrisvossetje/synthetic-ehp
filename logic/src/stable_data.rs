use crate::{MAX_STEM, types::{Differential, SyntheticSS, TauMult}};


pub fn synthetic_stable_e1(data: &mut SyntheticSS) {
        // Add all Synthetic stable sphere data
    let a = vec![
        ("6 5 3", Some(1)), ("15", Some(0)), // Stem 14/15
        ("5 1 2 3 3", Some(2)), ("14 1", Some(0)), // Stem 14/15
        ("3 4 4 1 1 1", Some(2)), ("13 1 1", Some(0)), // Stem 14/15
        ("1 2 4 3 3 3", Some(1)), ("8 3 3 3", Some(0)), // Stem 16/17
        ("2 2 4 3 3 3", Some(1)), ("5 7 3 3", Some(0)), // Stem 17/18 TODO: RHS source not clear, could be 9 3 3 3, but this has probably a h_0 to 4 5 3 3 3.
        ("1 1 2 4 3 3 3", Some(1)), ("4 5 3 3 3", Some(0)), // Stem 17/18
        ("5 1 2 3 4 4 1 1 1", Some(1)), ("13 1 2 4 1 1 1", Some(0)), // Stem 22/23
        ("3 4 4 1 1 2 4 1 1 1", Some(1)), ("12 1 1 2 4 1 1 1", Some(0)), // Stem 22/23
        ("1 2 4 1 1 2 4 3 3 3", Some(1)), ("8 1 1 2 4 3 3 3", Some(0)), // Stem 24/25
        ("3 6 2 3 4 4 1 1 1", Some(1)), ("6 2 4 5 3 3 3", Some(0)), // Stem 25/26
        ("2 2 4 1 1 2 4 3 3 3", Some(1)), ("4 2 2 4 5 3 3 3", Some(0)), // Stem 25/26
        ("1 1 2 4 1 1 2 4 3 3 3", Some(1)), ("2 2 2 2 4 5 3 3 3", Some(0)), // Stem 25/26
        ("4 2 2 2 4 5 3 3 3", Some(1)), ("6 2 3 5 7 3 3", Some(0)), // Stem 28/29
        ("2 2 2 2 2 4 5 3 3 3", Some(1)), ("3 6 2 4 5 3 3 3", Some(0)), // Stem 28/29
        ("2 2 2 2 3 5 7 3 3", Some(2)), ("12 4 5 3 3 3", Some(0)), // Stem 29/30
    ];
        
    for i in a {
        for s in 1..MAX_STEM {
            let gen_name = format!("{}[{}]", i.0, s);
            if let Some(g) = data.find_mut(&gen_name) {
                g.torsion = i.1;
            }
        }
    }
}

pub fn get_stable_diffs() -> Vec<Differential> {
    vec![
        // Stem 10/11
        Differential {
            from: "2(∞)[11]".to_string(),
            to: "3 3 3[1]".to_string(),
            coeff: 1,
            d: 10,
            proof: Some("??".to_string()),
            synthetic: Some(()),
            fake: false,
        },


        // Stem 14/15
        Differential {
            from: "[15]".to_string(),
            to: "5 3[6]".to_string(),
            coeff: 1,
            d: 9,
            proof: Some("Stable Diff".to_string()),
            synthetic: Some(()),
            fake: false,
        },
        // Stem 14/15
        Differential {
            from: "1[14]".to_string(),
            to: "1 2 3 3[5]".to_string(),
            coeff: 2,
            d: 9,
            proof: Some("Stable Diff".to_string()),
            synthetic: Some(()),
            fake: false,
        },
        // Stem 14/15
        Differential {
            from: "1 1[13]".to_string(),
            to: "4 4 1 1 1[3]".to_string(),
            coeff: 2,
            d: 10,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },


        // Stem 16/17
        Differential {
            from: "3 3 3[8]".to_string(),
            to: "2 4 3 3 3[1]".to_string(),
            coeff: 1,
            d: 7,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },

        
        // Stem 17/18
        Differential {
            from: "6 2 3 3[4]".to_string(),
            to: "2 4 3 3 3[2]".to_string(),
            coeff: 2,
            d: 2,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },
        // Stem 17/18
        Differential {
            from: "6 5 3[4]".to_string(),
            to: "2 4 3 3 3[2]".to_string(),
            coeff: 1,
            d: 2,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },
        // Stem 17/18
        Differential {
            from: "2 4 3 3 3[4]".to_string(),
            to: "3 6 2 3 3[1]".to_string(),
            coeff: 0,
            d: 3,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },
        // Stem 17/18
        Differential {
            from: "2 4 1 1 1[10]".to_string(),
            to: "12 1 1 1[3]".to_string(),
            coeff: 0,
            d: 7,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },


        // Stem 18/19
        Differential {
            from: "12 1 1 1[5]".to_string(),
            to: "2 3 4 4 1 1 1[3]".to_string(),
            coeff: 2,
            d: 2,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },
        // Stem 18/19
        Differential {
            from: "7 7[6]".to_string(),
            to: "13 3[3]".to_string(),
            coeff: 0,
            d: 3,
            proof: Some("Its not clear which of the two possible AF 3 targets this will hit. But this one is the most logical? It is also NOT relevant.".to_string()),
            synthetic: Some(()),
            fake: false,
        },
        
        
        // Stem 21/22
        Differential {
            from: "6 2 3 3[8]".to_string(),
            to: "2 4 3 3 3[6]".to_string(),
            coeff: 2,
            d: 2,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },
        // Stem 21/22
        Differential {
            from: "6 5 3[8]".to_string(),
            to: "2 4 3 3 3[6]".to_string(),
            coeff: 1,
            d: 2,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },

                
        // Stem 22/23
        Differential {
            from: "2 4 3 3 3[8]".to_string(),
            to: "3 6 2 3 3[5]".to_string(),
            coeff: 0,
            d: 3,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },


        // Stem 22/23
        Differential {
            from: "1 1 2 4 1 1 1[12]".to_string(),
            to: "4 4 1 1 2 4 1 1 1[3]".to_string(),
            coeff: 1,
            d: 9,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },

        // Stem 22/23
        Differential {
            from: "1 2 4 1 1 1[13]".to_string(),
            to: "1 2 3 4 4 1 1 1[5]".to_string(),
            coeff: 1,
            d: 8,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },
        // Stem 22/23
        Differential {
            from: "4 1 1 1[16]".to_string(),
            to: "3 4 4 1 1 1[8]".to_string(),
            coeff: 0,
            d: 8,
            proof: Some("This must be it as there is no room for this stable diff elsewhere on EHP".to_string()),
            synthetic: Some(()),
            fake: false,
        },
        // Stem 22/23
        Differential {
            from: "4 1 1 1[16]".to_string(),
            to: "12 1 1 1[7]".to_string(),
            coeff: 0,
            d: 9,
            proof: Some("This must be it as there is no room for this stable diff elsewhere on EHP".to_string()),
            synthetic: Some(()),
            fake: false,
        },
        // Stem 22/23
        Differential {
            from: "1 1 1[20]".to_string(),
            to: "4 4 1 1 1[11]".to_string(),
            coeff: 0,
            d: 9,
            proof: Some("Unsure about this target. Highly likely".to_string()),
            synthetic: Some(()),
            fake: false,
        },
        // Stem 22/23
        Differential {
            from: "1 1[21]".to_string(),
            to: "1 2 3 3[13]".to_string(),
            coeff: 0,
            d: 8,
            proof: None,
            synthetic: Some(()),
            fake: false,
        },


    ]
}

pub fn get_stable_tau_mults() -> Vec<TauMult> {
    vec![
        // Stem 16
        TauMult {
            from: "3 4 4 1 1 1[2]".to_string(),
            to: "12 1 1 1[1]".to_string(),
        },
        TauMult {
            from: "6 5 3[2]".to_string(),
            to: "6 2 3 3[2]".to_string(),
        },

        // Stem 18
        TauMult {
            from: "1 2 4 3 3 3[2]".to_string(),
            to: "3 6 2 3 3[1]".to_string(),
        },
        TauMult {
            from: "3 4 4 1 1 1[4]".to_string(),
            to: "12 1 1 1[3]".to_string(),
        },
        
        // Stem 19
        TauMult {
            from: "6 5 3[5]".to_string(),
            to: "13 3[3]".to_string(),
        },
        
        // Stem 20
        TauMult {
            from: "6 5 3[6]".to_string(),
            to: "6 2 3 3[6]".to_string(),
        },

        // Stem 22
        TauMult {
            from: "1 2 4 3 3 3[6]".to_string(),
            to: "3 6 2 3 3[5]".to_string(),
        },
        TauMult {
            from: "3 4 4 1 1 1[8]".to_string(),
            to: "12 1 1 1[7]".to_string(),
        },




        // // Stem 22
        // TauMult {
        //     from: "1 2 4 3 3 3[6]".to_string(),
        //     to: "3 6 2 3 3[5]".to_string(),
        // },
        // TauMult {
        //     from: "3 4 4 1 1 1[8]".to_string(),
        //     to: "12 1 1 1[7]".to_string(),
        // },
        // TauMult {
        //     from: "6 5 3[8]".to_string(),
        //     to: "6 2 3 3[8]".to_string(),
        // },

        // // Stem 23
        // TauMult {
        //     from: "4 1 1 2 4 3 3 3[2]".to_string(),
        //     to: "2 2 4 5 3 3 3[1]".to_string(),
        // },
        
        // // Stem 24
        // TauMult {
        //     from: "1 1 2 4 3 3 3[8]".to_string(),
        //     to: "2 3 5 7 3 3[2]".to_string(),
        // },
        // TauMult {
        //     from: "6 5 3[10]".to_string(),
        //     to: "6 2 3 3[10]".to_string(),
        // },

        // // Stem 25        
        // TauMult {
        //     from: "6 5 3[9]".to_string(),
        //     to: "13 3[7]".to_string(),
        // },
        // TauMult {
        //     from: "6 5 3[11]".to_string(),
        //     to: "11 7[7]".to_string(),
        // },

        // // Stem 26
        // TauMult {
        //     from: "6 5 3[12]".to_string(),
        //     to: "6 2 3 3[12]".to_string(),
        // },
        // TauMult {
        //     from: "6 2 4 3 3 3[5]".to_string(),
        //     to: "5 5 7 3 3[3]".to_string(),
        // },
        // TauMult {
        //     from: "2 2 4 3 3 3[9]".to_string(),
        //     to: "2 4 5 3 3 3[6]".to_string(),
        // },
        // TauMult {
        //     from: "1 2 4 3 3 3[10]".to_string(),
        //     to: "3 6 2 3 3[9]".to_string(),
        // },
        // TauMult {
        //     from: "3 4 4 1 1 1[12]".to_string(),
        //     to: "12 1 1 1[11]".to_string(),
        // },
        
        // // Stem 27
        // TauMult {
        //     from: "1 1 2 4 3 3 3[10]".to_string(),
        //     to: "6 2 3 4 4 1 1 1[5]".to_string(),
        // },
        // TauMult {
        //     from: "1 2 4 3 3 3[11]".to_string(),
        //     to: "3 5 7 3 3[6]".to_string(),
        // },
        // TauMult {
        //     from: "6 5 3[13]".to_string(),
        //     to: "13 3[11]".to_string(),
        // },
        // TauMult {
        //     from: "6 5 3[14]".to_string(),
        //     to: "6 2 3 3[14]".to_string(),
        // },
        
        // // Stem 28
        // TauMult {
        //     from: "2 2 4 3 3 3[11]".to_string(),
        //     to: "3 6 6 5 3[5]".to_string(),
        // },
        
        // // Stem 29
        // TauMult {
        //     from: "1 2 4 3 3 3[13]".to_string(),
        //     to: "2 3 5 7 7[5]".to_string(),
        // },
        
        
        
        // // Stem 30
        // TauMult {
        //     from: "2 2 2 2 2 4 5 3 3 3[2]".to_string(),
        //     to: "2 2 2 2 3 5 7 3 3[1]".to_string(),
        // },
        // TauMult {
        //     from: "1 2 4 1 1 2 4 3 3 3[6]".to_string(),
        //     to: "3 6 2 3 4 4 1 1 1[5]".to_string(),
        // },
        // TauMult {
        //     from: "1 2 4 3 3 3[14]".to_string(),
        //     to: "2 3 4 4 1 1 1[14]".to_string(),
        // },
    ]
}