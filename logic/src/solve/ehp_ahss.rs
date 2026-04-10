use crate::{domain::{model::SyntheticSS, process::compute_pages, ss::SSPages}, solve::issues::Issue, types::{Kind, Torsion}};


// (EHP -> AHSS, Lifts from AHSS -> EHP)
pub type SyntheticSSMap = (Vec<Option<usize>>, Vec<Option<usize>>);

pub fn in_metastable_range(y: i32, stem: i32) -> bool {
    stem < (y * 3)
}

pub fn set_metastable_range(ehp: &mut SyntheticSS, ahss: &SyntheticSS) -> Result<(), ()> {
    for g in ahss.model.gens() {
        if in_metastable_range(g.y, g.stem) {
            ehp.set_generator(&g.name, g.torsion)?;
        }
    }
    for ds in &ahss.diffs_page {
        for d in ds {
            let g_from = ahss.model.get(d.from);
            let g_to = ahss.model.get(d.to);
            if in_metastable_range(g_to.y, g_to.stem) {
                let proof = ahss.proven_from_to.get(&(d.from, d.to)).expect("If there is no reference to a proof here (note that the string can still be empty), then inserting differentials not done carefully enough.");
                ehp.add_diff_name(
                    g_from.name.clone(),
                    g_to.name.clone(),
                    proof.clone().map(|x| format!("{x} (Metastable)")),
                    Kind::Real
                )?;
            }
        }
    }

    for (&(from, to), proof) in &ahss.disproven_from_to {
        let g_from = ahss.model.get(from);
        let g_to = ahss.model.get(to);
        if in_metastable_range(g_to.y, g_to.stem) {
            let (from_name, to_name) = ahss.get_names(from, to);

            if ahss.model.stem(from) - ahss.model.stem(to) == 1 {
                let kind = if proof.is_some() { Kind::Fake } else { Kind::Unknown };
                ehp.add_diff_name(
                    g_from.name.clone(),
                    g_to.name.clone(),
                    proof.clone().map(|x| format!("{x} (Metastable)")),
                    kind
                )?;
            }
        }
    }

    for (page, ts) in ahss.internal_tau_page.iter().enumerate() {
        for t in ts {
            let g_from = ahss.model.get(t.from);
            let g_to = ahss.model.get(t.to);
            if in_metastable_range(g_to.y, g_to.stem) {
                let proof = ahss.proven_from_to.get(&(t.from, t.to)).expect("If there is no reference to a proof here (note that internally it can still have no proof), then inserting internal tau's not done carefully enough.");
                ehp.add_int_tau_name(
                    g_from.name.clone(),
                    g_to.name.clone(),
                    page as i32,
                    proof.clone().map(|x| format!("{x} (Metastable)")),
                    Kind::Real // TODO! <<
                )?;
            }
        }
    }

    for ess in &ahss.external_tau_page {
        for es in ess {
            for e in es {
                let g_from = ahss.model.get(e.from);
                let g_to = ahss.model.get(e.to);
                if in_metastable_range(g_to.y, g_to.stem) {
                    let proof = ahss.proven_from_to.get(&(e.from, e.to)).expect("If there is no reference to a proof here (note that internally it can still have no proof), then inserting external tau's not done carefully enough.");
                    ehp.add_ext_tau_name(
                        g_from.name.clone(),
                        g_to.name.clone(),
                        e.af,
                        proof.clone().map(|x| format!("{x} (Metastable)")),
                        Kind::Real // TODO! <<
                    )?;
                }
            }
        }
    }

    Ok(())
}

pub fn ehp_to_ahss_map(ehp: &SyntheticSS, ahss: &SyntheticSS) -> SyntheticSSMap {
    let ehp_ahss: Vec<_> = ehp.model.gens().iter().map(|g| ahss.model.try_index(&g.name)).collect();
    let ahss_ehp: Vec<_> = ahss.model.gens().iter().map(|g| ehp.model.try_index(&g.name)).collect();

    (ehp_ahss, ahss_ehp)
} 



fn check(a: &SyntheticSS, b: &SyntheticSS, a_p: &SSPages, b_p: &SSPages, a_b: &Vec<Option<usize>>, stem: i32, sphere: i32) -> Vec<Issue> {
    let mut issues = vec![];
    
    // We check if every AHSS diff between known generators also exists on EHP
    for (&(from, to), p) in &a.proven_from_to {
        // Skip Algebraic things
        // This must already have been commutative
        // Else the algebraic data was wrong, which i don't assume
        if p.is_none() {
            continue;
        }

        // If it is in the ehp pages then (if it maps to something)
        // it must also be in ahss 
        if !a_p.element_in_pages(from) || !a_p.element_in_pages(to) {
            continue;
        }

        if a.model.stem(to) != stem {
            continue;
        }

        if let Some(b_from) = a_b[from] {
            if let Some(b_to) = a_b[to] {
                if b_p.element_in_pages(b_from) && b_p.element_in_pages(b_to) {
                    // let from_g = ahss_p.element_at_page(d_y, from);
                    // let to_g = ahss_p.element_at_page(d_y, to);
    
                    let d_y = a.model.y(from) - a.model.y(to);
    
                    // We only check differentials.
                    // Tau extensions are usually clear to resolve
                    let d_stem = a.model.stem(from) - a.model.stem(to);
                    
                    if d_y > 0 && d_stem == 1 {
                        let (from_name, to_name) = a.get_names(from, to);

                        // This is a slightly looser check
                        // We check if they are non zero 1 page later
                        // We only want to check a "non"-existence of a diff
                        // Not the specific configuration at a page
                        let from_g_b = b_p.element_at_page(d_y+1, b_from);
                        let to_g_b = b_p.element_at_page(d_y+1, b_to);
        
                        if from_g_b.1.alive() && to_g_b.1.alive() {
                            if !b.proven_from_to.contains_key(&(b_from, b_to)) {
        
                                
                                issues.push(Issue::InvalidEHPAHSSMap { 
                                    from: from_name, 
                                    to: to_name, 
                                    stem, 
                                    sphere });
                            }
                        }
                    }
                }
            }
        }
    }

    issues
}


// This is a reduced version
// Below is the "official" version
pub fn compare_ehp_ahss(ehp: &SyntheticSS, ahss: &SyntheticSS, (ehp_ahss, ahss_ehp): &SyntheticSSMap, stem: i32, sphere: i32) -> Result<(), Vec<Issue>> {
    let (ehp_p, _) = compute_pages(ehp, 0, sphere - 1, stem, stem + 1, false);
    let (ahss_p, _) = compute_pages(ahss, 0, sphere - 1, stem, stem + 1, false);

    let mut issues = vec![];

    // First we check if the torsion on E1 is mapped correctly
    for y in 0..=(sphere-1) {
        for &ehp_id in ehp.model.gens_id_in_stem_y(stem, y) {
            if let Some(ahss_id) = ehp_ahss[ehp_id] {
                if ehp.model.original_torsion(ehp_id).alive() {
                    if ehp.model.original_torsion(ehp_id) > ahss.model.original_torsion(ahss_id) {
                        issues.push(Issue::InvalidEHPAHSSGen { name: ehp.model.name(ehp_id).to_string(), stem });
                    }
                }
                if ehp_p.element_in_pages(ehp_id) {
                    if ehp_p.element_final(ehp_id).1 > ahss_p.element_final(ehp_id).1 {
                        issues.push(Issue::InvalidEHPAHSSGen { name: ehp.model.name(ehp_id).to_string(), stem });
                    }
                }
            }
        }
    }

    issues.append(&mut check(ahss, ehp, &ahss_p, &ehp_p, ahss_ehp, stem, sphere));
    issues.append(&mut check(ehp, ahss, &ehp_p, &ahss_p, ehp_ahss, stem, sphere));

    if issues.len() == 0 {
        Ok(())
    } else {
        Err(issues)
    }
}


// // This is the original complete version
// // Here one can also start to understand the non trivial map from S^n -> QS
// // meaning, the map outside of those on E2 generators.
// // However it is too complex to also account / keep track of those.
// pub fn compare_ehp_ahss(ehp: &SyntheticSS, ahss: &SyntheticSS, (ehp_ahss, ahss_ehp): &SyntheticSSMap, stem: i32, top_trunc: i32) -> Result<(), Vec<Issue>> {
//     let (ehp_p, _) = compute_pages(ehp, 0, top_trunc, stem, stem, false);
//     let (ahss_p, _) = compute_pages(ahss, 0, top_trunc, stem, stem, false);

//     let mut issues = vec![];

//     // First we check if generators are mapped correctly
//     for y in 0..=top_trunc {
//         for &ehp_id in ehp.model.gens_id_in_stem_y(stem, y) {
//             if let Some(ahss_id) = ehp_ahss[ehp_id] {
//                 if ehp.model.original_torsion(ehp_id).alive() {
//                     if ehp.model.original_torsion(ehp_id) > ahss.model.original_torsion(ahss_id) {
//                         issues.push(Issue::InvalidEHPAHSSGen { name: ehp.model.name(ehp_id).to_string(), stem });
//                     }
//                 }
//             }
//         }
//     }


//     for (&(from, to), p) in &ehp.proven_from_to {
//         // Skip Algebraic things
//         // This must already have been commutative
//         // Else the algebraic data was wrong, which i don't assume
//         if p.is_none() {
//             continue;
//         }

//         // If it is in the ehp pages then (if it maps to something)
//         // it must also be in ahss 
//         if !ehp_p.element_in_pages(from) || !ehp_p.element_in_pages(to) {
//             continue;
//         }

//         if ehp.model.stem(from) != stem {
//             continue;
//         }

//         if let Some(a_to) = ehp_ahss[to] {
//             let d_y = ehp.model.y(from) - ehp.model.y(to);
//             let d_stem =  ehp.model.stem(from) - ehp.model.stem(to);

//             if d_y == 0 {
//                 // Some internal tau thing
//             } else if d_stem == 0 {
//                 // Some external tau thing

//                 // TODO
//                 // In general we should be able to figure these out without AHSS
//                 // Thus we skip for now

//             } else {
//                 // Differentials !
//                 let from_g = ehp_p.element_at_page(d_y, from);
//                 let to_g = ehp_p.element_at_page(d_y, to);

//                 if from_g.1.alive() && to_g.1.alive() {
//                     let coeff = to_g.0 - from_g.0 - 1;
//                     let to_g_ahss = ahss_p.element_at_page(d_y, a_to);

//                     if to_g_ahss.1 < Torsion::new(coeff) {
//                         if let Some(a_from) = ehp_ahss[from] {
//                             if !ahss.proven_from_to.contains_key(&(a_from, a_to)) {
//                                 // ISSUE
//                                 let (from_name, to_name) = ehp.get_names(from, to);
//                                 issues.push(Issue::InvalidEHPAHSSMap { 
//                                     from: from_name, 
//                                     to: to_name, 
//                                     stem, 
//                                     sphere: top_trunc + 1 });
//                             }
//                         } else {
//                             // ISSUE
//                             let (from_name, to_name) = ehp.get_names(from, to);
//                             issues.push(Issue::InvalidEHPAHSSMap { 
//                                 from: from_name, 
//                                 to: to_name, 
//                                 stem, 
//                                 sphere: top_trunc + 1 });
//                         }
//                     }
//                 }
//             }
//         }
//     }


//     for (&(from, to), p) in &ahss.proven_from_to {
//         // Skip Algebraic things
//         // This must already have been commutative
//         // Else the algebraic data was wrong, which i don't assume
//         if p.is_none() {
//             continue;
//         }

//         // If it is in the ehp pages then (if it maps to something)
//         // it must also be in ahss 
//         if !ahss_p.element_in_pages(from) || !ahss_p.element_in_pages(to) {
//             continue;
//         }

//         if ahss.model.stem(from) != stem {
//             continue;
//         }

//         if let Some(e_from) = ahss_ehp[from] {
//             let d_y = ahss.model.y(from) - ahss.model.y(to);
//             let d_stem =  ahss.model.stem(from) - ahss.model.stem(to);

//             if d_y == 0 {
//                 // Some internal tau thing
//             } else if d_stem == 0 {
//                 // Some external tau thing

//                 // TODO
//                 // In general we should be able to figure these out without AHSS
//                 // Thus we skip for now

//             } else {
//                 // Differentials !
//                 let from_g = ahss_p.element_at_page(d_y, from);
//                 let to_g = ahss_p.element_at_page(d_y, to);
                
//                 if from_g.1.alive() && to_g.1.alive() {
//                     let from_g_ehp = ehp_p.element_at_page(d_y, e_from);

//                     if from_g_ehp.1.alive() {
//                         if let Some(e_to) = ahss_ehp[to] {
//                             if !ehp.proven_from_to.contains_key(&(e_from, e_to)) {
//                                 // ISSUE
//                                 let (from_name, to_name) = ahss.get_names(from, to);
//                                 issues.push(Issue::InvalidEHPAHSSMap { 
//                                     from: from_name, 
//                                     to: to_name, 
//                                     stem, 
//                                     sphere: top_trunc + 1 });
//                             }
//                         } else {
//                             // ISSUE
//                             let (from_name, to_name) = ahss.get_names(from, to);
//                             issues.push(Issue::InvalidEHPAHSSMap { 
//                                 from: from_name, 
//                                 to: to_name, 
//                                 stem, 
//                                 sphere: top_trunc + 1 });
//                         }
//                     }
//                 }
//             }
//         }
//     }


//     if issues.len() == 0 {
//         Ok(())
//     } else {
//         Err(issues)
//     }
// }
