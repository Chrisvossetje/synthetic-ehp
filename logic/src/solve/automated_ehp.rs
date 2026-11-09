//! The automated EHP (unstable) solver — the counterpart to
//! [`crate::solve::automated_ahss`] for the EHP sequence. [`ehp_solver`] seeds the
//! metastable range from the AHSS, then runs the same speculative search
//! ([`ehp_iterate`], `try_diff`/`try_tau`) sphere by sphere, additionally
//! resolving the induced names that the EHP recursion introduces ([`fix_names`])
//! and lifting solved values upward via [`crate::domain::process::ehp_recursion`].

use core::panic;
use std::sync::{Arc, Mutex};

use crate::{
    MAX_STEM, MAX_VERIFY_STEM, data::{
        r#static::{ALGEBRAIC_SPHERE_PAGES, EHP_TO_AHSS, S0, algebraic_spheres},
        curtis::{DATA, MODEL, STABLE_MODEL},
    }, domain::{
        e1::E1, model::{Diff, ExtTauMult, SyntheticSS}, process::{compute_pages, ehp_recursion, try_compute_pages}, ss::SSPages
    }, solve::{
        action::{Action, process_action, revert_log_and_remake},
        automated::{
            ALWAYS_PRINT, Commitment, TauIssue, classify_tau_issue, commit_diff_choice,
            commit_tau_choice, filter_diff,
        },
        ehp_ahss::{in_metastable_range, set_metastable_range},
        generate::get_a_diff,
        issues::{
            Issue, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic,
        },
        proof::{ProofChoice, ProofTrace, SearchProof},
        search::{
            BranchResult, ChoiceResult, GetOut, SpeculativeBranchOutcome, discover_choice, check_getout, empty_getout, signal_parent_getout
        },
        solve::{suggest_tau_solution_algebraic, suggest_tau_solution_generator_synthetic},
    }, types::Kind
};


enum FixNamesResult {
    Contradiction(SearchProof),
    Applied(Vec<Action>),
    Open,
    Cancelled,
}

fn commit_induced_name_choice(
    data: &mut SyntheticSS,
    model: &E1,
    depth: i32,
    action: &mut Action,
    proof: SearchProof,
) -> Action {
    if ALWAYS_PRINT || depth == 0 {
        println!("Choosing induced name: {}", proof.summary());
    }
    let proof = if depth == 0 || ALWAYS_PRINT {
        proof.describe(model)
    } else {
        String::new()
    };
    if let Action::SetInducedName {
        proof: action_proof,
        ..
    } = action
    {
        *action_proof = proof.clone();
    }

    process_action(data, model, action, false).unwrap();
    action.clone()
}

/// Verify the EHP at a given (stem, sphere) against everything we know: its own
/// convergence (synthetic if `stem + 2 == sphere`, i.e. the stable cell, else
/// algebraic) and compatibility with the precomputed AHSS pages. Returns the
/// offending [`Issue`]s, which the caller may yet recognise as tau-fixable.
fn check_issue(data: &SyntheticSS, model: &E1, ahss_pages: &[SSPages; (MAX_STEM + 1) as usize], stem: i32, sphere: i32) -> Result<(), Vec<Issue>> {
    let pages = if stem + 2 == sphere {
        let pages = try_compute_pages(data, model, 0, sphere - 1, stem, stem, true)?;

        let observed = pages.convergence_at_stem(model, stem);

        compare_synthetic(&observed, &S0, 0, sphere - 1, stem)?;

        compare_algebraic_spectral_sequence(data, model, &pages, stem, 0, sphere - 1, false)?;

        pages
    } else {
        let pages = try_compute_pages(data, model, 0, sphere - 1, stem - 1, stem, true)?;

        let observed = pages.algebraic_convergence_at_stem(model, stem);

        compare_algebraic(&observed, algebraic_spheres(sphere), 0, sphere - 1, stem)?;
        pages
    };

    // EHP -> AHSS compatibility: each live EHP generator that has a stable
    // counterpart must, at every page, map into it without exceeding its torsion.
    for &f_id in model.gens_id_in_stem(stem) {
        if let Some(t_id) = EHP_TO_AHSS[f_id]
            && ahss_pages[(sphere - 1) as usize].element_in_pages(t_id)
        {
            if let Some(ps) = &pages.generators[f_id] {
                for (f_page, (f_af, f_torsion)) in ps {
                    if f_torsion.alive() {
                        let (t_af, t_torsion) = ahss_pages[(sphere - 1) as usize]
                            .element_at_page(*f_page, t_id);

                        if !t_torsion.can_map_with_coeff(&f_torsion, t_af - f_af) {
                            return Err(vec![Issue::InvalidEHPAHSSMap {
                                name: model.name(f_id).to_string(),
                                from_torsion: *f_torsion,
                                to_torsion: t_torsion,
                                stem,
                                sphere,
                            }]);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn filter_tau(
    data: &SyntheticSS,
    model: &E1,
    _alg_ahss: &SyntheticSS,
    _bot_trunc: i32,
    _top_trunc: i32,
    d: ExtTauMult,
) -> Option<Kind> {
    let _stem = model.stem(d.to);
    let _y = model.y(d.from);

    if data.generators[d.from].free() {
        Some(Kind::Unnecessary)
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
    model: &E1,
    alg_ehp: &SyntheticSS,
    ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize, Kind, Option<String>)>>>>,
    ahss_pages: &[SSPages; (MAX_STEM + 1) as usize],
    mut getout: GetOut,
    log: Arc<Mutex<Vec<Action>>>,
    mut stem: i32,
    mut top_trunc: i32,
    mut bot_trunc: i32,
    depth: i32,
    stop_at: (i32, i32),
) -> BranchResult {
    let mut trace = ProofTrace::default();
    // Same shape as the AHSS loop (see `automated_ahss::ahss_iterate`), but the
    // unstable EHP traversal is more intricate: `bot_trunc` walks the metastable
    // range, the spheres are stepped in the odd/even dance at the bottom of the
    // loop, induced names must be resolved (`fix_names`), and each solved sphere
    // is lifted to the next via `ehp_recursion`.
    loop {
        trace.at(getout.position);
        if depth == 0 && stem >= stop_at.0 && top_trunc >= stop_at.1 {
            return BranchResult::Open(trace.open(depth));
        }

        if depth > getout.max_depth() || getout.past_frontier() || stem >= MAX_STEM {
            return BranchResult::Open(trace.open(depth));
        }

        if check_getout(&getout) {
            return BranchResult::Cancelled(trace.cancelled(depth));
        }

        // While still inside the truncation window, try the next differential.
        if bot_trunc != 0 {
            let option = get_a_diff(&data, model, top_trunc, bot_trunc, stem);
            // Should only need first option here
            if let Some(d) = option {
                if depth >= getout.max_depth() {
                    return BranchResult::Open(trace.open(depth));
                }
                match try_diff(
                    &mut data,
                    model,
                    alg_ehp,
                    ahss_and_alg_data,
                    ahss_pages,
                    &getout,
                    &log,
                    stem,
                    top_trunc,
                    bot_trunc,
                    depth,
                    d,
                    &mut trace,
                ) {
                    ChoiceResult::Contradiction(proof) => {
                        signal_parent_getout(&mut getout, depth);
                        return BranchResult::Contradiction(trace.contradiction_by(depth, proof));
                    }
                    ChoiceResult::Chosen => {
                        continue;
                    }
                    ChoiceResult::Open => {
                        return BranchResult::Open(trace.open(depth));
                    }
                    ChoiceResult::Cancelled => {
                        return BranchResult::Cancelled(trace.cancelled(depth));
                    }
                }
            }
        } else {
            // At the bottom of the window: check convergence on this sphere and,
            // if a tau is needed, ask the matching suggester and try it.
            let potential_tau_thing =
                match is_tau_issue(&data, model, ahss_pages, stem, top_trunc + 1) {
                    Ok(tau_issue) => tau_issue,
                    Err(is) => {
                        signal_parent_getout(&mut getout, depth);
                        return BranchResult::Contradiction(trace.contradiction(depth, is));
                    }
                };

            if let Some((synthetic, mut issues)) = potential_tau_thing {
                let option = match synthetic {
                    TauIssue::AlgTauIssue => suggest_tau_solution_algebraic(
                        &data,
                        model,
                        &mut issues,
                        top_trunc,
                        bot_trunc,
                        stem,
                    ),
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
                    if depth >= getout.max_depth() {
                        return BranchResult::Open(trace.open(depth));
                    }
                    match try_tau(
                        &mut data,
                        model,
                        alg_ehp,
                        ahss_and_alg_data,
                        ahss_pages,
                        &getout,
                        &log,
                        stem,
                        top_trunc,
                        bot_trunc,
                        depth,
                        d,
                        &mut trace,
                    ) {
                        ChoiceResult::Contradiction(proof) => {
                            signal_parent_getout(&mut getout, depth);
                            return BranchResult::Contradiction(
                                trace.contradiction_by(depth, proof),
                            );
                        }
                        ChoiceResult::Chosen => {
                            continue;
                        }
                        ChoiceResult::Open => {
                            return BranchResult::Open(trace.open(depth));
                        }
                        ChoiceResult::Cancelled => {
                            return BranchResult::Cancelled(trace.cancelled(depth));
                        }
                    }
                } else {
                    signal_parent_getout(&mut getout, depth);

                    return BranchResult::Contradiction(trace.contradiction(
                        depth,
                        format!("Issue at S^{} | stem {}: {issues:?}", top_trunc + 1, stem),
                    ));
                }
            }
        }

        // Fold in the deferred (lifted/algebraic) differentials for this page and
        // step the window down; once it reaches the bottom, resolve induced names.
        if bot_trunc != 0 {
            // TODO: Do something wth this result ?
            let _ = add_diffs(
                &mut data,
                model,
                ahss_and_alg_data,
                stem,
                &log,
                top_trunc,
                bot_trunc,
                depth,
            );
            getout.position += 1;
            bot_trunc -= 1;
            continue;
        } else {
            if top_trunc & 1 == 0 && (top_trunc / 2) + stem < MAX_STEM {
                match fix_names(
                    &mut data,
                    model,
                    alg_ehp,
                    ahss_and_alg_data,
                    ahss_pages,
                    &getout,
                    &log,
                    stem,
                    top_trunc,
                    bot_trunc,
                    depth,
                    &mut trace,
                ) {
                    Ok(i) => match i {
                        FixNamesResult::Applied(mut actions) => {
                            if depth == 0 {
                                log.lock().unwrap().append(&mut actions);
                            }
                        }
                        FixNamesResult::Contradiction(proof) => {
                            signal_parent_getout(&mut getout, depth);
                            return BranchResult::Contradiction(
                                trace.contradiction_by(depth, proof),
                            );
                        }
                        FixNamesResult::Cancelled => {
                            return BranchResult::Cancelled(trace.cancelled(depth));
                        }
                        FixNamesResult::Open => return BranchResult::Open(trace.open(depth)),
                    },
                    Err(e) => {
                        signal_parent_getout(&mut getout, depth);
                        return BranchResult::Contradiction(trace.contradiction(depth, e));
                    }
                }
            }
        }

        // Once a sphere is fully solved, lift its values onto the next odd sphere.
        if top_trunc & 1 == 0 && top_trunc <= stem && (top_trunc / 2) + stem < MAX_STEM {
            let res =
                ehp_recursion(&mut data, model, top_trunc + 1, stem).map_err(|x| format!("{x:?}"));
            if res.is_err() {
                panic!();
            }
        }

        // Advance to the next (stem, sphere) cell. Spheres are visited in the order
        // the EHP recursion needs them (even spheres, then the 5/3/1 odd chain that
        // steps back two stems), not simply left-to-right.
        getout.position += 1;
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
    model: &E1,
    alg_ehp: &SyntheticSS,
    ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize, Kind, Option<String>)>>>>,
    ahss_pages: &[SSPages; (MAX_STEM + 1) as usize],
    getout: &GetOut,
    log: &Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
    trace: &mut ProofTrace,
) -> Result<FixNamesResult, String> {
    // The EHP recursion carries each generator a name describing where it came
    // from one sphere down. On a new sphere those induced names can become wrong
    // (the named class died, or sits at a different filtration). This step detects
    // such generators and decides which algebraic class each should now be called.
    let sphere = top_trunc + 1;

    let (pages, _) = compute_pages(data, model, 0, sphere - 1, stem, stem, true);
    let alg_pages = &ALGEBRAIC_SPHERE_PAGES[sphere as usize];

    // Collect the generators whose current induced name is inconsistent.
    let mut issues = vec![];

    for &id in model.gens_id_in_stem(stem) {
        // Synthetic Generators
        if let Some((id_af, id_torsion)) = pages.try_element_final(id)
            && id_torsion.alive()
        {
            let ind_name = data.get_name_at_sphere(model, id, sphere).to_string();
            let g = model.get_name(&ind_name);
            if let Some(dies) = g.dies
                && dies <= sphere
            {
                issues.push(Issue::InvalidName {
                    original_name: model.name(id).to_string(),
                    unexpected_name: ind_name,
                    sphere,
                    stem,
                    af: id_af,
                });
            } else if id_af != g.af {
                issues.push(Issue::InvalidName {
                    original_name: model.name(id).to_string(),
                    unexpected_name: ind_name,
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
            unexpected_name: _,
            sphere,
            stem,
            af,
        } = i
        {
            // Compare the live synthetic names at this filtration with the live
            // algebraic ones; the names present in one set but not the other are
            // the candidates this generator could be renamed to.
            let mut syn = vec![];
            let mut alg = vec![];
            for id in MODEL.gens_id_in_stem(*stem) {
                if pages.element_in_pages(*id) {
                    let g = pages.element_final(*id);
                    if g.1.alive() && g.0 == *af {
                        let name = data.get_name_at_sphere(model, *id, *sphere).to_string();
                        syn.push(name);
                    }
                }

                if alg_pages.element_in_pages(*id) {
                    let g = alg_pages.element_final(*id);
                    if g.1.alive() && g.0 == *af {
                        let name = MODEL.name(*id).to_string();
                        alg.push(name);
                    }
                }
            }

            let fil_syn: Vec<_> = syn.iter().filter(|i| !alg.contains(i)).collect();
            let fil_alg: Vec<_> = alg.iter().filter(|i| !syn.contains(i)).collect();

            if fil_alg.len() == 0 {
                return Err(format!(
                    "This should have been seen as an algebraic convergence issue"
                ));
            }
            // Unique candidate on each side: the rename is forced — apply it.
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

                trace.known(
                    depth,
                    ProofChoice::InducedName {
                        from: model.get_index(original_name),
                        to: model.get_index(name),
                        sphere: *sphere,
                    },
                    "unique compatible induced name",
                );
                process_action(data, model, &action, false).unwrap();
                sols.push(action);
            } else {
                // Two algebraic candidates: speculatively try each name and keep
                // the one whose world stays consistent (same with/without search).
                if fil_syn.len() == 1 && fil_alg.len() == 2 {
                    if depth >= getout.max_depth() {
                        return Ok(FixNamesResult::Open);
                    }
                    let mut a_action = Action::SetInducedName {
                        name: original_name.clone(),
                        new_name: fil_alg[0].to_string(),
                        sphere: *sphere,
                        proof: format!(""),
                    };
                    let mut b_action = Action::SetInducedName {
                        name: original_name.clone(),
                        new_name: fil_alg[1].to_string(),
                        sphere: *sphere,
                        proof: format!(""),
                    };

                    let run = |world: usize, control: GetOut| {
                        let mut world_data = data.clone();
                        let action = if world == 0 { &a_action } else { &b_action };
                        process_action(&mut world_data, model, action, false).unwrap();
                        ehp_iterate(
                            world_data, model, alg_ehp, ahss_and_alg_data, ahss_pages,
                            control, log.clone(), *stem, top_trunc, bot_trunc,
                            depth + 1, stop_at_default(),
                        )
                    };

                    if ALWAYS_PRINT || depth == 0 {
                        println!("Trying Induced name for {}", fil_syn[0]);
                    }

                    let choice = |name: &str| ProofChoice::InducedName {
                        from: model.get_index(original_name),
                        to: model.get_index(name),
                        sphere: *sphere,
                    };
                    match discover_choice(
                        depth, getout, [choice(fil_alg[0]), choice(fil_alg[1])], run,
                    ) {
                        SpeculativeBranchOutcome::Contradiction(proof) => {
                            return Ok(FixNamesResult::Contradiction(proof));
                        }
                        SpeculativeBranchOutcome::ChooseRight(e) => {
                            trace.resolved(depth, choice(fil_alg[1]), &e);
                            let action = commit_induced_name_choice(data, model, depth, &mut b_action, e);
                            sols.push(action);
                        }
                        SpeculativeBranchOutcome::ChooseLeft(e) => {
                            trace.resolved(depth, choice(fil_alg[0]), &e);
                            let action = commit_induced_name_choice(data, model, depth, &mut a_action, e);
                            sols.push(action);
                        }
                        SpeculativeBranchOutcome::Cancelled => {
                            return Ok(FixNamesResult::Cancelled);
                        }
                        SpeculativeBranchOutcome::BothOpen => {
                            return Ok(FixNamesResult::Open);
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

                    return Err(format!(
                        "We have two unknowns in one degree, we probably need to make better differential choices. Syn: {fil_syn:?} | Alg: {fil_alg:?} | Issue: {i:?}"
                    ));
                }
            }
        }
    }

    Ok(FixNamesResult::Applied(sols))
}

fn try_diff(
    data: &mut SyntheticSS,
    model: &E1,
    alg_ehp: &SyntheticSS,
    ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize, Kind, Option<String>)>>>>,
    ahss_pages: &[SSPages; (MAX_STEM + 1) as usize],
    getout: &GetOut,
    log: &Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
    d: Diff,
    trace: &mut ProofTrace,
) -> ChoiceResult {
    let (from_name, to_name) = model.get_names(d.from, d.to);

    let filter = filter_diff(&data, model, alg_ehp, bot_trunc, top_trunc, d, false);

    if let Some(kind) = filter {
        if depth == 0 {
            log.lock().unwrap().push(Action::AddDiff {
                from: from_name.clone(),
                to: to_name.clone(),
                proof: None,
                kind,
            });
        }

        if ALWAYS_PRINT || depth == 0 {
            println!("Finished diff by {kind:?}: {} -> {}", from_name, to_name);
        }
        trace.known(
            depth,
            ProofChoice::FilteredDiff(d, kind),
            format!("{kind:?}"),
        );
        data.add_diff(model, d.from, d.to, None, kind);
        return ChoiceResult::Chosen;
    }

    if ALWAYS_PRINT || depth == 0 {
        println!("Trying diff: {} -> {}", from_name, to_name);
    }

    let run = |world: usize, control: GetOut| {
        let mut world_data = data.clone();
        let kind = if world == 0 { Kind::Real } else { Kind::Fake };
        world_data.add_diff(model, d.from, d.to, None, kind);
        ehp_iterate(
            world_data,
            model,
            alg_ehp,
            ahss_and_alg_data,
            ahss_pages,
            control,
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
            stop_at_default(),
        )
    };

    match discover_choice(
        depth,
        getout,
        [ProofChoice::Diff(d, true), ProofChoice::Diff(d, false)],
        run,
    ) {
        SpeculativeBranchOutcome::Contradiction(proof) => ChoiceResult::Contradiction(proof),
        SpeculativeBranchOutcome::ChooseRight(e) => {
            trace.resolved(depth, ProofChoice::Diff(d, false), &e);
            commit_diff_choice(data, model, log, depth, d, Commitment::Fake(e), false);
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::ChooseLeft(e) => {
            trace.resolved(depth, ProofChoice::Diff(d, true), &e);
            commit_diff_choice(data, model, log, depth, d, Commitment::Real(e), false);
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::BothOpen => {
            commit_diff_choice(data, model, log, depth, d, Commitment::Unknown, false);
            ChoiceResult::Open
        }
        SpeculativeBranchOutcome::Cancelled => ChoiceResult::Cancelled,
    }
}

fn is_tau_issue(
    data: &SyntheticSS,
    model: &E1,
    ahss_pages: &[SSPages; (MAX_STEM + 1) as usize],
    real_stem: i32,
    sphere: i32,
) -> Result<Option<(TauIssue, Vec<Issue>)>, String> {
    match check_issue(data, model, ahss_pages, real_stem, sphere) {
        Ok(_) => Ok(None),
        Err(issues) => classify_tau_issue(issues, "For the stable Sphere", &format!("For S^{sphere}")),
    }
}

fn try_tau(
    data: &mut SyntheticSS,
    model: &E1,
    alg_ehp: &SyntheticSS,
    ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize, Kind, Option<String>)>>>>,
    ahss_pages: &[SSPages; (MAX_STEM + 1) as usize],
    getout: &GetOut,
    log: &Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
    d: ExtTauMult,
    trace: &mut ProofTrace,
) -> ChoiceResult {
    let (from_name, to_name) = model.get_names(d.from, d.to);

    let filter = filter_tau(data, model, alg_ehp, bot_trunc, top_trunc, d);

    if let Some(kind) = filter {
        if depth == 0 {
            log.lock().unwrap().push(Action::AddExt {
                from: from_name.clone(),
                to: to_name.clone(),
                af: d.af,
                kind,
                proof: None,
            });
        }

        if ALWAYS_PRINT || depth == 0 {
            println!("Finished tau by {kind:?}: {} -> {}", from_name, to_name);
        }
        trace.known(
            depth,
            ProofChoice::FilteredTau(d, kind),
            format!("{kind:?}"),
        );
        data.add_ext_tau(model, d.from, d.to, d.af, None, kind);
        return ChoiceResult::Chosen;
    }

    if ALWAYS_PRINT || depth == 0 {
        println!(
            "Trying tau: {} | {} | af: {} | S^{}",
            from_name,
            to_name,
            d.af,
            top_trunc + 1
        );
    }

    let run = |world: usize, control: GetOut| {
        let mut world_data = data.clone();
        let kind = if world == 0 { Kind::Real } else { Kind::Fake };
        world_data.add_ext_tau(model, d.from, d.to, d.af, None, kind);
        ehp_iterate(
            world_data,
            model,
            alg_ehp,
            ahss_and_alg_data,
            ahss_pages,
            control,
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
            stop_at_default(),
        )
    };

    match discover_choice(
        depth,
        getout,
        [ProofChoice::Tau(d, true), ProofChoice::Tau(d, false)],
        run,
    ) {
        SpeculativeBranchOutcome::Contradiction(proof) => ChoiceResult::Contradiction(proof),
        SpeculativeBranchOutcome::ChooseRight(e) => {
            trace.resolved(depth, ProofChoice::Tau(d, false), &e);
            commit_tau_choice(data, model, log, depth, d, Commitment::Fake(e), false);
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::ChooseLeft(e) => {
            trace.resolved(depth, ProofChoice::Tau(d, true), &e);
            commit_tau_choice(data, model, log, depth, d, Commitment::Real(e), false);
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::BothOpen => {
            commit_tau_choice(data, model, log, depth, d, Commitment::Unknown, false);
            if ALWAYS_PRINT || depth == 0 {
                println!("BothOpen: {from_name} | {to_name} tau multiple");
            }
            ChoiceResult::Open
        }
        SpeculativeBranchOutcome::Cancelled => ChoiceResult::Cancelled,
    }
}

/// Add the deferred differentials (lifted from AHSS, or algebraic) that belong
/// to the current page. Algebraic ones go in unconditionally; lifted ones only
/// when both endpoints are still alive at this truncation (otherwise the lift
/// isn't yet justified and we let the search find its own support first).
fn add_diffs(
    data: &mut SyntheticSS,
    model: &E1,
    ahss_and_alg_data: &Vec<Vec<Vec<Vec<(usize, usize, Kind, Option<String>)>>>>,
    stem: i32,
    log: &Arc<Mutex<Vec<Action>>>,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
) -> Result<(), String> {
    let d_y = top_trunc - bot_trunc + 1;


    for (from, to, k, p) in &ahss_and_alg_data[stem as usize][d_y as usize][top_trunc as usize] {
        if *k != Kind::Algebraic {
            let pages = try_compute_pages(&data, model, 0, top_trunc, stem, stem, false)
                .map_err(|x| format!("{x:?}"))?;

            if let Some((_, f_torsion)) = pages.try_element_final(*from)
                && f_torsion.alive()
            {
                if let Some((_, t_torsion)) = pages.try_element_final(*to)
                    && t_torsion.alive()
                {
                    if depth == 0 {
                        let (from_name, to_name) = model.get_names(*from, *to);
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
                    data.add_diff(model, *from, *to, None, *k);
                } else {

                    // This is too strong! as i have not yet given EHP the chance to find differentials which could support this.
                    // What we should check is that AFTER a top/bot truncation have been done, that it is compatible at each page wrt. torsion structure stuff ?
                    // return Err(format!("We have at E_{} that {} is alive while {} is dead. This is not compatible with the AHSS / AEHP.", top_trunc-bot_trunc, data.model.name(*from), data.model.name(*to)));
                }
            }
        } else {
            data.add_diff(model, *from, *to, None, *k);
        }
    }

    Ok(())
}

/// Entry point: run the automated EHP solver given an already-solved AHSS and an
/// optional starting log. Returns the produced log and the resulting sequence.
pub fn ehp_solver(ahss: &SyntheticSS, log: Option<Vec<Action>>) -> (Vec<Action>, SyntheticSS) {
    ehp_solver_until(ahss, log, stop_at_default())
}

fn stop_at_default() -> (i32, i32) {
    (MAX_VERIFY_STEM, 0)
}

fn ehp_solver_until(
    ahss: &SyntheticSS,
    log: Option<Vec<Action>>,
    stop_at: (i32, i32),
) -> (Vec<Action>, SyntheticSS) {
    // Start from the AHSS facts that are valid in the metastable range.
    let mut partial_ehp = SyntheticSS::empty(MODEL.clone());

    let _ = set_metastable_range(&mut partial_ehp, ahss);

    // Differentials longer than length 1 are deferred into `ahss_and_alg_data`
    // (keyed by stem / length / top filtration) to be added at the right page,
    // exactly as in the AHSS solver; length-1 ones are added immediately.
    let mut ahss_and_alg_data =
        vec![
            vec![vec![vec![]; (MAX_STEM + 2) as usize]; (MAX_STEM + 1) as usize];
            (MAX_STEM + 1) as usize
        ];

    let mut log = log.unwrap_or(vec![]);

    // Seed the EHP's own algebraic differentials (skipping the metastable ones,
    // already added above).
    for (&(from, to), (kind, _)) in &DATA.from_to {
        let d_y = MODEL.y(from) - MODEL.y(to);
        // Exclude metastable ones, as they have already been added
        if !in_metastable_range(MODEL.y(to), MODEL.stem(to)) {
            if d_y == 1 {
                partial_ehp.add_diff(&MODEL, from, to, None, *kind);
            } else {
                let stem = MODEL.stem(to);
                let top_trunc = MODEL.y(from);
                ahss_and_alg_data[stem as usize][d_y as usize][top_trunc as usize].push((
                    from,
                    to,
                    *kind,
                    None,
                ));
            }
        }
    }

    // Lift the AHSS's proven differentials into the EHP (the stable sequence's
    // facts must hold unstably too), skipping algebraic/unknown ones. Real
    // length-1 diffs and fakes go to the log; longer ones are deferred like above.
    for (&(from, to), (kind, _)) in &ahss.from_to {
        let d_y = STABLE_MODEL.y(from) - STABLE_MODEL.y(to);

        // Only add differentials here
        if STABLE_MODEL.stem(from) != STABLE_MODEL.stem(to) {
            if let Some(from_id) = MODEL.try_index(STABLE_MODEL.name(from)) {
                if let Some(to_id) = MODEL.try_index(STABLE_MODEL.name(to)) {
                    // Don't include the Unknown and Algebraic differentials
                    if *kind == Kind::Algebraic || *kind == Kind::Unknown {
                        continue;
                    }

                    let (from_name, to_name) = MODEL.get_names(from_id, to_id);
                    
                    if *kind == Kind::Real {
                        if d_y == 1 {
                                let (from_name, to_name) = MODEL.get_names(from_id, to_id);
                                log.push(Action::AddDiff {
                                    from: from_name,
                                    to: to_name,
                                    kind: *kind,
                                    proof: Some(format!("Lifted")),
                                });
                            } else {
                                let stem = MODEL.stem(to_id);
                                let top_trunc = MODEL.y(from_id);

                                ahss_and_alg_data[stem as usize][d_y as usize][top_trunc as usize]
                                    .push((
                                        from_id,
                                        to_id,
                                        *kind,
                                        Some(format!("Lifted")),
                                    ));
                            }
                    } else {
                        log.push(Action::AddDiff {
                            from: from_name,
                            to: to_name,
                            kind: *kind,
                            proof: Some(format!("Lifted")),
                        });
                    }
                }
            }
        }
    }

    // let mut fakes = vec![];

    // Add all external tau's
    // We won't worry about the fake ones
    for e in ahss.external_taus() {
        // Only lift taus whose endpoints both exist in the EHP model.
        if MODEL.try_index(STABLE_MODEL.name(e.from)).is_some()
            && MODEL.try_index(STABLE_MODEL.name(e.to)).is_some()
        {
            let (kind, _) = ahss
                .from_to
                .get(&(e.from, e.to))
                .unwrap().clone();
            log.push(Action::AddExt {
                from: STABLE_MODEL.name(e.from).to_string(),
                to: STABLE_MODEL.name(e.to).to_string(),
                af: e.af,
                kind: kind,
                proof: Some("Lifted".to_string()),
            });
        }
    }


    let ehp = revert_log_and_remake(0, &mut log, &MODEL, &partial_ehp, false);
    let log = Arc::new(Mutex::new(log));

    // Precompute each AHSS sphere's pages once, so `check_issue` can cheaply
    // verify EHP -> AHSS compatibility while the search runs.
    let ahss_pages = std::array::from_fn(|x| compute_pages(&ahss, &STABLE_MODEL, 0, x as i32, 0, 150, false).0);

    // Drive the search from the first cell; it appends every fact to `log`.
    let res = ehp_iterate(
        ehp,
        &MODEL,
        &DATA,
        &ahss_and_alg_data,
        &ahss_pages,
        empty_getout(),
        log.clone(),
        2,
        2,
        1,
        0,
        stop_at,
    );

    // Rebuild the final sequence from the produced log on top of the algebraic
    // differentials, so the result is exactly what replaying the log yields.
    for (&(from, to), _) in &DATA.from_to {
        partial_ehp.add_diff(&MODEL, from, to, None, Kind::Algebraic);
    }

    println!("{res:?}");

    let mut log = Arc::try_unwrap(log).unwrap().into_inner().unwrap();
    let ehp = revert_log_and_remake(0, &mut log, &MODEL, &partial_ehp, false);
    (log, ehp)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data::curtis::STABLE_DATA, io::import::get_log};
    use std::{collections::HashSet, time::Instant};

    fn without_proof(mut action: Action) -> Action {
        match &mut action {
            Action::AddDiff { proof, .. } | Action::AddExt { proof, .. } => *proof = None,
            Action::AddInt { proof, .. }
            | Action::SetE1 { proof, .. }
            | Action::SetInducedName { proof, .. } => proof.clear(),
            Action::Revert { .. } => {}
        }
        action
    }

    /// Recompute through stem 25 from the manual EHP facts, keeping the stable
    /// answer fixed. No generated data or saved logs are written by this test.
    #[test]
    #[ignore = "release-mode EHP stem 25 benchmark"]
    fn benchmark_ehp_stem_25() {
        let mut stable_log = get_log(false, true).unwrap();
        let ahss = revert_log_and_remake(0, &mut stable_log, &STABLE_MODEL, &STABLE_DATA, true);
        let manual = get_log(true, false).unwrap();
        let start = Instant::now();
        // Visit the low-sphere prerequisites in stems 26/27 before finishing 25.
        let (log, _) = ehp_solver_until(&ahss, Some(manual), (26, 4));
        println!(
            "EHP through stem 25: {:?}; {} actions",
            start.elapsed(),
            log.len()
        );
        let proof = log
            .iter()
            .find_map(|a| match a {
                Action::AddDiff {
                    from,
                    to,
                    kind: Kind::Real,
                    proof,
                } if from == "2 2 4 5 3 3 3[4]" && to == "2 4 1 1 2 4 3 3 3[2]" => proof.as_deref(),
                _ => None,
            })
            .expect("The stem 25 differential must be rediscovered with a proof");
        assert!(proof.starts_with("assume diff "), "{proof}");
        assert!(
            proof.contains("[2] assume"),
            "Nested evidence was lost: {proof}"
        );
        assert!(proof.contains("assume diff 2 2 4 5 3 3 3[4] -> 2 4 1 1 2 4 3 3 3[2] absent"));
        let (reduced, full) = proof.split_once("\nFull proof tree").expect("Both proof views must be saved");
        assert!(!reduced.contains("common L"));
        assert!(!reduced.contains("Companion root:"));
        assert!(!reduced.contains("assume diff 2 2 4 5 3 3 3[4] -> 2 4 1 1 2 4 3 3 3[2] exists"));
        assert!(full.contains("bounded companion:"));
        assert!(full.contains("assume diff 2 2 4 5 3 3 3[4] -> 2 4 1 1 2 4 3 3 3[2] exists"));
        let example = log.iter().find_map(|a| match a {
            Action::AddDiff { from, to, kind: Kind::Real, proof }
                if from == "7 3 3[5]" && to == "2 4 3 3 3[2]" => proof.as_deref(),
            _ => None,
        }).expect("The compact-chain example must be rediscovered");
        assert_eq!(example.split_once("\nFull proof tree").unwrap().0,
            concat!(
                "assume diff 7 3 3[5] -> 2 4 3 3 3[2] absent\n",
                "-> 3 3 3[9] -> 2 4 3 3 3[2] exists\n",
                "-> contradiction: Issue at S^5 | stem 21: [AlgebraicConvergence { bot_trunc: 0, top_trunc: 4, stem: 21, af: 6, expected: 0, observed: 1 }]\n",
            ));
        println!("Example proof: {} bytes reduced; {} bytes full", reduced.len(), full.len());
        if std::env::var_os("EHP_SHOW_PROOF").is_some() {
            println!("BEGIN STEM 25 PROOF\n{proof}END STEM 25 PROOF");
        }

        let expected: HashSet<_> = get_log(false, false)
            .unwrap()
            .into_iter()
            .map(without_proof)
            .collect();
        for action in &log {
            assert!(
                expected.contains(&without_proof(action.clone())),
                "Discovery changed a saved verdict: {action:?}"
            );
        }
    }
}
