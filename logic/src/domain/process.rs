//! Computing the pages of the spectral sequence from a [`SyntheticSS`].
//!
//! The flow is: [`instantiate_pages`] seeds every generator with its E1 state,
//! then [`apply_diff`] and [`apply_tau`] mutate that state page by page as the
//! asserted differentials and tau-multiplications are applied. [`compute_pages`]
//! drives the whole thing; anything inconsistent is reported as an [`Issue`]
//! rather than panicking. [`ehp_recursion`] is the EHP-specific step that lifts
//! one sphere's computed values up to the next.

use crate::{
    MAX_STEM,
    data::naming::{add_sphere_to_tag, generating_tag},
    domain::{e1::E1, model::SyntheticSS, ss::SSPages},
    solve::issues::Issue,
    types::{Kind, Torsion},
};

/// Seed every generator with its E1 state, keeping only those that fall within
/// the requested truncation window (`bot_trunc..=top_trunc` in y, plus a
/// one-stem margin around `from_stem..=to_stem`) and are still alive.
fn instantiate_pages(
    data: &SyntheticSS,
    model: &E1,
    bot_trunc: i32,
    top_trunc: i32,
    from_stem: i32,
    to_stem: i32,
) -> SSPages {
    let mut gens = vec![];

    let mut max_stem = 0;

    for (index, torsion) in data.generators.iter().enumerate() {
        let g = model.get(index);
        max_stem = max_stem.max(g.stem);

        if from_stem - 1 <= g.stem && g.stem <= to_stem + 1 {
            if bot_trunc <= g.y && g.y <= top_trunc && torsion.alive() {
                gens.push(Some(vec![(1, (g.af, *torsion))]));
                continue;
            }
        }

        gens.push(None);
    }

    SSPages {
        bot_trunc,
        top_trunc,
        generators: gens,
    }
}

/// Apply one differential `from -> to` on the given page, updating both
/// generators' (AF, torsion) state. Assumes differentials are applied in page
/// order. Inconsistencies (useless, negative coefficient, insufficient torsion)
/// are returned as an [`Issue`]; algebraic differentials silently no-op instead.
fn apply_diff(
    data: &SyntheticSS,
    model: &E1,
    pages: &mut SSPages,
    page: i32,
    from: usize,
    to: usize,
) -> Result<(), Issue> {
    // Current state of both endpoints (their (AF, torsion) on the latest page).
    let from_g = pages.element_final(from);
    let to_g = pages.element_final(to);

    let stem = model.stem(from);

    // We only do work when the source is still alive. The new states for the two
    // endpoints are computed in the branches below, then pushed onto page+1.
    let (new_from_g, new_to_g) = if from_g.1.alive() {
        // Source alive but target already dead: a real differential here has
        // nothing to hit (useless); an algebraic one is just expected, so no-op.
        if !to_g.1.alive() {
            if data.from_to[&(from, to)].0 != Kind::Algebraic {
                let (from_name, to_name) = model.get_names(from, to);
                return Err(Issue::UselessDifferential {
                    from,
                    to,
                    bot_trunc: pages.bot_trunc,
                    top_trunc: pages.top_trunc,
                    from_name,
                    to_name,
                    stem,
                });
            } else {
                return Ok(());
            }
        }

        // `coeff` = the tau-power this differential carries (the AF gap minus 1).
        // Negative means the geometry is impossible.
        let coeff = to_g.0 - from_g.0 - 1;
        if coeff < 0 {
            // TODO: Figure out if this should be branching or not ?
            // panic!("We encountered a negative coefficient, such a differential can not exist? And is unfixable ?");
            let (from_name, to_name) = model.get_names(from, to);
            return Err(Issue::InvalidCoeff {
                from,
                to,
                coeff,
                from_name,
                to_name,
            });
        }

        match to_g.1.0 {
            // Target is tau^to_t torsion.
            Some(to_t) => {
                // Delta represents how much F2 generators are actually hit
                let delta = to_t - coeff;
                // If this is non-positive then the differential is useless
                if delta > 0 {
                    match from_g.1.0 {
                        // Source torsion must be able to cover `delta`; otherwise
                        // the source isn't large enough to support this differential.
                        Some(from_t) => {
                            if delta > from_t {
                                let (from_name, to_name) = model.get_names(from, to);
                                return Err(Issue::InvalidTorsion {
                                    from,
                                    to,
                                    stem,
                                    to_needed: Torsion::new(from_t + coeff),
                                    from_name,
                                    to_name,
                                });
                            } else {
                                let new_from_af = from_g.0 - delta;
                                let new_from_t = from_t - delta;

                                let from = (new_from_af, Torsion::new(new_from_t));
                                let to = (to_g.0, Torsion::new(coeff));
                                (from, to)
                            }
                        }
                        None => {
                            let from = (from_g.0 - delta, Torsion::default());
                            let to = (to_g.0, Torsion::new(coeff));
                            (from, to)
                        }
                    }
                } else {
                    if data.from_to[&(from, to)].0 != Kind::Algebraic {
                        // Useless
                        let (from_name, to_name) = model.get_names(from, to);
                        return Err(Issue::UselessDifferential {
                            from,
                            to,
                            bot_trunc: pages.bot_trunc,
                            top_trunc: pages.top_trunc,
                            from_name,
                            to_name,
                            stem,
                        });
                    } else {
                        return Ok(());
                    }
                }
            }
            // Target is tau-free (infinite order). A torsion source can't kill a
            // free target — that would need impossibly large torsion — so error;
            // a free source kills it cleanly, leaving the source as tau^coeff.
            None => match from_g.1.0 {
                Some(t) => {
                    let (from_name, to_name) = model.get_names(from, to);
                    return Err(Issue::InvalidTorsion {
                        from,
                        to,
                        stem,
                        to_needed: Torsion::new(t + coeff),
                        from_name,
                        to_name,
                    });
                }
                None => {
                    let from = (from_g.0, Torsion::zero());
                    let to = (to_g.0, Torsion::new(coeff));
                    (from, to)
                }
            },
        }
    } else {
        // Source already dead: a real differential out of it is useless; if it
        // was algebraic (or the target is also dead) just keep the states as-is.
        if to_g.1.alive() && data.from_to[&(from, to)].0 != Kind::Algebraic {
            let (from_name, to_name) = model.get_names(from, to);
            return Err(Issue::UselessDifferential {
                from,
                to,
                bot_trunc: pages.bot_trunc,
                top_trunc: pages.top_trunc,
                from_name,
                to_name,
                stem,
            });
        } else {
            (from_g, to_g)
        }
    };

    // Record the post-differential states on the next page.
    pages.push(from, page + 1, new_from_g);
    pages.push(to, page + 1, new_to_g);

    Ok(())
}

/// Apply one tau-multiplication `from -> to` (internal or external), extending
/// `to`'s torsion from `from` where the bidegree allows it. `af` gates external
/// taus: the multiplication only fires once `from` has reached that AF.
fn apply_tau(
    model: &E1,
    pages: &mut SSPages,
    page: i32,
    af: i32,
    from: usize,
    to: usize,
) -> Result<(), Issue> {
    let from_g = pages.element_final(from);
    let to_g = pages.element_final(to);

    if from_g.1.alive() && to_g.1.alive() && from_g.0 >= af {
        if let Some(from_torsion) = from_g.1.0 {
            let af_diff = from_g.0 - to_g.0;

            let (new_from_torsion, new_to_torsion) = match to_g.1.0 {
                Some(to_torsion) => {
                    if from_torsion >= to_torsion + af_diff {
                        // This means the targeted generator is completely "inside" the torsion of from
                        // And thus cannot extend anything
                        return Ok(());
                    } else {
                        (
                            Torsion::new(to_torsion + af_diff),
                            Torsion::new(from_torsion - af_diff),
                        )
                    }
                }
                None => (Torsion::default(), Torsion::new(from_torsion - af_diff)),
            };


            // TODO : EHP requires >= and AHSS requires > ......
            if from_g.0 - from_torsion <= to_g.0 && af_diff >= 0 {
                pages.push(from, page, (from_g.0, new_from_torsion));
                pages.push(to, page, (to_g.0, new_to_torsion));
            } else if from_g.0 - from_torsion > to_g.0 {
                let (from_name, to_name) = model.get_names(from, to);
                return Err(Issue::InvalidTauMult {
                    from,
                    to,
                    from_name,
                    to_name,
                });
            }
        } else {
            // This means we have a "free" generator as source
            // And the tau multiplication should not happen at this "truncation"
        }
    }
    Ok(())
}

pub fn try_compute_pages(
    data: &SyntheticSS,
    model: &E1,
    bot_trunc: i32,
    top_trunc: i32,
    from_stem: i32,
    to_stem: i32,
    include_tau: bool,
) -> Result<SSPages, Vec<Issue>> {
    let (pages, issues) =
        compute_pages(data, model, bot_trunc, top_trunc, from_stem, to_stem, include_tau);

    if issues.len() != 0 {
        return Err(issues);
    } else {
        Ok(pages)
    }
}


pub fn compute_diffs_int_taus(
    data: &SyntheticSS,
    model: &E1,
    bot_trunc: i32,
    top_trunc: i32,
    from_stem: i32,
    to_stem: i32,
) -> (SSPages, Vec<Issue>) {
    let mut pages = instantiate_pages(data, model, bot_trunc, top_trunc, from_stem, to_stem);

    let mut issues = vec![];

    for page in 0..=MAX_STEM as usize {
        for t in &data.internal_tau_page[page] {
            if pages.element_in_pages(t.from) && pages.element_in_pages(t.to) {
                if let Err(i) = apply_tau(model, &mut pages, page as i32, 0, t.from, t.to) {
                    issues.push(i);
                }
            }
        }

        for d in &data.diffs_page[page] {
            if pages.element_in_pages(d.from) && pages.element_in_pages(d.to) {
                if let Err(i) = apply_diff(data, model, &mut pages, page as i32, d.from, d.to) {
                    issues.push(i);
                }
            }
        }
    }

    (pages, issues)
}

pub fn compute_ext_taus(
    pages: &mut SSPages,
    model: &E1,
    data: &SyntheticSS,
    _bot_trunc: i32,
    _top_trunc: i32,
    from_stem: i32,
    to_stem: i32,
) -> Vec<Issue> {
    let mut issues = vec![];
    for esss in &data.external_tau_page {
        for ess in esss {
            for es in ess {
                for e in es {
                    if pages.element_in_pages(e.from)
                        && pages.element_in_pages(e.to)
                        && from_stem <= model.stem(e.from)
                        && model.stem(e.from) <= to_stem
                    {
                        if let Err(i) = apply_tau(model, pages, 500, e.af, e.from, e.to) {
                            issues.push(i);
                        }
                    }
                }
            }
        }
    }

    issues
}

pub fn compute_pages(
    data: &SyntheticSS,
    model: &E1,
    bot_trunc: i32,
    top_trunc: i32,
    from_stem: i32,
    to_stem: i32,
    include_tau: bool,
) -> (SSPages, Vec<Issue>) {


    let (mut pages, mut issues) = compute_diffs_int_taus(data, model, bot_trunc, top_trunc, from_stem, to_stem);

    if include_tau {
        let mut issues2 = compute_ext_taus(&mut pages, model, data, bot_trunc, top_trunc, from_stem, to_stem);
        issues.append(&mut issues2);
    }

    (pages, issues)
}

/// One step of the EHP recursion: using the sequence computed up through
/// `sphere - 1`, write the surviving values in `stem` back onto the generators
/// they induce one sphere up. Only defined for odd spheres.
pub fn ehp_recursion(ehp: &mut SyntheticSS, model: &E1, sphere: i32, stem: i32) -> Result<(), Vec<Issue>> {
    if sphere % 2 == 0 {
        panic!("Can't call this on even spheres");
    }

    // Compute the sequence through the previous sphere, so we know what survives.
    let pages = try_compute_pages(ehp, model, 0, sphere - 1, stem, stem, true)?;

    // Clear the row this recursion is about to (re)populate.
    for id in model
    .gens_id_in_stem_y(stem + sphere / 2, sphere / 2)
    .clone()
    {
        ehp.generators[id] = Torsion::zero();
    }

    let mut issues = vec![];

    // For each generator surviving in this stem, find the generator it induces one
    // sphere up (by rewriting its name's tag) and copy its torsion onto it. The AF
    // must line up (off by exactly one) and the induced name must actually exist —
    // otherwise the recursion is inconsistent and we record an issue.
    for id in model.gens_id_in_stem(stem).clone() {
        if let Some(g) = pages.try_element_final(id)
            && g.1.alive()
        {
            let name = ehp.get_name_at_sphere(model, id, sphere).to_string();
            let gen_tag = generating_tag(&name);

            let target_name = add_sphere_to_tag(gen_tag, sphere);

            match model.try_index(&target_name) {
                Some(target_id) => {
                    let t_af = model.get(target_id).af;
                    if g.0 + 1 != t_af {
                        issues.push(Issue::InvalidAFRecursion {
                            from: id,
                            to: target_id,
                            from_name: name,
                            to_name: target_name,
                        });
                    } else {
                        ehp.generators[target_id] = g.1;
                    }
                }
                None => {
                    issues.push(Issue::InvalidName {
                        original_name: name.to_string(),
                        unexpected_name: target_name,
                        stem,
                        sphere,
                        af: g.0,
                    });
                }
            }
        }
    }

    if issues.len() != 0 {
        return Err(issues);
    } else {
        Ok(())
    }
}
