use core::panic;
use std::sync::{Arc, Mutex};

use itertools::Itertools;

use crate::{MAX_STEM, data::{self, compare::{algebraic_rp, rp_truncations, synthetic_rp}, curtis::STABLE_DATA}, domain::{model::{Diff, ExtTauMult, FromTo, SyntheticSS}, process::try_compute_pages}, solve::{action::{Action, D_R_REPEATS, revert_log_and_remake}, ahss::ahss_synthetic_e1_issue, ahss_e1::get_all_e1_solutions, generate::{get_a_diff, get_a_tau}, issues::{Issue, algebraic_issue_is_fixable_by_tau_extensions, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic, synthetic_issue_is_tau_structure_issue}, solve::{auto_deduce, suggest_tau_solution_algebraic, suggest_tau_solution_generator_synthetic, suggest_tau_solution_module_synthetic}}, types::Kind};

const PARALLEL_DEPTH: i32 = 2;
const ALWAYS_PRINT: bool = false;

fn check_issue(data: &SyntheticSS, stem: i32, bot_trunc: i32, top_trunc: i32) -> Result<(), Vec<Issue>> {
    for &(synthetic, bt, tt) in rp_truncations() {
        if (top_trunc == tt || (stem + 1 == top_trunc && tt == 256)) && bot_trunc == bt {
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

fn filter_diff(data: &SyntheticSS, alg_ahss: &SyntheticSS, bot_trunc: i32, top_trunc: i32, d: Diff) -> Option<(Kind, String)> {
    let stem = data.model.stem(d.to);
    let y = data.model.y(d.from);

    if y == 3 || y == 7 || (y == 15 && stem != 14) || (y == 31 && stem != 30) {
        Some((Kind::Fake, format!("By Hopf Invariant one things we cannot have a differential form the 2^i-1 sphere")))
    } else if data.in_diffs[d.to].iter().any(|from| data.model.y(*from) == top_trunc && data.model.original_torsion(*from).alive()){
        Some((Kind::Unknown, format!("As we are only interested in the module structure, we won't consider the case where two differentials target the same generator.")))
    } else if bot_trunc & 1 == 0 && 
            let Some(alg_to) = alg_ahss.out_diffs[d.from].first() && 
            data.model.original_torsion(*alg_to).alive() && 
            data.model.y(*alg_to) + 1 == bot_trunc {
        Some((Kind::Unknown, format!("We don't have enough Synthetic information to deduce this differential.")))
    } else if top_trunc & 1 == 1 && 
            let Some(dies) = data.model.get(d.to).dies && 
            let Some(source) = alg_ahss.in_diffs[d.to].first() &&
            data.model.original_torsion(*source).free() &&
            top_trunc + 2 == dies
        {
        Some((Kind::Unknown, format!("We don't have enough Synthetic information to deduce this differential.")))
    } else {
        None
    }
}

// TODO: Is a stem wise approach ENOUGH to conclude E1 stuff, lets hope E1 stuff can always be resolved on the current stem (sadly, i know this is not true :( )?
fn ahss_iterate(data: SyntheticSS, alg_ahss: &SyntheticSS, alg_data: &Vec<Vec<Vec<Vec<FromTo>>>>, mut getout: [Option<Arc<Mutex<bool>>>; PARALLEL_DEPTH as usize], log: Arc<Mutex<Vec<Action>>>, stem: i32, top_trunc: i32, bot_trunc: i32, depth: i32, bounded: bool, tau_mode: bool) -> Result<(), String> {
    for g in &getout {
        if let Some(g) = g {
            if *g.lock().unwrap() {
                return Ok(());
            }
        } 
    } 

    // The log here does nothing special, we do not rely on it.
    // This also means that we cannot reconstruct our data from this log
    // The reason this is fine is because we just look at a single stem
    // So any James periodicity additions will come later down the line

    let option = get_a_diff(&data, top_trunc, bot_trunc, stem);

    // Should only need first option here
    if let Some(d) = option {
        return try_diff(data, alg_ahss, alg_data, getout, &log, stem, top_trunc, bot_trunc, depth, bounded, d);
    }
    

    let potential_tau_thing = match is_tau_issue(&data, stem, top_trunc, bot_trunc, depth) {
        Ok(tau_issue) => tau_issue,
        Err(is) => {
            if depth <= PARALLEL_DEPTH && depth != 0 && let Some(getout) = &mut getout[(depth - 1) as usize] {
                *getout.lock().unwrap() = true;
            }
           return Err(is)
        },
    };

    if let Some((synthetic, issues)) = potential_tau_thing {
        let option = match synthetic {
            TauIssue::AlgTauIssue => suggest_tau_solution_algebraic(&data, &issues, top_trunc, bot_trunc, stem),
            TauIssue::SynTauGeneratorIssue => {suggest_tau_solution_generator_synthetic(&data, &issues, top_trunc, bot_trunc, stem)},
            TauIssue::SynTauModuleIssue => {
                // get_a_tau(&data, top_trunc, bot_trunc, stem)
                suggest_tau_solution_generator_synthetic(&data, &issues, top_trunc, bot_trunc, stem)
                }
            };
        
        if let Some(d) = option {
            return try_tau(data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded, d);
        } else {
            if depth <= PARALLEL_DEPTH && depth != 0 && let Some(getout) = &mut getout[(depth - 1) as usize] {
                *getout.lock().unwrap() = true;
            }
            return Err(format!("Issue at RP{bot_trunc}_{top_trunc}: {issues:?}"));
        }
    }

    // TODO: 
    // if bounded {
    //     return Ok(());
    // }
    
    if bot_trunc != 0 {
        return add_algebraic_diffs(data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded);
    }

    if top_trunc == stem + 1 {
        Ok(())
    } else {
        ahss_iterate(data, alg_ahss, alg_data, getout, log, stem, top_trunc + 1, top_trunc, depth, bounded, tau_mode)
    }
}

fn try_diff(mut data: SyntheticSS, alg_ahss: &SyntheticSS, alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>, mut getout: [Option<Arc<Mutex<bool>>>; PARALLEL_DEPTH as usize], log: &Arc<Mutex<Vec<Action>>>, stem: i32, top_trunc: i32, bot_trunc: i32, depth: i32, bounded: bool, d: Diff) -> Result<(), String> {
    let (from_name, to_name) = data.get_names(d.from, d.to);
    
    let filter = filter_diff(&data, alg_ahss, bot_trunc, top_trunc, d);
    
    if let Some((kind, reason)) = filter {
        if depth == 0 {
            log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof: Some(reason.clone()), kind });
        }
    
        data.add_diff(d.from, d.to, Some(reason), kind);
        return ahss_iterate(data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth, bounded, false)
    }


    if ALWAYS_PRINT || depth == 0 {
        println!("Trying diff: {} | {}", from_name, to_name);
    }    

    
    if depth < PARALLEL_DEPTH { 
        getout[depth as usize] = Some(Arc::new(Mutex::new(false)));
    }

    let with = || {
        let mut with_data = data.clone();
        with_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Real);
        ahss_iterate(with_data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth + 1, bounded, false)
    };
    let without = || {
        let mut without_data = data.clone();
        without_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Fake);
        ahss_iterate(without_data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth + 1, bounded, false)
    };

    if depth < PARALLEL_DEPTH {
        let (with_res, without_res) = rayon::join(
            with,
            without
        );
    
        if let Err(e) = with_res {
            // DISPROOF !
            let (from_name, to_name) = data.get_names(d.from, d.to);
        
            // Proof: Tau Issue: false | SyntheticConvergence { bot_trunc: 1, top_trunc: 30, stem: 30, af: 4, expected: [Torsion(Some(1))], observed: [Torsion(None)] }

            // Commit choice !
            if depth == 0 {
                println!("Disproven diff: {} | {} by {e}", from_name, to_name);            
                log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof: Some(e.clone()), kind: Kind::Fake });
            }
    
            data.add_diff(d.from, d.to, Some(e), Kind::Fake);
        
            // And iterate further
            getout[depth as usize] = None;
            return ahss_iterate(data, alg_ahss, alg_data, getout, log.clone(), stem, top_trunc, bot_trunc, depth, bounded, false)
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
                if kind == Kind::Unknown {
                    println!("Unknown diff: {} | {}", from_name, to_name);
                } else {
                    println!("Proven diff: {} | {} | {proof:?}", from_name, to_name);
                }
                log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof, kind });
            }
    
            getout[depth as usize] = None;
            return ahss_iterate(data, alg_ahss, alg_data, getout, log.clone(), stem, top_trunc, bot_trunc, depth, bounded, false)
        }

    } else {
        if let Err(e) = with() {
            // DISPROOF !
            let (from_name, to_name) = data.get_names(d.from, d.to);
        
            // Commit choice !
            if ALWAYS_PRINT || depth == 0 {
                println!("Disproven diff: {} | {} by {e}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof: Some(e.clone()), kind: Kind::Fake });
                getout = [const { None }; PARALLEL_DEPTH as usize];
            }
    
            data.add_diff(d.from, d.to, Some(e), Kind::Fake);
        
            // And iterate further
            return ahss_iterate(data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth, bounded, false)
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
    
            if ALWAYS_PRINT || depth == 0 {
                if kind == Kind::Unknown {
                    println!("Unknown diff: {} | {}", from_name, to_name);
                } else {
                    println!("Proven diff: {} | {} | {proof:?}", from_name, to_name);
                }
            }
            // Commit choice !
            if depth == 0 {
                log.lock().unwrap().push(Action::AddDiff { from: from_name, to: to_name, proof, kind });
                getout = [const { None }; PARALLEL_DEPTH as usize];
            }
    
            return ahss_iterate(data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth, bounded, false)
        }
    }
}

// Result Ok means fixable, Err is quit out
// Ok None means it is all fine!
// Ok Some gives back the errors

pub enum TauIssue {
    AlgTauIssue,
    SynTauGeneratorIssue,
    SynTauModuleIssue,
}

fn is_tau_issue(data: &SyntheticSS, stem: i32, top_trunc: i32, bot_trunc: i32, depth: i32) -> Result<Option<(TauIssue, Vec<Issue>)>, String> {
    match check_issue(data, stem, bot_trunc, top_trunc) {
        Ok(_) => { Ok(None) },
        Err(is) => {
            let all_synth_conv = if let Issue::SyntheticConvergence { bot_trunc, top_trunc, stem, af, expected, observed } = &is[0] {true} else {false};
            
            if all_synth_conv {
                let (solvable, generator) = synthetic_issue_is_tau_structure_issue(&is);
                if solvable {
                    if generator {
                        Ok(Some((TauIssue::SynTauGeneratorIssue, is)))
                    } else {
                        Ok(Some((TauIssue::SynTauModuleIssue, is)))
                    }
                } else {
                    Err(format!("For RP{bot_trunc}_{top_trunc} the F_2 vector space generators don't add up."))
                }
            } else {
                if algebraic_issue_is_fixable_by_tau_extensions(&is) {
                    Ok(Some((TauIssue::AlgTauIssue, is)))
                } else {
                    Err(format!("For RP{bot_trunc}_{top_trunc} there is no way to fix the algebraic convergence issues with tau extensions"))
                }
            }
        },
    }
}

fn try_tau(mut data: SyntheticSS, alg_ahss: &SyntheticSS, alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>, mut getout: [Option<Arc<Mutex<bool>>>; PARALLEL_DEPTH as usize], log: Arc<Mutex<Vec<Action>>>, stem: i32, top_trunc: i32, bot_trunc: i32, depth: i32, bounded: bool, d: ExtTauMult) -> Result<(), String> {
    let (from_name, to_name) = data.get_names(d.from, d.to);
    
    if ALWAYS_PRINT || depth == 0 {
        println!("Trying tau: {} | {} | af: {} | RP{bot_trunc}_{top_trunc}", from_name, to_name, d.af);
    }

    
    if depth < PARALLEL_DEPTH { 
        getout[depth as usize] = Some(Arc::new(Mutex::new(false)));
    }

    let with = || {
        let mut with_data = data.clone();
        with_data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Real);
        ahss_iterate(with_data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth + 1, bounded, true)
    };
    let without = || {
        let mut without_data = data.clone();
        without_data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Fake);
        ahss_iterate(without_data, alg_ahss, alg_data, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth + 1, bounded, true)
    };


    
    if depth < PARALLEL_DEPTH {
        let (with_res, without_res) = rayon::join(
            with,
            without
        );
    
        if let Err(e) = with_res {
            // DISPROOF !
            let (from_name, to_name) = data.get_names(d.from, d.to);
        
            // Commit choice !
            if depth == 0 {
                println!("Disproven tau: {} | {} by {e}", from_name, to_name);
                log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof: e.clone(), kind: Kind::Fake });
            }
        
            data.add_ext_tau(d.from, d.to, d.af, Some(e), Kind::Fake);
        
            // And iterate further
            getout[depth as usize] = None;
            return ahss_iterate(data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded, true)
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
                println!("Proven tau: {} | {} by {proof}", from_name, to_name);
                log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof, kind });
            }

            getout[depth as usize] = None;
            return ahss_iterate(data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded, true)
        }
    
    } else {
        if let Err(e) = with() {
            // DISPROOF !
            let (from_name, to_name) = data.get_names(d.from, d.to);
        
            // Commit choice !
            if depth == 0 {
                println!("Disproven tau: {} | {} by {e}", from_name, to_name);
                log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof: e.clone(), kind: Kind::Fake });
                getout = [const { None }; PARALLEL_DEPTH as usize];
            }
        
            data.add_ext_tau(d.from, d.to, d.af, Some(e), Kind::Fake);
        
            // And iterate further
            return ahss_iterate(data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded, true)
        } else {
            let without = without();
        
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
                println!("Proven tau: {} | {} by {proof}", from_name, to_name);
                log.lock().unwrap().push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof, kind });
                getout = [const { None }; PARALLEL_DEPTH as usize];
            }

            return ahss_iterate(data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded, true)
        }
    }
}

fn add_algebraic_diffs(mut data: SyntheticSS, alg_ahss: &SyntheticSS, alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>, getout: [Option<Arc<Mutex<bool>>>; PARALLEL_DEPTH as usize], log: Arc<Mutex<Vec<Action>>>, stem: i32, top_trunc: i32, bot_trunc: i32, depth: i32, bounded: bool) -> Result<(), String> {
    // As we are moving up a page for possible diffs,
    // we should add all adams differentials which could arise from here

    let d_y = top_trunc - bot_trunc + 1;
    for &(from, to) in &alg_data[stem as usize][d_y as usize][top_trunc as usize]  {
        if depth == 0 {
            let (from, to) = data.get_names(from, to);
            println!("Applying Algebraic diff {from} -> {to}");
        }
        data.add_diff(from, to, None, Kind::Real);
    }
    return ahss_iterate(data, alg_ahss, alg_data, getout.clone(), log, stem, top_trunc, bot_trunc-1, depth, bounded, false)
}


fn e1_to_ahss_loop(partial_ahss: &SyntheticSS, alg_ahss: &SyntheticSS, alg_data: &Vec<Vec<Vec<Vec<FromTo>>>>, mut log: Vec<Action>, stem: i32) -> (Result<(), String>, Vec<Action>) {
    let ahss = revert_log_and_remake(0, &mut log, &partial_ahss, true);
    let log = Arc::new(Mutex::new(log));
    let res = ahss_iterate(ahss, alg_ahss, &alg_data, [const { None }; PARALLEL_DEPTH as usize], log.clone(), stem, 2, 1, 0, false, false);

    (res, Arc::try_unwrap(log).unwrap().into_inner().unwrap())
}

fn e1_loop(ahss: SyntheticSS, partial_ahss: &SyntheticSS, alg_ahss: &SyntheticSS, alg_data: &Vec<Vec<Vec<Vec<FromTo>>>>, mut log: Vec<Action>, stem: i32) -> (Result<(), String>, Vec<Action>) {
    
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

            let res = e1_to_ahss_loop(partial_ahss, alg_ahss, alg_data, clone_log, stem);
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
        e1_to_ahss_loop(partial_ahss, alg_ahss, alg_data, log, stem)
    }
}

pub fn ahss_solver() -> (Vec<Action>, SyntheticSS) {
    let alg_ahss = STABLE_DATA.clone();
    let mut partial_ahss = SyntheticSS::empty(alg_ahss.model.clone());
    // We should add all d1's from the algebraic data
    
    let mut alg_data = vec![vec![vec![vec![]; (MAX_STEM + 2) as usize]; (MAX_STEM + 1) as usize]; (MAX_STEM + 1) as usize];

    for (&(from, to), _) in &alg_ahss.proven_from_to {
        let d_y = alg_ahss.model.y(from) - alg_ahss.model.y(to);
        let repeats = D_R_REPEATS[d_y as usize];
        if d_y == 1 || alg_ahss.model.y(to) - (repeats as i32) >= 1 {
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
        Action::AddDiff { from: "2 3 5 7 3 3[2]".to_string(), to: "2 4 1 1 2 4 3 3 3[1]".to_string(), proof: Some("Custom: By convergence on RP3_5 stem 26.".to_string()), kind: Kind::Real },


        Action::AddDiff { from: "3 5 7 3 3[10]".to_string(), to: "1 2 3 4 4 1 1 2 4 1 1 1[5]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Fake },
        // Action::AddDiff { from: "3 5 7 3 3[10]".to_string(), to: "2 3 4 4 1 1 2 4 1 1 1[6]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Fake },
        // Action::AddDiff { from: "[31]".to_string(), to: "1 2 3 3[21]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Real },
        // Action::AddDiff { from: "1[30]".to_string(), to: "4 4 1 1 1[19]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Real },
        // Action::AddDiff { from: "1 1[29]".to_string(), to: "12 1 1 1[15]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Real },
        // Action::AddDiff { from: "1 1 1[28]".to_string(), to: "2 3 4 4 1 1 1[14]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Real },
        
        // Action::AddDiff { from: "4 1 1 1[24]".to_string(), to: "1 2 3 4 4 1 1 1[13]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Real },
        // Action::AddDiff { from: "2 4 1 1 1[22]".to_string(), to: "4 4 1 1 2 4 1 1 1[11]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Real },
        // Action::AddDiff { from: "1 2 4 1 1 1[21]".to_string(), to: "8 4 1 1 2 4 1 1 1[7]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Real },
        // Action::AddDiff { from: "1 1 2 4 1 1 1[20]".to_string(), to: "2 3 4 4 1 1 2 4 1 1 1[6]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Real },
        // Action::AddDiff { from: "6 2 3 4 4 1 1 1[9]".to_string(), to: "6 2 2 4 5 3 3 3[2]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Real },
        
        // Action::AddExt { from: "1 2 3 3[21]".to_string(), to: "6 5 3[16]".to_string(), af: 5, proof: "Program can't figure this out.".to_string(), kind: Kind::Real },
        // Action::AddExt { from: "1 2 3 3[21]".to_string(), to: "13 3[14]".to_string(), af: 4, proof: "Program can't figure this out.".to_string(), kind: Kind::Real },
        // Action::AddExt { from: "4 4 1 1 1[19]".to_string(), to: "11 3 3[13]".to_string(), af: 5, proof: "Program can't figure this out.".to_string(), kind: Kind::Real },
        // Action::AddExt { from: "3 4 4 1 1 1[16]".to_string(), to: "9 3 3 3[12]".to_string(), af: 6, proof: "Program can't figure this out.".to_string(), kind: Kind::Real },
        // Action::AddExt { from: "2 3 4 4 1 1 1[14]".to_string(), to: "2 4 5 3 3 3[10]".to_string(), af: 8, proof: "Program can't figure this out.".to_string(), kind: Kind::Real },
        // Action::AddExt { from: "1 2 3 4 4 1 1 1[13]".to_string(), to: "6 2 3 4 4 1 1 1[8]".to_string(), af: 9, proof: "Program can't figure this out.".to_string(), kind: Kind::Real },
    ]; 
    
    for stem in 2..=31 {
        let ahss = revert_log_and_remake(0, &mut log, &partial_ahss, true);
        let (res, l) = e1_loop(ahss, &mut partial_ahss, &alg_ahss, &alg_data, log, stem);
        log = l;
        for dss in &alg_data[stem as usize] {
            for ds in dss {
                for &(from, to) in ds {
                    partial_ahss.add_diff(from, to, None, Kind::Real);
                }
            }
        }
        match res {
            Ok(_) => {
                println!("\n\nNEXT\n\n");
                
            },
            Err(e) => {
                println!("\n\nError on stem {stem}: {e}\n\n");
                break;
            },
        }
    }

    let ahss = revert_log_and_remake(0, &mut log, &partial_ahss, true);
    (log, ahss)
}