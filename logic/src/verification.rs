use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::{
    MAX_VERIFY_SPHERE, MAX_VERIFY_STEM, naming::generated_by_name, processor::get_filtered_data, types::{Category, SyntheticEHP}
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
            if from_gen.y - to_gen.y != diff.d {
                eprintln!(
                    "d_r (from EHP filtration) is not correct for diff: {} -> {}",
                    diff.from, diff.to
                );
                is_valid = false;
            }

            if from_gen.x - to_gen.x != 1 {
                eprintln!(
                    "x coordinate difference is not 1 for diff: {} -> {}",
                    diff.from, diff.to
                );
                is_valid = false;
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

    for y in 1..=MAX_VERIFY_STEM {
        let sphere = y * 2 + 1;

        let row_gens: Vec<_> = data
            .generators
            .iter()
            .filter(|g| g.y == y && g.x <= MAX_VERIFY_STEM && g.x != y)
            .collect();

        let mut conv_gens =
            get_filtered_data(data, Category::Synthetic, Some(sphere), 1000, true, None);

        let conv_gens: HashMap<_, _> = conv_gens
            .drain()
            .filter(|x| {
                let gx = data.find(&x.0).unwrap().x;
                0 < gx && gx <= MAX_VERIFY_STEM - y
            })
            .filter(|x| if let Some(t) = x.1.0 { t != 0 } else { true })
            .map(|x| (data.find(&x.0).unwrap().induced_name.clone(), x.1))
            .collect();

        let row_gen_translated: HashMap<_, _> = row_gens
            .iter()
            .map(|g| {
                (
                    generated_by_name(&g.name),
                    (g.torsion, g.adams_filtration - 1, g.name.clone()),
                )
            })
            .filter(|x| if let Some(t) = x.1.0 { t != 0 } else { true })
            .collect();

        for (r_name, (r_tor, r_af, r_original_name)) in &row_gen_translated {
            if !conv_gens.contains_key(r_name) {
                let orig_el = data.find(&r_original_name).unwrap();
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
    for sphere in (1..=MAX_VERIFY_SPHERE).into_iter().rev() {
        let alg_gens =
            get_filtered_data(data, Category::Algebraic, Some(sphere), 1000, true, None);
        let synth_gens =
            get_filtered_data(data, Category::Synthetic, Some(sphere), 1000, true, None);

        // (stem, adams filtration)
        let mut alg_map = HashMap::new();
        let mut synth_map = HashMap::new();

        let mut alg_names = HashSet::new();
        let mut synth_induced_names = HashSet::new();

        for g in alg_gens {
            if g.1.0.is_none() {
                let real_gen = data.find(&g.0).unwrap();
                if real_gen.x > MAX_VERIFY_STEM {
                    continue;
                }
                alg_names.insert(real_gen.name.clone());
                *alg_map.entry((real_gen.x, g.1.1)).or_insert(0) += 1;
            }
        }

        for g in synth_gens {
            let real_gen = data.find(&g.0).unwrap();
            if real_gen.x > MAX_VERIFY_STEM {
                continue;
            }
            let x = real_gen.x;
            if let Some(torsion) = g.1.0 {
                if torsion != 0 {
                    synth_induced_names.insert(real_gen.induced_name.clone());
                    *synth_map.entry((x, g.1.1)).or_insert(0) += 1;
                    if x + 1 <= MAX_VERIFY_STEM {
                        *synth_map.entry((x + 1, g.1.1 - (torsion + 1))).or_insert(0) += 1;
                    }
                }
            } else {
                synth_induced_names.insert(real_gen.induced_name.clone());
                *synth_map.entry((x, g.1.1)).or_insert(0) += 1;
            }
        }

        let alg_map_fold = alg_map.iter().fold(0, |c, x| c + *x.1);
        let synth_map_fold = synth_map.iter().fold(0, |c, x| c + *x.1);
        
        if alg_map_fold != synth_map_fold {
            eprintln!(
                "Algebraic {sphere} sphere different amount {alg_map_fold} of algebraic elements than the synthetic {synth_map_fold}",
            );
            is_valid = false;
        }

        for (k, val) in alg_map.iter().sorted_by_key(|k| k.0.0) {
            if !synth_map.contains_key(k) {
                eprintln!(
                    "Algebraic {sphere} sphere has {val} element(s) in stem {} | af {}, which synthetic does not",
                    k.0, k.1
                );
                is_valid = false;
            } else {
                let s_val = synth_map.get(k).unwrap();
                if val != s_val {
                    eprintln!(
                        "Algebraic {sphere} sphere has {val} element(s) in stem {} | af {}, while synthetic has {s_val}",
                        k.0, k.1
                    );
                    is_valid = false;
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

    let gens = get_filtered_data(data, Category::Synthetic, None, 1000, true, None);

    let mut count_gens = vec![0; (MAX_VERIFY_STEM + 1) as usize];
    let mut alg_count_gens = vec![0; (MAX_VERIFY_STEM + 1) as usize];

    let mut is_valid = true;

    for (name, (torsion, _filtration)) in gens {
        if let Some(real_gen) = data.find(&name) {
            let x = real_gen.x as usize;
            if x <= MAX_VERIFY_STEM as usize {
                if torsion.is_none() {
                    count_gens[x] += 1;
                    alg_count_gens[x] += 1;
                } else if let Some(t) = torsion {
                    if t > 0 {
                        alg_count_gens[x] += 1;
                        if x + 1 <= MAX_VERIFY_STEM as usize {
                            alg_count_gens[x + 1] += 1;
                        }
                    }
                }
            }
        }
    }

    for i in (0..=MAX_VERIFY_STEM as usize).rev() {
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

/// Verify stable stems match expected values
pub fn verify_classical(data: &SyntheticEHP) -> bool {
    let mut is_valid = true;

    const CLASSICAL_MAX_SPHERE: i32 = 32;
    const CLASSICAL_MAX_STEM: i32 = 30;

    // Row n = pi_n+*(S^*)
    // Col k = pi_*+k(S^k)
    let classical_gens: [[i32; CLASSICAL_MAX_SPHERE as usize]; (CLASSICAL_MAX_STEM + 1) as usize] = [[2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], [1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], [1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], [1, 2, 12, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24], [1, 12, 2, 4, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], [1, 2, 2, 4, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], [1, 2, 3, 72, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], [1, 3, 15, 15, 30, 60, 120, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240, 240], [1, 15, 2, 2, 2, 48, 8, 16, 8, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4], [1, 2, 4, 8, 8, 8, 16, 32, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8], [1, 4, 24, 2880, 144, 144, 48, 1152, 48, 24, 12, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6], [1, 24, 336, 2688, 2016, 2016, 1008, 1008, 1008, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504, 504], [1, 336, 4, 64, 8, 240, 1, 1, 1, 12, 2, 4, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], [1, 4, 6, 288, 12, 6, 6, 12, 6, 6, 12, 12, 6, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3], [1, 6, 30, 30240, 12, 24, 96, 23040, 64, 32, 32, 384, 32, 16, 8, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4], [1, 30, 30, 30, 60, 360, 960, 3840, 1920, 960, 480, 480, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960, 960], [1, 30, 12, 72, 4, 2016, 16, 128, 16, 480, 2, 2, 2, 48, 8, 16, 8, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4], [1, 12, 48, 4608, 16, 16, 16, 96, 16, 8, 8, 16, 16, 16, 32, 64, 32, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16], [1, 48, 48, 46080, 96, 288, 48, 24192, 48, 96, 64, 15360, 128, 128, 128, 3072, 128, 64, 32, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16], [1, 48, 264, 4224, 528, 8448, 528, 528, 528, 1584, 2112, 8448, 2112, 2112, 1056, 1056, 1056, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528, 528], [1, 264, 4, 64, 24, 5760, 24, 72, 24, 12096, 96, 768, 192, 5760, 24, 24, 24, 288, 48, 96, 48, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24], [1, 4, 2, 32, 4, 2, 4, 32, 8, 8, 16, 32, 32, 16, 8, 16, 8, 8, 16, 16, 8, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4], [1, 2, 2, 32, 8, 32, 64, 4096, 128, 64, 64, 512, 64, 128, 128, 2048, 128, 64, 64, 256, 32, 16, 8, 4, 4, 4, 4, 4, 4, 4, 4, 4], [1, 2, 4, 32, 32, 2048, 1024, 8192, 2048, 2048, 512, 512, 512, 512, 2048, 8192, 2048, 1024, 512, 512, 512, 256, 256, 256, 256, 256, 256, 256, 256, 256, 256, 256], [1, 4, 8, 32, 16, 512, 128, 4096, 128, 1024, 32, 16, 16, 128, 64, 512, 64, 128, 8, 4, 4, 16, 8, 16, 8, 4, 4, 4, 4, 4, 4, 4], [1, 8, 32, 2048, 128, 1024, 256, 32768, 256, 64, 64, 128, 32, 32, 64, 128, 64, 16, 8, 4, 4, 4, 8, 16, 8, 4, 4, 4, 4, 4, 4, 4], [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]];

    for sphere in (1..=MAX_VERIFY_SPHERE).into_iter().rev() {
        let gens =
            get_filtered_data(data, Category::Classical, Some(sphere), 1000, true, None);

        let mut conv_gens = vec![0; (CLASSICAL_MAX_STEM + 1) as usize];

        for g in gens {
            if g.1.0.is_none() {
                let real_g = data.find(&g.0).unwrap();
                if real_g.x <= CLASSICAL_MAX_STEM {
                    conv_gens[real_g.x as usize] += 1;
                }
            }
        }

        let classical_order: Vec<_> = (0..=CLASSICAL_MAX_STEM)
            .into_iter()
            .map(|x| classical_gens[x as usize][(sphere - 1) as usize].trailing_zeros())
            .collect();

        for a in 0..=MAX_VERIFY_STEM {
            if classical_order[a as usize] != conv_gens[a as usize] {
                eprintln!(
                    "Classical homotopy groups on the {sphere} Sphere do not agree on stem {a}. Expect: {}, Got: {}",
                    classical_order[a as usize], conv_gens[a as usize]
                );
                is_valid = false;
            }
        }
    }

    let gens = get_filtered_data(data, Category::Classical, None, 1, true, None);
    for y in 1..=((CLASSICAL_MAX_SPHERE - 1) / 2) {
        let sphere = y * 2 + 1;

        let filtrd: Vec<_> = gens
            .iter()
            .filter(|p| data.find(&p.0).unwrap().y == y)
            .collect();

        let mut classical_order: Vec<_> = (0..=(CLASSICAL_MAX_STEM - y))
            .into_iter()
            .map(|x| classical_gens[x as usize][(sphere - 1) as usize].trailing_zeros())
            .collect();
        classical_order[0] = 0;

        let mut conv_gens = vec![0; (CLASSICAL_MAX_STEM + 1) as usize];

        for g in filtrd {
            if g.1.0.is_none() {
                let real_g = data.find(&g.0).unwrap();
                if real_g.x != y && real_g.x <= CLASSICAL_MAX_STEM {
                    conv_gens[(real_g.x - y) as usize] += 1;
                }
            }
        }

        for a in 0..=(CLASSICAL_MAX_STEM - y) {
            if classical_order[a as usize] != conv_gens[a as usize] && a + y <= MAX_VERIFY_STEM {
                eprintln!(
                    "Classical homotopy groups on the ROW of {sphere} Sphere do not agree on stem {a} | {}. Expect: {}, Got: {}",
                    a + y,
                    classical_order[a as usize],
                    conv_gens[a as usize]
                );
                is_valid = false;
            }
        }
    }

    is_valid
}
