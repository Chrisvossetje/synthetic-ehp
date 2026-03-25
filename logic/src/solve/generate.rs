use crate::{
    domain::{model::SyntheticSS, process::compute_pages},
    types::Torsion,
};

// This list all OUTGOING differentials from a stem

pub fn get_all_diffs(data: &mut SyntheticSS, top_trunc: i32, stem: i32) {
    let (targets, _) = compute_pages(data, 0, top_trunc, stem, stem, false);

    for &t_id in data.model.gens_id_in_stem(stem) {
        let y = data.model.y(t_id);
        if y != top_trunc {
            if let Some((af, torsion)) = targets.try_element_final(t_id) {
                if torsion.alive() {
                    let diffs = get_sources_to_target(data, t_id, af, torsion, top_trunc);
                    for d in diffs {
                        println!("{:?}", data.get_names(d, t_id));

                        data.temp_fakes.insert((d, t_id));
                    }
                }
            }
        }
    }
}

pub fn get_sources_to_target(
    data: &SyntheticSS,
    target: usize,
    af: i32,
    torsion: Torsion,
    y_sources: i32,
) -> Vec<usize> {
    let stem = data.model.stem(target);
    let y = data.model.y(target);

    let (sources, _) = compute_pages(data, 0, 256, stem + 1, stem + 1, false);

    // let from_page = sources.available_on_page(target);
    // let (af, torsion) = sources.element_final(target);

    let mut options = vec![];

    for &g_id in data.model.gens_id_in_stem_y(stem + 1, y_sources) {
        if sources.element_in_pages(g_id) {
            let (s_af, s_torsion) = sources.element_final(g_id);
            let coeff = af - s_af - 1;
            if s_torsion.alive() && coeff >= 0 {
                if torsion.can_map_with_coeff(&s_torsion, coeff) {
                    options.push(g_id);
                }
            }
        }
    }

    options
}
