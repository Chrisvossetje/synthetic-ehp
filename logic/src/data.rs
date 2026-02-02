use crate::types::Differential;


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
        },
        // Stem 14/15, AF 2 -> 5
        Differential {
            from: "1[14]".to_string(),
            to: "1 2 3 3[5]".to_string(),
            coeff: 2,
            d: 9,
            proof: Some("Stable Diff".to_string()),
        },
        // Stem 14/15, AF 3 -> 6
        Differential {
            from: "1 1[13]".to_string(),
            to: "4 4 1 1 1[3]".to_string(),
            coeff: 2,
            d: 10,
            proof: Some("Stable Diff".to_string()),
        },
        // Stem 16/17, AF 4 -> 6
        Differential {
            from: "3 3 3[8]".to_string(),
            to: "2 4 3 3 3[1]".to_string(),
            coeff: 1,
            d: 7,
            proof: Some("Stable Diff".to_string()),
        },
        // Stem 17/18, AF 5 -> 7
        Differential {
            from: "5 3 3 3[4]".to_string(),
            to: "1 2 4 3 3 3[1]".to_string(),
            coeff: 1,
            d: 3,
            proof: Some("Stable Diff".to_string()),
        },
        // Stem 17/18, AF 4 -> 6
        Differential {
            from: "7 3 3[5]".to_string(),
            to: "2 4 3 3 3[2]".to_string(),
            coeff: 1,
            d: 3,
            proof: Some("Stable Diff".to_string()),
        },
        // Stem 22/23, AF 7 -> 9
        Differential {
            from: "1 2 4 1 1 1[13]".to_string(),
            to: "1 2 3 4 4 1 1 1[5]".to_string(),
            coeff: 1,
            d: 8,
            proof: Some("Stable Diff".to_string()),
        },
        // Stem 22/23, AF 8 -> 10
        Differential {
            from: "1 1 2 4 1 1 1[12]".to_string(),
            to: "4 4 1 1 2 4 1 1 1[3]".to_string(),
            coeff: 1,
            d: 9,
            proof: Some("Stable Diff".to_string()),
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
        },

        // Stem 21/22, AF 5 -> 7
        // Sphere 9
        Differential {
            from: "6 5 3[8]".to_string(),
            to: "5 1 2 3 3[7]".to_string(),
            coeff: 1,
            d: 1,
            proof: Some("If this diff does not exists, we would have an element with AF 3 on the algebraic 9 sphere.".to_string()),
        },

        // Stem 21/22, AF 5 -> 7
        // Sphere 9 -> 13
        Differential {
            from: "6 2 3 3[8]".to_string(),
            to: "2 4 3 3 3[6]".to_string(),
            coeff: 1,
            d: 2,
            proof: Some("Classical 9 - 13 Sphere needs a generator killed. This is the only viable differential.".to_string()),
        },



        // // Stem 22/23, AF 5 -> 7
        // // Sphere 9 -> ?
        // Differential {
        //     from: "2 4 3 3 3[8]".to_string(),
        //     to: "2 4 5 3 3 3[2]".to_string(),
        //     // to: "2 3 4 4 1 1 1[6]".to_string(),
        //     coeff: 1,
        //     d: 6,
        //     proof: Some("Classical 9 Sphere needed a generator killed. The only possible element to do is the source of this diff. The 9 and 10 AF element will be killed later by stable differentials. The 8 stem element can not be hit ?? This actually can. This leaves leaves the 7 stem as the only possible differential hit (note that on the E3 page, the AF filtration of the source is 5).
        //     THE ANSWER TO THIS DIFF COULD LIE IN THE ALGEBRAIC STRUCTURE OF THE 23'TH STEM.
        //     ".to_string()),
        // },


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

        // Maybe some names in some truncation of 21 ?
    ]
}
