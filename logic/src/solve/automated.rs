use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use crate::{
    MAX_AUTOMATED_TOP_TRUNC, MAX_STEM, MAX_VERIFY_STEM,
    data::{
        compare::{algebraic_rp, rp_truncations, synthetic_rp},
        curtis::STABLE_DATA,
    },
    domain::{
        model::{Diff, ExtTauMult, FromTo, SyntheticSS},
        process::compute_pages,
    },
    solve::{
        action::{Action, D_R_REPEATS, revert_log_and_remake},
        ahss::ahss_synthetic_e1_issue,
        ahss_e1::get_all_e1_solutions,
        generate::get_a_diff,
        issues::{
            Issue, algebraic_issue_is_fixable_by_tau_extensions, compare_algebraic,
            compare_algebraic_spectral_sequence, compare_synthetic,
            synthetic_issue_is_tau_structure_issue,
        },
        solve::{
            auto_deduce, suggest_tau_solution_algebraic, suggest_tau_solution_generator_synthetic,
        },
    },
    types::{Kind, Torsion},
};

pub const PARALLEL_DEPTH: i32 = 3;
pub const ALWAYS_PRINT: bool = false;

fn check_issue(
    data: &SyntheticSS,
    stem: i32,
    bot_trunc: i32,
    top_trunc: i32,
) -> Result<(), Vec<Issue>> {
    for &(synthetic, bt, tt) in rp_truncations() {
        if (top_trunc == tt || (stem + 1 == top_trunc && tt == 256)) && bot_trunc == bt {
            let pages = (if synthetic {
                let (pages, issues) = compute_pages(data, bt, tt, stem, stem, true);

                let observed = pages.convergence_at_stem(data, stem);

                compare_synthetic(&observed, synthetic_rp(bt, tt), bt, top_trunc, stem)?;

                if issues.len() != 0 {
                    return Err(issues);
                }
                pages
            } else {
                let (pages, issues) = compute_pages(data, bt, tt, stem - 1, stem, true);

                let observed = pages.algebraic_convergence_at_stem(data, stem);

                compare_algebraic(&observed, algebraic_rp(bt, tt), bt, tt, stem)?;

                if issues.len() != 0 {
                    return Err(issues);
                }
                pages
            });
            compare_algebraic_spectral_sequence(data, &pages, stem, bt, tt, true)?;
        }
    }
    Ok(())
}

fn filter_diff(
    data: &SyntheticSS,
    alg_ahss: &SyntheticSS,
    bot_trunc: i32,
    top_trunc: i32,
    d: Diff,
) -> Option<(Kind, String)> {
    let stem = data.model.stem(d.to);
    let y = data.model.y(d.from);

    if y == 3 || y == 7 || (y == 15 && stem != 14) || (y == 31 && stem != 30) {
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
        && let Some(alg_to) = alg_ahss.out_diffs[d.from].first()
        && data.model.original_torsion(*alg_to).alive()
        && data.model.y(*alg_to) + 1 == bot_trunc
    {
        Some((
            Kind::Unknown,
            format!("We don't have enough Synthetic information to deduce this differential."),
        ))
    } else if top_trunc & 1 == 1
        && !(top_trunc == 5 && bot_trunc == 3)
        && let Some(dies) = data.model.get(d.to).dies
        && let Some(source) = alg_ahss.in_diffs[d.to].first()
        && data.model.original_torsion(*source).free()
        && top_trunc + 2 == dies
    {
        Some((
            Kind::Unknown,
            format!("We don't have enough Synthetic information to deduce this differential."),
        ))
    } else {
        None
    }
}

fn ahss_iterate(
    data: SyntheticSS,
    alg_ahss: &SyntheticSS,
    alg_data: &Vec<Vec<Vec<Vec<FromTo>>>>,
    mut getout: [Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize],
    log: Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
    bounded: bool,
    tau_mode: bool,
) -> Result<bool, String> {
    for g in &getout {
        if let Some(g) = g {
            if g.load(Ordering::Relaxed) {
                return Ok(false);
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
        return try_diff(
            data, alg_ahss, alg_data, getout, &log, stem, top_trunc, bot_trunc, depth, bounded, d,
        );
    }

    let potential_tau_thing = match is_tau_issue(&data, stem, top_trunc, bot_trunc, depth) {
        Ok(tau_issue) => tau_issue,
        Err(is) => {
            if depth <= PARALLEL_DEPTH
                && depth != 0
                && let Some(getout) = &mut getout[(depth - 1) as usize]
            {
                getout.store(true, Ordering::Relaxed);
            }
            return Err(is);
        }
    };

    if let Some((synthetic, mut issues)) = potential_tau_thing {
        let option = match synthetic {
            TauIssue::AlgTauIssue => {
                suggest_tau_solution_algebraic(&data, &mut issues, top_trunc, bot_trunc, stem)
            }
            TauIssue::SynTauGeneratorIssue => suggest_tau_solution_generator_synthetic(
                &data,
                &mut issues,
                top_trunc,
                bot_trunc,
                stem,
            ),
            TauIssue::SynTauModuleIssue => {
                // get_a_tau(&data, top_trunc, bot_trunc, stem)
                suggest_tau_solution_generator_synthetic(
                    &data,
                    &mut issues,
                    top_trunc,
                    bot_trunc,
                    stem,
                )
            }
        };

        if let Some(d) = option {
            return try_tau(
                data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded,
                d,
            );
        } else {
            if depth <= PARALLEL_DEPTH
                && depth != 0
                && let Some(getout) = &mut getout[(depth - 1) as usize]
            {
                getout.store(true, Ordering::Relaxed);
            }
            return Err(format!("Issue at RP{bot_trunc}_{top_trunc}: {issues:?}"));
        }
    }

    // TODO:
    // if bounded {
    //     return Ok(true);
    // }

    if bot_trunc != 0 {
        return add_algebraic_diffs(
            data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded,
        );
    }

    if top_trunc == stem + 1 || top_trunc >= MAX_AUTOMATED_TOP_TRUNC {
        Ok(true)
    } else {
        ahss_iterate(
            data,
            alg_ahss,
            alg_data,
            getout,
            log,
            stem,
            top_trunc + 1,
            top_trunc,
            depth,
            bounded,
            tau_mode,
        )
    }
}

fn try_diff(
    mut data: SyntheticSS,
    alg_ahss: &SyntheticSS,
    alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>,
    mut getout: [Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize],
    log: &Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
    bounded: bool,
    d: Diff,
) -> Result<bool, String> {
    let (from_name, to_name) = data.get_names(d.from, d.to);

    let filter = filter_diff(&data, alg_ahss, bot_trunc, top_trunc, d);

    if ALWAYS_PRINT || depth == 0 {
        println!("Trying diff: {} | {}", from_name, to_name);
    }

    if let Some((kind, reason)) = filter {
        if depth == 0 {
            log.lock().unwrap().push(Action::AddDiff {
                from: from_name,
                to: to_name,
                proof: Some(reason.clone()),
                kind,
            });
        }

        data.add_diff(d.from, d.to, Some(reason), kind);
        return ahss_iterate(
            data,
            alg_ahss,
            alg_data,
            getout.clone(),
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth,
            bounded,
            false,
        );
    }

    if depth < PARALLEL_DEPTH {
        getout[depth as usize] = Some(Arc::new(AtomicBool::new(false)));
    }

    let with = || {
        let mut with_data = data.clone();
        with_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Real);
        ahss_iterate(
            with_data,
            alg_ahss,
            alg_data,
            getout.clone(),
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
            bounded,
            false,
        )
    };
    let without = || {
        let mut without_data = data.clone();
        without_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Fake);
        ahss_iterate(
            without_data,
            alg_ahss,
            alg_data,
            getout.clone(),
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
            bounded,
            false,
        )
    };

    if depth < PARALLEL_DEPTH {
        let (with_res, without_res) = rayon::join(with, without);

        if let Err(e) = with_res {
            // DISPROOF !
            let (from_name, to_name) = data.get_names(d.from, d.to);

            // Proof: Tau Issue: false | SyntheticConvergence { bot_trunc: 1, top_trunc: 30, stem: 30, af: 4, expected: [Torsion(Some(1))], observed: [Torsion(None)] }

            if ALWAYS_PRINT || depth == 0 {
                println!("Disproven diff: {} | {} by {e}", from_name, to_name);
            }
            // Commit choice !
            if depth == 0 {
                log.lock().unwrap().push(Action::AddDiff {
                    from: from_name,
                    to: to_name,
                    proof: Some(e.clone()),
                    kind: Kind::Fake,
                });
            }

            data.add_diff(d.from, d.to, Some(e), Kind::Fake);

            // And iterate further
            getout[depth as usize] = None;
            return ahss_iterate(
                data,
                alg_ahss,
                alg_data,
                getout,
                log.clone(),
                stem,
                top_trunc,
                bot_trunc,
                depth,
                bounded,
                false,
            );
        } else if let Err(e) = without_res {
            let (from_name, to_name) = data.get_names(d.from, d.to);

            data.add_diff(d.from, d.to, Some(e.clone()), Kind::Real);

            if ALWAYS_PRINT || depth == 0 {
                println!(
                    "Proven diff: {} | {} | {:?}",
                    from_name,
                    to_name,
                    Some(e.clone())
                );
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddDiff {
                    from: from_name,
                    to: to_name,
                    proof: Some(e),
                    kind: Kind::Real,
                });
            }

            getout[depth as usize] = None;
            return ahss_iterate(
                data,
                alg_ahss,
                alg_data,
                getout,
                log.clone(),
                stem,
                top_trunc,
                bot_trunc,
                depth,
                bounded,
                false,
            );
        } else if !matches!(with_res, Ok(true)) || !matches!(without_res, Ok(true)) {
            return Ok(false);
        } else {
            let without = without_res;

            let (from_name, to_name) = data.get_names(d.from, d.to);

            if matches!(without, Ok(true)) && depth == 0 {
                println!(
                    "WITH OR WITHOUT ARE BOTH FINE FOR THE DIFFERENTIAL: {:?} | {:?}",
                    d,
                    data.get_names(d.from, d.to)
                );
            }

            // If with and without are both Ok then we continue WITHOUT the differential
            // But we do remember that we don't know the result of this diff
            let kind = Kind::Unknown;
            let proof = None;

            data.add_diff(d.from, d.to, proof.clone(), kind);

            // Commit choice !
            if ALWAYS_PRINT || depth == 0 {
                if kind == Kind::Unknown {
                    println!("Unknown diff: {} | {}", from_name, to_name);
                } else {
                    println!("Proven diff: {} | {} | {proof:?}", from_name, to_name);
                }
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddDiff {
                    from: from_name,
                    to: to_name,
                    proof,
                    kind,
                });
            }

            getout[depth as usize] = None;
            return ahss_iterate(
                data,
                alg_ahss,
                alg_data,
                getout,
                log.clone(),
                stem,
                top_trunc,
                bot_trunc,
                depth,
                bounded,
                false,
            );
        }
    } else {
        let with_res = with();
        if let Err(e) = with_res {
            // DISPROOF !
            let (from_name, to_name) = data.get_names(d.from, d.to);

            // Commit choice !
            if ALWAYS_PRINT || depth == 0 {
                println!("Disproven diff: {} | {} by {e}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddDiff {
                    from: from_name,
                    to: to_name,
                    proof: Some(e.clone()),
                    kind: Kind::Fake,
                });
                getout = [const { None }; PARALLEL_DEPTH as usize];
            }

            data.add_diff(d.from, d.to, Some(e), Kind::Fake);

            // And iterate further
            return ahss_iterate(
                data,
                alg_ahss,
                alg_data,
                getout.clone(),
                log.clone(),
                stem,
                top_trunc,
                bot_trunc,
                depth,
                bounded,
                false,
            );
        } else if !matches!(with_res, Ok(true)) {
            return Ok(false);
        } else {
            let without = without();

            let (from_name, to_name) = data.get_names(d.from, d.to);

            if matches!(without, Ok(true)) && depth == 0 {
                println!(
                    "WITH OR WITHOUT ARE BOTH FINE FOR THE DIFFERENTIAL: {:?} | {:?}",
                    d,
                    data.get_names(d.from, d.to)
                );
            }

            // If with and without are both Ok then we continue WITHOUT the differential
            // But we do remember that we don't know the result of this diff
            let (kind, proof) = match without {
                Err(e) => (Kind::Real, Some(e)),
                Ok(true) => (Kind::Unknown, None),
                Ok(false) => return Ok(false),
            };

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
                log.lock().unwrap().push(Action::AddDiff {
                    from: from_name,
                    to: to_name,
                    proof,
                    kind,
                });
                getout = [const { None }; PARALLEL_DEPTH as usize];
            }

            return ahss_iterate(
                data,
                alg_ahss,
                alg_data,
                getout.clone(),
                log.clone(),
                stem,
                top_trunc,
                bot_trunc,
                depth,
                bounded,
                false,
            );
        }
    }
}

// Result Ok means fixable, Err is quit out
// Ok None means it is all fine!
// Ok Some gives back the errors

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TauIssue {
    AlgTauIssue,
    SynTauGeneratorIssue,
    SynTauModuleIssue,
}

fn is_tau_issue(
    data: &SyntheticSS,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
) -> Result<Option<(TauIssue, Vec<Issue>)>, String> {
    match check_issue(data, stem, bot_trunc, top_trunc) {
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
    mut data: SyntheticSS,
    alg_ahss: &SyntheticSS,
    alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>,
    mut getout: [Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize],
    log: Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
    bounded: bool,
    d: ExtTauMult,
) -> Result<bool, String> {
    let (from_name, to_name) = data.get_names(d.from, d.to);

    if ALWAYS_PRINT || depth == 0 {
        println!(
            "Trying tau: {} | {} | af: {} | RP{bot_trunc}_{top_trunc}",
            from_name, to_name, d.af
        );
    }

    if depth < PARALLEL_DEPTH {
        getout[depth as usize] = Some(Arc::new(AtomicBool::new(false)));
    }

    let with = || {
        let mut with_data = data.clone();
        with_data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Real);
        ahss_iterate(
            with_data,
            alg_ahss,
            alg_data,
            getout.clone(),
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
            bounded,
            true,
        )
    };
    let without = || {
        let mut without_data = data.clone();
        without_data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Fake);
        ahss_iterate(
            without_data,
            alg_ahss,
            alg_data,
            getout.clone(),
            log.clone(),
            stem,
            top_trunc,
            bot_trunc,
            depth + 1,
            bounded,
            true,
        )
    };

    if depth < PARALLEL_DEPTH {
        let (with_res, without_res) = rayon::join(with, without);

        if let Err(e) = with_res {
            // DISPROOF !
            let (from_name, to_name) = data.get_names(d.from, d.to);

            // Commit choice !
            if ALWAYS_PRINT || depth == 0 {
                println!("Disproven tau: {} | {} by {e}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddExt {
                    from: from_name,
                    to: to_name,
                    af: d.af,
                    proof: e.clone(),
                    kind: Kind::Fake,
                });
            }

            data.add_ext_tau(d.from, d.to, d.af, Some(e), Kind::Fake);

            // And iterate further
            getout[depth as usize] = None;
            return ahss_iterate(
                data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded,
                true,
            );
        } else if let Err(e) = without_res {
            let (from_name, to_name) = data.get_names(d.from, d.to);
            let proof = e.clone();

            data.add_ext_tau(d.from, d.to, d.af, Some(proof.clone()), Kind::Real);

            if ALWAYS_PRINT || depth == 0 {
                println!("Proven tau: {} | {} by {proof}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddExt {
                    from: from_name,
                    to: to_name,
                    af: d.af,
                    proof,
                    kind: Kind::Real,
                });
            }

            getout[depth as usize] = None;
            return ahss_iterate(
                data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded,
                true,
            );
        } else if !matches!(with_res, Ok(true)) || !matches!(without_res, Ok(true)) {
            return Ok(false);
        } else {
            let without = without_res;

            let (from_name, to_name) = data.get_names(d.from, d.to);

            if matches!(without, Ok(true)) && depth == 0 {
                println!(
                    "WITH OR WITHOUT ARE BOTH FINE FOR THE TAU: {:?} | {:?}",
                    d,
                    data.get_names(d.from, d.to)
                );
            }

            // If with and without are both Ok then we continue WITHOUT the differential
            // But we do remember that we don't know the result of this diff
            let kind = Kind::Unknown;
            let proof = "".to_string();

            data.add_ext_tau(d.from, d.to, d.af, Some(proof.clone()), kind);

            // Commit choice !
            if ALWAYS_PRINT || depth == 0 {
                println!("Proven tau: {} | {} by {proof}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddExt {
                    from: from_name,
                    to: to_name,
                    af: d.af,
                    proof,
                    kind,
                });
            }

            getout[depth as usize] = None;
            return ahss_iterate(
                data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded,
                true,
            );
        }
    } else {
        let with_res = with();
        if let Err(e) = with_res {
            // DISPROOF !
            let (from_name, to_name) = data.get_names(d.from, d.to);

            // Commit choice !
            if ALWAYS_PRINT || depth == 0 {
                println!("Disproven tau: {} | {} by {e}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddExt {
                    from: from_name,
                    to: to_name,
                    af: d.af,
                    proof: e.clone(),
                    kind: Kind::Fake,
                });
                getout = [const { None }; PARALLEL_DEPTH as usize];
            }

            data.add_ext_tau(d.from, d.to, d.af, Some(e), Kind::Fake);

            // And iterate further
            return ahss_iterate(
                data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded,
                true,
            );
        } else if !matches!(with_res, Ok(true)) {
            return Ok(false);
        } else {
            let without = without();

            let (from_name, to_name) = data.get_names(d.from, d.to);

            if matches!(without, Ok(true)) && depth == 0 {
                println!(
                    "WITH OR WITHOUT ARE BOTH FINE FOR THE TAU: {:?} | {:?}",
                    d,
                    data.get_names(d.from, d.to)
                );
            }

            // If with and without are both Ok then we continue WITHOUT the differential
            // But we do remember that we don't know the result of this diff
            let (kind, proof) = match without {
                Err(e) => (Kind::Real, e),
                Ok(true) => (Kind::Unknown, "".to_string()),
                Ok(false) => return Ok(false),
            };

            data.add_ext_tau(d.from, d.to, d.af, Some(proof.clone()), kind);

            // Commit choice !
            if ALWAYS_PRINT || depth == 0 {
                println!("Proven tau: {} | {} by {proof}", from_name, to_name);
            }
            if depth == 0 {
                log.lock().unwrap().push(Action::AddExt {
                    from: from_name,
                    to: to_name,
                    af: d.af,
                    proof,
                    kind,
                });
                getout = [const { None }; PARALLEL_DEPTH as usize];
            }

            return ahss_iterate(
                data, alg_ahss, alg_data, getout, log, stem, top_trunc, bot_trunc, depth, bounded,
                true,
            );
        }
    }
}

fn add_algebraic_diffs(
    mut data: SyntheticSS,
    alg_ahss: &SyntheticSS,
    alg_data: &Vec<Vec<Vec<Vec<(usize, usize)>>>>,
    getout: [Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize],
    log: Arc<Mutex<Vec<Action>>>,
    stem: i32,
    top_trunc: i32,
    bot_trunc: i32,
    depth: i32,
    bounded: bool,
) -> Result<bool, String> {
    // As we are moving up a page for possible diffs,
    // we should add all adams differentials which could arise from here

    let d_y = top_trunc - bot_trunc + 1;
    for &(from, to) in &alg_data[stem as usize][d_y as usize][top_trunc as usize] {
        if depth == 0 {
            let (from, to) = data.get_names(from, to);
            println!("Applying Algebraic diff {from} -> {to}");
        }
        data.add_diff(from, to, None, Kind::Real);
    }
    return ahss_iterate(
        data,
        alg_ahss,
        alg_data,
        getout.clone(),
        log,
        stem,
        top_trunc,
        bot_trunc - 1,
        depth,
        bounded,
        false,
    );
}

fn e1_to_ahss_loop(
    partial_ahss: &SyntheticSS,
    alg_ahss: &SyntheticSS,
    alg_data: &Vec<Vec<Vec<Vec<FromTo>>>>,
    mut log: Vec<Action>,
    stem: i32,
) -> (Result<bool, String>, Vec<Action>) {
    let ahss = revert_log_and_remake(0, &mut log, &partial_ahss, true);
    let log = Arc::new(Mutex::new(log));
    let res = ahss_iterate(
        ahss,
        alg_ahss,
        &alg_data,
        [const { None }; PARALLEL_DEPTH as usize],
        log.clone(),
        stem,
        2,
        1,
        0,
        false,
        false,
    );

    (res, Arc::try_unwrap(log).unwrap().into_inner().unwrap())
}

fn e1_loop(
    ahss: SyntheticSS,
    partial_ahss: &SyntheticSS,
    alg_ahss: &SyntheticSS,
    alg_data: &Vec<Vec<Vec<Vec<FromTo>>>>,
    mut log: Vec<Action>,
    stem: i32,
) -> (Result<bool, String>, Vec<Action>) {
    println!("\nStarted stem {stem}\n");

    let mut proper_issues = vec![];
    match ahss_synthetic_e1_issue(&ahss, stem) {
        Ok(_) => {}
        Err(issues) => {
            for i in issues {
                // First we solve all the e1 issues we can resolve
                match auto_deduce(&ahss, &i) {
                    Ok(mut a) => log.append(&mut a),
                    Err(_) => proper_issues.push(i),
                }
            }
        }
    }

    if proper_issues.len() != 0 {
        println!("We need to try multiple E1 options");
        println!("{proper_issues:?}");

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

        let positives = res.iter().fold(0, |acc, r| {
            if matches!(r.0, Ok(true)) {
                acc + 1
            } else {
                acc
            }
        });
        if positives != 1 {
            let first = res[0].1.clone();
            return (
                Err(format!(
                    "We have {positives} positives. Which means we can't decide on E1 stuff sadge :("
                )),
                first,
            );
        } else {
            for r in res {
                if matches!(r.0, Ok(true)) {
                    return r;
                }
            }
            unreachable!("There was one positive result.")
        }
    } else {
        e1_to_ahss_loop(partial_ahss, alg_ahss, alg_data, log, stem)
    }
}

pub fn ahss_solver(log: Option<Vec<Action>>) -> Result<(Vec<Action>, SyntheticSS), ()> {
    let alg_ahss = STABLE_DATA.clone();
    let mut partial_ahss = SyntheticSS::empty(alg_ahss.model.clone());
    // We should add all d1's from the algebraic data

    let mut alg_data = vec![
        vec![vec![vec![]; (MAX_STEM + 2) as usize]; (MAX_STEM + 1) as usize];
        (MAX_STEM + 1) as usize
    ];

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

    let mut log = if let Some(log) = log {
        log
    } else {
        vec![
        Action::AddInt { 
            from: "6 5 3[2]".to_string(), 
            to: "6 2 3 3[2]".to_string(), 
            page: 2, 
            proof: "Unique solution to RP1_2".to_string(), 
            kind: Kind::Real 
        },
        Action::AddDiff { from: "2 3 5 7 3 3[2]".to_string(), to: "2 4 1 1 2 4 3 3 3[1]".to_string(), proof: Some("This differential cannot be deduced by just looking at the current stem. It must be deduced by convergence on RP1_4 stem 26.".to_string()), kind: Kind::Real },

        Action::SetE1 { tag: "17 7 7 7".to_string(), torsion: Torsion(Some(0)), proof: "This E1 generator cannot be deduced by looking at the stem in which it occurs. It can only be concluded by looking further + James periodicity.".to_string() },
        Action::AddDiff { from: "3 5 7 3 3[10]".to_string(), to: "1 2 3 4 4 1 1 2 4 1 1 1[5]".to_string(), proof: Some("Program takes a long time to deduce this differential + results in Unknown. We just say its fake.".to_string()), kind: Kind::Fake },
        Action::AddDiff { from: "3 5 7 3 3[10]".to_string(), to: "2 3 4 4 1 1 2 4 1 1 1[6]".to_string(), proof: Some("Program takes a long time to deduce this differential + results in Unknown. We just say its fake.".to_string()), kind: Kind::Fake },
        Action::AddDiff { from: "15 15[6]".to_string(), to: "9 3 5 7 7[4]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Fake },
        Action::AddDiff { from: "12 1 1 1[5]".to_string(), to: "2 3 4 4 1 1 1[3]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Real },
        Action::AddDiff { from: "14 4 5 7 7[8]".to_string(), to: "19 7 7 7[4]".to_string(), proof: Some("Program can't figure this out.".to_string()), kind: Kind::Real },
        Action::AddDiff {
            from: "14 5 7 7[8]".to_string(),
            to: "14 4 5 7 7[3]".to_string(),
            kind: Kind::Real,
            proof: Some("If we don't have this differential we have a problem at RP3_47. (This could not have been deduced by just looking at RP1_8 at most)".to_string()),
        },
        // "AddExt": {
        //     "from": "6 2 3 4 4 1 1 2 4 1 1 2 4 1 1 1[5]",
        //     "to": "2 2 2 2 2 2 2 2 2 3 5 7 3 3[4]",
        //     "af": 16,
        //     "kind": "Real",
        //     "proof": "Without this extension the problems in AF 5 at RP4_12 stem 43 cannot be resolved."
        // }
        //  "AddDiff": {
        //     "from": "11 15 7 7[5]",
        //     "to": "9 11 7 7 7[3]",d
        //     "kind": "Real",
        //     "proof": "The source of this must also be this one, not the other AF5 element in this grid point. For RP3_5 the F_2 vector space generators don't add up. [SyntheticConvergence { bot_trunc: 3, top_trunc: 5, stem: 44, af: 7, expected: [Torsion(None), Torsion(Some(1))], observed: [Torsion(None), Torsion(None)] }, SyntheticConvergence { bot_trunc: 3, top_trunc: 5, stem: 44, af: 11, expected: [Torsion(None), Torsion(Some(1))], observed: [Torsion(None), Torsion(None)] }]"
        // }
        //         "AddDiff": {
        //     "from": "27 3 3[13]",
        //     "to": "21 11 3 3[7]",
        //     "kind": "Unknown",
        //     "proof": "(Custom) This differential must be here to support resolve the Synthetic convergence for RP7_256, stem 45, af 7. "
        // }
    ]
    };

    for stem in 2..MAX_VERIFY_STEM {
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
            Ok(true) => {
                println!("\n\nNEXT\n\n");
            }
            Ok(false) => {
                println!("\n\nCancelled on stem {stem}\n\n");
                break;
            }
            Err(e) => {
                println!("\n\nError on stem {stem}: {e}\n\n");
                break;
            }
        }
    }

    let ahss = revert_log_and_remake(0, &mut log, &partial_ahss, true);
    Ok((log, ahss))
}
