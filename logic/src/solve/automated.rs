use std::sync::{
    Arc, Mutex,
};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    MAX_STEM, MAX_VERIFY_STEM, data::{
        compare::{RADON_HURWITZ_NUMBERS, algebraic_rp, rp_truncations, synthetic_rp},
        curtis::{STABLE_DATA, STABLE_MODEL}, naming::generate_names_from_tag,
    }, domain::{
        e1::E1,
        model::{Diff, ExtTauMult, SyntheticSS},
        process::compute_pages,
    }, solve::{
        action::{Action, D_R_REPEATS, process_action, revert_log_and_remake},
        ahss::ahss_synthetic_e1_issue,
        ahss_e1::get_all_e1_solutions,
        generate::get_a_diff,
        issues::{
            Issue, algebraic_issue_is_fixable_by_tau_extensions, compare_algebraic,
            compare_algebraic_spectral_sequence, compare_synthetic,
            synthetic_issue_is_tau_structure_issue,
        },
        search::{
            BranchResult, ChoiceResult, GetOut, SpeculativeBranchOutcome, branch_on_speculative_worlds, check_getout, create_getout, empty_getout, signal_parent_getout
        },
        solve::{
            auto_deduce, suggest_tau_solution_algebraic, suggest_tau_solution_generator_synthetic,
        },
    }, types::{Kind, Torsion}
};

pub const PARALLEL_DEPTH: i32 = 6;
pub const ALWAYS_PRINT: bool = false;
pub const MAX_DEPTH: i32 = 10;

enum Commitment {
    Real(String),
    Fake(String),
    Unknown,
}

fn commit_diff_choice(
    data: &mut SyntheticSS,
    model: &E1,
    log: &Arc<Mutex<Vec<Action>>>,
    depth: i32,
    d: Diff,
    commitment: Commitment,
) {
    let (from_name, to_name) = model.get_names(d.from, d.to);

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
            data.add_diff(model, d.from, d.to, Some(proof), Kind::Fake);
        }
        Commitment::Real(proof) => {
            if ALWAYS_PRINT || depth == 0 {
                println!("Proven diff: {} | {} | {:?}", from_name, to_name, proof);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddDiff {
                    from: from_name.clone(),
                    to: to_name.clone(),
                    proof: Some(proof.clone()),
                    kind: Kind::Real,
                });
            }

            let action = Action::AddDiff {
                from: model.name(d.from).to_string(),
                to: model.name(d.to).to_string(),
                kind: Kind::Real,
                proof: Some(proof)
            };

            let _ = process_action(data, model, &action, true);
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
            data.add_diff(model, d.from, d.to, None, Kind::Unknown);
        }
    }
}

fn commit_tau_choice(
    data: &mut SyntheticSS,
    model: &E1,
    log: &Arc<Mutex<Vec<Action>>>,
    depth: i32,
    d: ExtTauMult,
    commitment: Commitment,
) {
    let (from_name, to_name) = model.get_names(d.from, d.to);

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
                    proof: Some(proof),
                    kind: Kind::Fake,
                });
            }

            let action = Action::AddExt {
                from: model.name(d.from).to_string(),
                to: model.name(d.to).to_string(),
                af: d.af,
                proof: None,
                kind: Kind::Fake,
            };

            let _ = process_action(data, model, &action, true);
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
                    proof: Some(proof),
                    kind: Kind::Real,
                });
            }

            let action = Action::AddExt {
                from: model.name(d.from).to_string(),
                to: model.name(d.to).to_string(),
                af: d.af,
                kind: Kind::Real,
                proof: None
            };

            let _ = process_action(data, model, &action, true);
        }
        Commitment::Unknown => {
            if ALWAYS_PRINT || depth == 0 {
                println!("Unknown tau: {} | {}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddExt {
                    from: from_name,
                    to: to_name,
                    af: d.af,
                    proof: None,
                    kind: Kind::Unknown,
                });
            }
            data.add_ext_tau(model, d.from, d.to, d.af, Some("".to_string()), Kind::Unknown);
        }
    }
}

fn check_issue(
    data: &SyntheticSS,
    model: &E1,
    stem: i32,
    bot_trunc: i32,
    top_trunc: i32,
) -> Result<(), Vec<Issue>> {
    for &(synthetic, bt, tt) in rp_truncations() {
        if (top_trunc == tt || (stem + 1 == top_trunc && tt == 256)) && bot_trunc == bt {
            let pages = if synthetic {
                let (pages, issues) = compute_pages(data, model, bt, tt, stem, stem, true);

                let observed = pages.convergence_at_stem(model, stem);

                compare_synthetic(&observed, synthetic_rp(bt, tt), bt, top_trunc, stem)?;

                if issues.len() != 0 {
                    return Err(issues);
                }
                pages
            } else {
                let (pages, issues) = compute_pages(data, model, bt, tt, stem - 1, stem, true);

                let observed = pages.algebraic_convergence_at_stem(model, stem);

                compare_algebraic(&observed, algebraic_rp(bt, tt), bt, tt, stem)?;

                if issues.len() != 0 {
                    return Err(issues);
                }
                pages
            };
            compare_algebraic_spectral_sequence(data, model, &pages, stem, bt, tt, true)?;
        }
    }
    Ok(())
}

fn filter_diff(
    data: &SyntheticSS,
    model: &E1,
    alg_ahss: &SyntheticSS,
    bot_trunc: i32,
    top_trunc: i32,
    d: Diff,
) -> Option<Kind> {
    let stem = model.stem(d.to);
    let y = model.y(d.from);

    if y - model.y(d.to) < RADON_HURWITZ_NUMBERS[y as usize] {
        Some(Kind::MinimalLength)
    } else if data.in_diffs[d.to]
        .iter()
        .any(|from| model.y(*from) == top_trunc && data.generators[*from].alive())
    {
        Some(Kind::AdditiveStructure)
    } else if bot_trunc & 1 == 0
        && let Some(alg_to) = alg_ahss.out_diffs[d.from].first()
        && data.generators[*alg_to].alive()
        && model.y(*alg_to) + 1 == bot_trunc
    {
        Some(Kind::Invisible)
    } else if top_trunc & 1 == 1
        && !(top_trunc == 5 && bot_trunc == 3)
        && let Some(dies) = model.get(d.to).dies
        && let Some(source) = alg_ahss.in_diffs[d.to].first()
        && data.generators[*source].free()
        && top_trunc + 2 == dies
    {
        Some(Kind::Unnecessary)
    } else {
        None
    }
}

fn iterate_e1_issues(
    data: &mut SyntheticSS,
    model: &E1,
    alg_ahss: &SyntheticSS,
    alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>,
    e1_issues: &Vec<Vec<(Vec<(usize, Torsion)>, Vec<Action>)>>,
    getout: &mut GetOut,
    log: &Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
) -> Option<BranchResult> {
    if e1_issues[stem as usize].len() == 0 {
        None
    } else {
        if depth >= MAX_DEPTH {
            return Some(BranchResult::Open);
        }

        let outcomes: Vec<_> = if depth < PARALLEL_DEPTH {
            let g = create_getout(&getout, e1_issues[stem as usize].len() as i32, depth);

            e1_issues[stem as usize].par_iter().enumerate().map(|(index, x)| {
                if depth == 0 || ALWAYS_PRINT {
                    println!("Trying splitting with {index} on stem {stem} | depth {depth}");
                }
                let mut data = data.clone();


                // Apply possible solution to torsion on E_1 page
                for j in &x.0 {
                    data.generators[j.0] = j.1;
                }

                let res = ahss_iterate(data, model, alg_ahss, alg_data, e1_issues, g.clone(), log.clone(), stem, top_trunc, bot_trunc, depth + 1);
                if depth == 0 || ALWAYS_PRINT {
                    println!("Finish splitting with {index} on stem {stem} by {res:?}");
                }
                res
            }).collect()
        } else {
            e1_issues[stem as usize].iter().enumerate().map(|(index, x)| {
                if depth == 0 || ALWAYS_PRINT {
                    println!("Trying splitting with {index} on stem {stem} | depth {depth}");
                }
                let mut data = data.clone();

                // Apply possible solution to torsion on E_1 page
                for j in &x.0 {
                    data.generators[j.0] = j.1;
                }

                let res = ahss_iterate(data, model, alg_ahss, alg_data, e1_issues, getout.clone(), log.clone(), stem, top_trunc, bot_trunc, depth + 1);
                if depth == 0 || ALWAYS_PRINT {
                    println!("Finish splitting with {index} on stem {stem} by {res:?}");
                }
                res
            }).collect()
        };

        if depth == 0 || ALWAYS_PRINT {
            println!("{outcomes:?}");
        }

        let positives = outcomes.iter().fold(0, |acc, r| {
            if let BranchResult::Contradiction(_) = r {
                acc
            } else {
                acc + 1
            }
        });


        let opens = outcomes.iter().fold(0, |acc, r| {
            if let BranchResult::Open = r {
                acc + 1
            } else {
                acc
            }
        });


        if check_getout(&getout) {
            return Some(BranchResult::Cancelled);
        }

        if positives == 1 {
            for (index, r) in outcomes.into_iter().enumerate() {
                if let BranchResult::Contradiction(_) = r {

                }  else {
                    if depth == 0 {
                        let actions = e1_issues[stem as usize][index].1.clone();
                        for a in actions {
                            if depth == 0 || ALWAYS_PRINT {
                                if let Action::SetE1 { tag, torsion, proof: _ } = &a {
                                    println!("Set E1 torsion {index}: {} | {:?}", tag, torsion);
                                }
                            }
                            log.lock().unwrap().push(a);
                        }
                    }

                    for j in &e1_issues[stem as usize][index].0 {
                        if (depth == 0 || ALWAYS_PRINT) && model.get(j.0).y == 1 {
                            println!("Set E1 torsion {index}: {} | {:?}", model.get(j.0).name, j.1);
                        }
                        data.generators[j.0] = j.1;
                    }
                    return None;
                }
            }
        } else if opens >= 2 {
            if depth == 0 || ALWAYS_PRINT {
                println!("We have {opens} opens");

                for (index, j) in outcomes.iter().enumerate() {
                    if let BranchResult::Open = j {
                        println!("{index} | {:?}", e1_issues[stem as usize][index].1)
                    }
                }
            }
            return Some(BranchResult::Open)
        }

        // signal_parent_getout(getout, depth);
        return Some(BranchResult::Contradiction(
            format!(
                "We have {positives} positives and {opens} opens. Which means we can't decide on E1 stuff :("
            )
        ));
    }
}

fn ahss_iterate(
    mut data: SyntheticSS,
    model: &E1,
    alg_ahss: &SyntheticSS,
    alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>,
    e1_issues: &Vec<Vec<(Vec<(usize, Torsion)>, Vec<Action>)>>,
    mut getout: GetOut,
    log: Arc<Mutex<Vec<Action>>>,
    mut stem: i32,
    mut top_trunc: i32,
    mut bot_trunc: i32,
    depth: i32,
) -> BranchResult {
    loop {
        if depth == 0 && stem >= MAX_VERIFY_STEM {
            return BranchResult::Open;
        }

        if depth > MAX_DEPTH || stem >= MAX_STEM {
            if ALWAYS_PRINT {
                println!("DEPTH REACHED");
            }
            return BranchResult::Open;
        }

        if check_getout(&getout) {
            return BranchResult::Cancelled;
        }

        if bot_trunc <= D_R_REPEATS[(top_trunc - bot_trunc) as usize] as i32 {
            let option = get_a_diff(&data, model, top_trunc, bot_trunc, stem);

            if let Some(d) = option {
                if depth >= MAX_DEPTH {
                    return BranchResult::Open;
                }
                match try_diff(
                    &mut data,
                    model,
                    alg_ahss,
                    alg_data,
                    e1_issues,
                    &getout,
                    &log,
                    stem,
                    top_trunc,
                    bot_trunc,
                    depth,
                    d,
                ) {
                    ChoiceResult::Chosen => continue,
                    ChoiceResult::Open => return BranchResult::Open,
                    ChoiceResult::Cancelled => return BranchResult::Cancelled,
                }
            }
        }

        let potential_tau_thing = match is_tau_issue(&data, model, stem, top_trunc, bot_trunc) {
            Ok(tau_issue) => tau_issue,
            Err(is) => {
                signal_parent_getout(&mut getout, depth);
                return BranchResult::Contradiction(is);
            }
        };

        if let Some((synthetic, mut issues)) = potential_tau_thing {
            let option = match synthetic {
                TauIssue::AlgTauIssue => {
                    suggest_tau_solution_algebraic(&data, model, &mut issues, top_trunc, bot_trunc, stem)
                }
                TauIssue::SynTauGeneratorIssue => suggest_tau_solution_generator_synthetic(
                    &data,
                    model,
                    &mut issues,
                    top_trunc,
                    bot_trunc,
                    stem,
                ),
                TauIssue::SynTauModuleIssue => suggest_tau_solution_generator_synthetic(
                    &data,
                    model,
                    &mut issues,
                    top_trunc,
                    bot_trunc,
                    stem,
                ),
            };

            if let Some(d) = option {
                if depth >= MAX_DEPTH {
                    return BranchResult::Open;
                }
                match try_tau(
                    &mut data,
                    model,
                    alg_ahss,
                    alg_data,
                    e1_issues,
                    &getout,
                    &log,
                    stem,
                    top_trunc,
                    bot_trunc,
                    depth,
                    d,
                ) {
                    ChoiceResult::Chosen => continue,
                    ChoiceResult::Open => return BranchResult::Open,
                    ChoiceResult::Cancelled => return BranchResult::Cancelled,
                }
            } else {
                signal_parent_getout(&mut getout, depth);
                return BranchResult::Contradiction(format!(
                    "Issue at RP{bot_trunc}_{top_trunc}: {issues:?}"
                ));
            }
        }

        if bot_trunc != 0 {
            let d_y = top_trunc - bot_trunc + 1;
            for &(from, to) in &alg_data[stem as usize][d_y as usize][top_trunc as usize] {
                if depth == 0 {
                    let (from_name, to_name) = model.get_names(from, to);
                    println!("Applying Algebraic diff {from_name} -> {to_name}");
                }
                data.add_diff(model, from, to, None, Kind::Algebraic);
            }
            bot_trunc -= 1;
            continue;
        }

        if top_trunc == stem + 1 {
            stem += 1;
            top_trunc = 2;
            bot_trunc = top_trunc - 1;

            if let Some(r) = iterate_e1_issues(
                &mut data,
                model,
                alg_ahss,
                alg_data,
                e1_issues,
                &mut getout,
                &log,
                stem,
                top_trunc,
                bot_trunc,
                depth,
            ) {
                return r;
            }
        } else {
            top_trunc += 1;
            bot_trunc = top_trunc - 1;
        }


        if depth == 0 {
            println!("Current stem: {stem} | top_trunc: {top_trunc}");
        }
    }
}

fn try_diff(
    data: &mut SyntheticSS,
    model: &E1,
    alg_ahss: &SyntheticSS,
    alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>,
    e1_issues: &Vec<Vec<(Vec<(usize, Torsion)>, Vec<Action>)>>,
    getout: &GetOut,
    log: &Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
    d: Diff,
) -> ChoiceResult {
    let (from_name, to_name) = model.get_names(d.from, d.to);

    let filter = filter_diff(data, model, alg_ahss, bot_trunc, top_trunc, d);

    if let Some(kind) = filter {
        if ALWAYS_PRINT || depth == 0 {
            println!(
                "Finished diff by: {} | {} -> {kind:?}",
                from_name, to_name
            );
        }
        if depth == 0 {
            log.lock().unwrap().push(Action::AddDiff {
                from: from_name,
                to: to_name,
                proof: None,
                kind,
            });
        }
        data.add_diff(model, d.from, d.to, None, kind);
        return ChoiceResult::Chosen;
    }

    if ALWAYS_PRINT || depth == 0 {
        println!("Trying diff: {} | {}", from_name, to_name);
    }

    let g = create_getout(getout, 2, depth);

    let with = || {
        let mut with_data = data.clone();

        let action = Action::AddDiff {
            from: model.name(d.from).to_string(),
            to: model.name(d.to).to_string(),
            kind: Kind::Real,
            proof: Some("".to_string())
        };

        let _ = process_action(&mut with_data, model, &action, true);

        ahss_iterate(
            with_data,
            model,
            alg_ahss,
            alg_data,
            e1_issues,
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
        without_data.add_diff(model, d.from, d.to, Some("".to_string()), Kind::Fake);
        ahss_iterate(
            without_data,
            model,
            alg_ahss,
            alg_data,
            e1_issues,
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
            commit_diff_choice(data, model, log, depth, d, Commitment::Fake(e));
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::ChooseLeft(e) => {
            commit_diff_choice(data, model, log, depth, d, Commitment::Real(e));
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::BothOpen => {
            commit_diff_choice(data, model, log, depth, d, Commitment::Unknown);
            ChoiceResult::Open
        }
        SpeculativeBranchOutcome::Cancelled => ChoiceResult::Cancelled,
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TauIssue {
    AlgTauIssue,
    SynTauGeneratorIssue,
    SynTauModuleIssue,
}

fn is_tau_issue(
    data: &SyntheticSS,
    model: &E1,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
) -> Result<Option<(TauIssue, Vec<Issue>)>, String> {
    match check_issue(data, model, stem, bot_trunc, top_trunc) {
        Ok(_) => Ok(None),
        Err(is) => {
            let all_synth_conv = if let Issue::SyntheticConvergence {
                bot_trunc: _,
                top_trunc: _,
                stem: _,
                af: _,
                expected: _,
                observed: _,
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
                        "For RP{bot_trunc}_{top_trunc} the F_2 vector space generators don't add up. {is:?}"
                    ))
                }
            } else {
                if algebraic_issue_is_fixable_by_tau_extensions(&is) {
                    Ok(Some((TauIssue::AlgTauIssue, is)))
                } else {
                    Err(format!(
                        "For RP{bot_trunc}_{top_trunc} there is no way to fix the algebraic convergence issues with tau extensions. {is:?}"
                    ))
                }
            }
        }
    }
}

fn try_tau(
    data: &mut SyntheticSS,
    model: &E1,
    alg_ahss: &SyntheticSS,
    alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>,
    e1_issues: &Vec<Vec<(Vec<(usize, Torsion)>, Vec<Action>)>>,
    getout: &GetOut,
    log: &Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
    d: ExtTauMult,
) -> ChoiceResult {
    let (from_name, to_name) = model.get_names(d.from, d.to);

    if ALWAYS_PRINT || depth == 0 {
        println!(
            "Trying tau: {} | {} | af: {} | RP{bot_trunc}_{top_trunc}",
            from_name, to_name, d.af
        );
    }

    let g = create_getout(getout, 2, depth);

    let with = || {
        let mut with_data = data.clone();

        let action = Action::AddExt {
            from: model.name(d.from).to_string(),
            to: model.name(d.to).to_string(),
            af: d.af,
            kind: Kind::Real,
            proof: None
        };

        let _ = process_action(&mut with_data, model, &action, true);


        ahss_iterate(
            with_data,
            model,
            alg_ahss,
            alg_data,
            e1_issues,
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

        let action = Action::AddExt {
            from: model.name(d.from).to_string(),
            to: model.name(d.to).to_string(),
            af: d.af,
            kind: Kind::Fake,
            proof: None
        };

        let _ = process_action(&mut without_data, model, &action, true);



        ahss_iterate(
            without_data,
            model,
            alg_ahss,
            alg_data,
            e1_issues,
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
            commit_tau_choice(data, model, log, depth, d, Commitment::Fake(e));
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::ChooseLeft(e) => {
            commit_tau_choice(data, model, log, depth, d, Commitment::Real(e));
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::BothOpen => {
            commit_tau_choice(data, model, log, depth, d, Commitment::Unknown);
            if ALWAYS_PRINT || depth == 0 {
                println!("BothOpen: {from_name} | {to_name} tau multiple");
            }
            ChoiceResult::Open
        }
        SpeculativeBranchOutcome::Cancelled => ChoiceResult::Cancelled,
    }
}


pub fn ahss_solve_e1_issues(ahss: &SyntheticSS, model: &E1, log: &mut Vec<Action>) -> Vec<Vec<(Vec<(usize, Torsion)>, Vec<Action>)>> {
    let mut stem_sols = vec![vec![]; (MAX_STEM + 1) as usize];

    for stem in 2..=MAX_STEM {
        let mut proper_issues = vec![];
        match ahss_synthetic_e1_issue(&ahss, model, stem) {
            Ok(_) => {}
            Err(issues) => {
                for i in issues {
                    // First we solve all the e1 issues we can resolve
                    match auto_deduce(&ahss, model, &i) {
                        Ok(mut a) => log.append(&mut a),
                        Err(_) => {
                            proper_issues.push(i);
                        },
                    }
                }
            }
        }

        if proper_issues.len() == 0 {
            continue;
        }

        let proper_sols = get_all_e1_solutions(ahss, model, &proper_issues);

        let l: Vec<_> = proper_sols.iter().map(|x|  {
            let ids: Vec<_> = x.into_iter().flat_map(|y| {
                if let Action::SetE1 { tag, torsion, proof: _ }  = y {
                    let mut ids = vec![];
                    for g in generate_names_from_tag(&tag, 1, 1) {
                        if let Some(id) = model.try_index(&g) {
                            ids.push((id, *torsion));
                        } else {
                            break;
                        }
                    }
                    ids
                } else {
                    unreachable!()
                }
            }).collect();

            (ids, x.clone())
        }).collect();

        stem_sols[stem as usize] = l;
    }

    stem_sols
}

pub fn ahss_solver(log: Option<Vec<Action>>) -> (Vec<Action>, SyntheticSS) {
    let alg_ahss = STABLE_DATA.clone();
    let model: &E1 = &STABLE_MODEL;
    let mut partial_ahss = SyntheticSS::empty(model.clone());
    // We should add all d1's from the algebraic data

    let mut alg_data = vec![
        vec![vec![vec![]; (MAX_STEM + 2) as usize]; (MAX_STEM + 1) as usize];
        (MAX_STEM + 1) as usize
    ];

    for (&(from, to), (kind, _)) in &alg_ahss.from_to {
        let d_y = model.y(from) - model.y(to);
        let repeats = D_R_REPEATS[d_y as usize];
        if d_y == 1 || model.y(to) - (repeats as i32) >= 1 {
            partial_ahss.add_diff(model, from, to, None, *kind);
        } else {
            let stem = model.stem(to);
            let top_trunc = model.y(from);
            alg_data[stem as usize][d_y as usize][top_trunc as usize].push((from, to));
        }
    }


    let mut log = log.unwrap_or(vec![]);


    // Set E1 actions should already have happened here
    let ahss = revert_log_and_remake(0, &mut log, model, &partial_ahss, true);
    let e1_issues = ahss_solve_e1_issues(&ahss, model, &mut log);


    let ahss = revert_log_and_remake(0, &mut log, model, &ahss, true);
    let log = Arc::new(Mutex::new(log));

    let res = ahss_iterate(ahss, model, &alg_ahss, &alg_data, &e1_issues, empty_getout(), log.clone(),
    2, 2, 1, 0);

    // Add AHSS Algebraic Diffs
    for (&(from, to), (kind, _)) in &alg_ahss.from_to {
        partial_ahss.add_diff(model, from, to, None, *kind);
    }

    println!("{res:?}");

    let mut log = Arc::try_unwrap(log).unwrap().into_inner().unwrap();
    let ahss = revert_log_and_remake(0, &mut log, model, &partial_ahss, true);
    (log, ahss)
}
