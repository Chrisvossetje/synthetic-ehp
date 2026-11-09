//! The automated AHSS solver. [`ahss_solver`] drives a recursive search
//! ([`ahss_iterate`]) that walks every stem and RP^n truncation, proposing
//! differentials and tau-multiplications and committing to each as Real or Fake
//! by speculatively exploring both worlds (`try_diff`/`try_tau` via
//! [`crate::solve::search`]). E1-page torsion ambiguities are resolved by
//! branching over the candidate assignments in [`iterate_e1_issues`]. The
//! `filter_*` helpers shortcut differentials whose status is forced by theory
//! (James periodicity / minimal length / additive structure).

use std::sync::{
    Arc, Mutex,
};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    MAX_STEM, MAX_VERIFY_STEM, data::{
        curtis::{STABLE_DATA, STABLE_MODEL}, naming::generate_names_from_tag, r#static::{algebraic_rp, rp_truncations, synthetic_rp}
    }, domain::{
        e1::E1,
        model::{Diff, ExtTauMult, SyntheticSS},
        process::compute_pages,
    }, solve::{
        action::{Action, D_R_REPEATS, process_action, revert_log_and_remake},
        ahss::{ahss_synthetic_e1_issue, get_all_e1_solutions},
        automated::{
            ALWAYS_PRINT, Commitment, PARALLEL_DEPTH, TauIssue, classify_tau_issue,
            commit_diff_choice, commit_tau_choice, filter_diff,
        },
        generate::get_a_diff,
        issues::{
            Issue, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic,
        },
        proof::{ProofChoice, ProofTrace, SearchProof},
        search::{
            BranchResult, ChoiceResult, GetOut, SpeculativeBranchOutcome, discover_choice, check_getout, create_getout, empty_getout, signal_parent_getout
        },
        solve::{
            auto_deduce, suggest_tau_solution_algebraic, suggest_tau_solution_generator_synthetic,
        },
    }, types::{Kind, Torsion}
};

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
    trace: &mut ProofTrace,
) -> Option<BranchResult> {
    trace.at(getout.position);
    let candidates = &e1_issues[stem as usize];
    if candidates.is_empty() {
        return None;
    }
    if depth >= getout.max_depth() {
        return Some(BranchResult::Open(std::mem::take(trace).open(depth)));
    }
    let choice = |index: usize| ProofChoice::E1 {
        stem,
        assignments: candidates[index]
            .0
            .iter()
            .copied()
            .filter(|(id, _)| model.y(*id) == 1)
            .collect(),
    };
    let g = create_getout(getout, candidates.len() as i32, depth);
    let run = |(index, candidate): (usize, &(Vec<(usize, Torsion)>, Vec<Action>))| {
        let mut world = data.clone();
        for &(id, torsion) in &candidate.0 {
            world.generators[id] = torsion;
        }
        ahss_iterate(
            world,
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
        .assuming(depth + 1, choice(index))
    };
    let outcomes: Vec<_> = if depth < PARALLEL_DEPTH {
        candidates.par_iter().enumerate().map(run).collect()
    } else {
        candidates.iter().enumerate().map(run).collect()
    };
    if check_getout(getout) {
        return Some(BranchResult::Cancelled(
            std::mem::take(trace).cancelled(depth),
        ));
    }
    let viable: Vec<_> = outcomes
        .iter()
        .enumerate()
        .filter(|(_, r)| !matches!(r, BranchResult::Contradiction(_)))
        .map(|(i, _)| i)
        .collect();
    let refutations: Vec<_> = outcomes
        .iter()
        .filter_map(|r| match r {
            BranchResult::Contradiction(proof) => Some(proof.clone()),
            _ => None,
        })
        .collect();
    let proof =
        (!refutations.is_empty()).then(|| SearchProof::cases(depth, getout.position, refutations));
    if let [index] = viable.as_slice() {
        let index = *index;
        if let Some(proof) = &proof {
            trace.resolved(depth, choice(index), proof);
        } else {
            trace.known(depth, choice(index), "unique E1 assignment");
        }
        if depth == 0 {
            for mut action in candidates[index].1.clone() {
                if let Some(proof) = &proof
                    && let Action::SetE1 { proof: text, .. } = &mut action
                {
                    *text = proof.describe(model);
                }
                log.lock().unwrap().push(action);
            }
        }
        for &(id, torsion) in &candidates[index].0 {
            data.generators[id] = torsion;
        }
        return None;
    }
    if viable.is_empty() {
        signal_parent_getout(getout, depth);
        return Some(BranchResult::Contradiction(
            std::mem::take(trace).contradiction_by(depth, proof.unwrap()),
        ));
    }
    if outcomes.iter().any(|r| matches!(r, BranchResult::Open(_))) {
        Some(BranchResult::Open(std::mem::take(trace).open(depth)))
    } else {
        Some(BranchResult::Cancelled(
            std::mem::take(trace).cancelled(depth),
        ))
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
    let mut trace = ProofTrace::default();
    // The solver sweeps (stem, top_trunc) outward. Each iteration tries to make
    // progress at the current cell; once nothing is left to do there it advances
    // the truncation/stem at the bottom of the loop. We never recurse here —
    // recursion happens inside try_diff/try_tau when a choice must be guessed.
    loop {
        trace.at(getout.position);
        // Depth-0 (the real run) stops once it has verified the whole range;
        // deeper speculative branches stop at MAX_DEPTH and report Open (no
        // contradiction found, so the guess that spawned them stays plausible).
        if depth == 0 && stem >= MAX_VERIFY_STEM {
            return BranchResult::Open(trace.open(depth));
        }

        if depth > getout.max_depth() || getout.past_frontier() || stem >= MAX_STEM {
            if ALWAYS_PRINT {
                println!("DEPTH REACHED");
            }
            return BranchResult::Open(trace.open(depth));
        }

        // A sibling branch already decided this subtree; bail out early.
        if check_getout(&getout) {
            return BranchResult::Cancelled(trace.cancelled(depth));
        }

        // 1. Propose the next plausible differential at this cell and try it.
        //    (Only within the James-periodicity window for this page length.)
        if bot_trunc <= D_R_REPEATS[(top_trunc - bot_trunc) as usize] as i32 {
            let option = get_a_diff(&data, model, top_trunc, bot_trunc, stem);

            if let Some(d) = option {
                if depth >= getout.max_depth() {
                    return BranchResult::Open(trace.open(depth));
                }
                match try_diff(
                    &mut data, model, alg_ahss, alg_data, e1_issues, &getout, &log, stem,
                    top_trunc, bot_trunc, depth, d, &mut trace,
                ) {
                    ChoiceResult::Contradiction(proof) => {
                        signal_parent_getout(&mut getout, depth);
                        return BranchResult::Contradiction(trace.contradiction_by(depth, proof));
                    }
                    ChoiceResult::Chosen => continue,
                    ChoiceResult::Open => return BranchResult::Open(trace.open(depth)),
                    ChoiceResult::Cancelled => {
                        return BranchResult::Cancelled(trace.cancelled(depth));
                    }
                }
            }
        }

        // 2. No differential to add — check convergence at this cell. A clean
        //    result means we move on; a contradiction kills this branch; a
        //    tau-fixable mismatch tells us which kind of tau to look for next.
        let potential_tau_thing = match is_tau_issue(&data, model, stem, top_trunc, bot_trunc) {
            Ok(tau_issue) => tau_issue,
            Err(is) => {
                signal_parent_getout(&mut getout, depth);
                return BranchResult::Contradiction(trace.contradiction(depth, is));
            }
        };

        if let Some((synthetic, mut issues)) = potential_tau_thing {
            // Ask the matching suggester for a concrete tau to try.
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
                    &mut data, model, alg_ahss, alg_data, e1_issues, &getout, &log, stem,
                    top_trunc, bot_trunc, depth, d, &mut trace,
                ) {
                    ChoiceResult::Contradiction(proof) => {
                        signal_parent_getout(&mut getout, depth);
                        return BranchResult::Contradiction(trace.contradiction_by(depth, proof));
                    }
                    ChoiceResult::Chosen => continue,
                    ChoiceResult::Open => return BranchResult::Open(trace.open(depth)),
                    ChoiceResult::Cancelled => {
                        return BranchResult::Cancelled(trace.cancelled(depth));
                    }
                }
            } else {
                // We know a tau is needed but couldn't construct one: dead end.
                signal_parent_getout(&mut getout, depth);
                return BranchResult::Contradiction(trace.contradiction(
                    depth,
                    format!("Issue at RP{bot_trunc}_{top_trunc}: {issues:?}"),
                ));
            }
        }

        // 3. Nothing left to decide at this cell. Fold in the algebraic
        //    differentials that belong to this page, then widen the truncation
        //    window (bot_trunc down) and revisit.
        if bot_trunc != 0 {
            let d_y = top_trunc - bot_trunc + 1;
            for &(from, to) in &alg_data[stem as usize][d_y as usize][top_trunc as usize] {
                if depth == 0 {
                    let (from_name, to_name) = model.get_names(from, to);
                    println!("Applying Algebraic diff {from_name} -> {to_name}");
                }
                data.add_diff(model, from, to, None, Kind::Algebraic);
            }
            getout.position += 1;
            bot_trunc -= 1;
            continue;
        }

        // 4. This stem's truncations are exhausted. Advance: either climb to the
        //    next top_trunc, or roll over to the next stem and first resolve any
        //    E1-page torsion ambiguities it introduces.
        getout.position += 1;
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
                &mut trace,
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
    trace: &mut ProofTrace,
) -> ChoiceResult {
    let (from_name, to_name) = model.get_names(d.from, d.to);

    // If theory already settles this differential, record it and skip the search.
    let filter = filter_diff(data, model, alg_ahss, bot_trunc, top_trunc, d, true);

    if let Some(kind) = filter {
        if ALWAYS_PRINT || depth == 0 {
            println!("Finished diff by: {} | {} -> {kind:?}", from_name, to_name);
        }
        if depth == 0 {
            log.lock().unwrap().push(Action::AddDiff {
                from: from_name,
                to: to_name,
                proof: None,
                kind,
            });
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
        println!("Trying diff: {} | {}", from_name, to_name);
    }

    // Otherwise guess: explore two cloned worlds, one where the differential is
    // Real and one where it is Fake, each continuing the search one level deeper.
    // Whichever world hits a contradiction rules out that choice for the other.
    let run = |world: usize, control: GetOut| {
        let mut world_data = data.clone();
        let kind = if world == 0 { Kind::Real } else { Kind::Fake };
        let action = Action::AddDiff {
            from: from_name.clone(),
            to: to_name.clone(),
            kind,
            proof: None,
        };
        if kind == Kind::Real {
            process_action(&mut world_data, model, &action, true).unwrap();
        } else {
            world_data.add_diff(model, d.from, d.to, None, kind);
        }
        ahss_iterate(
            world_data,
            model,
            alg_ahss,
            alg_data,
            e1_issues,
            control,
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
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
            commit_diff_choice(data, model, log, depth, d, Commitment::Fake(e), true);
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::ChooseLeft(e) => {
            trace.resolved(depth, ProofChoice::Diff(d, true), &e);
            commit_diff_choice(data, model, log, depth, d, Commitment::Real(e), true);
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::BothOpen => {
            commit_diff_choice(data, model, log, depth, d, Commitment::Unknown, true);
            ChoiceResult::Open
        }
        SpeculativeBranchOutcome::Cancelled => ChoiceResult::Cancelled,
    }
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
        Err(issues) => {
            let context = format!("For RP{bot_trunc}_{top_trunc}");
            classify_tau_issue(issues, &context, &context)
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
    trace: &mut ProofTrace,
) -> ChoiceResult {
    let (from_name, to_name) = model.get_names(d.from, d.to);

    if ALWAYS_PRINT || depth == 0 {
        println!(
            "Trying tau: {} | {} | af: {} | RP{bot_trunc}_{top_trunc}",
            from_name, to_name, d.af
        );
    }

    let run = |world: usize, control: GetOut| {
        let mut world_data = data.clone();
        let kind = if world == 0 { Kind::Real } else { Kind::Fake };
        let action = Action::AddExt {
            from: from_name.clone(),
            to: to_name.clone(),
            af: d.af,
            kind,
            proof: None,
        };
        process_action(&mut world_data, model, &action, true).unwrap();
        ahss_iterate(
            world_data,
            model,
            alg_ahss,
            alg_data,
            e1_issues,
            control,
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
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
            commit_tau_choice(data, model, log, depth, d, Commitment::Fake(e), true);
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::ChooseLeft(e) => {
            trace.resolved(depth, ProofChoice::Tau(d, true), &e);
            commit_tau_choice(data, model, log, depth, d, Commitment::Real(e), true);
            ChoiceResult::Chosen
        }
        SpeculativeBranchOutcome::BothOpen => {
            commit_tau_choice(data, model, log, depth, d, Commitment::Unknown, true);
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

/// Entry point: run the automated AHSS solver from an optional starting log,
/// returning the full action log it produced and the resulting spectral sequence.
pub fn ahss_solver(log: Option<Vec<Action>>) -> (Vec<Action>, SyntheticSS) {
    let alg_ahss = STABLE_DATA.clone();
    let model: &E1 = &STABLE_MODEL;
    let mut partial_ahss = SyntheticSS::empty(model.clone());

    // Seed the algebraic differentials. The short ones (length 1, or whose James
    // period fits below the bottom cell) can go straight into the data; the rest
    // are deferred into `alg_data`, indexed by (stem, length, top filtration), so
    // the search adds them at the right page as it widens the truncation window.
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


    // Replay any starting log (its SetE1 actions establish E1 torsion), then
    // precompute the candidate E1 assignments the search will branch over.
    let ahss = revert_log_and_remake(0, &mut log, model, &partial_ahss, true);
    let e1_issues = ahss_solve_e1_issues(&ahss, model, &mut log);


    let ahss = revert_log_and_remake(0, &mut log, model, &ahss, true);
    let log = Arc::new(Mutex::new(log));

    // Drive the search from the very first cell (stem 2, RP^1_2). It appends
    // every committed fact to the shared `log`.
    let res = ahss_iterate(ahss, model, &alg_ahss, &alg_data, &e1_issues, empty_getout(), log.clone(),
    2, 2, 1, 0);

    // Rebuild the final sequence from the produced log on top of the algebraic
    // differentials, so the result is exactly what replaying the log yields.
    for (&(from, to), (kind, _)) in &alg_ahss.from_to {
        partial_ahss.add_diff(model, from, to, None, *kind);
    }

    println!("{res:?}");

    let mut log = Arc::try_unwrap(log).unwrap().into_inner().unwrap();
    let ahss = revert_log_and_remake(0, &mut log, model, &partial_ahss, true);
    (log, ahss)
}
