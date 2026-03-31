use crate::domain::{model::{Diff, ExtTauMult, SyntheticSS}, process::compute_pages};


pub fn get_a_diff(data: &SyntheticSS, top_trunc: i32, target_y: i32, stem: i32) -> Option<Diff> {

    // We can look at targets in RP1_256 as we have not added the adams diffs yet! 
    let (elements, _) = compute_pages(data, 0, 256, stem, stem + 1, false);

    for &t_id in data.model.gens_id_in_stem_y(stem, target_y) {
        
        if let Some((af, torsion)) = elements.try_element_final(t_id) && torsion.alive() {
            for &s_id in data.model.gens_id_in_stem_y(stem + 1, top_trunc) {
                if let Some((page, (s_af, s_torsion))) = elements.try_element_final_with_page(s_id) {
                    let coeff = af - s_af - 1;
                    if top_trunc - target_y >= page && s_torsion.alive() && coeff >= 0 {
                        
                        // Would be useless diff
                        if let Some(t_torsion) = torsion.0 && t_torsion - coeff <= 0 {
                            continue;
                        }
                        
                        // Would have been seen algebraicly
                        if coeff == 0 && af == data.model.original_af(t_id) && s_af == data.model.original_af(s_id) {
                            continue;
                        }
                        if !data.disproven_from_to.contains_key(&(s_id,t_id)) {
                            if torsion.can_map_with_coeff(&s_torsion, coeff) {
                                return Some(Diff { from: s_id, to: t_id })
                            }
                        }
                    }
                }
            }            
        }
    }
    
    None
}

pub fn get_a_tau(data: &SyntheticSS, top_trunc: i32, target_y: i32, stem: i32) -> Option<ExtTauMult> {

    // We can look at targets in RP1_256 as we have not added the adams diffs yet! 
    let (elements, _) = compute_pages(data, target_y, top_trunc, stem, stem, false);

    for &s_id in data.model.gens_id_in_stem(stem) {
        if let Some((s_af, s_torsion)) = elements.try_element_final(s_id) && s_torsion.alive() {
            if let Some(s_torsion) = s_torsion.0 {
                for &t_id in data.model.gens_id_in_stem(stem) {
                    if let Some((t_af, t_torsion)) = elements.try_element_final(t_id) && t_torsion.alive() {
                        if !data.proven_from_to.contains_key(&(s_id,t_id)) && 
                            !data.disproven_from_to.contains_key(&(s_id,t_id)) {
                            let d_y = data.model.y(s_id) - data.model.y(t_id);
                            if d_y > 0 {
                                if s_af > t_af && s_af - s_torsion <= t_af {
                                    return Some(ExtTauMult { from: s_id, to: t_id, af: s_af })
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
