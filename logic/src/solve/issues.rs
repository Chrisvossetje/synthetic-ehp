use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    MAX_STEM,
    data::{
        compare::EMPTY_LIST_TORSION,
        curtis::{DATA_PAGES, STABLE_DATA_PAGES},
    },
    domain::{model::SyntheticSS, process::try_compute_pages},
    types::Torsion,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Issue {
    SyntheticE1Page {
        stem: i32,
        af: i32,

        expected: Vec<Torsion>,
        observed: Vec<Torsion>,
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

    InvalidTorsion {
        from: usize,
        to: usize,
        stem: i32,
        from_name: String,
        to_name: String,
        to_needed: Torsion,
    },

    InvalidCoeff {
        from: usize,
        to: usize,
        from_name: String,
        to_name: String,
        coeff: i32,
    },

    InvalidAFRecursion {
        from: usize,
        to: usize,
        from_name: String,
        to_name: String,
    },

    InvalidAEHP {
        from: usize,
        to: usize,
        stem: i32,
        from_name: String,
        to_name: String,
    },

    InvalidTauMult {
        from: usize,
        to: usize,
        from_name: String,
        to_name: String,
    },

    UselessDifferential {
        from: usize,
        to: usize,
        bot_trunc: i32,
        top_trunc: i32,
        from_name: String,
        to_name: String,
    },

    InvalidName {
        original_name: String,
        unexpected_name: String,
        sphere: i32,
        stem: i32,
        af: i32,
    },

    InvalidEHPAHSSGen {
        name: String,
        stem: i32,
    },

    InvalidEHPAHSSMap {
        from: String,
        to: String,
        stem: i32,
        sphere: i32,
    }
}

pub fn compare_synthetic(
    observed: &HashMap<i32, Vec<Torsion>>,
    expected: &HashMap<(i32, i32), Vec<Torsion>>,
    bot_trunc: i32,
    top_trunc: i32,
    stem: i32,
) -> Result<(), Vec<Issue>> {
    let mut issues = vec![];

    for (s, af) in expected.keys() {
        if *s == stem && !observed.contains_key(&(af)) {
            issues.push(Issue::SyntheticConvergence {
                bot_trunc,
                top_trunc,
                stem,
                af: *af,
                expected: expected.get(&(stem, *af)).unwrap().clone(),
                observed: vec![],
            });
        }
    }

    for &af in observed.keys() {
        let obs = &observed[&(af)];
        let exp = expected.get(&(stem, af)).unwrap_or(&EMPTY_LIST_TORSION);
        if exp != obs {
            issues.push(Issue::SyntheticConvergence {
                bot_trunc,
                top_trunc,
                stem,
                af,
                expected: exp.clone(),
                observed: obs.clone(),
            });
        }
    }
    if issues.len() == 0 {
        Ok(())
    } else {
        Err(issues)
    }
}

pub fn compare_algebraic(
    observed: &HashMap<i32, usize>,
    expected: &HashMap<(i32, i32), usize>,
    bot_trunc: i32,
    top_trunc: i32,
    stem: i32,
) -> Result<(), Vec<Issue>> {
    let mut issues = vec![];

    for &(s, af) in expected.keys() {
        if s == stem && !observed.contains_key(&(af)) {
            issues.push(Issue::AlgebraicConvergence {
                bot_trunc,
                top_trunc,
                stem,
                af,
                expected: expected.get(&(stem, af)).unwrap().clone(),
                observed: 0,
            });
        }
    }

    for &af in observed.keys() {
        let obs = &observed[&(af)];
        let exp = expected.get(&(stem, af)).unwrap_or(&0);
        if exp != obs {
            issues.push(Issue::AlgebraicConvergence {
                bot_trunc,
                top_trunc,
                stem,
                af,
                expected: *exp,
                observed: *obs,
            });
        }
    }
    if issues.len() == 0 {
        Ok(())
    } else {
        Err(issues)
    }
}

pub fn compare_algebraic_spectral_sequence(
    data: &SyntheticSS,
    stem: i32,
    bot_trunc: i32,
    top_trunc: i32,
    ahss: bool,
) -> Result<(), Vec<Issue>> {
    // We just check the algebraic diffs, and then if their sources are correct ?
    // Most notably differerntials OUT of a stem
    let pages = try_compute_pages(data, bot_trunc, top_trunc, stem, stem)?;

    let alg_pages = if ahss {
        &STABLE_DATA_PAGES
    } else {
        &DATA_PAGES
    };

    let mut issues = vec![];

    // TODO: This should be somewhat dependent on the top_truncation.
    // At least for EHP this is important

    for (&from, tos) in &data.out_diffs {
        for &to in tos {
            if data.model.stem(to) == stem
                && bot_trunc <= data.model.y(to)
                && data.model.y(from) <= top_trunc
            {
                let alg = data.proven_from_to.get(&(from, to)).unwrap().is_none();
                if alg {
                    if !data.model.original_torsion(to).alive()
                        && data.model.original_torsion(from).alive()
                    {
                        let page = data.model.y(from) - data.model.y(to);
                        // From should die before the corresponding page.
                        let from_g = pages.element_at_page(page, from);
                        if from_g.1.alive() && from_g.0 == data.model.original_af(from) {
                            let (from_name, to_name) = data.get_names(from, to);
                            issues.push(Issue::InvalidAEHP {
                                from,
                                to,
                                stem,
                                from_name,
                                to_name,
                            });
                        }
                    }
                } else {
                    if data.model.original_torsion(from).alive() {
                        let coeff = data.model.original_af(to) - data.model.original_af(from);
                        let page = data.model.y(from) - data.model.y(to);
                        let (af, tor) = pages.element_at_page(page, from);
                        let or_af = data.model.original_af(from);

                        // af == or_af means that from actually maps to something in Algebraic
                        if coeff == 1 && af == or_af && tor.alive() {
                            // Now we could have that the target was already dead in the Algebraic part
                            if alg_pages.element_at_page(page, to).1.alive() {
                                let (from_name, to_name) = data.get_names(from, to);
                                issues.push(Issue::InvalidAEHP {
                                    from,
                                    to,
                                    stem,
                                    from_name,
                                    to_name,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // We should also check that any coeff 0 diff (seen from the ORIGINAL AF) is an algebraic one

    if issues.len() == 0 {
        Ok(())
    } else {
        Err(issues)
    }
}

/// This function checks if the synthetic convergence can be fixed by tau extensions
/// Meaning do the amount of F2 generators (up until AF=0) coincide
/// If so, we only need to fix the tau module structure, and not add differentials
pub fn synthetic_issue_is_tau_structure_issue(issues: &Vec<Issue>) -> bool {
    let mut count: [i8; MAX_STEM as usize] = [0; MAX_STEM as usize];

    for i in issues {
        if let Issue::SyntheticConvergence {
            bot_trunc,
            top_trunc,
            stem,
            af,
            expected,
            observed,
        } = i
        {
            for i in expected {
                let c = if let Some(t) = i.0 { af - t + 1 } else { 0 };
                for j in c..=*af {
                    count[j as usize] += 1;
                }
            }
            for i in observed {
                let c = if let Some(t) = i.0 { af - t + 1 } else { 0 };
                for j in c..=*af {
                    count[j as usize] -= 1;
                }
            }
        } else {
            return false;
        }
    }
    count.iter().all(|x| *x == 0)
}
