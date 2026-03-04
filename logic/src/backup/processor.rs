use crate::data::{get_diffs, get_induced_names, get_tau_mults};
use crate::stable_data::{get_stable_diffs, get_stable_tau_mults};
use crate::{MAX_STEM, MAX_VERIFY_STEM};
use crate::types::{Category, Kind, SyntheticSS};
use crate::naming::{generating_name};
use core::panic;
use std::collections::{HashMap};
use itertools::Itertools;



/// Add differentials to the data
pub fn add_diffs(data: &mut SyntheticSS) {
    for d in get_diffs() {
        data.insert_diff(d);
    }
}

/// Add unstable differentials to the data
pub fn add_induced_names(data: &mut SyntheticSS) {
    for (g, induced_name) in get_induced_names() {
        if let Some(g) = data.find_mut(&g) {
            for n in induced_name {
                g.induced_name.insert(0, n);
            }
        }
    }
}

pub fn add_tau_mults(data: &mut SyntheticSS) {
    data.tau_mults = get_tau_mults();
}

/// Add stable differentials to the data
pub fn add_stable_diffs(data: &mut SyntheticSS) {
    for d in get_stable_diffs() {
        data.insert_diff(d);
    }
}

pub fn add_stable_tau_mults(data: &mut SyntheticSS) {
    data.tau_mults = get_stable_tau_mults();
}

/// Compute inductive generators
pub fn compute_inductive_generators(data: &mut SyntheticSS) {
    for x in 3..MAX_STEM {
        for y in 1..=MAX_STEM {
            let sphere = y * 2 + 1;
            
            panic!("This whole function should be done by stem, and not by row.
                    Make the function take in both the sphere and stem it should calculate for.");
        //     let gens = get_filtered_data(data, Category::Synthetic, 0, sphere, 1000, Some(x));


        //     // First set everything to zero and then see what should NOT have been zero
        //     for g in &mut data.generators {
        //         if g.x == x + y && g.y == y {
        //             g.torsion = Some(0);
        //         }
        //     }
            

        //     for (name, (torsion, filtration)) in gens.iter().sorted_by_key(|x|  data.find(&x.0).unwrap().y) {
        //         if let Some(g) = data.find(&name).cloned() {
        //             if g.x == x {
        //                 let target_name = format!("{}[{}]", generating_name(&g.get_induced_name(sphere)), y);
        //                 if let Some(g_target) = data.find_mut(&target_name) {
        //                     g_target.torsion = *torsion;
        //                     if let Some(t) = *torsion {
        //                         if t != 0 {
        //                             g_target.adams_filtration = filtration + 1;
        //                         }
        //                     } else {
        //                         g_target.adams_filtration = filtration + 1;
        //                     }
        //                 }
        //             }
        //         }
        //     }
        }
    }
}

// Return all generators which can support a differential
// (page, torsion, af)
// page: the page which supported its last diff. Meaning it can support a diff on page + 1
// torsion: Torsion at the last diff / page
// af: AF at the last diff / page
pub fn find_possible_sources_for_differentials_in_stem(data: &SyntheticSS, stem: i32, bot_trunc: i32, top_trunc: i32) -> HashMap<String, (i32, Option<i32>, i32)> {
    let mut g = get_filtered_data(data, Category::Synthetic, 0, 1000, 1000, Some(stem));
    g.generators.retain(|k, v| v[0].1 != Some(0) && bot_trunc <= data.find(&k).unwrap().y && data.find(&k).unwrap().y <= top_trunc);


    g.generators.into_iter().map(|(k,v)| {
        (k,v[0])
    }).collect()
}

pub struct PageSS {
    pub generators: HashMap<String, Vec<(i32, Option<i32>, i32)>>
}

impl PageSS {
    pub fn fill(data: &SyntheticSS,
            category: Category,
            bot_trunc: i32,
            top_trunc: i32,
            limit_x: Option<i32>,) -> Self {
        let mut torsion_map: HashMap<String, Vec<(i32, Option<i32>, i32)>> = HashMap::new();

        // Initialize generator torsion map
        for g in &data.generators {
            let in_x_limit = limit_x.map_or(true, |lx| lx - 1 <= g.x && g.x <= lx + 1);

            if bot_trunc <= g.y && g.y < top_trunc && in_x_limit {
                match category {
                    Category::Algebraic => {
                        torsion_map.insert(g.name.clone(), vec![(0, None, g.adams_filtration)]);
                    }
                    Category::Geometric => {
                        if g.torsion.is_none() {
                            torsion_map.insert(g.name.clone(), vec![(0, None, g.adams_filtration)]);
                        }
                    }
                    Category::Synthetic => {
                        torsion_map.insert(g.name.clone(), vec![(0, g.torsion, g.adams_filtration)]);
                    }
                }
            }
        }
        PageSS { generators: torsion_map }

    }

    pub fn contains(&self, el: &str) -> bool {
        self.generators.contains_key(el)
    }
    
    pub fn get_recent(&self, el: &str) -> (i32, Option<i32>, i32) {
        let a = self.generators.get(el).unwrap();
        let (_, el_torsion, el_filt) = a.first().unwrap();
        let (_, _, original_filt) = a.last().unwrap();
        (*original_filt, *el_torsion, *el_filt)
    }

    pub fn insert(&mut self, el: &str, page: i32, torsion: Option<i32>, filtration: i32) {
        // TODO: Make this just Push ??
        self.generators.get_mut(el).unwrap().insert(0, (page, torsion, filtration));
    }

    pub fn reduce_to_stem(&mut self, data: &SyntheticSS, stem: i32) {
        self.generators.retain(|k, _| data.find(k).unwrap().x == stem); 
    }

    pub fn get_final_page(self) -> HashMap<String, (Option<i32>, i32)> {
        self.generators.into_iter().map(|(k, v)| (k, (v[0].1, v[0].2))).collect()
    }
}


/// Get filtered data for a given spectral sequence page
/// Returns (generators_map, differentials_list)
/// generators_map: name -> (torsion, adams_filtration)
pub fn get_filtered_data(
    data: &SyntheticSS,
    category: Category,
    bot_trunc: i32,
    top_trunc: i32,
    page: i32,
    limit_x: Option<i32>,
) -> PageSS {
    let mut pages = PageSS::fill(data, category, bot_trunc, top_trunc, limit_x);

    // Process differentials
    // The coefficients of a differential are somewhat nuanced
    // The coefficient of a differential is to be defined to go from the corresponding AF on page the differential is defined.
    // And wrt to target where it considers the AF to be the one the element has on the E1 ! page
    for diff in &data.differentials {
        if !pages.contains(&diff.from) || !pages.contains(&diff.to) {
            continue;
        }
        
        if diff.d < page && diff.kind == Kind::Real {
            match category {
                Category::Synthetic => {                    
                    let (to_original_filtration, to_torsion, to_filt) = pages.get_recent(&diff.to);
                    let (_, from_torsion, from_filt) = pages.get_recent(&diff.from);

                    let difference_expected_filtration = to_original_filtration - to_filt;

                    let coeff = diff.coeff - difference_expected_filtration;

                    if let Some(t) = to_torsion {
                        if t == 0 {continue;}
                    }
                    if let Some(t) = from_torsion {
                        if t == 0 {continue;}
                    }

                    if to_filt != from_filt + 1 + coeff && data.find(&diff.to).unwrap().x <= MAX_VERIFY_STEM {
                        eprintln!("Oh Oh. From: {} | {from_filt} and to: {} | {to_filt} do not have compatible filtration, coeff: {}", diff.from, diff.to, coeff);
                    }

                    if to_torsion.is_none() {
                        if from_torsion.is_some() {
                            if data.find(&diff.to).unwrap().x <= MAX_VERIFY_STEM {
                                eprintln!("Oh oh, we have an induced map from torsion to torsion free. {} -> {}", diff.from, diff.to);
                            }
                        }

                        // Target is torsion-free, source dies and target becomes torsion
                        pages.insert(&diff.from, diff.d, Some(0), from_filt);
                        pages.insert(&diff.to, diff.d, Some(coeff), to_filt);
                    } else if let Some(to_torsion_val) = to_torsion {
                        if to_torsion_val != 0 {
                            let new_from_filt = from_filt - to_torsion_val + coeff;

                            if let Some(from_torsion_val) = from_torsion {
                                // The following would map lower to higher torsion (with coeff)
                                if from_torsion_val >= 0 && from_torsion_val + coeff < to_torsion_val && data.find(&diff.to).unwrap().x <= MAX_VERIFY_STEM {
                                    eprintln!(
                                        "For {} -> {}, from_torsion={:?}, to_torsion={:?}. Mapping from lower to higher torsion! Coeff: {}",
                                        diff.from, diff.to, from_torsion, to_torsion, coeff
                                    );
                                }
                                
                                let new_to_torsion_val = from_torsion_val - to_torsion_val + coeff;
                                if new_to_torsion_val < 0 &&  data.find(&diff.to).unwrap().x <= MAX_VERIFY_STEM {
                                    eprintln!("Oh oh, we have negative torsion. This error should have been caught earlier (in mapping from lower torsion to higher torsion). From: {} | to: {}", diff.from, diff.to)
                                }
                                
                                pages.insert(&diff.from, diff.d, Some(new_to_torsion_val), new_from_filt);
                            } else {
                                pages.insert(&diff.from, diff.d, None, new_from_filt);
                            }
                            pages.insert(&diff.to, diff.d, Some(coeff), to_filt);
                        }
                    }
                }


                Category::Algebraic => {
                    if diff.coeff == 0 && diff.synthetic.is_none() {
                        let from_filt = pages.get_recent(&diff.from).2;
                        let to_filt = pages.get_recent(&diff.to).2;

                        pages.insert(&diff.from, diff.d, Some(0), from_filt);
                        pages.insert(&diff.to, diff.d, Some(0), to_filt);
                    }
                }


                Category::Geometric => {
                    let to_torsion = pages.get_recent(&diff.to).1;
                    let to_filt = pages.get_recent(&diff.to).2;
                    let from_filt = pages.get_recent(&diff.from).2;

                    if to_torsion.is_none() || to_torsion != Some(0) {
                        pages.insert(&diff.from, diff.d, Some(0), from_filt);
                        pages.insert(&diff.to, diff.d, Some(0), to_filt);
                    }
                }
            }
        }
    }

    // If fixed on stem, filter out around that stem which could have been targets of differentials
    if let Some(stem) = limit_x {
        pages.reduce_to_stem(data, stem); 
    }

    if category == Category::Synthetic && page > 999 {
        for tm in &data.tau_mults {
            if !pages.contains(&tm.from) || !pages.contains(&tm.to) {
                continue;
            }
    

            let (_ ,from_torsion, from_af) = pages.get_recent(&tm.from);
            let (_, to_torsion, to_af) = pages.get_recent(&tm.to);


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
    
                pages.insert(&tm.to, page, Some(0), to_af);
                if let Some(to_torsion_val) = to_torsion {
                    // TODO
                    pages.insert(&tm.from, 10, Some(from_torsion_val + to_torsion_val), from_af);
                } else {
                    // TODO
                    pages.insert(&tm.from, 10, None, from_af);
                }
            } else {
                eprintln!("In tau Multiplications, from torsion cannot be None. For tau mult: {tm:?}");
            }
    
        }
    }

    pages
}

