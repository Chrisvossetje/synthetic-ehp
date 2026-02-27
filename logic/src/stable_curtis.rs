use std::collections::HashMap;

use crate::{MAX_STEM, curtis::{Tagged, Untagged}, types::{Differential, Generator, Kind, SyntheticSS}};

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

    for unt in untagged {
        if unt.stem <= MAX_STEM {
            let (first, second) = unt.tag.strip_prefix('(').unwrap().split_once(')').unwrap();
            let y: i32 = first.parse().unwrap();

            generators.push(Generator::new(
                format!("{}[{}]", second.trim(), first.trim()),
                unt.stem,
                y,
                unt.filt,
                y + 1,
                None
            ));
        }
    }

    for tag in tagged {
        if tag.stem <= MAX_STEM {
            let (first_l, second_l) = tag.left_tag.strip_prefix('(').unwrap().split_once(')').unwrap();
            let y: i32 = first_l.parse().unwrap();

            let to = format!("{}[{}]", second_l.trim(), first_l.trim());

            let (first_r, second_r) = tag.right_tag.strip_prefix('(').unwrap().split_once(')').unwrap();
            let y_2: i32 = first_r.parse().unwrap();
            
            let from = format!("{}[{}]", second_r.trim(), first_r.trim());
            
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

            differentials.push(Differential {
                from,
                to,
                coeff: 0,
                d: (y_2 - y),
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