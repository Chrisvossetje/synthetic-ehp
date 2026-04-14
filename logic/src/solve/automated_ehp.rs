// use core::panic;
// use std::{fmt::format, sync::{Arc, LockResult, Mutex, atomic::{AtomicBool, Ordering}}};

// use itertools::Itertools;
// use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

// use crate::{MAX_STEM, data::{compare::{S0, algebraic_rp, algebraic_spheres, rp_truncations, synthetic_rp}, curtis::{DATA, STABLE_DATA}}, domain::{model::{Diff, FromTo, SyntheticSS}, process::try_compute_pages}, solve::{action::{Action, D_R_REPEATS, revert_log_and_remake}, ahss::ahss_synthetic_e1_issue, ahss_e1::get_all_e1_solutions, automated::{ALWAYS_PRINT, PARALLEL_DEPTH}, ehp_ahss::set_metastable_range, generate::{get_a_diff, get_a_tau}, issues::{Issue, algebraic_issue_is_fixable_by_tau_extensions, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic, synthetic_issue_is_tau_structure_issue}, solve::auto_deduce}, types::Kind};

// pub const MAX_DEPTH: i32 = 0;

// fn check_issue(data: &SyntheticSS, stem: i32, sphere: i32) -> Result<(), Vec<Issue>> {
//     if stem + 2 == sphere {
//         let pages = try_compute_pages(data, 0, sphere - 1, stem, stem)?;
        
//         let observed = pages.convergence_at_stem(data, stem);

//         compare_synthetic(
//             &observed,
//             &S0,
//             0,
//             sphere - 1,
//             stem,
//         )?;
//     } else {
//         let pages = try_compute_pages(data, 0, sphere - 1, stem - 1, stem)?;
        
//         let observed = pages.algebraic_convergence_at_stem(data, stem);

//         compare_algebraic(
//             &observed,
//             algebraic_spheres(sphere - 1),
//             0,
//             sphere - 1,
//             stem,
//         )?;
//     }

//     compare_algebraic_spectral_sequence(data, stem, 0, sphere - 1, false)
// }

// fn filter_diff(data: &SyntheticSS, alg_ahss: &SyntheticSS, bot_trunc: i32, top_trunc: i32, d: Diff) -> Option<(Kind, String)> {
//     let stem = data.model.stem(d.to);
//     let y = data.model.y(d.from);

//     if y == 3 || y == 7 || (y == 15 && stem != 14) || (y == 31 && stem != 30) {
//         Some((Kind::Fake, format!("By Hopf Invariant one things we cannot have a differential form the 2^i-1 sphere")))
//     } else if data.in_diffs[d.to].iter().any(|from| data.model.y(*from) == top_trunc && data.model.original_torsion(*from).alive()){
//         Some((Kind::Unknown, format!("As we are only interested in the module structure, we won't consider the case where two differentials target the same generator.")))
//     } else if bot_trunc & 1 == 0 && 
//             let Some(alg_to) = alg_ahss.out_diffs[d.from].first() && 
//             data.model.original_torsion(*alg_to).alive() && 
//             data.model.y(*alg_to) + 1 == bot_trunc {
//         Some((Kind::Unknown, format!("We don't have enough Synthetic information to deduce this differential.")))
//     } else if top_trunc & 1 == 1 && !(top_trunc == 5 && bot_trunc == 3) &&
//             let Some(dies) = data.model.get(d.to).dies && 
//             let Some(source) = alg_ahss.in_diffs[d.to].first() &&
//             data.model.original_torsion(*source).free() &&
//             top_trunc + 2 == dies
//         {
//         Some((Kind::Unknown, format!("We don't have enough Synthetic information to deduce this differential.")))
//     } else {
//         None
//     }
// }

// fn stem_to_real_stem(stem: i32, sphere: i32) -> i32 {
//     stem - ((sphere - 1) / 2)
// }

// fn ehp_iterate(mut data: SyntheticSS, alg_ehp: &SyntheticSS, ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>, getout: [Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize], log: Arc<Mutex<Vec<Action>>>, stem: i32, sphere: i32, bot_trunc: i32, depth: i32, bounded: bool) -> Result<bool, String> {
//     for g in &getout {
//         if let Some(g) = g {
//             if g.load(Ordering::Relaxed) {
//                 return Ok(false);
//             }
//         } 
//     } 

//     let real_stem = stem_to_real_stem(stem, sphere);

//     let option = get_a_diff(&data, sphere, bot_trunc, real_stem);

//     // Should only need first option here
//     if let Some(d) = option {
//         return try_diff(data, alg_ehp, ahss_and_alg_data, getout, &log, stem, sphere, bot_trunc, depth, bounded, d);
//     }
    

//     let potential_tau_thing = match is_tau_issue(&data, real_stem, sphere) {
//         Ok(tau_issue) => tau_issue,
//         Err(is) => {
//             if depth <= PARALLEL_DEPTH && depth != 0 && let Some(getout) = &mut getout[(depth - 1) as usize] {
//                 getout.store(true, Ordering::Relaxed);
//             }
//            return Err(is)
//         },
//     };

//     if let Some((synthetic, mut issues)) = potential_tau_thing {
//         let option = match synthetic {
//             TauIssue::AlgTauIssue => suggest_tau_solution_algebraic(&data, &mut issues, sphere, bot_trunc, stem),
//             TauIssue::SynTauGeneratorIssue => {suggest_tau_solution_generator_synthetic(&data, &mut issues, sphere, bot_trunc, stem)},
//             TauIssue::SynTauModuleIssue => {
//                 // get_a_tau(&data, top_trunc, bot_trunc, stem)
//                 suggest_tau_solution_generator_synthetic(&data, &mut issues, sphere, bot_trunc, stem)
//                 }
//             };
        
//         if let Some(d) = option {
//             return try_tau(data, alg_ahss, alg_data, getout, log, stem, sphere, bot_trunc, depth, bounded, d);
//         } else {
//             if depth <= PARALLEL_DEPTH && depth != 0 && let Some(getout) = &mut getout[(depth - 1) as usize] {
//                 getout.store(true, Ordering::Relaxed);
//             }
//             return Err(format!("Issue at RP{bot_trunc}_{sphere}: {issues:?}"));
//         }
//     }

//     // TODO: 
//     // if bounded {
//     //     return Ok(true);
//     // }
    
//     if bot_trunc != 0 {
//         return add_diffs(data, alg_ehp, ahss_and_alg_data, getout, log, stem, sphere, bot_trunc, depth, bounded);
//     }

//     if sphere == stem + 1 || sphere >= MAX_AUTOMATED_TOP_TRUNC {
//         Ok(true)
//     } else {
//         ahss_iterate(data, alg_ahss, alg_data, getout, log, stem, sphere + 1, sphere, depth, bounded, tau_mode)
//     }
// }



// fn try_diff(mut data: SyntheticSS, alg_ahss: &SyntheticSS, alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>, mut getout: [Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize], log: &Arc<Mutex<Vec<Action>>>, stem: i32, top_trunc: i32, bot_trunc: i32, depth: i32, bounded: bool, d: Diff) -> Result<bool, String> {
//     let (from_name, to_name) = data.get_names(d.from, d.to);
    
//     let filter = filter_diff(&data, alg_ahss, bot_trunc, top_trunc, d);

//     if ALWAYS_PRINT || depth == 0 {
//         println!("Trying diff: {} | {}", from_name, to_name);
//     }    
    
//     if let Some((kind, reason)) = filter {
//         if depth == 0 {
//             log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof: Some(reason.clone()), kind });
//         }
    
//         data.add_diff(d.from, d.to, Some(reason), kind);
//         return ehp_iterate(data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth, bounded, false)
//     }

    

    
//     if depth < PARALLEL_DEPTH { 
//         getout[depth as usize] = Some(Arc::new(AtomicBool::new(false)));
//     }

//     let with = || {
//         let mut with_data = data.clone();
//         with_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Real);
//         ehp_iterate(with_data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth + 1, bounded, false)
//     };
//     let without = || {
//         let mut without_data = data.clone();
//         without_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Fake);
//         ehp_iterate(without_data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth + 1, bounded, false)
//     };

//     if depth < PARALLEL_DEPTH {
//         let (with_res, without_res) = rayon::join(
//             with,
//             without
//         );
    
//         if let Err(e) = with_res {
//             // DISPROOF !
//             let (from_name, to_name) = data.get_names(d.from, d.to);
        
//             // Proof: Tau Issue: false | SyntheticConvergence { bot_trunc: 1, top_trunc: 30, stem: 30, af: 4, expected: [Torsion(Some(1))], observed: [Torsion(None)] }

//             if ALWAYS_PRINT || depth == 0 {
//                 println!("Disproven diff: {} | {} by {e}", from_name, to_name);            
//             }
//             // Commit choice !
//             if depth == 0 {
//                 log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof: Some(e.clone()), kind: Kind::Fake });
//             }
    
//             data.add_diff(d.from, d.to, Some(e), Kind::Fake);
        
//             // And iterate further
//             getout[depth as usize] = None;
//             return ahss_iterate(data, alg_ahss, alg_data, getout, log.clone(), stem, top_trunc, bot_trunc, depth, bounded, false)
//         } else if let Err(e) = without_res {
//             let (from_name, to_name) = data.get_names(d.from, d.to);
    
//             data.add_diff(d.from, d.to, Some(e.clone()), Kind::Real);
    
//             if ALWAYS_PRINT || depth == 0 {
//                 println!("Proven diff: {} | {} | {:?}", from_name, to_name, Some(e.clone()));
//             }
//             if depth == 0 {
//                 log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof: Some(e), kind: Kind::Real });
//             }

//             getout[depth as usize] = None;
//             return ahss_iterate(data, alg_ahss, alg_data, getout, log.clone(), stem, top_trunc, bot_trunc, depth, bounded, false)
//         } else if !matches!(with_res, Ok(true)) || !matches!(without_res, Ok(true)) {
//             return Ok(false);
//         } else {
//             let without = without_res;
    
//             let (from_name, to_name) = data.get_names(d.from, d.to);
    
//             if matches!(without, Ok(true)) && depth == 0 {
//                 println!("WITH OR WITHOUT ARE BOTH FINE FOR THE DIFFERENTIAL: {:?} | {:?}", d, data.get_names(d.from, d.to));
//             }
    
//             // If with and without are both Ok then we continue WITHOUT the differential
//             // But we do remember that we don't know the result of this diff    
//             let kind = Kind::Unknown;
//             let proof = None;
        
//             data.add_diff(d.from, d.to, proof.clone(), kind);
    
//             // Commit choice !
//             if ALWAYS_PRINT || depth == 0 { 
//                 if kind == Kind::Unknown {
//                     println!("Unknown diff: {} | {}", from_name, to_name);
//                 } else {
//                     println!("Proven diff: {} | {} | {proof:?}", from_name, to_name);
//                 }
//             }
//             if depth == 0 { 
//                 log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof, kind });
//             }
    
//             getout[depth as usize] = None;
//             return ahss_iterate(data, alg_ahss, alg_data, getout, log.clone(), stem, top_trunc, bot_trunc, depth, bounded, false)
//         }

//     } else {
//         let with_res = with();
//         if let Err(e) = with_res {
//             // DISPROOF !
//             let (from_name, to_name) = data.get_names(d.from, d.to);
        
//             // Commit choice !
//             if ALWAYS_PRINT || depth == 0 {
//                 println!("Disproven diff: {} | {} by {e}", from_name, to_name);
//             }
//             if depth == 0 {
//                 log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof: Some(e.clone()), kind: Kind::Fake });
//                 getout = [const { None }; PARALLEL_DEPTH as usize];
//             }
    
//             data.add_diff(d.from, d.to, Some(e), Kind::Fake);
        
//             // And iterate further
//             return ahss_iterate(data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth, bounded, false)
//         } else if !matches!(with_res, Ok(true)) {
//             return Ok(false);
//         } else {
//             let without = without();
    
//             let (from_name, to_name) = data.get_names(d.from, d.to);
    
//             if matches!(without, Ok(true)) && depth == 0 {
//                 println!("WITH OR WITHOUT ARE BOTH FINE FOR THE DIFFERENTIAL: {:?} | {:?}", d, data.get_names(d.from, d.to));
//             }
    
//             // If with and without are both Ok then we continue WITHOUT the differential
//             // But we do remember that we don't know the result of this diff    
//             let (kind, proof) = match without {
//                 Err(e) => (Kind::Real, Some(e)),
//                 Ok(true) => (Kind::Unknown, None),
//                 Ok(false) => return Ok(false),
//             };
        
//             data.add_diff(d.from, d.to, proof.clone(), kind);
    
//             if ALWAYS_PRINT || depth == 0 {
//                 if kind == Kind::Unknown {
//                     println!("Unknown diff: {} | {}", from_name, to_name);
//                 } else {
//                     println!("Proven diff: {} | {} | {proof:?}", from_name, to_name);
//                 }
//             }
//             // Commit choice !
//             if depth == 0 {
//                 log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof, kind });
//                 getout = [const { None }; PARALLEL_DEPTH as usize];
//             }
    
//             return ahss_iterate(data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth, bounded, false)
//         }
//     }
// }

// // Result Ok means fixable, Err is quit out
// // Ok None means it is all fine!
// // Ok Some gives back the errors

// pub enum TauIssue {
//     AlgTauIssue,
//     SynTauGeneratorIssue,
//     SynTauModuleIssue,
// }

// fn is_tau_issue(data: &SyntheticSS, real_stem: i32, sphere: i32) -> Result<Option<(TauIssue, Vec<Issue>)>, String> {
//     match check_issue(data, real_stem, sphere) {
//         Ok(_) => { Ok(None) },
//         Err(is) => {
//             let all_synth_conv = if let Issue::SyntheticConvergence { bot_trunc, top_trunc, stem, af, expected, observed } = &is[0] {true} else {false};
            
//             if all_synth_conv {
//                 let (solvable, generator) = synthetic_issue_is_tau_structure_issue(&is);
//                 if solvable {
//                     if generator {
//                         Ok(Some((TauIssue::SynTauGeneratorIssue, is)))
//                     } else {
//                         Ok(Some((TauIssue::SynTauModuleIssue, is)))
//                     }
//                 } else {
//                     Err(format!("For the stable Sphere the F_2 vector space generators don't add up. {is:?}"))
//                 }
//             } else {
//                 if algebraic_issue_is_fixable_by_tau_extensions(&is) {
//                     Ok(Some((TauIssue::AlgTauIssue, is)))
//                 } else {
//                     Err(format!("For S^{sphere} there is no way to fix the algebraic convergence issues with tau extensions. {is:?}"))
//                 }
//             }
//         },
//     }
// }

// fn try_tau(mut data: SyntheticSS, alg_ahss: &SyntheticSS, alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>, mut getout: [Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize], log: Arc<Mutex<Vec<Action>>>, stem: i32, top_trunc: i32, bot_trunc: i32, depth: i32, bounded: bool, d: ExtTauMult) -> Result<bool, String> {
//     let (from_name, to_name) = data.get_names(d.from, d.to);
    
//     if ALWAYS_PRINT || depth == 0 {
//         println!("Trying tau: {} | {} | af: {} | RP{bot_trunc}_{top_trunc}", from_name, to_name, d.af);
//     }

    
//     if depth < PARALLEL_DEPTH { 
//         getout[depth as usize] = Some(Arc::new(AtomicBool::new(false)));
//     }

//     let with = || {
//         let mut with_data = data.clone();
//         with_data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Real);
//         ehp_iterate(with_data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth + 1, bounded, true)
//     };
//     let without = || {
//         let mut without_data = data.clone();
//         without_data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Fake);
//         ehp_iterate(without_data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth + 1, bounded, true)
//     };


    
//     if depth < PARALLEL_DEPTH {
//         let (with_res, without_res) = rayon::join(
//             with,
//             without
//         );
    
//         if let Err(e) = with_res {
//             // DISPROOF !
//             let (from_name, to_name) = data.get_names(d.from, d.to);
        
//             // Commit choice !
//             if ALWAYS_PRINT || depth == 0 {
//                 println!("Disproven tau: {} | {} by {e}", from_name, to_name);
//             }
//             if depth == 0 {
//                 log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof: e.clone(), kind: Kind::Fake });
//             }
        
//             data.add_ext_tau(d.from, d.to, d.af, Some(e), Kind::Fake);
        
//             // And iterate further
//             getout[depth as usize] = None;
//             return ahss_iterate(data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded, true)
//         } else if let Err(e) = without_res {
//             let (from_name, to_name) = data.get_names(d.from, d.to);
//             let proof = e.clone();
        
//             data.add_ext_tau(d.from, d.to, d.af, Some(proof.clone()), Kind::Real);
        
//             if ALWAYS_PRINT || depth == 0 { 
//                 println!("Proven tau: {} | {} by {proof}", from_name, to_name);
//             }
//             if depth == 0 { 
//                 log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof, kind: Kind::Real });
//             }

//             getout[depth as usize] = None;
//             return ahss_iterate(data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded, true)
//         } else if !matches!(with_res, Ok(true)) || !matches!(without_res, Ok(true)) {
//             return Ok(false);
//         } else {
//             let without = without_res;
        
//             let (from_name, to_name) = data.get_names(d.from, d.to);
    
//             if matches!(without, Ok(true)) && depth == 0 {
//                 println!("WITH OR WITHOUT ARE BOTH FINE FOR THE TAU: {:?} | {:?}", d, data.get_names(d.from, d.to));
//             }
        
//             // If with and without are both Ok then we continue WITHOUT the differential
//             // But we do remember that we don't know the result of this diff    
//             let kind = Kind::Unknown;
//             let proof = "".to_string();
        
//             data.add_ext_tau(d.from, d.to, d.af, Some(proof.clone()), kind);
        
//             // Commit choice !
//             if ALWAYS_PRINT || depth == 0 { 
//                 println!("Proven tau: {} | {} by {proof}", from_name, to_name);
//             }
//             if depth == 0 { 
//                 log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof, kind });
//             }

//             getout[depth as usize] = None;
//             return ahss_iterate(data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded, true)
//         }
    
//     } else {
//         let with_res = with();
//         if let Err(e) = with_res {
//             // DISPROOF !
//             let (from_name, to_name) = data.get_names(d.from, d.to);
        
//             // Commit choice !
//             if ALWAYS_PRINT || depth == 0 {
//                 println!("Disproven tau: {} | {} by {e}", from_name, to_name);
//             }
//             if depth == 0 {
//                 log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof: e.clone(), kind: Kind::Fake });
//                 getout = [const { None }; PARALLEL_DEPTH as usize];
//             }
        
//             data.add_ext_tau(d.from, d.to, d.af, Some(e), Kind::Fake);
        
//             // And iterate further
//             return ahss_iterate(data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded, true)
//         } else if !matches!(with_res, Ok(true)) {
//             return Ok(false);
//         } else {
//             let without = without();
        
//             let (from_name, to_name) = data.get_names(d.from, d.to);
    
//             if matches!(without, Ok(true)) && depth == 0 {
//                 println!("WITH OR WITHOUT ARE BOTH FINE FOR THE TAU: {:?} | {:?}", d, data.get_names(d.from, d.to));
//             }
        
//             // If with and without are both Ok then we continue WITHOUT the differential
//             // But we do remember that we don't know the result of this diff    
//             let (kind, proof) = match without {
//                 Err(e) => (Kind::Real, e),
//                 Ok(true) => (Kind::Unknown, "".to_string()),
//                 Ok(false) => return Ok(false),
//             };
        
//             data.add_ext_tau(d.from, d.to, d.af, Some(proof.clone()), kind);

//             // Commit choice !
//             if ALWAYS_PRINT || depth == 0 { 
//                 println!("Proven tau: {} | {} by {proof}", from_name, to_name);
//             }
//             if depth == 0 { 
//                 log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof, kind });
//                 getout = [const { None }; PARALLEL_DEPTH as usize];
//             }

//             return ahss_iterate(data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded, true)
//         }
//     }
// }

// fn add_diffs(mut data: SyntheticSS, alg_ehp: &SyntheticSS, ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>, getout: [Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize], log: Arc<Mutex<Vec<Action>>>, stem: i32, top_trunc: i32, bot_trunc: i32, depth: i32, bounded: bool) -> Result<bool, String> {
//     // As we are moving up a page for possible diffs,
//     // we should add all AHSS / adams differentials which could arise from here


//     // TODO: there are 

//     let d_y = top_trunc - bot_trunc + 1;
//     for &(from, to) in &ahss_and_alg_data[stem as usize][d_y as usize][top_trunc as usize]  {
//         if depth == 0 {
//             let (from, to) = data.get_names(from, to);
//             println!("Applying Algebraic diff {from} -> {to}");
//         }
//         data.add_diff(from, to, None, Kind::Real);
//     }
//     return ahss_iterate(data, alg_ahss, alg_data, getout.clone(), log, stem, top_trunc, bot_trunc-1, depth, bounded, false)
// }


// pub fn ehp_solver(ahss: &SyntheticSS, log: Option<Vec<Action>>) -> (Vec<Action>, SyntheticSS) {
//     let alg_ehp = DATA.clone();
//     let mut partial_ehp = SyntheticSS::empty(alg_ehp.model.clone());
//     // We should add all d1's from the algebraic data
    
//     let _ = set_metastable_range(&mut partial_ehp, ahss);

//     let mut ahss_and_alg_data = vec![vec![vec![vec![]; (MAX_STEM + 2) as usize]; (MAX_STEM + 1) as usize]; (MAX_STEM + 1) as usize];

//     // Add EHP Algebraic Diffs
//     for (&(from, to), _) in &alg_ehp.proven_from_to {
//         let d_y = alg_ehp.model.y(from) - alg_ehp.model.y(to);
//         if d_y == 1 {
//             partial_ehp.add_diff(from, to, None, Kind::Real);
//         } else {
//             let stem = alg_ehp.model.stem(to);
//             let top_trunc = alg_ehp.model.y(from);
//             ahss_and_alg_data[stem as usize][d_y as usize][top_trunc as usize].push((from, to, Kind::Real, None));
//         }
//     }

//     // Add compatible AHSS diffs
//     for (&(from, to), proof) in &ahss.proven_from_to {
//         let d_y = ahss.model.y(from) - ahss.model.y(to);

//         if let Some(from_id) = alg_ehp.model.try_index(ahss.model.name(from)) {
//             if let Some(to_id) = alg_ehp.model.try_index(ahss.model.name(to)) {
//                 // Differentials 
//                 if ahss.model.stem(from) != ahss.model.stem(to) {
//                     // Don't include the Algebraic diffs of AHSS
//                     if let Some(p) = proof {
//                         if d_y == 1 {
//                             partial_ehp.add_diff(from_id, to_id, Some(format!("Lifted from Stable EHP")), Kind::Real);
//                         } else {
//                             let stem = alg_ehp.model.stem(to_id);
//                             let top_trunc = alg_ehp.model.y(from_id);
//                             ahss_and_alg_data[stem as usize][d_y as usize][top_trunc as usize].push((from_id, to_id, Kind::Real, Some(format!("Lifted from Stable EHP"))));
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     // Add disproven compatible AHSS diffs
//     for (&(from, to), proof) in &ahss.disproven_from_to {
//         let d_y = ahss.model.y(from) - ahss.model.y(to);

//         // Only add differentials here
//         if ahss.model.stem(from) != ahss.model.stem(to) {
//             if let Some(from_id) = alg_ehp.model.try_index(ahss.model.name(from)) {
//                 if let Some(to_id) = alg_ehp.model.try_index(ahss.model.name(to)) {
//                     // Don't include the Unknown differentials
//                     if let Some(p) = proof {
//                         partial_ehp.add_diff(from_id, to_id, Some(format!("Lifted from Stable EHP")), Kind::Fake);
//                     }
//                 } 
//             }
//         }
//     }

//     // Add all external tau's 
//     // We won't worry about the fake ones
//     for esss in &ahss.external_tau_page {
//         for ess in esss {
//             for es in ess {
//                 for e in es {
//                     if let Some(from_id) = alg_ehp.model.try_index(ahss.model.name(e.from)) {
//                         if let Some(to_id) = alg_ehp.model.try_index(ahss.model.name(e.to)) {
//                             partial_ehp.add_ext_tau(from_id, to_id, e.af, Some(format!("Lifted from Stable EHP")), Kind::Real);
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     let mut log = if let Some(log) = log {log} else { vec![
    
//     ]};

//     let ehp = revert_log_and_remake(0, &mut log, &partial_ehp, true);
//     let log = Arc::new(Mutex::new(log));

//     ehp_iterate(ehp, &alg_ehp, ahss_and_alg_data, [const { None }; PARALLEL_DEPTH as usize], log, 2, 2, 0, 0, false);

//     // for stem in 2..=35 {
//     //     let ahss = revert_log_and_remake(0, &mut log, &partial_ehp, true);
//     //     let (res, l) = e1_loop(ahss, &mut partial_ehp, &alg_ehp, &alg_data, log, stem);
//     //     log = l;
//     //     for dss in &alg_data[stem as usize] {
//     //         for ds in dss {
//     //             for &(from, to) in ds {
//     //                 partial_ehp.add_diff(from, to, None, Kind::Real);
//     //             }
//     //         }
//     //     }
//     //     match res {
//     //         Ok(_) => {},
//     //         Err(e) => {
//     //             println!("Error on stem {stem}: {e}");
//     //             break;
//     //         },
//     //     }
//     // }

//     let ehp = revert_log_and_remake(0, &mut log, &partial_ehp, true);
//     (log, ehp)
// }