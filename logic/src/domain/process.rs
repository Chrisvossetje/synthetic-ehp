use crate::{MAX_STEM, data::naming::{add_sphere_to_tag, generating_tag}, domain::{model::SyntheticSS, ss::SSPages}, solve::issues::Issue, types::Torsion};


fn instantiate_pages(data: &SyntheticSS, bot_trunc: i32, top_trunc: i32, from_stem: i32, to_stem: i32) -> SSPages {
    let mut gens = vec![];

    let mut max_stem = 0;

    for (_, g) in data.model.enumerate() {
        max_stem = max_stem.max(g.stem);

        if from_stem - 1 <= g.stem && g.stem <= to_stem + 1 {
            if bot_trunc <= g.y && g.y <= top_trunc && g.torsion.alive() {
                gens.push(
                    Some(vec![(1, (g.af, g.torsion))])
                );
                continue;
            }
        }
        
        gens.push(None);
    }

    SSPages { bot_trunc, top_trunc, generators: gens }
}

// Assume differentials are applied in correct order
fn apply_diff(data: &SyntheticSS, pages: &mut SSPages, page: i32, from: usize, to: usize) -> Result<(), Issue> {
    let from_g = pages.element_final(from);
    let to_g = pages.element_final(to);

    let stem = data.model.stem(from);

    let (new_from_g, new_to_g) = 
        if from_g.1.alive() {
            if !to_g.1.alive() {
                let (from_name, to_name) = data.get_names(from, to);
                return Ok(());
                // TODO: 
                // UselessDifferential { from: 925, to: 806, bot_trunc: 3, top_trunc: 11, from_name: "3 6 2 3 3[11]", to_name: "2 3 5 7 3 3[4]" }
                // return Err(Issue::UselessDifferential { from, to, bot_trunc: pages.bot_trunc, top_trunc: pages.top_trunc, from_name, to_name });
            }

            let coeff = to_g.0 - from_g.0 - 1;
            if coeff < 0 {
                // TODO: Figure out if this should be branching or not ?
                // panic!("We encountered a negative coefficient, such a differential can not exist? And is unfixable ?");
                let (from_name, to_name) = data.get_names(from, to);
                return Err(Issue::InvalidCoeff { from, to, coeff, from_name, to_name });
            }
        
        
            match to_g.1.0 {
                Some(to_t) => match from_g.1.0 {
                    Some(from_t) => {
                        // Delta represents how much F2 generators are actually hit
                        let delta = to_t - coeff; 
                        if delta > from_t {
                            let (from_name, to_name) = data.get_names(from, to);
                            return Err(Issue::InvalidTorsion { from, to, stem, to_needed: Torsion::new(from_t + coeff), from_name, to_name })
                        } else {
                            let new_from_af =  from_g.0 - delta;
                            let new_from_t = from_t - delta;
                            
                            let from = (new_from_af, Torsion::new(new_from_t));
                            let to = (to_g.0, Torsion::new(coeff));
                            (from, to)
                        }
                    },
                    None => {
                        let from = (from_g.0 - to_t + coeff, Torsion::default());
                        let to = (to_g.0, Torsion::new(coeff));
                        (from, to)
                    },
                },
                None => match from_g.1.0 {
                    Some(t) => { 
                        let (from_name, to_name) = data.get_names(from, to);
                        return Err(Issue::InvalidTorsion { from, to, stem, to_needed: Torsion::new(t + coeff), from_name, to_name })
                    },
                    None => {
                        let from = (from_g.0, Torsion::zero());
                        let to = (to_g.0, Torsion::new(coeff));
                        (from, to)
                    },
                },
            }
        } else {
            (from_g, to_g)
        };

    pages.push(from, page + 1, new_from_g);
    pages.push(to, page + 1, new_to_g);

    Ok(())
}

fn apply_tau(data: &SyntheticSS, pages: &mut SSPages, page: i32, from: usize, to: usize) -> Result<(),Issue> {
    let from_g = pages.element_final(from);
    let to_g = pages.element_final(to);

    if from_g.1.alive() && to_g.1.alive() {
        if let Some(from_torsion) = from_g.1.0 {
            let new_torsion = match to_g.1.0 {
                Some(t) => Torsion::new(from_torsion + t),
                None => Torsion::default(),
            };
            let af_diff = from_g.0 - to_g.0;
            
            if from_g.0 - from_torsion <= to_g.0 && af_diff > 0 {
                // If this is negative, it means that we actually have smallar torsion on the other element
                let leftover_torsion = Torsion::new(from_torsion - af_diff);
                pages.push(from, page, (from_g.0, new_torsion));
                pages.push(to, page, (to_g.0, leftover_torsion));
            } else if from_g.0 - from_torsion > to_g.0 {
                let (from_name, to_name) = data.get_names(from, to);
                return Err(
                    Issue::InvalidTauMult { from, to, from_name, to_name }
                );
            }
        } else {
            // This means we have a "free" generator as source
            // And the tau multiplication should not happen at this "truncation"
        }
    } 
    Ok(())
}

pub fn try_compute_pages(data: &SyntheticSS, bot_trunc: i32, top_trunc: i32, from_stem: i32, to_stem: i32) -> Result<SSPages, Vec<Issue>> {
    let (pages, issues) = compute_pages(data, bot_trunc, top_trunc, from_stem, to_stem);

    if issues.len() != 0 {
        return Err(issues);
    } else {
        Ok(pages)
    }
}

pub fn compute_pages(data: &SyntheticSS, bot_trunc: i32, top_trunc: i32, from_stem: i32, to_stem: i32) -> (SSPages, Vec<Issue>) {
    let mut pages = instantiate_pages(data, bot_trunc, top_trunc, from_stem, to_stem);

    let mut issues = vec![];

    for page in 0..=MAX_STEM as usize {
        for t in &data.internal_tau_page[page] {
            if pages.element_in_pages(t.from) && pages.element_in_pages(t.from)  {
                if let Err(i) = apply_tau(data, &mut pages, page as i32, t.from, t.to) {
                    issues.push(i);
                }
            }
        }

        for d in &data.diffs_page[page] {
            if pages.element_in_pages(d.from) && pages.element_in_pages(d.to) {
                if let Err(i) = apply_diff(data, &mut pages, page as i32, d.from, d.to) {
                    issues.push(i);
                }
            }
        }
    }
    
    // TODO: The order in which these appear right now is IMPORTANT
    for e in &data.external_tau_page {
        if pages.element_in_pages(e.from) && pages.element_in_pages(e.to)  {
            if let Err(i) = apply_tau(data, &mut pages, 500, e.from, e.to) {
                issues.push(i);
            }
        }
    }

    (pages, issues)
}



pub fn ehp_recursion(ehp: &mut SyntheticSS, sphere: i32, stem: i32) -> Result<(), Vec<Issue>> {
    if sphere % 2 == 0 {
        panic!("Can't call this on even spheres");
    }

    let pages = try_compute_pages(ehp, 0, sphere - 1, stem, stem)?;

    // Set everything to zero
    for id in ehp.model.gens_id_in_stem_y(stem + sphere/2, sphere/2).clone() {
        ehp.model.get_mut(id).torsion = Torsion::zero();
    }
    
    let mut issues = vec![];
    
    // Set everything to what we expect
    for id in ehp.model.gens_id_in_stem(stem).clone() {
        if pages.element_in_pages(id) {
            let g = pages.element_final(id);
            
            let name = ehp.get_name_at_sphere(id, sphere).to_string();
            let gen_tag = generating_tag(&name);

            let target_name = add_sphere_to_tag(gen_tag, sphere);
            if g.1.alive() {
                match ehp.model.try_index(&target_name) {
                    Some(target_id) => {
                        let t_g = ehp.model.get_mut(target_id);
                        if g.0 + 1 != t_g.af {
                            issues.push(Issue::InvalidAFRecursion { 
                                from: id, 
                                to: target_id, 
                                from_name: name, 
                                to_name: target_name, 
                            });
                        } else {
                            t_g.torsion = g.1;
                        }
                    },
                    None => {
                        issues.push(Issue::InvalidName { 
                            original_name: name.to_string(),
                            unexpected_name: target_name, 
                            stem,
                            sphere,
                            af: g.0 });
                    },
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