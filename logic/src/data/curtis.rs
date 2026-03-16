use std::collections::{HashMap, HashSet};

use crate::{MAX_STEM, domain::{e1::E1, model::{Diff, SyntheticSS}}, io::export::Differential, data::naming::{name_get_tag, name_to_sphere}, types::{Generator, Kind}};

static CURTIS_TXT: &str = include_str!("../../../curtis_table.txt");
static STABLE_CURTIS_TXT: &str = include_str!("../../../stable_curtis_table.txt");

#[derive(Debug, Clone)]
pub struct Untagged {
    pub stem: i32,
    pub filt: i32,
    pub tag: String,
    pub origin: i32,
}

#[derive(Debug, Clone)]
pub struct Tagged {
    pub stem: i32,
    pub filt: i32,
    pub left_tag: String,
    pub left_origin: i32,
    pub right_tag: String,
    pub right_origin: i32,
}

fn name_to_tag_origin(name: &str) -> (i32, String) {
    match name.split_once(" ") {
        Some((first, second)) => {
            let initial: i32 = first.parse().unwrap();
            (initial, second.to_string())
        },
        None => {
            let initial: i32 = name.parse().unwrap();
            (initial, "".to_string())
        },
    }
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
            let lname = parts[1].trim_start_matches('(').split(')').next().unwrap().to_string();
            let (left_origin, left_tag) = name_to_tag_origin(&lname);
            
            let rname = parts[2].trim_start_matches('(').split(')').next().unwrap().to_string();
            let (right_origin, right_tag) = name_to_tag_origin(&rname);
            
            tagged.push(Tagged {
                stem,
                filt,
                left_tag,
                left_origin,
                right_tag,
                right_origin,
            });
        } else {
            // Untagged entry
            let unt = parts[1].trim_start_matches('(').split(')').next().unwrap().to_string();
            let (origin, tag) = name_to_tag_origin(&unt);
            untagged.push(Untagged {
                stem,
                filt,
                tag,
                origin,
            });
        }
    }

    (untagged, tagged)
}



fn stable_name_to_tag_origin(name: &str) -> (i32, String) {
    let (first, second) = name.strip_prefix('(').unwrap().split_once(')').unwrap();
    let y: i32 = first.parse().unwrap();
    (y, second.trim().to_string())
}

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
            let (left_origin, left_tag) = stable_name_to_tag_origin(left.trim());
            let (right_origin, right_tag) = stable_name_to_tag_origin(right.trim());

            tagged.push(Tagged {
                stem: current_degree.0,
                filt: current_degree.1,
                left_tag,
                left_origin,
                right_tag,
                right_origin,

            });            
        } else {
            // Untagged
            let (origin, tag) = stable_name_to_tag_origin(line.trim());
            untagged.push(Untagged {
                stem: current_degree.0,
                filt: current_degree.1,
                tag,
                origin
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

    // Add initial generator
    generators.push(Generator::new("[0]".to_string(), 0, 0, 0, 1, None));

    
    // Stable generators
    for unt in untagged {
        if unt.stem <= MAX_STEM {
            generators.push(Generator::new(
                format!("{}[{}]", unt.tag, unt.origin),
                unt.stem,
                unt.origin,
                unt.filt,
                unt.origin + 1,
                None,
            ));
        }
    }

    // Differentials
    for tag in tagged {
        if tag.stem <= MAX_STEM {
            let to = format!("{}[{}]", tag.left_tag, tag.left_origin);
            let from = format!("{}[{}]", tag.right_tag, tag.right_origin);
            generators.push(Generator::new(
                to.clone(),
                tag.stem,
                tag.left_origin,
                tag.filt,
                tag.left_origin + 1,
                Some(tag.right_origin + 1),
            ));
                
            generators.push(Generator::new(
                from.clone(),
                tag.stem + 1,
                tag.right_origin,
                tag.filt - 1,
                tag.right_origin + 1,
                Some(tag.right_origin + 1),
            ));
    
            let d_r: i32 = tag.right_origin - tag.left_origin;
    
            // TODO: This is weird
            differentials.push(Differential {
                from,
                to,
                coeff: 0,
                d: d_r,
                proof: None,
                kind: Kind::Real,
            });
        }
    }

    generators.sort_by_key(|x| x.af);
    generators.sort_by_key(|x| x.y);
    generators.sort_by_key(|x| x.stem);

    let mut diffs_page = vec![vec![]; (MAX_STEM + 1) as usize];
    let internal_tau_page = vec![vec![]; (MAX_STEM + 1) as usize];
    let external_tau_page = vec![];
    let mut proven_from_to = HashMap::new();

    let model = E1::new(generators);


    for d in differentials {
        let from = model.get_index(&d.from);
        let to = model.get_index(&d.to);
        diffs_page[d.d as usize].push(Diff { from, to });
        proven_from_to.insert((from, to), None);
    }

    let in_diffs = proven_from_to.iter().map(|x| (x.0.1, x.0.0)).collect();

    let data = SyntheticSS {
        model,
        diffs_page,
        internal_tau_page,
        external_tau_page,
        proven_from_to,
        disproven_from_to: HashMap::default(),
        in_diffs,
        out_ext_tau: HashMap::default(),
        temp_fakes: HashSet::default(),
    };

    data
}

pub fn generate_algebraic_data() -> SyntheticSS {
    let (untagged, tagged) = parse_curtis_table();
    parse_algebraic_data(&untagged, &tagged)
}

pub fn generate_stable_algebraic_data() -> SyntheticSS {
    let (untagged, tagged) = parse_stable_curtis_table();
    parse_algebraic_data(&untagged, &tagged)
}