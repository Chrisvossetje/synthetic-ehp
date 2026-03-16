use crate::{MAX_STEM, solve::issues::Issue, domain::model::{SSPages, SyntheticSS, map_between_generators}, types::Torsion};


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
fn apply_diff(pages: &mut SSPages, page: i32, from: usize, to: usize) -> Result<(), Issue> {
    let from_g = pages.element_final(from);
    let to_g = pages.element_final(to);


    if let Ok((new_from_g, new_to_g)) = map_between_generators(from_g, to_g) {
        pages.push(from, page + 1, new_from_g);
        pages.push(to, page + 1, new_to_g);
    } else {
        // TODO: 
        // println!("Some illegal thing between {} | {}", from, to);
    }
    Ok(())
}

fn apply_tau(pages: &mut SSPages, page: i32, from: usize, to: usize) -> Result<(),Issue> {
    let from_g = pages.element_final(from);
    let to_g = pages.element_final(to);

    if from_g.1.alive() && to_g.1.alive() {
        if let Some(from_torsion) = from_g.1.0 {
            let new_torsion = match to_g.1.0 {
                Some(t) => Torsion::new(from_torsion + t),
                None => Torsion::default(),
            };
            if from_g.0 - from_torsion == to_g.0 {
                pages.push(from, page, (from_g.0, new_torsion));
                pages.push(to, page, (to_g.0, Torsion::zero()));
            } else {
                panic!("Torsion from part does not correctly attach to target of this Tau mult.") 
            }
        } else {
            // This means we have a "free" generator as source
        }
    } 
    Ok(())
}

pub fn compute_pages(data: &SyntheticSS, bot_trunc: i32, top_trunc: i32, from_stem: i32, to_stem: i32) -> (SSPages, Vec<Issue>) {
    let mut pages = instantiate_pages(data, bot_trunc, top_trunc, from_stem, to_stem);

    let mut issues = vec![];

    for page in 0..=MAX_STEM as usize {
        for t in &data.internal_tau_page[page] {
            if pages.element_in_pages(t.from) && pages.element_in_pages(t.from)  {
                if let Err(i) = apply_tau(&mut pages, page as i32, t.from, t.to) {
                    issues.push(i);
                }
            }
        }

        for d in &data.diffs_page[page] {
            if pages.element_in_pages(d.from) && pages.element_in_pages(d.to) {
                if let Err(i) = apply_diff(&mut pages, page as i32, d.from, d.to) {
                    issues.push(i);
                }
            }
        }
    }
    
    // TODO: The order in which these appear right now is IMPORTANT
    for e in &data.external_tau_page {
        if pages.element_in_pages(e.from) && pages.element_in_pages(e.to)  {
            if let Err(i) = apply_tau(&mut pages, 500, e.from, e.to) {
                issues.push(i);
            }
        }
    }

    (pages, issues)
}