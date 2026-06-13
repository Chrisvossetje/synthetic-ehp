//! The [`Issue`] enum — every way a computed spectral sequence can disagree
//! with what we expect — and the comparison routines that produce them:
//! `compare_synthetic`/`compare_algebraic` check convergence against the loaded
//! reference data, and `compare_algebraic_spectral_sequence` checks the
//! algebraic differentials. Also includes the heuristics that decide whether a
//! convergence mismatch can be repaired purely by tau-multiplications.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    MAX_STEM,
    data::{
        r#static::EMPTY_LIST_TORSION,
        curtis::{DATA_PAGES, STABLE_DATA_PAGES},
    },
    domain::{e1::E1, model::SyntheticSS, ss::SSPages},
    types::{Kind, Torsion},
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
        stem: i32,
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
        name: String,
        from_torsion: Torsion,
        to_torsion: Torsion,
        stem: i32,
        sphere: i32,
    },
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
    model: &E1,
    pages: &SSPages,
    stem: i32,
    bot_trunc: i32,
    top_trunc: i32,
    ahss: bool,
) -> Result<(), Vec<Issue>> {
    // Cross-check the synthetic data against the purely-algebraic spectral
    // sequence by looking at the differentials leaving the stem above this one:
    // each algebraic differential should still behave algebraically here, and
    // each non-algebraic one shouldn't be silently overriding an algebraic fact.
    let alg_pages = if ahss {
        &STABLE_DATA_PAGES
    } else {
        &DATA_PAGES
    };

    let mut issues = vec![];

    // TODO: This should be somewhat dependent on the top_truncation.
    // At least for EHP this is important

    for &from in model.gens_id_in_stem(stem + 1) {
        let tos = &data.out_diffs[from];
        for &to in tos {
            // Only differentials landing in this stem, within the truncation window.
            if model.stem(to) == stem
                && bot_trunc <= model.y(to)
                && model.y(from) <= top_trunc
            {
                let alg = data.from_to.get(&(from, to)).unwrap().0 == Kind::Algebraic;
                if alg {
                    // Algebraic differential whose target is dead but source alive:
                    // the source must already be dead by the differential's page,
                    // otherwise the algebraic structure has been violated.
                    if !data.generators[to].alive()
                    && data.generators[from].alive()
                    {
                        let page = model.y(from) - model.y(to);
                        // From should die before the corresponding page.
                        let from_g = pages.element_at_page(page, from);
                        if from_g.1.alive() && from_g.0 == model.af(from) {
                            let (from_name, to_name) = model.get_names(from, to);
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
                    // Non-algebraic differential that, at its page, still looks like
                    // a length-1 map out of the original AF onto a target that is
                    // alive algebraically — i.e. it should have been algebraic.
                    if data.generators[from].alive() {
                        let coeff = model.af(to) - model.af(from);
                        let page = model.y(from) - model.y(to);
                        let (af, tor) = pages.element_at_page(page, from);
                        let or_af = model.af(from);

                        // af == or_af means that from actually maps to something in Algebraic
                        if coeff == 1 && af == or_af && tor.alive() {
                            // Now we could have that the target was already dead in the Algebraic part
                            if alg_pages.element_at_page(page, to).1.alive() {
                                let (from_name, to_name) = model.get_names(from, to);
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
///
/// Returns `(solvable, generator)`: `solvable` is true when expected and observed
/// agree as F2-vector-spaces filtered by AF (so only the tau structure, not the
/// generator count, is wrong); `generator` further distinguishes the sub-case.
pub fn synthetic_issue_is_tau_structure_issue(issues: &Vec<Issue>) -> (bool, bool) {
    // `count[af]` tallies expected-minus-observed F2 generators living at or below
    // each AF; `total_gens` tallies the raw expected-minus-observed generator count.
    let mut count: [i8; MAX_STEM as usize] = [0; MAX_STEM as usize];
    let mut total_gens: i32 = 0;

    for i in issues {
        if let Issue::SyntheticConvergence {
            af,
            expected,
            observed,
            ..
        } = i
        {
            total_gens += expected.len() as i32;
            total_gens -= observed.len() as i32;
            // Each class contributes to every AF level from where its tau-tower
            // starts (af - torsion + 1, or 0 if free) up to its own AF.
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
            // Any non-convergence issue means this isn't a pure tau-structure fix.
            return (false, false);
        }
    }

    // If the totals don't cancel at every AF, the generator counts genuinely
    // differ and tau extensions alone can't reconcile them.
    if total_gens > 0 || count.iter().any(|x| *x != 0) {
        (false, false)
    } else {
        if issues.len() == 1 {
            panic!();
        }
        (true, total_gens < 0)
    }
}

pub fn algebraic_issue_is_fixable_by_tau_extensions(issues: &Vec<Issue>) -> bool {
    let mut count: [i8; MAX_STEM as usize] = [0; MAX_STEM as usize];

    for i in issues {
        if let Issue::AlgebraicConvergence {
            af,
            expected,
            observed,
            ..
        } = i
        {
            count[*af as usize] += *expected as i8;
            count[*af as usize] -= *observed as i8;
        } else {
            return false;
        }
    }

    count.iter().all(|x| *x <= 0)
}
