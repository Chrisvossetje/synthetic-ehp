use crate::domain::{
    model::{Diff, ExtTauMult, SyntheticSS},
    process::compute_pages,
    ss::SSPages,
};

pub fn get_a_diff(data: &SyntheticSS, top_trunc: i32, bot_trunc: i32, stem: i32) -> Option<Diff> {
    // We can look at targets in RP1_256 as we have not added the adams diffs yet!
    // let (targets, _) = compute_pages(data, 0, top_trunc, stem, stem, false);
    let (sources, _) = compute_pages(data, 0, 256, stem, stem + 1, false);

    let d_y = top_trunc - bot_trunc;

    for &t_id in data.model.gens_id_in_stem_y(stem, bot_trunc) {
        if let Some((t_af, t_torsion)) = sources.try_element_final(t_id)
            && t_torsion.alive()
        {
            for &s_id in data.model.gens_id_in_stem_y(stem + 1, top_trunc) {
                let (from_name, to_name) = data.get_names(s_id, t_id);
                if let Some((_, s_torsion)) = sources.try_element_final(s_id)
                    && s_torsion.alive()
                {
                    let (s_af, _) = sources.element_at_page(d_y + 1, s_id);

                    let coeff = t_af - s_af - 1;

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
                            if t_af == data.model.original_af(t_id)
                                && s_af == data.model.original_af(s_id)
                            {
                                if let Some(died) = data.model.get(t_id).dies {
                                    if died > data.model.y(s_id) {
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                        }

                        if !data.disproven_from_to.contains_key(&(s_id, t_id))
                            && !data.proven_from_to.contains_key(&(s_id, t_id))
                        {
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

pub fn get_a_tau(
    data: &SyntheticSS,
    top_trunc: i32,
    target_y: i32,
    stem: i32,
) -> Option<ExtTauMult> {
    let (elements, _) = compute_pages(data, target_y, top_trunc, stem, stem, false);

    for &s_id in data.model.gens_id_in_stem(stem) {
        if let Some((s_af, s_torsion)) = elements.try_element_final(s_id)
            && s_torsion.alive()
        {
            if let Some(s_torsion) = s_torsion.0 {
                for &t_id in data.model.gens_id_in_stem(stem) {
                    let (from_name, to_name) = data.get_names(s_id, t_id);

                    if let Some((t_af, t_torsion)) = elements.try_element_final(t_id)
                        && t_torsion.alive()
                    {
                        if !data.proven_from_to.contains_key(&(s_id, t_id))
                            && !data.disproven_from_to.contains_key(&(s_id, t_id))
                        {
                            let y = data.model.y(t_id);
                            if !data.out_taus[s_id].iter().any(|to| data.model.y(*to) == y) {
                                let d_y = data.model.y(s_id) - data.model.y(t_id);
                                if d_y > 0 {
                                    if s_af > t_af && s_af - s_torsion <= t_af {
                                        return Some(ExtTauMult {
                                            from: s_id,
                                            to: t_id,
                                            af: t_af + 1,
                                        });
                                    }
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

pub fn get_a_tau_for_t_ids(
    data: &SyntheticSS,
    elements: &SSPages,
    target_ids: &Vec<usize>,
    stem: i32,
) -> Option<ExtTauMult> {
    for &t_id in target_ids {
        if let Some((t_af, t_torsion)) = elements.try_element_final(t_id)
            && t_torsion.alive()
        {
            for &s_id in data.model.gens_id_in_stem(stem) {
                if let Some((s_af, s_torsion)) = elements.try_element_final(s_id)
                    && s_torsion.alive()
                {
                    if let Some(s_torsion) = s_torsion.0 {
                        let (from_name, to_name) = data.get_names(s_id, t_id);

                        if !data.proven_from_to.contains_key(&(s_id, t_id))
                            && !data.disproven_from_to.contains_key(&(s_id, t_id))
                        {
                            let y = data.model.y(t_id);
                            if !data.out_taus[s_id].iter().any(|to| data.model.y(*to) == y) {
                                let d_y = data.model.y(s_id) - data.model.y(t_id);
                                if d_y > 0 {
                                    if s_af > t_af && s_af - s_torsion <= t_af {
                                        return Some(ExtTauMult {
                                            from: s_id,
                                            to: t_id,
                                            af: t_af + 1,
                                        });
                                    }
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
    elements: &SSPages,
    target_ids: &Vec<usize>,
    source_ids: &Vec<usize>,
    stem: i32,
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
                        let (from_name, to_name) = data.get_names(s_id, t_id);
                        if !data.proven_from_to.contains_key(&(s_id, t_id))
                            && !data.disproven_from_to.contains_key(&(s_id, t_id))
                        {
                            let y = data.model.y(t_id);
                            if !data.out_taus[s_id].iter().any(|to| data.model.y(*to) == y) {
                                let d_y = data.model.y(s_id) - data.model.y(t_id);
                                if d_y > 0 {
                                    if s_af > t_af && s_af - s_torsion <= t_af {
                                        return Some(ExtTauMult {
                                            from: s_id,
                                            to: t_id,
                                            af: t_af + 1,
                                        });
                                    }
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
