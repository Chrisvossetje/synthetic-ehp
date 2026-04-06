use std::collections::HashMap;

use crate::{
    data::{curtis::DATA, naming::name_get_tag},
    domain::{
        model::SyntheticSS,
        process::compute_pages,
        ss::{PagesGeneratorState, SSPages},
    },
    solve::{action::Action, issues::Issue},
    types::Torsion,
};

pub fn auto_deduce(data: &SyntheticSS, issue: &Issue) -> Result<Action, ()> {
    match issue {
        Issue::SyntheticE1Page {
            stem,
            af,
            expected,
            observed,
        } => {
            if expected.len() == 1 {
                assert_eq!(observed.len(), 1);
                for id in data.model.gens_id_in_stem_af(*stem, *af) {
                    if data.model.y(*id) == 1 {
                        let name = data.model.name(*id);
                        return Ok(Action::SetE1 {
                            tag: name_get_tag(name).to_string(),
                            torsion: expected[0],
                            proof: format!("Only one generator in stem {stem}, af {af}. (auto)"),
                        });
                    }
                }
            }
            Err(())
        }
        Issue::InvalidName {
            original_name,
            unexpected_name,
            sphere,
            stem,
            af,
        } => {
            let (pages, _) = compute_pages(data, 0, sphere - 1, *stem, *stem, true);
            let (alg_pages, _) = compute_pages(&DATA, 0, sphere - 1, *stem, *stem, true);

            let mut syn = vec![];
            let mut alg = vec![];
            for id in DATA.model.gens_id_in_stem(*stem) {
                if pages.element_in_pages(*id) {
                    let g = pages.element_final(*id);
                    if g.1.alive() && g.0 == *af {
                        let name = data.get_name_at_sphere(*id, *sphere);
                        syn.push(name);
                    }
                }

                if alg_pages.element_in_pages(*id) {
                    let g = alg_pages.element_final(*id);
                    if g.1.alive() && g.0 == *af {
                        let name = DATA.model.name(*id);
                        alg.push(name);
                    }
                }
            }

            let fil_syn: Vec<_> = syn.iter().filter(|i| !alg.contains(i)).collect();
            let fil_alg: Vec<_> = alg.iter().filter(|i| !syn.contains(i)).collect();

            if fil_syn.len() == 1 && fil_alg.len() == 1 {
                let name = fil_alg[0];
                return Ok(Action::SetInducedName {
                    name: original_name.clone(),
                    new_name: name.to_string(),
                    sphere: *sphere,
                    proof: format!("Only one choice which could represent this recursion. (auto)"),
                });
            }
            if fil_alg.len() == 0 {
                println!(
                    "{} should be killed here. And might want to check algebraic convergence stuff here",
                    original_name
                );
            }
            Err(())
        }
        _ => Err(()),
    }
}