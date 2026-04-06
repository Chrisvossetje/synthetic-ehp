use core::panic;
use std::sync::{Arc, LockResult, Mutex};

use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{MAX_STEM, data::{compare::{algebraic_rp, rp_truncations, synthetic_rp}, curtis::STABLE_DATA}, domain::{model::{Diff, FromTo, SyntheticSS}, process::try_compute_pages}, solve::{action::{Action, revert_log_and_remake}, ahss::ahss_synthetic_e1_issue, ahss_e1::get_all_e1_solutions, generate::{get_a_diff, get_a_tau}, issues::{Issue, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic, synthetic_issue_is_tau_structure_issue}, solve::auto_deduce}, types::Kind};


fn check_issue(data: &SyntheticSS, stem: i32, bot_trunc: i32, top_trunc: i32) -> Result<(), Vec<Issue>> {
    for &(synthetic, bt, tt) in rp_truncations() {
        if (top_trunc == tt || (stem + 1 == top_trunc  && tt == 256)) && bot_trunc == bt {
            if synthetic {
                let pages = try_compute_pages(data, bt, tt, stem, stem)?;
                
                let observed = pages.convergence_at_stem(data, stem);

                compare_synthetic(
                    &observed,
                    synthetic_rp(bt, tt),
                    bt,
                    top_trunc,
                    stem,
                )?;
            } else {
                let pages = try_compute_pages(data, bt, tt, stem - 1, stem)?;
                
                let observed = pages.algebraic_convergence_at_stem(data, stem);

                compare_algebraic(
                    &observed,
                    algebraic_rp(bt, tt),
                    bt,
                    tt,
                    stem,
                )?;
            }
            compare_algebraic_spectral_sequence(data, stem, bt, tt, true)?;
        }
    }
    
    Ok(())
}

fn filter_diff(data: &SyntheticSS, d: Diff) -> bool {
    let stem = data.model.stem(d.to);
    let y = data.model.y(d.from);

    // TODO: HIGHER Hopf Invariant One thing  
    y == 3 || y == 7 || (y == 15 && stem != 14) || (y == 31 && stem != 30)
}

// TODO: Is a stem wise approach ENOUGH to conclude E1 stuff, lets hope E1 stuff can always be resolved on the current stem (sadly, i know this is not true :( )?
fn ahss_iterate(mut data: SyntheticSS, alg_data: &Vec<Vec<Vec<Vec<FromTo>>>>, mut getout: Option<Arc<Mutex<bool>>>, log: Arc<Mutex<Vec<Action>>>, stem: i32, top_trunc: i32, target_y: i32, depth: i32, bounded: bool) -> Result<(), String> {    
    
    if let Some(g) = getout {
        if *g.lock().unwrap() {
            println!("HOI@!!!");
            return Ok(());
        }
        getout = Some(g);
    }


    // The log here does nothing special, we do not rely on it.
    // This also means that we cannot reconstruct our data from this log
    // The reason this is fine is because we just look at a single stem
    // So any James periodicity additions will come later down the line

    let option = get_a_diff(&data, top_trunc, target_y, stem);

    // Should only need first option here
    if let Some(d) = option {
        let (from_name, to_name) = data.get_names(d.from, d.to);
        
        let filter = filter_diff(&data, d);
    
        if filter {
            let proof = "Hopf Invariant One.".to_string();
            if depth == 0 {
                log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof: Some(proof.clone()), kind: Kind::Fake });
            }
    
            data.add_diff(d.from, d.to, Some(proof), Kind::Fake);
            return ahss_iterate(data, alg_data, getout.clone(), log.clone(), stem, top_trunc, target_y, depth, bounded)
        }

        if depth == 0 {
            println!("Trying diff: {} | {}", from_name, to_name);
        }

        let mut getout = if getout.is_some() { getout } else { Some(Arc::new(Mutex::new(false))) };

        let with = || {
            let mut with_data = data.clone();
            with_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Real);
            ahss_iterate(with_data, alg_data, getout.clone(), log.clone(), stem, top_trunc, target_y, depth + 1, bounded)
        };
        
        let without = || {
            let mut without_data = data.clone();
            without_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Fake);
            ahss_iterate(without_data, alg_data, getout.clone(), log.clone(), stem, top_trunc, target_y, depth + 1, bounded)
        };

        if depth == 0 {
            let (with_res, without_res) = rayon::join(
                with,
                without
            );
            
            if let Err(e) = with_res {
                // DISPROOF !
                let (from_name, to_name) = data.get_names(d.from, d.to);
                
                // Commit choice !
                if depth == 0 {
                    log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof: Some(e.clone()), kind: Kind::Fake });
                }
    
                data.add_diff(d.from, d.to, Some(e), Kind::Fake);
                
                // And iterate further
                return ahss_iterate(data, alg_data, None, log, stem, top_trunc, target_y, depth, bounded)
            } else {
                let without = without_res;
    
                let (from_name, to_name) = data.get_names(d.from, d.to);
            
                if without.is_ok() && depth == 0 {
                    println!("WITH OR WITHOUT ARE BOTH FINE FOR THE DIFFERENTIAL: {:?} | {:?}", d, data.get_names(d.from, d.to));
                }
    
                // If with and without are both Ok then we continue WITHOUT the differential
                // But we do remember that we don't know the result of this diff    
                let kind = if without.is_err() { Kind::Real } else { Kind::Unknown };
                let proof = if let Err(e) = without { Some(e) } else { None };
                
                data.add_diff(d.from, d.to, proof.clone(), kind);
    
                // Commit choice !
                if depth == 0 { 
                    log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof, kind });
                }
    
                return ahss_iterate(data, alg_data, None, log, stem, top_trunc, target_y, depth, bounded)
            }

        } else {
            if let Err(e) = with() {
                // DISPROOF !
                let (from_name, to_name) = data.get_names(d.from, d.to);
                
                // Commit choice !
                if depth == 0 {
                    println!("Disproven diff: {} | {} by {e}", from_name, to_name);
                    log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof: Some(e.clone()), kind: Kind::Fake });
                    getout = None;
                }
    
                data.add_diff(d.from, d.to, Some(e), Kind::Fake);
                
                // And iterate further
                return ahss_iterate(data, alg_data, getout.clone(), log, stem, top_trunc, target_y, depth, bounded)
            } else {
                let without = without();
    
                let (from_name, to_name) = data.get_names(d.from, d.to);
            
                if without.is_ok() && depth == 0 {
                    println!("WITH OR WITHOUT ARE BOTH FINE FOR THE DIFFERENTIAL: {:?} | {:?}", d, data.get_names(d.from, d.to));
                }
    
                // If with and without are both Ok then we continue WITHOUT the differential
                // But we do remember that we don't know the result of this diff    
                let kind = if without.is_err() { Kind::Real } else { Kind::Unknown };
                let proof = if let Err(e) = without { Some(e) } else { None };
                
                data.add_diff(d.from, d.to, proof.clone(), kind);
    
                
                // Commit choice !
                if depth == 0 {
                    if kind == Kind::Unknown {
                        println!("Unknown diff: {} | {}", from_name, to_name);
                    } else {
                        println!("Proven diff: {} | {}", from_name, to_name);
                    }
                    log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof, kind });
                    getout = None;
                }
    
                return ahss_iterate(data, alg_data, getout.clone(), log, stem, top_trunc, target_y, depth, bounded)
            }
        }
    }

    let potential_tau_thing = match check_issue(&data, stem, target_y, top_trunc) {
        Ok(_) => { false },
        Err(is) => {
            let all_synth_conv = is.iter().all(|i| if let Issue::SyntheticConvergence { bot_trunc, top_trunc, stem, af, expected, observed } = i { true } else { false });
            
            if all_synth_conv {
                synthetic_issue_is_tau_structure_issue(&is)
            } else {
                for i in is {
                    match &i {
                        Issue::SyntheticE1Page { stem, af, expected, observed } => {
                            panic!("Cannot find E1 issues here {:?}", i)
                        },
                        Issue::InvalidName { original_name, unexpected_name, sphere, stem, af } => {
                            panic!("Cannot occur in AHSS {:?}", i)
                        },
                        Issue::InvalidEHPAHSSGen { name, stem } => {
                            panic!("Cannot occur in AHSS {:?}", i)
                            
                        },
                        Issue::InvalidEHPAHSSMap { from, to, stem, sphere } => {
                            panic!("Cannot occur in AHSS {:?}", i)
                        },
                        Issue::InvalidAFRecursion { from, to, from_name, to_name } => {
                            panic!("Cannot occur in AHSS {:?}", i)
                        },
                        Issue::SyntheticConvergence { bot_trunc, top_trunc, stem, af, expected, observed } => {},
                        Issue::AlgebraicConvergence { bot_trunc, top_trunc, stem, af, expected, observed } => {},
                        _ => {
                            if let Some(getout) = getout && depth == 0 {
                                *getout.lock().unwrap() = true;
                            }
                            return Err(format!("{:?}", i))
                        }
                    }
                }
                true
            }
        },
    };

    if potential_tau_thing {
        let option = get_a_tau(&data, top_trunc, target_y, stem);
    
        // Should only need first option here
        if let Some(d) = option {
            let (from_name, to_name) = data.get_names(d.from, d.to);
            
            if depth == 0 {
                println!("Trying tau: {} | {} | af: {}", from_name, to_name, d.af);
            }
    
            let mut getout = if getout.is_some() { getout } else { Some(Arc::new(Mutex::new(false))) };
    
            let with = || {
                let mut with_data = data.clone();
                with_data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Real);
                ahss_iterate(with_data, alg_data, getout.clone(), log.clone(), stem, top_trunc, target_y, depth + 1, bounded)
            };
            
            let without = || {
                let mut without_data = data.clone();
                without_data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Fake);
                ahss_iterate(without_data, alg_data, getout.clone(), log.clone(), stem, top_trunc, target_y, depth + 1, bounded)
            };
    
            if depth == -1 {
                let (with_res, without_res) = rayon::join(
                    with,
                    without
                );
                
                if let Err(e) = with_res {
                    // DISPROOF !
                    let (from_name, to_name) = data.get_names(d.from, d.to);
                    
                    // Commit choice !
                    if depth == 0 {
                        log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof: e.clone(), kind: Kind::Fake });
                    }
        
                    data.add_ext_tau(d.from, d.to, d.af, Some(e), Kind::Fake);
                    
                    // And iterate further
                    return ahss_iterate(data, alg_data, None, log, stem, top_trunc, target_y, depth, bounded)
                } else {
                    let without = without_res;
        
                    let (from_name, to_name) = data.get_names(d.from, d.to);
                
                    if without.is_ok() && depth == 0 {
                        println!("WITH OR WITHOUT ARE BOTH FINE FOR THE TAU: {:?} | {:?}", d, data.get_names(d.from, d.to));
                    }
        
                    // If with and without are both Ok then we continue WITHOUT the differential
                    // But we do remember that we don't know the result of this diff    
                    let kind = if without.is_err() { Kind::Real } else { Kind::Unknown };
                    let proof = if let Err(e) = without { e } else { "".to_string() };
                    
                    data.add_ext_tau(d.from, d.to, d.af, Some(proof.clone()), kind);
        
                    // Commit choice !
                    if depth == 0 { 
                        log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof, kind });
                    }
        
                    return ahss_iterate(data, alg_data, None, log, stem, top_trunc, target_y, depth, bounded)
                }
    
            } else {
                if let Err(e) = with() {
                    // DISPROOF !
                    let (from_name, to_name) = data.get_names(d.from, d.to);
                    
                    // Commit choice !
                    if depth == 0 {
                        println!("Disproven tau: {} | {} by {e}", from_name, to_name);
                        log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof: e.clone(), kind: Kind::Fake });
                        getout = None;
                    }
        
                    data.add_ext_tau(d.from, d.to, d.af, Some(e), Kind::Fake);
                    
                    // And iterate further
                    return ahss_iterate(data, alg_data, getout.clone(), log, stem, top_trunc, target_y, depth, bounded)
                } else {
                    let without = without();
        
                    let (from_name, to_name) = data.get_names(d.from, d.to);
                
                    if without.is_ok() && depth == 0 {
                        println!("WITH OR WITHOUT ARE BOTH FINE FOR THE DIFFERENTIAL: {:?} | {:?}", d, data.get_names(d.from, d.to));
                    }
        
                    // If with and without are both Ok then we continue WITHOUT the differential
                    // But we do remember that we don't know the result of this diff    
                    let kind = if without.is_err() { Kind::Real } else { Kind::Unknown };
                    let proof = if let Err(e) = without { e } else { "".to_string() };
                    
                    data.add_ext_tau(d.from, d.to, d.af, Some(proof.clone()), kind);
        
                    // Commit choice !
                    if depth == 0 { 
                        println!("Proven tau: {} | {} by {proof}", from_name, to_name);
                        log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof, kind });
                        getout = None;
                    }
        
                    return ahss_iterate(data, alg_data, getout.clone(), log, stem, top_trunc, target_y, depth, bounded)
                }
            }
        } 
    }

    match check_issue(&data, stem, target_y, top_trunc).map_err(|x| format!("Tau Issue: {potential_tau_thing} | {}", x.into_iter().map(|f| format!("{:?}", f)).join(", "))) {
        Ok(_) => {},
        Err(e) => {
            if let Some(getout) = getout && depth == 0 {
                *getout.lock().unwrap() = true;
            }
            return Err(e);
        },
    }    

    if bounded {
        return Ok(());
    }
    
    if target_y != 0 {
        // As we are moving up a page for possible diffs,
        // we should add all adams differentials which could arise from here
        
        let d_y = top_trunc - target_y + 1;
        for &(from, to) in &alg_data[stem as usize][d_y as usize][top_trunc as usize]  {
            if depth == 0 {
                let (from, to) = data.get_names(from, to);
                println!("Applying Algebraic diff {from} -> {to}");
            }
            data.add_diff(from, to, None, Kind::Real);
        }
        
        return ahss_iterate(data, alg_data, getout.clone(), log, stem, top_trunc, target_y-1, depth, bounded)
    }


    if top_trunc == stem + 1 {
        return Ok(());
    } else {
        ahss_iterate(data, alg_data, getout.clone(), log, stem, top_trunc + 1, top_trunc, depth, bounded)
    }
}

fn e1_to_ahss_loop(partial_ahss: &SyntheticSS, alg_data: &Vec<Vec<Vec<Vec<FromTo>>>>, mut log: Vec<Action>, stem: i32) -> (Result<(), String>, Vec<Action>) {
    let ahss = revert_log_and_remake(0, &mut log, &partial_ahss, true);
    let log_length = log.len();

    // // First we do a quick scan of diffs which cannot exist based directly on the E1 side.
    // let log = Arc::new(Mutex::new(log));
    // let res = ahss_iterate(ahss, &alg_data, None, log.clone(), stem, 2, 1, 0, true);
    
    // let mut log = Arc::try_unwrap(log).unwrap().into_inner().unwrap();
    
    // let mut start = 0;
    // log.retain(|x| {
    //     let res = match x {
    //         Action::AddDiff { from, to, proof, kind } => {
    //             *kind == Kind::Unknown
    //         },
    //         Action::AddInt { from, to, page, proof, kind } => {
    //             *kind == Kind::Unknown
    //         },
    //         Action::AddExt { from, to, af, proof, kind } => {
    //             *kind == Kind::Unknown
    //         },
    //         _ => {
    //             false
    //         }
    //     };
    //     start += 1;
    //     res && start > log_length  
    // });
    
    // let ahss = revert_log_and_remake(0, &mut log, &partial_ahss, true);

    let log = Arc::new(Mutex::new(log));
    let res = ahss_iterate(ahss, &alg_data, None, log.clone(), stem, 2, 1, 0, false);

    (res, Arc::try_unwrap(log).unwrap().into_inner().unwrap())
}

fn e1_loop(ahss: SyntheticSS, partial_ahss: &SyntheticSS, alg_data: &Vec<Vec<Vec<Vec<FromTo>>>>, mut log: Vec<Action>, stem: i32) -> (Result<(), String>, Vec<Action>) {
    
    println!("\nStarted stem {stem}\n");
    
    let mut proper_issues = vec![];
    match ahss_synthetic_e1_issue(&ahss, stem) {
        Ok(_) => {},
        Err(issues) => {
            for i in issues {
                // First we solve all the e1 issues we can resolve 
                match auto_deduce(&ahss, &i){
                    Ok(a) => log.push(a),
                    Err(_) => proper_issues.push(i),
                }
            }
        },
    }

    if proper_issues.len() != 0 {
        println!("We need to try multiple E1 options");


        // TODO: Par iter
        let res: Vec<_> = get_all_e1_solutions(&ahss, &proper_issues).into_iter().map(|options| {
            let mut clone_log = log.clone();
            for a in &options {
                clone_log.push(a.clone());
            } 

            let res = e1_to_ahss_loop(partial_ahss, alg_data, clone_log, stem);
            println!("\nIn the following option we have the following result: \n{options:?} \n{:?}\n", res.0);
            res
        }).collect();

        let positives = res.iter().fold(0, |acc, r| if r.0.is_ok() {acc + 1} else {acc});
        if positives != 1 {
            let first = res[0].1.clone();
            return (Err(format!("We have {positives} positives. Which means we can't decide on E1 stuff sadge :(")), first)
        } else {
            for r in res {
                if r.0.is_ok() {
                    return r
                }
            }
            unreachable!("There was one positive result.")
        }
    } else {
        e1_to_ahss_loop(partial_ahss, alg_data, log, stem)
    }
}

pub fn ahss_solver() -> (Vec<Action>, SyntheticSS) {
    let alg_ahss = STABLE_DATA.clone();
    let mut partial_ahss = SyntheticSS::empty(alg_ahss.model.clone());
    // We should add all d1's from the algebraic data
    
    let mut alg_data = vec![vec![vec![vec![]; (MAX_STEM + 2) as usize]; (MAX_STEM + 1) as usize]; (MAX_STEM + 1) as usize];

    for (&(from, to), _) in &alg_ahss.proven_from_to {
        let d_y = alg_ahss.model.y(from) - alg_ahss.model.y(to);
        if d_y == 1 {
            partial_ahss.add_diff(from, to, None, Kind::Real);
        } else {
            let stem = alg_ahss.model.stem(to);
            let top_trunc = alg_ahss.model.y(from);
            alg_data[stem as usize][d_y as usize][top_trunc as usize].push((from, to));
        }
    }

    let mut log = vec![
        Action::AddInt { 
            from: "6 5 3[2]".to_string(), 
            to: "6 2 3 3[2]".to_string(), 
            page: 2, 
            proof: "Unique solution to RP1_2".to_string(), 
            kind: Kind::Real 
        },
        Action::AddDiff { from: "2 3 5 7 3 3[2]".to_string(), to: "2 4 1 1 2 4 3 3 3[1]".to_string(), proof: Some("Custom: By convergence on RP3_5 stem 26.".to_string()), kind: Kind::Real }
    ]; 
    
    for stem in 2..=25 {
        let ahss = revert_log_and_remake(0, &mut log, &partial_ahss, true);
        let (res, l) = e1_loop(ahss, &mut partial_ahss, &alg_data, log, stem);
        log = l;
        for dss in &alg_data[stem as usize] {
            for ds in dss {
                for &(from, to) in ds {
                    partial_ahss.add_diff(from, to, None, Kind::Real);
                }
            }
        }
        match res {
            Ok(_) => {},
            Err(e) => {
                println!("Error on stem {stem}: {e}");
                break;
            },
        }
    }

    let ahss = revert_log_and_remake(0, &mut log, &partial_ahss, true);
    (log, ahss)
}