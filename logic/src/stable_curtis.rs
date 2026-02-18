use std::collections::HashMap;

use crate::{MAX_STEM, curtis::{Tagged, Untagged}, types::{Differential, Generator, SyntheticSS}};

static STABLE_CURTIS_TXT: &str = include_str!("../../stable_curtis_table.txt");

fn parse_stable_curtis_table() -> (Vec<Untagged>, Vec<Tagged>) {
    let mut untagged = Vec::new();
    let mut tagged = Vec::new();

    let mut current_degree = (0,0);

    for line in STABLE_CURTIS_TXT.lines() {
        if line.len() == 0 {
            continue;
        }
        if &line[0..=0] != "(" {
            continue;
        }
        if line.contains(',') {
            let strip: Vec<_> = line.strip_prefix('(').unwrap().strip_suffix(')').unwrap().split(',').collect();
            let (l,r) = (strip[0], strip[1]);
            current_degree = (l.trim().parse().unwrap(), r.trim().parse().unwrap());
            continue;
        }

        if line.contains('←') {
            // Tagged
            let (left, right) = line.split_once('←').unwrap();

            tagged.push(Tagged {
                stem: current_degree.0,
                filt: current_degree.1,
                left_tag: left.trim().to_string(),
                right_tag: right.trim().to_string(),
            });            
        } else {
            // Untagged
            untagged.push(Untagged {
                stem: current_degree.0,
                filt: current_degree.1,
                tag: line.trim().to_string(),
            });            
        }

        // let line = line.trim_end_matches(")\n").trim_start_matches("((");
        // let parts: Vec<&str> = line.split('#').collect();

        // let stemfilt = parts[0].trim_end_matches(") ").split(' ').collect::<Vec<&str>>();
        // let stem: i32 = stemfilt[0].parse().unwrap();
        // let filt: i32 = stemfilt[1].parse().unwrap();

        // if parts.len() == 3 {
        //     // Tagged entry
        //     let ltag = parts[1].trim_start_matches('(').split(')').next().unwrap().to_string();
        //     let rtag = parts[2].trim_start_matches('(').split(')').next().unwrap().to_string();
        //     tagged.push(Tagged {
        //         stem,
        //         filt,
        //         left_tag: ltag,
        //         right_tag: rtag,
        //     });
        // } else {
        //     // Untagged entry
        //     let unt = parts[1].trim_start_matches('(').split(')').next().unwrap().to_string();
        //     untagged.push(Untagged {
        //         stem,
        //         filt,
        //         tag: unt,
        //     });
        // }
    }

    (untagged, tagged)
}


fn parse_stable_algebraic_data(
    untagged: &[Untagged],
    tagged: &[Tagged],
) -> SyntheticSS {
    let mut generators = Vec::new();
    let mut differentials = Vec::new();
    let multiplications = Vec::new();

    // Add initial generator
    // generators.push(Generator::new("[0]".to_string(), 0, 0, 0));

    // // Generate the degree zero parts
    // for n in (3..MAX_UNEVEN_INPUT).step_by(4) {
    //     let y = n / 2;
    //     generators.push(Generator::new(format!("[{}]", y), y, y, 1));
    // }

    // Stable generators
    // for n in (3..MAX_UNEVEN_INPUT).step_by(2) {
    //     let y = n / 2;
        for unt in untagged {
            if unt.stem <= MAX_STEM {
                // let initial: i32 = unt.tag.strip_prefix('(').unwrap().split_once(')').unwrap().0.parse().unwrap();
                let (first, second) = unt.tag.strip_prefix('(').unwrap().split_once(')').unwrap();
                let y: i32 = first.parse().unwrap();
                // if initial < y {
                generators.push(Generator::new(
                    format!("{}[{}]", second.trim(), first.trim()),
                    unt.stem,
                    y,
                    unt.filt
                ));
            }
            // }
        }
    // }

    // Unstable generators
    // for n in (3..MAX_UNEVEN_INPUT).step_by(2) {
    // let y = n / 2;
    for tag in tagged {
        if tag.stem <= MAX_STEM {
            let (first_l, second_l) = tag.left_tag.strip_prefix('(').unwrap().split_once(')').unwrap();
            let y: i32 = first_l.parse().unwrap();

            let to = format!("{}[{}]", second_l.trim(), first_l.trim());

            generators.push(Generator::new(
                to.clone(),
                tag.stem,
                y,
                tag.filt
            ));
            
            let (first_r, second_r) = tag.right_tag.strip_prefix('(').unwrap().split_once(')').unwrap();
            let y_2: i32 = first_r.parse().unwrap();

            let from = format!("{}[{}]", second_r.trim(), first_r.trim());

            generators.push(Generator::new(
                from.clone(),
                tag.stem + 1,
                y_2,
                tag.filt - 1
            ));

            differentials.push(Differential {
                from,
                to,
                coeff: 0,
                d: (y_2 - y),
                proof: Some("Lifted AEHP differential.".to_string()),
                synthetic: None,
            });

        }
    }
    // }

    // // Differentials
    // for tag in tagged {
    //     if tag.stem <= MAX_STEM {
    //         let rspl: Vec<&str> = tag.right_tag.split_whitespace().collect();
    //         let f = format!("{}[{}]", rspl[1..].join(" "), rspl[0]);
            
    //         let lspl: Vec<&str> = tag.left_tag.split_whitespace().collect();
    //         let t = format!("{}[{}]", lspl[1..].join(" "), lspl[0]);

    //         let d_r: i32 = rspl[0].parse::<i32>().unwrap() - lspl[0].parse::<i32>().unwrap();

    //         differentials.push(Differential {
    //             from: f.clone(),
    //             to: t,
    //             coeff: 0,
    //             d: d_r,
    //             proof: Some("Lifted AEHP differential.".to_string()),
    //             synthetic: None,
    //         });

    //         if tag.stem == MAX_STEM {
    //             let y: i32 = rspl[0].parse().unwrap();
    //             let af = tag.right_tag.split_whitespace().count() as i32;
    //             generators.push(Generator::new(f, tag.stem + 1, y, af));
    //         }
    //     }
    // }

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



pub fn generate_stable_algebraic_data() -> SyntheticSS {
    let (untagged, tagged) = parse_stable_curtis_table();


    let mut data = parse_stable_algebraic_data(&untagged, &tagged);
    
    // Note: Sorting on y is quite important !!!
    data.generators.sort_by_key(|x| x.y);
    data.generators.sort_by_key(|x| x.adams_filtration);
    data.differentials.sort();
    data.build_find_map();
    data
}