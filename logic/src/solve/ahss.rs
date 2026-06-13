//! Issue-finding for the AHSS (stable) sequence: [`find_ahss_issues`] computes
//! the relevant RP^n truncations for a stem and checks both synthetic and
//! algebraic convergence against the reference data, while
//! [`ahss_synthetic_e1_issue`] checks the E1 page itself against S0.

use std::collections::HashMap;

use itertools::Itertools;

use crate::{
    data::{naming::name_get_tag, r#static::{S0_ZEROES, algebraic_rp, rp_truncations, synthetic_rp}},
    domain::{e1::E1, model::SyntheticSS, process::try_compute_pages, ss::SSPages},
    solve::{action::Action, issues::{
        Issue, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic,
        synthetic_issue_is_tau_structure_issue,
    }}, types::Torsion,
};

fn verify_convergence(
    model: &E1,
    pages: &SSPages,
    bot_trunc: i32,
    top_trunc: i32,
    stem: i32,
) -> Result<(), Vec<Issue>> {
    let observed = pages.convergence_at_stem(model, stem);

    compare_synthetic(
        &observed,
        synthetic_rp(bot_trunc, top_trunc),
        bot_trunc,
        top_trunc,
        stem,
    )
}

fn verify_algebraic_convergence(
    model: &E1,
    pages: &SSPages,
    bot_trunc: i32,
    top_trunc: i32,
    stem: i32,
) -> Result<(), Vec<Issue>> {
    let observed = pages.algebraic_convergence_at_stem(model, stem);

    compare_algebraic(
        &observed,
        algebraic_rp(bot_trunc, top_trunc),
        bot_trunc,
        top_trunc,
        stem,
    )
}

pub fn find_ahss_issues(data: &SyntheticSS, model: &E1, stem: i32) -> Result<(), Vec<Issue>> {
    ahss_synthetic_e1_issue(data, model, stem)?;

    for &(synthetic, bot_trunc, top_trunc) in rp_truncations() {
        let pages = if synthetic {
            let pages = try_compute_pages(data, model, bot_trunc, top_trunc, stem, stem, true)?;

            verify_convergence(model, &pages, bot_trunc, top_trunc, stem).map_err(|x| {
                println!(
                    "Tau issues: {}",
                    synthetic_issue_is_tau_structure_issue(&x).0
                );
                x
            })?;
            pages
        } else {
            let pages = try_compute_pages(data, model, bot_trunc, top_trunc, stem - 1, stem, true)?;

            verify_algebraic_convergence(model, &pages, bot_trunc, top_trunc, stem)?;
            pages
        };
        compare_algebraic_spectral_sequence(data, model, &pages, stem, bot_trunc, top_trunc, true)?;
    }

    Ok(())
}

pub fn ahss_synthetic_e1_issue(data: &SyntheticSS, model: &E1, stem: i32) -> Result<(), Vec<Issue>> {
    let mut observed = HashMap::new();
    for id in model.gens_id_in_stem(stem) {
        let g = model.get(*id);
        if g.y == 1 && g.stem == stem {
            observed.entry(g.af - 1).or_insert(vec![]).push(data.generators[*id]);
        }
    }

    for j in &mut observed {
        j.1.sort();
    }

    compare_synthetic(&observed, &S0_ZEROES, 1, 1, stem - 1).map_err(|x| {
        x.into_iter()
            .map(|e| {
                if let Issue::SyntheticConvergence {
                    bot_trunc: _,
                    top_trunc: _,
                    stem: _,
                    af,
                    expected,
                    observed,
                } = e
                {
                    Issue::SyntheticE1Page {
                        stem: stem,
                        af: af + 1,
                        expected,
                        observed,
                    }
                } else {
                    panic!("We should only expect Synthetic Convergence issues here")
                }
            })
            .collect()
    })
}

pub fn get_all_e1_solutions(data: &SyntheticSS, model: &E1, issues: &Vec<Issue>) -> Vec<Vec<Action>> {
    let sols: Vec<_> = issues.iter().map(|i| get_e1_solutions(data, model, i)).collect();

    sols.iter()
        .map(|s| 0..s.len())
        .multi_cartesian_product()
        .map(|idxs| {
            sols.iter()
                .zip(idxs)
                .flat_map(|(sol, i)| sol[i].clone())
                .collect()
        })
        .collect()
}

pub fn get_e1_solutions(_data: &SyntheticSS, model: &E1, issue: &Issue) -> Vec<Vec<Action>> {
    // Give a list of options which one could / should change
    // This is the only time i (should) do this forward approach. Aka giving potential solutions.
    // In other cases i should just "go" and see if some option resolves some issue

    // The solutions should be some combinatorial thing
    // It should be "unique" permutations!

    if let Issue::SyntheticE1Page {
        stem,
        af,
        expected,
        ..
    } = issue
    {
        let stem_af_to_index = model.gens_id_in_stem_af(*stem, *af);

        // let stem = stem + 1;

        let mut changes = vec![];

        for p in expected.iter().permutations(expected.len()).unique() {
            let mut change = vec![];
            for (id, &torsion) in p.into_iter().enumerate() {
                if torsion != Torsion::default() {
                    let real_id = stem_af_to_index[id];
                    let tag = name_get_tag(model.name(real_id)).to_string();
                    change.push(Action::SetE1 {
                        tag,
                        torsion,
                        proof: "".to_string(),
                    });
                }
            }
            changes.push(change);
        }

        changes
    } else {
        panic!("Can only call this function on Synthetic E1 error")
    }
}
