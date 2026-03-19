use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{data::compare::{S0_ZEROES, algebraic_rp, algebraic_rp_truncations, rp_truncations, synthetic_rp, synthetic_rp_truncations}, domain::{model::SyntheticSS, process::{compute_pages, try_compute_pages}, ss::SSPages}, solve::issues::{Issue, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic}, types::Torsion};

fn verify_convergence(data: &SyntheticSS, pages: &SSPages, bot_trunc: i32, top_trunc: i32, stem: i32) -> Result<(), Vec<Issue>> {
    let observed = pages.convergence_at_stem(data, stem);
    
    compare_synthetic(&observed, synthetic_rp(bot_trunc, top_trunc), bot_trunc, top_trunc, stem)
}

fn verify_algebraic_convergence(data: &SyntheticSS, pages: &SSPages, bot_trunc: i32, top_trunc: i32, stem: i32) -> Result<(), Vec<Issue>> {
    let observed = pages.algebraic_convergence_at_stem(data, stem);
    
    compare_algebraic(&observed, algebraic_rp(bot_trunc, top_trunc), bot_trunc, top_trunc, stem)
}

pub fn find_ahss_issues(data: &SyntheticSS, stem: i32) -> Result<(), Vec<Issue>> {
    ahss_synthetic_e1_issue(data, stem)?;

    for &(synthetic, bot_trunc, top_trunc) in rp_truncations() {
        if synthetic {
            let pages = try_compute_pages(data, bot_trunc, top_trunc, stem, stem)?;
        
            verify_convergence(&data, &pages, bot_trunc, top_trunc, stem)?;
        } else {
            let pages = try_compute_pages(data, bot_trunc, top_trunc, stem-1, stem)?;
        
            verify_algebraic_convergence(&data, &pages, bot_trunc, top_trunc, stem)?;
        }
    }

    compare_algebraic_spectral_sequence(data, stem, true)?;
    
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

    compare_synthetic(&observed, &S0_ZEROES, 1, 1, stem - 1).map_err(|x|
    x.into_iter().map(|e| 
        if let Issue::SyntheticConvergence { bot_trunc: _, top_trunc: _, stem: _, af, expected, observed } = e {
            Issue::SyntheticE1Page { 
                stem: stem, 
                af: af + 1, 
                expected, 
                observed }
        } else {
            panic!("We should only expect Synthetic Convergence issues here")
        }
    ).collect())
    
    // let mut issues = vec![];


    // for (s, af) in synthetic_s0_keys() {
    //     if s == stem - 1 && !observed.contains_key(&(s, af)) && s != 0 {
    //         panic!("1) Algebraic Data is strictly smaller then what i can compare it to! On E1 page, y=1, stem={}, af={}", stem, af+1);
    //     }
    // }
    
    // for &(s, af) in observed.keys().sorted_by_key(|x| x.1) {
    //     let obs = &observed[&(s, af)];
    //     if synthetic_s0(s, af) != obs {
    //         if obs.len() != synthetic_s0(s, af).len() {
    //             panic!("2) Algebraic Data is strictly smaller then what i can compare it to! On E1 page, y=1, stem={}, af={}", stem, af);
    //         }
    //         issues.push(Issue::SyntheticE1Page { 
    //             stem, 
    //             af: af + 1, 
    //             observed: obs.clone(), 
    //             expected: synthetic_s0(s, af).clone() 
    //         });
    //     }
    // }
    // if issues.is_empty() {
    //     Ok(())
    // } else {
    //     Err(issues)
    // }
}