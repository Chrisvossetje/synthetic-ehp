use crate::types::Differential;

pub fn get_diffs() -> Vec<Differential> {
    vec![
        Differential {
            from: "[15]".to_string(),
            to: "5 3[6]".to_string(),
            coeff: 1,
            d: 9,
            proof: None,
        },
        Differential {
            from: "1[14]".to_string(),
            to: "1 2 3 3[5]".to_string(),
            coeff: 2,
            d: 9,
            proof: None,
        },
        Differential {
            from: "1 1[13]".to_string(),
            to: "4 4 1 1 1[3]".to_string(),
            coeff: 2,
            d: 10,
            proof: None,
        },
        Differential {
            from: "3 3 3[8]".to_string(),
            to: "2 4 3 3 3[1]".to_string(),
            coeff: 1,
            d: 7,
            proof: None,
        },
        Differential {
            from: "5 3 3 3[4]".to_string(),
            to: "1 2 4 3 3 3[1]".to_string(),
            coeff: 1,
            d: 3,
            proof: None,
        },
        Differential {
            from: "7 3 3[5]".to_string(),
            to: "2 4 3 3 3[2]".to_string(),
            coeff: 1,
            d: 3,
            proof: None,
        },
        Differential {
            from: "7 3 3 3[4]".to_string(),
            to: "1 2 4 3 3 3[3]".to_string(),
            coeff: 1,
            d: 1,
            proof: None,
        },
    ]
}


pub fn get_induced_names() -> Vec<(String, String)> {
    // (Original name , induced name)
    vec![
        ("2 4 3 3 3[5]".to_string(), "7 3 3 3[4]".to_string()),
        ("3 6 2 3 3[3]".to_string(), "4 5 3 3 3[2]".to_string()),
        ("3 6 2 3 3[4]".to_string(), "5 7 3 3[3]".to_string()),
    ]
}
