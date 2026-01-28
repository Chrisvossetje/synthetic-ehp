use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::{
    MAX_STEM, MAX_UNEVEN_INPUT,
    naming::{generated_by_name, generating_name},
    processor::get_filtered_data,
    types::{Category, SyntheticEHP},
};

/// Verify data integrity
pub fn verify_integrity(data: &SyntheticEHP) -> bool {
    let mut names = HashSet::new();
    let mut is_valid = true;

    // Check for duplicate generator names
    for generator in &data.generators {
        if names.contains(&generator.name) {
            eprintln!("Duplicate generator name: {}", generator.name);
            is_valid = false;
        }
        names.insert(generator.name.clone());
    }

    // Verify differentials reference existing generators
    for diff in &data.differentials {
        if !names.contains(&diff.from) {
            eprintln!(
                "Differential references unknown 'from' generator: {}",
                diff.from
            );
            is_valid = false;
        }
        if !names.contains(&diff.to) {
            eprintln!(
                "Differential references unknown 'to' generator: {}",
                diff.to
            );
            is_valid = false;
        }

        if let (Some(from_gen), Some(to_gen)) = (data.find(&diff.from), data.find(&diff.to)) {
            if to_gen.adams_filtration != from_gen.adams_filtration + diff.coeff + 1 {
                eprintln!(
                    "Adams filtration does not coincide with expectations for diff: {} -> {}",
                    diff.from, diff.to
                );
            }

            if from_gen.y - to_gen.y != diff.d {
                eprintln!(
                    "d_r (from EHP filtration) is not correct for diff: {} -> {}",
                    diff.from, diff.to
                );
            }

            if from_gen.x - to_gen.x != 1 {
                eprintln!(
                    "x coordinate difference is not 1 for diff: {} -> {}",
                    diff.from, diff.to
                );
            }
        }
    }

    // Verify multiplications reference existing generators
    for mult in &data.multiplications {
        if !names.contains(&mult.from) {
            eprintln!(
                "Multiplication references unknown 'from' generator: {}",
                mult.from
            );
            is_valid = false;
        }
        if !names.contains(&mult.to) {
            eprintln!(
                "Multiplication references unknown 'to' generator: {}",
                mult.to
            );
            is_valid = false;
        }
    }

    is_valid
}

// Here i check if the E_\infty of SEHP coincides with the input to the SEHP
pub fn verify_self_coherence(data: &SyntheticEHP) -> bool {
    let mut is_valid = true;

    for y in 1..=1 {
        let sphere = y * 2 + 1;

        let row_gens: Vec<_> = data.generators.iter().filter(|g| g.y == y).collect();

        let (mut conv_gens, _) =
            get_filtered_data(data, Category::Synthetic, Some(sphere), 1000, true, None);

        let conv_gens: HashMap<_, _> = conv_gens.drain().filter(|x| data.find(&x.0).unwrap().x <= MAX_STEM - y - 1).filter(|x| if let Some(t) = x.1.0 { t != 0} else {true}).collect();

        let row_gen_translated: HashMap<_, _> = row_gens
            .iter()
            .map(|g| {
                let gen_name = generated_by_name(&g.name);
                (
                    data.find(&gen_name).unwrap().induced_name.clone(),
                    (g.torsion, g.adams_filtration - 1, g.name.clone()),
                )
            }).filter(|x| data.find(&x.0).unwrap().x <= MAX_STEM - 2)
            .filter(|x| if let Some(t) = x.1.0 { t != 0} else {true})
            .collect();

        // dbg!()

        for (r_name, (r_tor, r_af, r_original_name)) in &row_gen_translated {
            if !conv_gens.contains_key(r_name) {
                let orig_el = data.find(&r_name).unwrap();
                eprintln!(
                    "Synthetic row has an element not in the SS on sphere {sphere}. Expected: {r_name} | original_name: {r_original_name} | orig_xy: ({},{})",
                    orig_el.x, orig_el.y
                );
                is_valid = false;
                break;
            } else {
                let c_gen = conv_gens.get(r_name).unwrap();
                let c_y = data.find(r_name).unwrap().y;
                if c_gen != &(*r_tor, *r_af) {
                    eprintln!(
                        "Synthetic row has elements which do not coincide wrt. torsion/adams filtration on sphere {sphere} | y: {c_y}. Original_name: {r_original_name} | row_values: ({:?}, {}) | ss_values: ({:?}, {})",
                        r_tor, r_af, c_gen.0, c_gen.1
                    );
                    is_valid = false;
                    break;
                }
            }
        }

        for (c_name, _) in &conv_gens {
            if !row_gen_translated.contains_key(c_name) {
                let c_gen = data.find(&c_name).unwrap();
                eprintln!(
                    "Synthetic SS has an element not in the row on sphere {sphere}. Expected: {c_name} | xy:  ({},{})",
                    c_gen.x, c_gen.y
                );
                is_valid = false;
                break;
            }
        }
    }
    is_valid
}

pub fn verify_algebraic(data: &SyntheticEHP) -> bool {
    let mut is_valid = true;
    for sphere in (2..=MAX_STEM).into_iter().rev() {
        let (alg_gens, _) =
            get_filtered_data(data, Category::Algebraic, Some(sphere), 1000, true, None);
        let (synth_gens, _) =
            get_filtered_data(data, Category::Synthetic, Some(sphere), 1000, true, None);

        // (stem, adams filtration)
        let mut alg_map = HashMap::new();
        let mut synth_map = HashMap::new();

        let mut alg_names = HashSet::new();
        let mut synth_induced_names = HashSet::new();

        for g in alg_gens {
            if g.1.0.is_none() {
                let real_gen = data.find(&g.0).unwrap();
                if real_gen.x <= MAX_STEM {
                    continue;
                }
                alg_names.insert(real_gen.name.clone());
                *alg_map.entry((real_gen.x, g.1.1)).or_insert(0) += 1;
            }
        }

        for g in synth_gens {
            let real_gen = data.find(&g.0).unwrap();
            if real_gen.x <= MAX_STEM {
                continue;
            }
            let x = real_gen.x;
            if let Some(torsion) = g.1.0 {
                if torsion != 0 {
                    synth_induced_names.insert(real_gen.induced_name.clone());
                    *synth_map.entry((x, g.1.1)).or_insert(0) += 1;
                    *synth_map.entry((x + 1, g.1.1 - (torsion + 1))).or_insert(0) += 1;
                }
            } else {
                synth_induced_names.insert(real_gen.induced_name.clone());
                *synth_map.entry((x, g.1.1)).or_insert(0) += 1;
            }
        }

        for (k, val) in alg_map.iter().sorted_by_key(|k| k.0.0) {
            if !synth_map.contains_key(k) {
                eprintln!(
                    "Algebraic {sphere} sphere has {val} element(s) in stem {} | af {}, which synthetic does not",
                    k.0, k.1
                );
                is_valid = false;
                break;
            } else {
                let s_val = synth_map.get(k).unwrap();
                if val != s_val {
                    eprintln!(
                        "Algebraic {sphere} sphere has {val} element(s) in stem {} | af {}, while synthetic has {s_val}",
                        k.0, k.1
                    );
                    is_valid = false;
                    break;
                }
            }
        }

        for (k, val) in synth_map.iter().sorted_by_key(|k| k.0.0) {
            if !alg_map.contains_key(k) {
                eprintln!(
                    "Algebraic {sphere} sphere has 0 element(s) in stem {} | af {}, while synthetic has {val}",
                    k.0, k.1
                );
                is_valid = false;
                break;
            }
        }

        for n in synth_induced_names
            .iter()
            .sorted_by_key(|x| data.find(&x).unwrap().x)
        {
            if !alg_names.contains(n) {
                let g = data.find(n).unwrap();
                eprintln!(
                    "Synthetic {sphere} sphere has element with induced name {n} which is not in the algebraic sphere. Location ({},{}) on synthetic",
                    g.x, g.y
                );
                is_valid = false;
                break;
            }
        }
    }
    is_valid
}

/// Verify stable stems match expected values
pub fn verify_stable(data: &SyntheticEHP) -> bool {
    let stable_gens: Vec<i32> = vec![
        1, 1, 1, 3, 0, 0, 1, 4, 2, 3, 1, 3, 0, 0, 2, 6, 2, 4, 4, 4, 3, 2, 2, 8, 2, 2, 2, 3, 1, 0,
        1, 8, 4, 5, 5, 5,
    ];
    let alg_stable_gens: Vec<i32> = vec![
        1, 1, 1, 3, 0, 0, 1, 4, 2, 3, 1, 3, 0, 0, 5, 9, 3, 7, 6, 4, 3, 2, 4, 10, 3, 6, 5, 3, 3, 3,
        13, 22, 8, 10, 11, 7,
    ];

    let (gens, _) = get_filtered_data(data, Category::Synthetic, None, 1000, true, None);

    let mut count_gens = vec![0; (MAX_STEM + 1) as usize];
    let mut alg_count_gens = vec![0; (MAX_STEM + 1) as usize];

    let mut is_valid = true;

    for (name, (torsion, _filtration)) in gens {
        if let Some(real_gen) = data.find(&name) {
            let x = real_gen.x as usize;
            if x <= MAX_STEM as usize {
                if torsion.is_none() {
                    count_gens[x] += 1;
                    alg_count_gens[x] += 1;
                } else if let Some(t) = torsion {
                    if t > 0 {
                        alg_count_gens[x] += 1;
                        if x + 1 <= MAX_STEM as usize {
                            alg_count_gens[x + 1] += 1;
                        }
                    }
                }
            }
        }
    }

    for i in (0..=MAX_STEM as usize).rev() {
        if i < alg_stable_gens.len() && alg_count_gens[i] != alg_stable_gens[i] {
            eprintln!(
                "Algebraic stable stem in degree {} do not agree. We have {} and expect {}",
                i, alg_count_gens[i], alg_stable_gens[i]
            );
            is_valid = false;
        }
        if i < stable_gens.len() && count_gens[i] != stable_gens[i] {
            eprintln!(
                "Classical stable stem in degree {} do not agree. We have {} and expect {}",
                i, count_gens[i], stable_gens[i]
            );
            is_valid = false;
        }
    }

    is_valid
}
