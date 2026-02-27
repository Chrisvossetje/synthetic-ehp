use std::{char::MAX, collections::HashMap};

use crate::{MAX_STEM, MAX_UNEVEN_INPUT, types::{Differential, Generator, Kind, SyntheticSS}};

static CURTIS_TXT: &str = include_str!("../../curtis_table.txt");

#[derive(Debug, Clone)]
pub struct Untagged {
    pub stem: i32,
    pub filt: i32,
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct Tagged {
    pub stem: i32,
    pub filt: i32,
    pub left_tag: String,
    pub right_tag: String,
}

fn parse_curtis_table() -> (Vec<Untagged>, Vec<Tagged>) {
    let mut untagged = Vec::new();
    let mut tagged = Vec::new();

    for line in CURTIS_TXT.lines() {
        let line = line.trim_end_matches(")\n").trim_start_matches("((");
        let parts: Vec<&str> = line.split('#').collect();

        let stemfilt = parts[0].trim_end_matches(") ").split(' ').collect::<Vec<&str>>();
        let stem: i32 = stemfilt[0].parse().unwrap();
        let filt: i32 = stemfilt[1].parse().unwrap();

        if parts.len() == 3 {
            // Tagged entry
            let ltag = parts[1].trim_start_matches('(').split(')').next().unwrap().to_string();
            let rtag = parts[2].trim_start_matches('(').split(')').next().unwrap().to_string();
            tagged.push(Tagged {
                stem,
                filt,
                left_tag: ltag,
                right_tag: rtag,
            });
        } else {
            // Untagged entry
            let unt = parts[1].trim_start_matches('(').split(')').next().unwrap().to_string();
            untagged.push(Untagged {
                stem,
                filt,
                tag: unt,
            });
        }
    }

    (untagged, tagged)
}


fn parse_algebraic_data(
    untagged: &[Untagged],
    tagged: &[Tagged],
) -> SyntheticSS {
    let mut generators = Vec::new();
    let mut differentials = Vec::new();
    let multiplications = Vec::new();

    // Add initial generator
    generators.push(Generator::new("[0]".to_string(), 0, 0, 0, 1, None));

    
    // Stable generators
    for unt in untagged {
        if unt.stem <= MAX_STEM {
            let (y, rest) = match unt.tag.split_once(" ") {
                Some((first, second)) => {
                    let initial: i32 = first.parse().unwrap();
                    (initial, second)
                },
                None => {
                    let initial: i32 = unt.tag.parse().unwrap();
                    (initial, "")
                },
            };

            generators.push(Generator::new(
                format!("{}[{}]", rest, y),
                unt.stem,
                y,
                unt.filt,
                y + 1,
                None
            ));
        }
    }

    // Differentials
    for tag in tagged {
        if tag.stem <= MAX_STEM {
            let (y, rest) = match tag.left_tag.split_once(" ") {
                Some((first, second)) => {
                    let initial: i32 = first.parse().unwrap();
                    (initial, second)
                },
                None => {
                    let initial: i32 = tag.left_tag.parse().unwrap();
                    (initial, "")
                },
            };
            let (y_2, rest_2) = match tag.right_tag.split_once(" ") {
                Some((first, second)) => {
                    let initial: i32 = first.parse().unwrap();
                    (initial, second)
                },
                None => {
                    let initial: i32 = tag.right_tag.parse().unwrap();
                    (initial, "")
                },
            };

            let to = format!("{}[{}]", rest, y);
            let from = format!("{}[{}]", rest_2, y_2);
    
            generators.push(Generator::new(
                to.clone(),
                tag.stem,
                y,
                tag.filt,
                y + 1,
                Some(y_2 + 1)
            ));
                
            generators.push(Generator::new(
                from.clone(),
                tag.stem + 1,
                y_2,
                tag.filt - 1,
                y_2 + 1,
                Some(y_2 + 1)
            ));
    
            let d_r: i32 = y_2 - y;
    
            differentials.push(Differential {
                from,
                to,
                coeff: 0,
                d: d_r,
                proof: Some("Lifted AEHP differential.".to_string()),
                synthetic: None,
                kind: Kind::Real,
            });
        }
    }

    let mut data = SyntheticSS {
        generators,
        differentials,
        multiplications,
        tau_mults: vec![],
        find_map: HashMap::new(),
    };

    data.build_find_map();
    data
}

pub fn generate_algebraic_data() -> SyntheticSS {
    let (untagged, tagged) = parse_curtis_table();
    let mut data = parse_algebraic_data(&untagged, &tagged);
    
    // Note: Sorting on y is quite important !!!
    data.generators.sort_by_key(|x| x.y);
    data.generators.sort_by_key(|x| x.adams_filtration);
    data.differentials.sort();
    data.build_find_map();
    data
}
