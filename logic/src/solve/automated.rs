//! Logic shared by the two automated solvers, [`crate::solve::automated_ahss`]
//! and [`crate::solve::automated_ehp`].
//!
//! Both solvers perform the same speculative branch-and-bound: propose a
//! differential or tau-multiplication, explore the "it exists" and "it doesn't"
//! worlds, and commit once one side is contradicted. The pieces that don't
//! depend on which sequence we're solving live here:
//!
//! - the search-tuning constants and the [`TauIssue`]/[`Commitment`] enums,
//! - committing a resolved choice to the log + data ([`commit_diff_choice`],
//!   [`commit_tau_choice`]) — the `ahss` flag selects whether the fact is
//!   replayed through [`process_action`] (so AHSS gets James-periodicity
//!   expansion) or applied directly (EHP),
//! - the heuristic that shortcuts a differential whose status is forced by
//!   theory ([`filter_diff`]),
//! - classifying a batch of convergence issues into the kind of tau-problem
//!   they represent ([`classify_tau_issue`]).
//!
//! What stays per-solver is the truncation traversal (`*_iterate`), the
//! `check_issue` against each sequence's reference data, and EHP-only steps
//! like induced-name resolution.

use std::sync::{Arc, Mutex};

use crate::{
    data::r#static::RADON_HURWITZ_NUMBERS,
    domain::{
        e1::E1,
        model::{Diff, ExtTauMult, SyntheticSS},
    },
    solve::{
        action::{Action, process_action},
        issues::{
            Issue, algebraic_issue_is_fixable_by_tau_extensions,
            synthetic_issue_is_tau_structure_issue,
        },
    },
    types::Kind,
};

/// Depth up to which speculative branches are explored in parallel (via rayon)
/// rather than sequentially.
pub const PARALLEL_DEPTH: i32 = 6;
/// When true, every solver step prints, not just the top-level (depth 0) ones.
pub const ALWAYS_PRINT: bool = false;
/// Maximum recursion depth before a branch gives up and reports `Open`.
pub const MAX_DEPTH: i32 = 10;

/// Which flavour of convergence problem a batch of issues represents, and hence
/// which `suggest_tau_solution_*` routine should try to repair it.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TauIssue {
    AlgTauIssue,
    SynTauGeneratorIssue,
    SynTauModuleIssue,
}

/// The verdict the speculative search reached for a proposed fact: it is real
/// (with a proof), it is fake/disproven (with a refutation), or both worlds
/// stayed open so we record it as unknown.
pub enum Commitment {
    Real(String),
    Fake(String),
    Unknown,
}

/// Record a resolved differential in the log (at depth 0) and apply it to the
/// working data. `ahss` selects how a *real* differential is applied: the AHSS
/// solver replays it through [`process_action`] so James periodicity fans it
/// out across spheres, while the EHP solver adds the single differential.
pub fn commit_diff_choice(
    data: &mut SyntheticSS,
    model: &E1,
    log: &Arc<Mutex<Vec<Action>>>,
    depth: i32,
    d: Diff,
    commitment: Commitment,
    ahss: bool,
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
            if ahss {
                let action = Action::AddDiff {
                    from: model.name(d.from).to_string(),
                    to: model.name(d.to).to_string(),
                    kind: Kind::Real,
                    proof: Some(proof),
                };
                let _ = process_action(data, model, &action, true);
            } else {
                data.add_diff(model, d.from, d.to, Some(proof), Kind::Real);
            }
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

/// Record a resolved external tau-multiplication in the log (at depth 0) and
/// apply it to the working data. As with [`commit_diff_choice`], `ahss` decides
/// whether real/fake taus are replayed through [`process_action`] (AHSS, for
/// James periodicity) or added directly (EHP).
pub fn commit_tau_choice(
    data: &mut SyntheticSS,
    model: &E1,
    log: &Arc<Mutex<Vec<Action>>>,
    depth: i32,
    d: ExtTauMult,
    commitment: Commitment,
    ahss: bool,
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
                    proof: Some(proof.clone()),
                    kind: Kind::Fake,
                });
            }
            apply_tau_fact(data, model, d, Kind::Fake, ahss);
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
                    proof: Some(proof.clone()),
                    kind: Kind::Real,
                });
            }
            apply_tau_fact(data, model, d, Kind::Real, ahss);
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
            data.add_ext_tau(model, d.from, d.to, d.af, None, Kind::Unknown);
        }
    }
}

/// Apply a real/fake tau to the data: through [`process_action`] in AHSS mode
/// (so James periodicity fans it out) or directly otherwise.
fn apply_tau_fact(data: &mut SyntheticSS, model: &E1, d: ExtTauMult, kind: Kind, ahss: bool) {
    if ahss {
        let action = Action::AddExt {
            from: model.name(d.from).to_string(),
            to: model.name(d.to).to_string(),
            af: d.af,
            kind,
            proof: None,
        };
        let _ = process_action(data, model, &action, true);
    } else {
        data.add_ext_tau(model, d.from, d.to, d.af, None, kind);
    }
}

/// Decide whether a proposed differential's status is already forced by theory,
/// short-circuiting the speculative search. Returns the [`Kind`] to record:
/// - `MinimalLength`: shorter than the Radon–Hurwitz bound, so it cannot exist;
/// - `AdditiveStructure`: another live source already maps in at the top filtration;
/// - `Invisible` / `Unnecessary`: forced by the algebraic data at this truncation.
///
/// The `ahss` flag enables one AHSS-only exception in the `Unnecessary` case.
pub fn filter_diff(
    data: &SyntheticSS,
    model: &E1,
    alg: &SyntheticSS,
    bot_trunc: i32,
    top_trunc: i32,
    d: Diff,
    ahss: bool,
) -> Option<Kind> {
    let y = model.y(d.from);

    if y - model.y(d.to) < RADON_HURWITZ_NUMBERS[y as usize] {
        Some(Kind::MinimalLength)
    } else if data.in_diffs[d.to]
        .iter()
        .any(|from| model.y(*from) == top_trunc && data.generators[*from].alive())
    {
        Some(Kind::AdditiveStructure)
    } else if bot_trunc & 1 == 0
        && let Some(alg_to) = alg.out_diffs[d.from].first()
        && data.generators[*alg_to].alive()
        && model.y(*alg_to) + 1 == bot_trunc
    {
        Some(Kind::Invisible)
    } else if top_trunc & 1 == 1
        && !(ahss && top_trunc == 5 && bot_trunc == 3)
        && let Some(dies) = model.get(d.to).dies
        && let Some(source) = alg.in_diffs[d.to].first()
        && data.generators[*source].free()
        && top_trunc + 2 == dies
    {
        Some(Kind::Unnecessary)
    } else {
        None
    }
}

/// Classify the convergence issues a `check_issue` produced into the kind of
/// tau-problem they are, so the caller knows which tau-suggestion routine to run.
///
/// If the issues are all synthetic-convergence ones, we ask whether they can be
/// repaired purely by tau-extensions (rather than new differentials) and, if so,
/// whether the fix is at the generator or module level. Otherwise we check the
/// algebraic convergence the same way. When no tau-extension can fix them the
/// issues are a genuine contradiction, reported as `Err` using the caller's
/// `f2_context`/`alg_context` labels (which name the truncation or sphere).
pub fn classify_tau_issue(
    issues: Vec<Issue>,
    f2_context: &str,
    alg_context: &str,
) -> Result<Option<(TauIssue, Vec<Issue>)>, String> {
    let all_synth_conv = matches!(&issues[0], Issue::SyntheticConvergence { .. });

    if all_synth_conv {
        let (solvable, generator) = synthetic_issue_is_tau_structure_issue(&issues);
        if solvable {
            if generator {
                Ok(Some((TauIssue::SynTauGeneratorIssue, issues)))
            } else {
                Ok(Some((TauIssue::SynTauModuleIssue, issues)))
            }
        } else {
            Err(format!(
                "{f2_context} the F_2 vector space generators don't add up. {issues:?}"
            ))
        }
    } else if algebraic_issue_is_fixable_by_tau_extensions(&issues) {
        Ok(Some((TauIssue::AlgTauIssue, issues)))
    } else {
        Err(format!(
            "{alg_context} there is no way to fix the algebraic convergence issues with tau extensions. {issues:?}"
        ))
    }
}
