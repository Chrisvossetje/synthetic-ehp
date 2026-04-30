use core::panic;
use std::sync::{
        Arc, Mutex,
        atomic::AtomicBool
    };


use crate::{
    ALGEBRAIC_SPHERE_PAGES, MAX_STEM, STABLE_SYNTHETIC_PAGES, data::{
        compare::{EHP_TO_AHSS, S0, algebraic_spheres},
        curtis::DATA,
    }, domain::{
        model::{Diff, ExtTauMult, SyntheticSS},
        process::{compute_pages, ehp_recursion, try_compute_pages},
    }, solve::{
        action::{Action, process_action, revert_log_and_remake}, automated::{ALWAYS_PRINT, PARALLEL_DEPTH, TauIssue}, ehp_ahss::{in_metastable_range, set_metastable_range}, generate::get_a_diff, issues::{
            Issue, algebraic_issue_is_fixable_by_tau_extensions, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic, synthetic_issue_is_tau_structure_issue
        }, search::{BranchResult, ChoiceResult, SpeculativeBranchOutcome, branch_on_speculative_worlds, check_getout, create_getout, signal_parent_getout}, solve::{
            suggest_tau_solution_algebraic, suggest_tau_solution_generator_synthetic,
        }
    }, types::Kind
};

pub const MAX_DEPTH: i32 = 14;

enum Commitment {
    Real(String),
    Fake(String),
    Unknown,
}

enum FixNamesResult {
    Applied(Vec<Action>),
    Open,
    Cancelled,
}

fn commit_diff_choice(
    data: &mut SyntheticSS,
    log: &Arc<Mutex<Vec<Action>>>,
    depth: i32,
    d: Diff,
    commitment: Commitment,
) {
    let (from_name, to_name) = data.get_names(d.from, d.to);

    match commitment {
        Commitment::Fake(proof) => {
            if ALWAYS_PRINT || depth == 0 {
                println!("Disproven diff: {} | {} by {proof}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddDiff {
                    from: from_name,
                    to: to_name,
                    proof: Some(proof.clone()),
                    kind: Kind::Fake,
                });
            }
            data.add_diff(d.from, d.to, Some(proof), Kind::Fake);
        }
        Commitment::Real(proof) => {
            if ALWAYS_PRINT || depth == 0 {
                println!("Proven diff: {} | {} | {:?}", from_name, to_name, proof);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddDiff {
                    from: from_name,
                    to: to_name,
                    proof: Some(proof.clone()),
                    kind: Kind::Real,
                });
            }
            data.add_diff(d.from, d.to, Some(proof), Kind::Real);
        }
        Commitment::Unknown => {
            if ALWAYS_PRINT || depth == 0 {
                println!("Unknown diff: {} | {}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddDiff {
                    from: from_name,
                    to: to_name,
                    proof: None,
                    kind: Kind::Unknown,
                });
            }
            data.add_diff(d.from, d.to, None, Kind::Unknown);
        }
    }
}

fn commit_tau_choice(
    data: &mut SyntheticSS,
    log: &Arc<Mutex<Vec<Action>>>,
    depth: i32,
    d: ExtTauMult,
    commitment: Commitment,
) {
    let (from_name, to_name) = data.get_names(d.from, d.to);

    match commitment {
        Commitment::Fake(proof) => {
            if ALWAYS_PRINT || depth == 0 {
                println!("Disproven tau: {} | {} by {proof}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddExt {
                    from: from_name,
                    to: to_name,
                    af: d.af,
                    proof: proof.clone(),
                    kind: Kind::Fake,
                });
            }
            data.add_ext_tau(d.from, d.to, d.af, Some(proof), Kind::Fake);
        }
        Commitment::Real(proof) => {
            if ALWAYS_PRINT || depth == 0 {
                println!("Proven tau: {} | {} by {proof}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddExt {
                    from: from_name,
                    to: to_name,
                    af: d.af,
                    proof: proof.clone(),
                    kind: Kind::Real,
                });
            }
            data.add_ext_tau(d.from, d.to, d.af, Some(proof), Kind::Real);
        }
        Commitment::Unknown => {
            if ALWAYS_PRINT || depth == 0 {
                println!("Unknown tau: {} | {} by ", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddExt {
                    from: from_name,
                    to: to_name,
                    af: d.af,
                    proof: "".to_string(),
                    kind: Kind::Unknown,
                });
            }
            data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Unknown);
        }
    }
}

fn commit_induced_name_choice(
    data: &mut SyntheticSS,
    depth: i32,
    action: &mut Action,
    proof: String,
) -> Action {
    if let Action::SetInducedName { proof: action_proof, .. } = action {
        *action_proof = proof.clone();
    }

    if ALWAYS_PRINT || depth == 0 {
        println!("Choosing induced name: {:?} | because {proof}", action);
    }

    process_action(data, action, false).unwrap();
    action.clone()
}

fn check_issue(data: &SyntheticSS, stem: i32, sphere: i32) -> Result<(), Vec<Issue>> {
    let pages = if stem + 2 == sphere {
        let pages = try_compute_pages(data, 0, sphere - 1, stem, stem, true)?;

        let observed = pages.convergence_at_stem(data, stem);

        compare_synthetic(&observed, &S0, 0, sphere - 1, stem)?;
        
        compare_algebraic_spectral_sequence(data, &pages, stem, 0, sphere - 1, false)?;
        
        pages
    } else {
        let pages = try_compute_pages(data, 0, sphere - 1, stem - 1, stem, true)?;
        
        let observed = pages.algebraic_convergence_at_stem(data, stem);
        
        compare_algebraic(&observed, algebraic_spheres(sphere), 0, sphere - 1, stem)?;
        pages
    };
    
    
    for &f_id in data.model.gens_id_in_stem(stem) {
        if let Some(t_id) = EHP_TO_AHSS[f_id] && STABLE_SYNTHETIC_PAGES.get().unwrap()[(sphere - 1) as usize].element_in_pages(t_id) {
            if let Some(ps) = &pages.generators[f_id] {
                for (f_page, (f_af, f_torsion)) in ps {
                    if f_torsion.alive() {
                        let (t_af, t_torsion) = STABLE_SYNTHETIC_PAGES.get().unwrap()[(sphere - 1) as usize].element_at_page(*f_page, t_id);

                        if !t_torsion.can_map_with_coeff(&f_torsion, t_af - f_af) {
                            return Err(vec![
                                Issue::InvalidEHPAHSSMap { 
                                    name: data.model.name(f_id).to_string(), 
                                    from_torsion: *f_torsion,
                                    to_torsion: t_torsion,
                                    stem, 
                                    sphere 
                                }
                            ]);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn filter_diff(
    data: &SyntheticSS,
    alg_ehp: &SyntheticSS,
    bot_trunc: i32,
    top_trunc: i32,
    d: Diff,
) -> Option<(Kind, String)> {
    let stem = data.model.stem(d.to);
    let y = data.model.y(d.from);

    if y == 3 || y == 7 {
        Some((
            Kind::Fake,
            format!(
                "By Hopf Invariant one things we cannot have a differential form the 2^i-1 sphere"
            ),
        ))
    } else if data.in_diffs[d.to]
        .iter()
        .any(|from| data.model.y(*from) == top_trunc && data.model.original_torsion(*from).alive())
    {
        Some((
            Kind::Unknown,
            format!(
                "As we are only interested in the module structure, we won't consider the case where two differentials target the same generator."
            ),
        ))
    } else if bot_trunc & 1 == 0
        && let Some(alg_to) = alg_ehp.out_diffs[d.from].first()
        && data.model.original_torsion(*alg_to).alive()
        && data.model.y(*alg_to) + 1 == bot_trunc
    {
        Some((
            Kind::Unknown,
            format!("We don't have enough Synthetic information to deduce this differential."),
        ))
    } else if top_trunc & 1 == 1
        && let Some(dies) = data.model.get(d.to).dies
        && let Some(source) = alg_ehp.in_diffs[d.to].first()
        && data.model.original_torsion(*source).free()
        && top_trunc + 2 == dies
    {
        Some((
            Kind::Unknown,
            format!("We don't have enough Synthetic information to deduce this differential."),
        ))
    } else if data.out_diffs[d.from].last().map(|x| if let Some(target) = data.out_taus[*x].last() { *target != d.to } else {false}).unwrap_or(false) {
        Some((
            Kind::Unknown,
            format!("Can't have a differential which earlier hits some tau multiple and then later does not."),
        ))
    } else {
        None
    }
}

fn filter_tau(
    data: &SyntheticSS,
    alg_ahss: &SyntheticSS,
    bot_trunc: i32,
    top_trunc: i32,
    d: ExtTauMult,
) -> Option<(Kind, String)> {
    let stem = data.model.stem(d.to);
    let y = data.model.y(d.from);

    if data.model.original_torsion(d.from).free() {
        Some((
            Kind::Fake,
            format!("We have not enough information to see the difference between this tau extension and having the incoming differential just go to the thing this tau extension is targeting."),
        ))
    } else {
        None
    }
}

fn get_first_non_metastable_range(stem: i32, top_trunc: i32) -> i32 {
    if in_metastable_range(top_trunc, stem) {
        if top_trunc == (stem / 3) + 1 {
            stem / 3
        } else {
            (stem / 3) + 1
        }
    } else {
        top_trunc - 1
    }
}

fn ehp_iterate(
    mut data: SyntheticSS,
    alg_ehp: &SyntheticSS,
    ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize, Kind, Option<String>)>>>>,
    mut getout: [Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize],
    log: Arc<Mutex<Vec<Action>>>,
    mut stem: i32,
    mut top_trunc: i32,
    mut bot_trunc: i32,
    mut depth: i32,
) -> BranchResult {
    loop {
        if depth == 0 && stem >= 47 {
            return BranchResult::Open;
        }

        if depth > MAX_DEPTH || stem >= 48 {
            return BranchResult::Open;
        }

        if check_getout(&getout) {
            return BranchResult::Cancelled;
        }

        if bot_trunc != 0 {
            let option = get_a_diff(&data, top_trunc, bot_trunc, stem);

            // Should only need first option here
            if let Some(d) = option {
                match try_diff(
                    &mut data,
                    alg_ehp,
                    ahss_and_alg_data,
                    &getout,
                    &log,
                    stem,
                    top_trunc,
                    bot_trunc,
                    depth,
                    d,
                ) {
                    ChoiceResult::Chosen => {
                        continue;
                    },
                    ChoiceResult::Open => {
                        return BranchResult::Open;
                    },
                    ChoiceResult::Cancelled => {
                        return BranchResult::Cancelled;
                    },
                }
            }
        } else {
            let potential_tau_thing = match is_tau_issue(&data, stem, top_trunc + 1) {
                Ok(tau_issue) => tau_issue,
                Err(is) => {
                    signal_parent_getout(&mut getout, depth);
                    return BranchResult::Contradiction(is);
                }
            };


            if let Some((synthetic, mut issues)) = potential_tau_thing {
                let option = match synthetic {
                    TauIssue::AlgTauIssue => suggest_tau_solution_algebraic(
                        &data,
                        &mut issues,
                        top_trunc,
                        bot_trunc,
                        stem,
                    ),
                    TauIssue::SynTauGeneratorIssue => suggest_tau_solution_generator_synthetic(
                        &data,
                        &mut issues,
                        top_trunc,
                        bot_trunc,
                        stem,
                    ),
                    TauIssue::SynTauModuleIssue => suggest_tau_solution_generator_synthetic(
                        &data,
                        &mut issues,
                        top_trunc,
                        bot_trunc,
                        stem,
                    ),
                };

                if let Some(d) = option {
                    match try_tau(
                        &mut data,
                        alg_ehp,
                        ahss_and_alg_data,
                        &getout,
                        &log,
                        stem,
                        top_trunc,
                        bot_trunc,
                        depth,
                        d,
                    ) {
                        ChoiceResult::Chosen => {
                            continue;
                        },
                        ChoiceResult::Open => {
                            return BranchResult::Open;
                        },
                        ChoiceResult::Cancelled => {
                            return BranchResult::Cancelled;
                        },
                    }
                } else {
                    signal_parent_getout(&mut getout, depth);

                    return BranchResult::Contradiction(format!(
                        "Issue at S^{} | stem {}: {issues:?}",
                        top_trunc + 1,
                        stem
                    ));
                }
            }
        }

        
        if bot_trunc != 0 {
            // TODO: Do something wth this result ?
            let _ = add_diffs(
                &mut data,
                ahss_and_alg_data,
                stem,
                &log,
                top_trunc,
                bot_trunc,
                depth,
            );
            bot_trunc -= 1;
            continue;
        } else {
            if top_trunc & 1 == 0 && (top_trunc/2) + stem < MAX_STEM {
                match fix_names(&mut data,
                    alg_ehp,
                    ahss_and_alg_data,
                    &getout,
                    &log,
                    stem,
                    top_trunc,
                    bot_trunc,
                    depth) {
                        Ok(i) => match i {
                            FixNamesResult::Applied(mut actions) => {
                                if depth == 0 {
                                    log.lock().unwrap().append(&mut actions);
                                }
                            }
                            FixNamesResult::Cancelled => return BranchResult::Cancelled,
                            FixNamesResult::Open => return BranchResult::Open,
                        },
                        Err(e) => return BranchResult::Contradiction(e),
                    }
            }
        }
        
        if top_trunc & 1 == 0 && top_trunc <= stem && (top_trunc/2) + stem < MAX_STEM {
            let res = ehp_recursion(&mut data, top_trunc + 1, stem).map_err(|x| format!("{x:?}"));
            if res.is_err() {
                panic!();
            }
        }
        
        
        // Stupid complicated formula
        // But all this works out to always traverse all sphere in such a manner that the recursion is always "just in time"
        // Aka, we move through the SS in a slanted way
        // Sadly this also means that we can only use the synthetic knowledge "very late"
        // But we have the benifit of seeing tau torsion like issues early
        // if top_trunc == 1 {
        //     top_trunc = 2*((stem+2)/3);
        //     stem = ((2*(stem+1))+1)/3;
        // } else {
        //     if top_trunc == 3 || (top_trunc & 1 == 0 && top_trunc == stem + 1) {
        //         stem += 1;
        //         top_trunc -= 2;
        //     } else if top_trunc & 1 == 0 {
        //         top_trunc += 1;
        //     } else {
        //         stem += 1;
        //         top_trunc -= 3;
        //     }
        // }


        // Slightly less complicated formula
        if top_trunc >= stem + 1 {
            stem += 1;
            top_trunc = 4;
        } else if top_trunc == 5 {
            stem += 1;
            top_trunc = 2;
        } else if top_trunc == 3 {
            stem += 1;
            top_trunc = 1;
        } else if top_trunc == 1 {
            stem -= 2;
            top_trunc = 6;
        } else {
            top_trunc += 1;
        }


        // // Simple formula
        // if top_trunc == stem + 1 {
        //     stem += 1;
        //     top_trunc = 2;
        // } else {
        //     top_trunc += 1;
        // }





        bot_trunc = get_first_non_metastable_range(stem, top_trunc);
        if depth == 0 {
            println!("Current stem: {stem} | top_trunc: {top_trunc}");
        }
    }
}

fn fix_names(
    data: &mut SyntheticSS,
    alg_ehp: &SyntheticSS,
    ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize, Kind, Option<String>)>>>>,
    getout: &[Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize],
    log: &Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
) -> Result<FixNamesResult, String> {
    let sphere = top_trunc + 1;

    let (pages, _) = compute_pages(data, 0, sphere - 1, stem, stem, true);
    let alg_pages = &ALGEBRAIC_SPHERE_PAGES.get().unwrap()[sphere as usize];

    let mut issues = vec![];

    for &id in data.model.gens_id_in_stem(stem) {
        // Synthetic Generators
        if let Some((id_af, id_torsion)) = pages.try_element_final(id)
            && id_torsion.alive()
        {
            let ind_name = data.get_name_at_sphere(id, sphere);
            let g = data.model.get_name(ind_name);
            if let Some(dies) = g.dies && dies <= sphere {
                // PROBLEM 

                // Here we should already try to solve this shit
                issues.push(Issue::InvalidName {
                    original_name: data.model.name(id).to_string(),
                    unexpected_name: ind_name.to_string(),
                    sphere,
                    stem,
                    af: id_af,
                });  
            } else if id_af != g.af {
                // Here we should already try to solve this shit
                issues.push(Issue::InvalidName {
                    original_name: data.model.name(id).to_string(),
                    unexpected_name: ind_name.to_string(),
                    sphere,
                    stem,
                    af: id_af,
                });  
            } 
        }
    }

    let mut sols = vec![];

    for i in &issues {
        if let Issue::InvalidName {
            original_name,
            unexpected_name,
            sphere,
            stem,
            af,
        } = i
        {
            // Problem things
            let mut syn = vec![];
            let mut alg = vec![];
            for id in DATA.model.gens_id_in_stem(*stem) {
                if pages.element_in_pages(*id) {
                    let g = pages.element_final(*id);
                    if g.1.alive() && g.0 == *af {
                        let name = data.get_name_at_sphere(*id, *sphere);
                        syn.push(name);
                    }
                }

                if alg_pages.element_in_pages(*id) {
                    let g = alg_pages.element_final(*id);
                    if g.1.alive() && g.0 == *af {
                        let name = DATA.model.name(*id);
                        alg.push(name);
                    }
                }
            }

            let fil_syn: Vec<_> = syn.iter().filter(|i| !alg.contains(i)).collect();
            let fil_alg: Vec<_> = alg.iter().filter(|i| !syn.contains(i)).collect();

            if fil_alg.len() == 0 {
                    return Err(format!("This should have been seen as an algebraic convergence issue"))
            }
            if fil_syn.len() == 1 && fil_alg.len() == 1 {
                let name = fil_alg[0];
                let action = Action::SetInducedName {
                    name: original_name.clone(),
                    new_name: name.to_string(),
                    sphere: *sphere,
                    proof: format!(
                        "We must have that {original_name} is represented by {name} as it is the only unique choice."
                    ),
                };

                process_action(data, &action, false).unwrap();
                sols.push(action);
            } else {

                // TODO: i have to collect all the problems and THEN fix the stuffzies


                // Go do a branching search to find best candidates
                if fil_syn.len() == 1 && fil_alg.len() == 2 {
                    let g = create_getout(getout, depth);

                    let mut a_action = Action::SetInducedName {
                            name: original_name.clone(),
                            new_name: fil_alg[0].to_string(),
                            sphere: *sphere,
                            proof: format!(
                                ""
                            ),
                        };
                    let mut b_action = Action::SetInducedName {
                            name: original_name.clone(),
                            new_name: fil_alg[1].to_string(),
                            sphere: *sphere,
                            proof: format!(
                                ""
                            ),
                        };

                    let a = || {
                        let mut with_data = data.clone();
                        process_action(&mut with_data, &a_action, false).unwrap();

                        ehp_iterate(
                            with_data,
                            alg_ehp,
                            ahss_and_alg_data,
                            g.clone(),
                            log.clone(),
                            *stem,
                            top_trunc,
                            bot_trunc,
                            depth + 1,
                        )
                    };
                    let b = || {
                        let mut without_data = data.clone();
                        process_action(&mut without_data, &b_action, false).unwrap();

                        ehp_iterate(
                            without_data,
                            alg_ehp,
                            ahss_and_alg_data,
                            g.clone(),
                            log.clone(),
                            *stem,
                            top_trunc,
                            bot_trunc,
                            depth + 1,
                        )
                    };

                    if ALWAYS_PRINT || depth == 0 {
                        println!("Trying Induced name for {}", fil_syn[0]);
                    }

                    match branch_on_speculative_worlds(depth, a, b) {
                        SpeculativeBranchOutcome::ChooseRight(e) => {
                            let action = commit_induced_name_choice(data, depth, &mut b_action, e);
                            sols.push(action);
                        }
                        SpeculativeBranchOutcome::ChooseLeft(e) => {
                            let action = commit_induced_name_choice(data, depth, &mut a_action, e);
                            sols.push(action);
                        }
                        SpeculativeBranchOutcome::Cancelled => {
                            return Ok(FixNamesResult::Cancelled);
                        }
                        SpeculativeBranchOutcome::BothOpen => {
                            return Ok(FixNamesResult::Open);
                            // if depth == 0 {
                            //     unreachable!("Induced-name branching should distinguish the two choices.");
                            // } else {
                            //     // What should i do here ?
                            //     // I probably can't see the result yet ?
                            //     let action = commit_induced_name_choice(data, depth, &mut a_action, "".to_string());
                            //     sols.push(action);
                            // }
                        }
                    }

                } else {
                    if depth == 0 {
                        println!("{i:?}");
                        println!("{fil_syn:?}");
                        println!("{fil_alg:?}");
    
                        println!("This is not necc. an error. It means i DO have to think harder and split on multiple name choices :(.
                        But i probably want to do this manual anyway. Implementing this logic for the few cases where it occurs might not be worth it.");  
                    }

                    return Err(format!("We have two unknowns in one degree, we probably need to make better differential choices. Syn: {fil_syn:?} | Alg: {fil_alg:?} | Issue: {i:?}"));
                }
            }
        }
    }

    Ok(FixNamesResult::Applied(sols))
}

fn try_diff(
    data: &mut SyntheticSS,
    alg_ehp: &SyntheticSS,
    ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize, Kind, Option<String>)>>>>,
    getout: &[Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize],
    log: &Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
    d: Diff,
) -> ChoiceResult {
    let (from_name, to_name) = data.get_names(d.from, d.to);

    let filter = filter_diff(&data, alg_ehp, bot_trunc, top_trunc, d);

    if let Some((kind, reason)) = filter {
        if depth == 0 {
            log.lock().unwrap().push(Action::AddDiff {
                from: from_name,
                to: to_name,
                proof: Some(reason.clone()),
                kind,
            });
        }

        let (from_name, to_name) = data.get_names(d.from, d.to);

        if ALWAYS_PRINT || depth == 0 {
            println!(
                "Finished diff by: {} | {} -> {kind:?} + {reason}",
                from_name, to_name
            );
        }
        data.add_diff(d.from, d.to, Some(reason), kind);
        return ChoiceResult::Chosen;
    }

    if ALWAYS_PRINT || depth == 0 {
        println!("Trying diff: {} | {}", from_name, to_name);
    }

    let g = create_getout(getout, depth);

    let with = || {
        // println!("{depth} With Trying diff: {} | {}", from_name, to_name);
        let mut with_data = data.clone();
        with_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Real);
        ehp_iterate(
            with_data,
            alg_ehp,
            ahss_and_alg_data,
            g.clone(),
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
        )
    };
    let without = || {
        // println!("{depth} Without Trying diff: {} | {}", from_name, to_name);
        let mut without_data = data.clone();
        without_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Fake);
        ehp_iterate(
            without_data,
            alg_ehp,
            ahss_and_alg_data,
            g.clone(),
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
        )
    };

    match branch_on_speculative_worlds(depth, with, without) {
        SpeculativeBranchOutcome::ChooseRight(e) => {
            commit_diff_choice(data, log, depth, d, Commitment::Fake(e));
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::ChooseLeft(e) => {
            commit_diff_choice(data, log, depth, d, Commitment::Real(e));
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::BothOpen => {
            // In this case it merely means that i don't know what happened for sure
            // if im in depth == 0 i HAVE to make a choice
            // Else i just return and try to follow the other branch 
            commit_diff_choice(data, log, depth, d, Commitment::Unknown);
            ChoiceResult::Open
        }
        SpeculativeBranchOutcome::Cancelled => ChoiceResult::Cancelled,
    }
}

fn is_tau_issue(
    data: &SyntheticSS,
    real_stem: i32,
    sphere: i32,
) -> Result<Option<(TauIssue, Vec<Issue>)>, String> {
    match check_issue(data, real_stem, sphere) {
        Ok(_) => Ok(None),
        Err(is) => {
            let all_synth_conv = if let Issue::SyntheticConvergence {
                bot_trunc,
                top_trunc,
                stem,
                af,
                expected,
                observed,
            } = &is[0]
            {
                true
            } else {
                false
            };

            if all_synth_conv {
                let (solvable, generator) = synthetic_issue_is_tau_structure_issue(&is);
                if solvable {
                    if generator {
                        Ok(Some((TauIssue::SynTauGeneratorIssue, is)))
                    } else {
                        Ok(Some((TauIssue::SynTauModuleIssue, is)))
                    }
                } else {
                    Err(format!(
                        "For the stable Sphere the F_2 vector space generators don't add up. {is:?}"
                    ))
                }
            } else {
                if algebraic_issue_is_fixable_by_tau_extensions(&is) {
                    Ok(Some((TauIssue::AlgTauIssue, is)))
                } else {
                    Err(format!(
                        "For S^{sphere} there is no way to fix the algebraic convergence issues with tau extensions. {is:?}"
                    ))
                }
            }
        }
    }
}

fn try_tau(
    data: &mut SyntheticSS,
    alg_ehp: &SyntheticSS,
    ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize, Kind, Option<String>)>>>>,
    getout: &[Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize],
    log: &Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
    d: ExtTauMult,
) -> ChoiceResult {
    let (from_name, to_name) = data.get_names(d.from, d.to);

    let filter  = filter_tau(data, alg_ehp, bot_trunc, top_trunc, d);

    if let Some((kind, reason)) = filter {
        if depth == 0 {
            log.lock().unwrap().push(Action::AddExt {
                from: from_name,
                to: to_name,
                af: d.af,
                proof: reason.clone(),
                kind,
            });
        }

        let (from_name, to_name) = data.get_names(d.from, d.to);

        
        if ALWAYS_PRINT || depth == 0 {
            println!(
                "Finished tau by: {} | {} -> {kind:?} + {reason}",
                from_name, to_name
            );
        }
        data.add_ext_tau(d.from, d.to, d.af, Some(reason), kind);
        return ChoiceResult::Chosen;
    }

    if ALWAYS_PRINT || depth == 0 {
        println!(
            "Trying tau: {} | {} | af: {} | S^{}",
            from_name, to_name, d.af, top_trunc + 1
        );
    }
    
    let g = create_getout(getout, depth);

    let with = || {
        let mut with_data = data.clone();
        with_data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Real);
        ehp_iterate(
            with_data,
            alg_ehp,
            ahss_and_alg_data,
            g.clone(),
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
        )
    };
    let without = || {
        let mut without_data = data.clone();
        without_data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Fake);
        ehp_iterate(
            without_data,
            alg_ehp,
            ahss_and_alg_data,
            g.clone(),
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
        )
    };

    match branch_on_speculative_worlds(depth, with, without) {
        SpeculativeBranchOutcome::ChooseRight(e) => {
            commit_tau_choice(data, log, depth, d, Commitment::Fake(e));
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::ChooseLeft(e) => {
            commit_tau_choice(data, log, depth, d, Commitment::Real(e));
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::BothOpen => {
            // commit_tau_choice(data, log, depth, d, Commitment::Unknown);
            ChoiceResult::Open
        }
        SpeculativeBranchOutcome::Cancelled => {
            ChoiceResult::Cancelled
        },
    }
}

fn add_diffs(
    data: &mut SyntheticSS,
    ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize, Kind, Option<String>)>>>>,
    stem: i32,
    log: &Arc<Mutex<Vec<Action>>>,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
) -> Result<(), String> {
    // As we are moving up a page for possible diffs,
    // we should add all AHSS / adams differentials which could arise from here

    
    let d_y = top_trunc - bot_trunc + 1;


    for (from, to, k, p) in &ahss_and_alg_data[stem as usize][d_y as usize][top_trunc as usize] {    
        if let Some(p) = p {
            let pages = try_compute_pages(&data, 0, top_trunc, stem, stem, false).map_err(|x| format!("{x:?}"))?;

            if let Some((f_af, f_torsion)) = pages.try_element_final(*from) && f_torsion.alive() {
                if let Some((t_af, t_torsion)) = pages.try_element_final(*to) && t_torsion.alive() {
                    let coeff = t_af - f_af - 1;
                    // if t_torsion.can_map_with_coeff(&f_torsion, coeff) {

                        if depth == 0 {
                            let (from_name, to_name) = data.get_names(*from, *to);
                            if ALWAYS_PRINT || depth == 0 {
                                println!("Lifted diff: {} | {}", from_name, to_name);
                            }
                            log.lock().unwrap().push(Action::AddDiff {
                                from: from_name,
                                to: to_name,
                                proof: Some(format!("(Lifted from AHSS) - {:?}", p)),
                                kind: *k,
                            });
                        }
                        // If added torsion is invalid we will see it in useless diff / invalid torsion / EHPAHSS map error
                        data.add_diff(*from, *to, Some("".to_string()), *k);
                    // } else {
                    //     return Err(format!("We want to lift the diff from {} to {}. However, this would give ")
                    //     println!("stem: {stem} | {:?}", data.get_names(*from, *to));
                    //     panic!("");
                    // }
                } else {

                    // This is too strong! as i have not yet given EHP the chance to find differentials which could support this.
                    // What we should check is that AFTER a top/bot truncation have been done, that it is compatible at each page wrt. torsion structure stuff ?
                    // return Err(format!("We have at E_{} that {} is alive while {} is dead. This is not compatible with the AHSS / AEHP.", top_trunc-bot_trunc, data.model.name(*from), data.model.name(*to)));
                }
            }
        } else {
            data.add_diff(*from, *to, None, *k);
        }
    }

    Ok(())
}

pub fn ehp_solver(ahss: &SyntheticSS, log: Option<Vec<Action>>) -> (Vec<Action>, SyntheticSS) {
    let alg_ehp = DATA.clone();
    let mut partial_ehp = SyntheticSS::empty(alg_ehp.model.clone());
    // We should add all d1's from the algebraic data

    let _ = set_metastable_range(&mut partial_ehp, ahss);

    let mut ahss_and_alg_data =
        vec![
            vec![vec![vec![]; (MAX_STEM + 2) as usize]; (MAX_STEM + 1) as usize];
            (MAX_STEM + 1) as usize
        ];

        
    let mut log = log.unwrap_or(vec![
        
    ]);


    // Add EHP Algebraic Diffs
    for (&(from, to), _) in &alg_ehp.proven_from_to {
        let d_y = alg_ehp.model.y(from) - alg_ehp.model.y(to);
        // Exclude metastable ones, as they have already been added
        if !in_metastable_range(alg_ehp.model.y(to), alg_ehp.model.stem(to)) {
            // if in_metastable_range(alg_ehp.model.y(to) + 1, alg_ehp.model.stem(to)) {
            //     partial_ehp.add_diff(from, to, None, Kind::Real);
            // } else {
                if d_y == 1 {
                    partial_ehp.add_diff(from, to, None, Kind::Real);
                } else {
                    let stem = alg_ehp.model.stem(to);
                    let top_trunc = alg_ehp.model.y(from);
                    ahss_and_alg_data[stem as usize][d_y as usize][top_trunc as usize].push((
                        from,
                        to,
                        Kind::Real,
                        None,
                    ));
                }
            // }
        }
    }

    // Add compatible AHSS diffs
    for (&(from, to), proof) in &ahss.proven_from_to {
        let d_y = ahss.model.y(from) - ahss.model.y(to);
        if let Some(from_id) = alg_ehp.model.try_index(ahss.model.name(from)) {
            if let Some(to_id) = alg_ehp.model.try_index(ahss.model.name(to)) {
                // Differentials
                if ahss.model.stem(from) != ahss.model.stem(to) {
                    // Exclude metastable ones, as they have already been added
                    if !in_metastable_range(ahss.model.y(to), ahss.model.stem(to)) {
                        // if in_metastable_range(ahss.model.y(to) + 1, ahss.model.stem(to)) {
                        //     if let Some(p) = proof {
                        //         partial_ehp.add_diff(
                        //             from_id,
                        //             to_id,
                        //             Some(format!("Lifted from Stable EHP")),
                        //             Kind::Real,
                        //         );
                        //     }
                        // } else {
                            // Don't include the Algebraic diffs of AHSS
                            if let Some(p) = proof && ahss.model.name(from) != "5 6 5 2 3 5 7 7[4]" {
                                if d_y == 1 {
                                    let (from_name, to_name) = alg_ehp.get_names(from_id,to_id);
                                    log.push(Action::AddDiff { 
                                        from: from_name, 
                                        to: to_name, 
                                        kind: Kind::Real, 
                                        proof: Some(format!("(Lifted from AHSS) - {:?}", p)) 
                                    });
                                } else {
                                    let stem = alg_ehp.model.stem(to_id);
                                    let top_trunc = alg_ehp.model.y(from_id);
                                    ahss_and_alg_data[stem as usize][d_y as usize][top_trunc as usize]
                                        .push((
                                            from_id,
                                            to_id,
                                            Kind::Real,
                                            Some(format!("(Lifted from AHSS) - {:?}", p)),
                                        ));
                                }
                            }
                        // }
                    }
                }
            }
        }
    }

    // Add disproven compatible AHSS diffs
    for (&(from, to), proof) in &ahss.disproven_from_to {
        let d_y = ahss.model.y(from) - ahss.model.y(to);

        // Only add differentials here
        if ahss.model.stem(from) != ahss.model.stem(to) {
            if let Some(from_id) = alg_ehp.model.try_index(ahss.model.name(from)) {
                if let Some(to_id) = alg_ehp.model.try_index(ahss.model.name(to)) {
                    // Don't include the Unknown differentials
                    if let Some(p) = proof {
                        let (from_name, to_name) = alg_ehp.get_names(from_id,to_id);
                        log.push(Action::AddDiff { 
                            from: from_name, 
                            to: to_name, 
                            kind: Kind::Fake, 
                            proof: Some(format!("(Lifted from AHSS) - {:?}", p)) 
                        });
                        // partial_ehp.add_diff(
                        //     from_id,
                        //     to_id,
                        //     Some(format!("Lifted from Stable EHP")),
                        //     Kind::Fake,
                        // );
                    }
                }
            }
        }
    }

    // let mut fakes = vec![];

    // Add all external tau's
    // We won't worry about the fake ones
    for esss in &ahss.external_tau_page {
        for ess in esss {
            for es in ess {
                for e in es {
                    if let Some(from_id) = alg_ehp.model.try_index(ahss.model.name(e.from)) {
                        if let Some(to_id) = alg_ehp.model.try_index(ahss.model.name(e.to)) {
                            let p = ahss.proven_from_to.get(&(e.from, e.to)).unwrap().as_ref().unwrap();
                            log.push(
                                Action::AddExt { 
                                    from: ahss.model.name(e.from).to_string(), 
                                    to: ahss.model.name(e.to).to_string(), 
                                    af: e.af, kind: Kind::Real, 
                                    proof: format!("(Lifted from AHSS) - {:?}", p)
                                }
                            );
                        // } else {
                        //     if ahss.out_diffs[e.to].len() > 0 {
                        //         fakes.push((ahss.model.stem(e.from), ahss.model.name(e.from), ahss.model.name(e.to)));
                        //     }
                        }
                    }
                } 
            }
        }
    }   

    // fakes.sort();
    // for f in fakes {
    //     println!("AHSS Ext lift potential (stem {}) : {} -> {}", f.0, f.1, f.2);
    // }

    // panic!();

    let ehp = revert_log_and_remake(0, &mut log, &partial_ehp, false);
    let log = Arc::new(Mutex::new(log));

    let res = ehp_iterate(
        ehp,
        &alg_ehp,
        &ahss_and_alg_data,
        [const { None }; PARALLEL_DEPTH as usize],
        log.clone(),
        2,
        2,
        1,
        0,
    );

    // Add EHP Algebraic Diffs
    for (&(from, to), _) in &alg_ehp.proven_from_to {
        partial_ehp.add_diff(from, to, None, Kind::Real);
    }

    println!("{res:?}");

    let mut log = Arc::try_unwrap(log).unwrap().into_inner().unwrap();
    let ehp = revert_log_and_remake(0, &mut log, &partial_ehp, false);
    (log, ehp)
}
