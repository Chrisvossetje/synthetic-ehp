use crate::{MAX_STEM, model::{SSPages, SyntheticSS, map_between_generators}, types::Torsion};


fn instantiate_pages(data: &SyntheticSS, bot_trunc: i32, top_trunc: i32) -> SSPages {
    let mut gens = vec![];

    let mut max_stem = 0;

    for (index, g) in data.model.enumerate() {
        max_stem = max_stem.max(g.stem);

        if bot_trunc <= g.y && g.y <= top_trunc {
            gens.push(
                Some(vec![(1, (g.af, g.torsion))])
            );
        } else {
            gens.push(None);
        } 
    }

    SSPages { bot_trunc, top_trunc, generators: gens }
}

// Assume differentials are applied in correct order
fn apply_diff(pages: &mut SSPages, page: i32, from: usize, to: usize) {
    let from_g = pages.element_final(from);
    let to_g = pages.element_final(to);

    let (new_from_g, new_to_g) = map_between_generators(from_g, to_g).unwrap();
    pages.push(from, page, new_from_g);
    pages.push(to, page, new_to_g);
}

fn apply_tau(pages: &mut SSPages, page: i32, from: usize, to: usize) {
    let from_g = pages.element_final(from);
    let to_g = pages.element_final(to);

    let from_torsion = from_g.1.0.unwrap();
    let new_torsion = match to_g.1.0 {
        Some(t) => Torsion::new(from_torsion + t),
        None => Torsion::default(),
    };

    pages.push(from, page, (from_g.0, new_torsion));
    pages.push(to, page, (to_g.0, Torsion::zero()));

}

pub fn compute_pages(data: &SyntheticSS, bot_trunc: i32, top_trunc: i32) -> SSPages {
    let mut pages = instantiate_pages(data, bot_trunc, top_trunc);

    // We assume that NO invalid differentials exist on the SyntheticSS
    for page in 0..=MAX_STEM as usize {
        for d in &data.diffs_page[page] {
            if pages.generators[d.from].is_some() && pages.generators[d.to].is_some() {
                apply_diff(&mut pages, page as i32, d.from, d.to);
            }
        }
        
        for t in &data.internal_tau_page[page] {
            if pages.generators[t.from].is_some() && pages.generators[t.to].is_some() {
                apply_tau(&mut pages, page as i32, t.from, t.to);
            }
        }
    }

    for e in &data.external_tau_page {
        apply_tau(&mut pages, 1000, e.from, e.to);
    }

    pages
}