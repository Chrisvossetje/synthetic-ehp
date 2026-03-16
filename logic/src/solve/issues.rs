use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{data::compare::{algebraic_rp, algebraic_rp_keys, algebraic_rp_truncations, rp_truncations, synthetic_rp, synthetic_rp_keys, synthetic_s0, synthetic_s0_keys}, domain::{model::{SSPages, SyntheticSS}, process::compute_pages}, types::Torsion};


#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Issue {
    SyntheticE1Page {
        stem: i32,
        af: i32,

        expected: Vec<Torsion>,
        observed: Vec<Torsion>,
    },
    InvalidTorsion {
        from: usize,

        // Only to can be the problem, as it must be lower by some page
        to: usize,
        to_needed: Torsion,
    },
    SyntheticConvergence {
        bot_trunc: i32,
        top_trunc: i32,
        stem: i32,
        af: i32,

        expected: Vec<Torsion>,
        observed: Vec<Torsion>,
    },
    AlgebraicConvergence {
        bot_trunc: i32,
        top_trunc: i32,
        stem: i32,
        af: i32,
    
        expected: usize,
        observed: usize,
    },
    InvalidAEHP {
        from: usize,
        to: usize,
    },
}


pub fn verify_convergence(data: &SyntheticSS, pages: &SSPages, bot_trunc: i32, top_trunc: i32, stem: i32) -> Result<(), Vec<Issue>> {
    let observed = pages.convergence_at_stem(data, stem);
    
    let mut issues = vec![];

    for (s, af) in synthetic_rp_keys(bot_trunc, top_trunc) {
        if  s == stem && !observed.contains_key(&(af)) {
            issues.push(Issue::SyntheticConvergence { 
                bot_trunc, top_trunc, stem, af,
                expected: synthetic_rp(bot_trunc, top_trunc, stem, af).clone(), 
                observed: vec![] });
            
        }
    }

    for &af in observed.keys().sorted() {
        let obs = &observed[&(af)];
        if synthetic_rp(bot_trunc, top_trunc, stem, af) != obs {
            issues.push(Issue::SyntheticConvergence { 
                bot_trunc, top_trunc, stem, af,
                expected: synthetic_rp(bot_trunc, top_trunc, stem, af).clone(), 
                observed:  obs.clone() 
            });
        }
    }
    if issues.len() == 0 {
        Ok(())
    } else {
        Err(issues)
    }
}

pub fn verify_algebraic_convergence(data: &SyntheticSS, pages: &SSPages, bot_trunc: i32, top_trunc: i32, stem: i32) -> Result<(), Vec<Issue>> {
    let observed_minus_one = pages.convergence_at_stem(data, stem - 1);
    let observed = pages.convergence_at_stem(data, stem);

    let mut observed: HashMap<_, _> = observed.iter().map(|(k,v)| (*k, v.len())).collect();
    for (j, l) in &observed_minus_one {
        for i in l {
            if let Some(torsion) = i.0 {
                let b = j - torsion - 1;
                *observed.entry(b).or_insert(0) += 1;
            }
        }
    }
    
    let mut issues = vec![];

    for (s, af) in algebraic_rp_keys(bot_trunc, top_trunc) {
        if  s == stem && !observed.contains_key(&(af)) {
            issues.push(Issue::AlgebraicConvergence { 
                bot_trunc, top_trunc, stem, af,
                expected: *algebraic_rp(bot_trunc, top_trunc, stem, af), 
                observed: 0 
            });
        }
    }

    for &af in observed.keys().sorted() {
        let obs = &observed[&(af)];
        if algebraic_rp(bot_trunc, top_trunc, stem, af) != obs {
            issues.push(Issue::AlgebraicConvergence { 
                bot_trunc, top_trunc, stem, af,
                expected: *algebraic_rp(bot_trunc, top_trunc, stem, af), 
                observed: *obs 
            });
        }
    }
    if issues.len() == 0 {
        Ok(())
    } else {
        Err(issues)
    }
}

pub fn find_ahss_issue(data: &SyntheticSS, stem: i32) -> Result<(), Vec<Issue>> {
    ahss_synthetic_e1_issue(data, stem)?;

    for &(bot_trunc, top_trunc) in algebraic_rp_truncations() {
        let (pages, issues) = compute_pages(data, bot_trunc, top_trunc, stem-1, stem);
        // First check convergence
        verify_algebraic_convergence(&data, &pages, bot_trunc, top_trunc, stem)?;

        // Then check for structural issues in the computed pages
        if issues.len() != 0 {
            return Err(issues);
        }
    }
    for &(bot_trunc, top_trunc) in rp_truncations() {
        let (pages, issues) = compute_pages(data, bot_trunc, top_trunc, stem, stem);
        // First check convergence
        verify_convergence(&data, &pages, bot_trunc, top_trunc, stem)?;

        // Then check for structural issues in the computed pages
        if issues.len() != 0 {
            return Err(issues);
        }
    }
    

    
    Ok(())
}

pub fn ahss_synthetic_e1_issue(data: &SyntheticSS, stem: i32) -> Result<(), Vec<Issue>> {
    let mut observed = HashMap::new();
    for id in data.model.gens_id_in_stem(stem) {
        let g = data.model.get(*id);
        if g.y == 1 && g.stem == stem {
            observed.entry((g.stem - 1, g.af - 1)).or_insert(vec![]).push(g.torsion);
        }
    }

    let mut issues = vec![];

    for j in &mut observed {
        j.1.sort();
    }

    for (s, af) in synthetic_s0_keys() {
        if s == stem - 1 && !observed.contains_key(&(s, af)) && s != 0 {
            panic!("1) Algebraic Data is strictly smaller then what i can compare it to! On E1 page, y=1, stem={}, af={}", stem, af+1);
        }
    }
    
    for &(s, af) in observed.keys().sorted_by_key(|x| x.1) {
        let obs = &observed[&(s, af)];
        if synthetic_s0(s, af) != obs {
            if obs.len() != synthetic_s0(s, af).len() {
                panic!("2) Algebraic Data is strictly smaller then what i can compare it to! On E1 page, y=1, stem={}, af={}", stem, af);
            }
            issues.push(Issue::SyntheticE1Page { 
                stem, 
                af: af + 1, 
                observed: obs.clone(), 
                expected: synthetic_s0(s, af).clone() 
            });
        }
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}