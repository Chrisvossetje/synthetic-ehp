use std::collections::HashMap;

use crate::{MAX_UNEVEN_INPUT, MAX_STEM, types::{Differential, Generator, SyntheticSS}};

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
    generators.push(Generator::new("[0]".to_string(), 0, 0, 0));

    // Generate the degree zero parts
    for n in (3..MAX_UNEVEN_INPUT).step_by(4) {
        let y = n / 2;
        generators.push(Generator::new(format!("[{}]", y), y, y, 1));
    }

    // Stable generators
    for n in (3..MAX_UNEVEN_INPUT).step_by(2) {
        let y = n / 2;
        for unt in untagged {
            if unt.stem + y <= MAX_STEM {
                let initial: i32 = unt.tag.split_whitespace().next().unwrap().parse().unwrap();
                if initial < n {
                    generators.push(Generator::new(
                        format!("{}[{}]", unt.tag, y),
                        unt.stem + y,
                        y,
                        unt.filt + 1
                    ));
                }
            }
        }
    }

    // Unstable generators
    for n in (3..MAX_UNEVEN_INPUT).step_by(2) {
        let y = n / 2;
        for tag in tagged {
            if tag.stem + y <= MAX_STEM {
                let initial: i32 = tag.left_tag.split_whitespace().next().unwrap().parse().unwrap();
                let initial_tag: i32 = tag.right_tag.split_whitespace().next().unwrap().parse().unwrap();
                if initial < n && n <= initial_tag {
                    generators.push(Generator::new(
                        format!("{}[{}]", tag.left_tag, y),
                        tag.stem + y,
                        y,
                        tag.filt + 1
                    ));
                }
            }
        }
    }

    // Differentials
    for tag in tagged {
        if tag.stem <= MAX_STEM {
            let rspl: Vec<&str> = tag.right_tag.split_whitespace().collect();
            let f = format!("{}[{}]", rspl[1..].join(" "), rspl[0]);
            
            let lspl: Vec<&str> = tag.left_tag.split_whitespace().collect();
            let t = format!("{}[{}]", lspl[1..].join(" "), lspl[0]);

            let d_r: i32 = rspl[0].parse::<i32>().unwrap() - lspl[0].parse::<i32>().unwrap();

            differentials.push(Differential {
                from: f.clone(),
                to: t,
                coeff: 0,
                d: d_r,
                proof: Some("Lifted AEHP differential.".to_string()),
                synthetic: None,
                fake: false,
            });

            if tag.stem == MAX_STEM {
                let y: i32 = rspl[0].parse().unwrap();
                let af = tag.right_tag.split_whitespace().count() as i32;
                generators.push(Generator::new(f, tag.stem + 1, y, af));
            }
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
