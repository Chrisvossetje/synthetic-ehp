//! Proposing candidate facts for the solver to try: [`get_a_diff`] finds the
//! next plausible differential in a stem, and the `get_a_tau*` family finds the
//! next plausible external tau-multiplication (optionally restricted to given
//! source/target generators). These only suggest; the solver decides.

use crate::{
    domain::{
        e1::E1,
        model::{Diff, ExtTauMult, SyntheticSS},
        process::compute_pages,
        ss::SSPages,
    },
    types::Torsion,
};

/// Build the external tau `s_id -> t_id` if it is dimensionally valid: `from`
/// must sit above `to`, reach far enough down in AF to land on it, and (when
/// `to` is itself torsion) not already be covered by `to`'s existing torsion.
/// `s_torsion` is `from`'s (finite) torsion; `t_torsion` is `to`'s full state.
fn make_ext_tau(
    model: &E1,
    s_id: usize,
    s_af: i32,
    s_torsion: i32,
    t_id: usize,
    t_af: i32,
    t_torsion: Torsion,
) -> Option<ExtTauMult> {
    let d_y = model.y(s_id) - model.y(t_id);
    if d_y > 0 && s_af > t_af && s_af - s_torsion <= t_af {
        let valid = match t_torsion.0 {
            Some(t_torsion) => t_af - t_torsion < s_af - s_torsion,
            None => true,
        };
        if valid {
            return Some(ExtTauMult {
                from: s_id,
                to: t_id,
                af: t_af + 1,
            });
        }
    }
    None
}

pub fn get_a_diff(data: &SyntheticSS, model: &E1, top_trunc: i32, bot_trunc: i32, stem: i32) -> Option<Diff> {
    // We can look at targets in RP1_256 as we have not added the adams diffs yet!
    // let (targets, _) = compute_pages(data, 0, top_trunc, stem, stem, false);
    let (sources, _) = compute_pages(data, model, 0, 256, stem, stem + 1, false);

    let d_y = top_trunc - bot_trunc;

    for &t_id in model.gens_id_in_stem_y(stem, bot_trunc) {
        if let Some((t_af, t_torsion)) = sources.try_element_final(t_id)
            && t_torsion.alive()
        {
            for &s_id in model.gens_id_in_stem_y(stem + 1, top_trunc) {
                if let Some((s_af_final, s_torsion)) = sources.try_element_final(s_id)
                && s_torsion.alive()
                {
                    
                    let (t_af_at_page, _) = sources.element_at_page(d_y, t_id);
                    let (s_af_at_page, _) = sources.element_at_page(d_y, s_id);
                    
                    let coeff = t_af - s_af_final - 1;


                    // Make sure the diff is valid at that page
                    // The restriction to only have 1 differential per generator per page is to strong.
                    // We could have a diff from free to torsion, which will need to also have a tau multiple diff to a higher free generator for example.
                    
                    if t_af_at_page - s_af_at_page < 1 {
                        continue;
                    }
                    

                    // TODO: top_trunc - target_y >= s_page
                    // This should only hold whenever the s_page came from an actual
                    if coeff >= 0 {
                        // Would be useless diff
                        if let Some(t_torsion) = t_torsion.0
                            && t_torsion - coeff <= 0
                        {
                            continue;
                        }

                        // Would have been seen algebraically
                        if coeff == 0 {
                            if t_af == model.af(t_id)
                                && s_af_final == model.af(s_id)
                            {
                                if let Some(died) = model.get(t_id).dies {
                                    if died > model.y(s_id) {
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                        }

                        if !data.from_to.contains_key(&(s_id, t_id)) {
                            if t_torsion.can_map_with_coeff(&s_torsion, coeff) {
                                return Some(Diff {
                                    from: s_id,
                                    to: t_id,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

#[allow(dead_code)]
pub fn get_a_tau(
    data: &SyntheticSS,
    model: &E1,
    top_trunc: i32,
    target_y: i32,
    stem: i32,
) -> Option<ExtTauMult> {
    let (elements, _) = compute_pages(data, model, target_y, top_trunc, stem, stem, false);

    for &s_id in model.gens_id_in_stem(stem) {
        if let Some((s_af, s_torsion)) = elements.try_element_final(s_id)
            && s_torsion.alive()
        {
            if let Some(s_torsion) = s_torsion.0 {
                for &t_id in model.gens_id_in_stem(stem) {
                    if let Some((t_af, t_torsion)) = elements.try_element_final(t_id)
                        && t_torsion.alive()
                        && !data.from_to.contains_key(&(s_id, t_id))
                    {
                        let y = model.y(t_id);
                        if !data.out_taus[s_id].iter().any(|to| model.y(*to) == y) {
                            if let Some(tau) =
                                make_ext_tau(model, s_id, s_af, s_torsion, t_id, t_af, t_torsion)
                            {
                                return Some(tau);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn get_a_tau_for_t_ids(
    data: &SyntheticSS,
    model: &E1,
    elements: &SSPages,
    target_ids: &Vec<usize>,
) -> Option<ExtTauMult> {
    for &t_id in target_ids {
        let stem = model.stem(t_id);
        if let Some((t_af, t_torsion)) = elements.try_element_final(t_id)
            && t_torsion.alive()
        {
            for &s_id in model.gens_id_in_stem(stem) {
                if let Some((s_af, s_torsion)) = elements.try_element_final(s_id)
                    && s_torsion.alive()
                {
                    if let Some(s_torsion) = s_torsion.0 {
                        if !data.from_to.contains_key(&(s_id, t_id)) {
                            let y = model.y(t_id);
                            if !data.out_taus[s_id].iter().any(|to| {
                                model.y(*to) == y && data.generators[*to].alive()
                            }) {
                                if let Some(tau) =
                                    make_ext_tau(model, s_id, s_af, s_torsion, t_id, t_af, t_torsion)
                                {
                                    return Some(tau);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn get_a_tau_for_t_ids_s_ids(
    data: &SyntheticSS,
    model: &E1,
    elements: &SSPages,
    target_ids: &Vec<usize>,
    source_ids: &Vec<usize>,
) -> Option<ExtTauMult> {
    for &t_id in target_ids {
        if let Some((t_af, t_torsion)) = elements.try_element_final(t_id)
            && t_torsion.alive()
        {
            for &s_id in source_ids {
                if let Some((s_af, s_torsion)) = elements.try_element_final(s_id)
                    && s_torsion.alive()
                {
                    if let Some(s_torsion) = s_torsion.0 {
                        if !data.from_to.contains_key(&(s_id, t_id)) {
                            let y = model.y(t_id);
                            if !data.out_taus[s_id].iter().any(|to| model.y(*to) == y) {
                                if let Some(tau) =
                                    make_ext_tau(model, s_id, s_af, s_torsion, t_id, t_af, t_torsion)
                                {
                                    return Some(tau);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
