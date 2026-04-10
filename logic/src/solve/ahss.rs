use std::collections::HashMap;

use crate::{
    data::compare::{S0_ZEROES, algebraic_rp, rp_truncations, synthetic_rp},
    domain::{model::SyntheticSS, process::try_compute_pages, ss::SSPages},
    solve::issues::{
        Issue, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic,
        synthetic_issue_is_tau_structure_issue,
    },
};

fn verify_convergence(
    data: &SyntheticSS,
    pages: &SSPages,
    bot_trunc: i32,
    top_trunc: i32,
    stem: i32,
) -> Result<(), Vec<Issue>> {
    let observed = pages.convergence_at_stem(data, stem);

    compare_synthetic(
        &observed,
        synthetic_rp(bot_trunc, top_trunc),
        bot_trunc,
        top_trunc,
        stem,
    )
}

fn verify_algebraic_convergence(
    data: &SyntheticSS,
    pages: &SSPages,
    bot_trunc: i32,
    top_trunc: i32,
    stem: i32,
) -> Result<(), Vec<Issue>> {
    let observed = pages.algebraic_convergence_at_stem(data, stem);

    compare_algebraic(
        &observed,
        algebraic_rp(bot_trunc, top_trunc),
        bot_trunc,
        top_trunc,
        stem,
    )
}

pub fn find_ahss_issues(data: &SyntheticSS, stem: i32) -> Result<(), Vec<Issue>> {
    ahss_synthetic_e1_issue(data, stem)?;

    for &(synthetic, bot_trunc, top_trunc) in rp_truncations() {
        if synthetic {
            let pages = try_compute_pages(data, bot_trunc, top_trunc, stem, stem)?;

            verify_convergence(&data, &pages, bot_trunc, top_trunc, stem).map_err(|x| {
                println!("Tau issues: {}", synthetic_issue_is_tau_structure_issue(&x).0);
                x
            })?;
        } else {
            let pages = try_compute_pages(data, bot_trunc, top_trunc, stem - 1, stem)?;

            verify_algebraic_convergence(&data, &pages, bot_trunc, top_trunc, stem)?;
        }
        compare_algebraic_spectral_sequence(data, stem, bot_trunc, top_trunc, true)?;
    }

    Ok(())
}

pub fn ahss_synthetic_e1_issue(data: &SyntheticSS, stem: i32) -> Result<(), Vec<Issue>> {
    let mut observed = HashMap::new();
    for id in data.model.gens_id_in_stem(stem) {
        let g = data.model.get(*id);
        if g.y == 1 && g.stem == stem {
            observed.entry(g.af - 1).or_insert(vec![]).push(g.torsion);
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
