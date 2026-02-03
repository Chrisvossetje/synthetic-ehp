use crate::data::{get_diffs, get_induced_names, get_tau_mults};
use crate::{MAX_STEM, MAX_VERIFY_STEM};
use crate::types::{Category, Differential, SyntheticEHP};
use crate::naming::{generating_name};
use std::collections::{HashMap};
use itertools::Itertools;



/// Add differentials to the data
pub fn add_diffs(data: &mut SyntheticEHP) {
    for d in get_diffs() {
        
        data.insert_diff(d);
    }
}

/// Add unstable differentials to the data
pub fn add_induced_names(data: &mut SyntheticEHP) {
    for (g, induced_name) in get_induced_names() {
        if let Some(g) = data.find_mut(&g) {
            g.induced_name = induced_name[0].1.clone();
        }
    }
}

pub fn add_tau_mults(data: &mut SyntheticEHP) {
    data.tau_mults = get_tau_mults();
}

/// Compute inductive generators
pub fn compute_inductive_generators(data: &mut SyntheticEHP) {
    for x in 3..MAX_STEM {
        for y in 1..=MAX_STEM {
            let sphere = y * 2 + 1;
            let gens = get_filtered_data(data, Category::Synthetic, Some(sphere), 1000, false, Some(x));

            for (name, (torsion, filtration)) in gens.iter().sorted_by_key(|x|  data.find(&x.0).unwrap().y) {
                if let Some(g) = data.find(&name).cloned() {
                    if g.x == x {
                        let target_name = format!("{}[{}]", generating_name(&g.induced_name), y);
                        if let Some(g_target) = data.find_mut(&target_name) {
                            g_target.torsion = *torsion;
                            g_target.adams_filtration = filtration + 1;
                        }
                    }
                }
            }
        }
    }
}

/// Get filtered data for a given spectral sequence page
/// Returns (generators_map, differentials_list)
/// generators_map: name -> (torsion, adams_filtration)
pub fn get_filtered_data(
    data: &SyntheticEHP,
    category: Category,
    truncation: Option<i32>,
    page: i32,
    _all_diffs: bool,
    limit_x: Option<i32>,
) -> HashMap<String, (Option<i32>, i32)> {
    let mut torsion_map: HashMap<String, (Option<i32>, i32)> = HashMap::new();

    // Initialize generator torsion map
    for g in &data.generators {
        let in_truncation = truncation.map_or(true, |t| g.y < t);
        let in_x_limit = limit_x.map_or(true, |lx| g.x <= lx + 1);

        if in_truncation && in_x_limit {
            match category {
                Category::Algebraic => {
                    torsion_map.insert(g.name.clone(), (None, g.adams_filtration));
                }
                Category::Classical => {
                    if g.torsion.is_none() {
                        torsion_map.insert(g.name.clone(), (None, g.adams_filtration));
                    }
                }
                Category::Synthetic => {
                    torsion_map.insert(g.name.clone(), (g.torsion, g.adams_filtration));
                }
            }
        }
    }

    // Process differentials
    for diff in &data.differentials {
        if !torsion_map.contains_key(&diff.from) || !torsion_map.contains_key(&diff.to) {
            continue;
        }
        
        if diff.d < page {
            match category {
                Category::Synthetic => {
                    // Get values without holding mutable references
                    let &(to_torsion, to_filt) = torsion_map.get(&diff.to).unwrap();
                    let &(from_torsion, from_filt) = torsion_map.get(&diff.from).unwrap();

                    if let Some(t) = to_torsion {
                        if t == 0 {continue;}
                    }
                    if let Some(t) = from_torsion {
                        if t == 0 {continue;}
                    }

                    if to_filt != from_filt + 1 + diff.coeff && data.find(&diff.to).unwrap().x <= MAX_VERIFY_STEM {
                        eprintln!("Oh Oh. From: {} | {from_filt} and to: {} | {to_filt} do not have compatible filtration, coeff: {}", diff.from, diff.to, diff.coeff);
                    }

                    if to_torsion.is_none() {
                        if from_torsion.is_some() {
                            if data.find(&diff.to).unwrap().x <= MAX_VERIFY_STEM {
                                eprintln!("Oh oh, we have an induced map from torsion to torsion free. {} -> {}", diff.from, diff.to);
                            }
                        }

                        // Target is torsion-free, source dies and target becomes torsion
                        torsion_map.insert(diff.from.clone(), (Some(0), from_filt));
                        torsion_map.insert(diff.to.clone(), (Some(diff.coeff), to_filt));
                    } else if let Some(to_torsion_val) = to_torsion {
                        if to_torsion_val != 0 {
                            let new_from_filt = from_filt - to_torsion_val + diff.coeff;

                            if let Some(from_torsion_val) = from_torsion {
                                // The following would map lower to higher torsion (with coeff)
                                if from_torsion_val >= 0 && from_torsion_val + diff.coeff < to_torsion_val {
                                    eprintln!(
                                        "For {} -> {}, from_torsion={:?}, to_torsion={:?}. Mapping from lower to higher torsion!",
                                        diff.from, diff.to, from_torsion, to_torsion
                                    );
                                }
                                
                                let new_to_torsion_val = from_torsion_val - to_torsion_val + diff.coeff;
                                if new_to_torsion_val < 0 {eprintln!("Oh oh, we have negative torsion. This error should have been caught earlier (in mapping from lower torsion to higher torsion). From: {} | to: {}", diff.from, diff.to)}
                                
                                torsion_map.insert(diff.from.clone(), (Some(new_to_torsion_val), new_from_filt));
                            } else {
                                torsion_map.insert(diff.from.clone(), (None, new_from_filt));
                            }
                            torsion_map.insert(diff.to.clone(), (Some(diff.coeff), to_filt));
                        }
                    }
                }


                Category::Algebraic => {
                    if diff.coeff == 0 {
                        let from_filt = torsion_map.get(&diff.from).unwrap().1;
                        let to_filt = torsion_map.get(&diff.to).unwrap().1;

                        torsion_map.insert(diff.from.clone(), (Some(0), from_filt));
                        torsion_map.insert(diff.to.clone(), (Some(0), to_filt));
                    }
                }


                Category::Classical => {
                    let to_torsion = torsion_map.get(&diff.to).unwrap().0;
                    let to_filt = torsion_map.get(&diff.to).unwrap().1;
                    let from_filt = torsion_map.get(&diff.from).unwrap().1;

                    if to_torsion.is_none() || to_torsion != Some(0) {
                        torsion_map.insert(diff.from.clone(), (Some(0), from_filt));
                        torsion_map.insert(diff.to.clone(), (Some(0), to_filt));
                    }
                }
            }
        }
    }

    if category == Category::Synthetic {
        for tm in &data.tau_mults {
            if !torsion_map.contains_key(&tm.from) || !torsion_map.contains_key(&tm.to) {
                continue;
            }
    
            
    
            let &(from_torsion, from_af) = torsion_map.get(&tm.from).unwrap();
            let &(to_torsion, to_af) = torsion_map.get(&tm.to).unwrap();
    
            if let Some(from_torsion_val) = from_torsion {
                if from_torsion_val == 0 {
                    continue;
                }
            }
            if let Some(to_torsion_val) = to_torsion {
                if to_torsion_val == 0 {
                    continue;
                }
            }
    
            if let Some(from_torsion_val) = from_torsion {
                if from_af - from_torsion_val != to_af {
                    eprintln!("AF does not match up for {tm:?}. From torsion: {from_torsion_val} | from_af: {from_af} | to_af: {to_af}");
                }
    
                torsion_map.insert(tm.to.clone(), (Some(0), to_af));
                if let Some(to_torsion_val) = to_torsion {
                    torsion_map.insert(tm.from.clone(), (Some(from_torsion_val + to_torsion_val), from_af));
                } else {
                    torsion_map.insert(tm.from.clone(), (None, from_af));
                }
            } else {
                eprintln!("In tau Multiplications, from torsion cannot be None. For tau mult: {tm:?}");
            }
    
        }
    }

    torsion_map
}

