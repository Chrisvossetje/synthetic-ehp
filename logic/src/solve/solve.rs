use std::collections::HashMap;

use crate::{
    data::{curtis::DATA, naming::name_get_tag},
    domain::{
        model::{ExtTauMult, SyntheticSS},
        process::compute_pages,
        ss::{PagesGeneratorState, SSPages},
    },
    solve::{action::Action, ahss_e1::{get_all_e1_solutions, get_e1_solutions}, generate::{get_a_tau_for_t_ids, get_a_tau_for_t_ids_s_ids}, issues::Issue},
    types::Torsion,
};

pub fn auto_deduce(data: &SyntheticSS, issue: &Issue) -> Result<Vec<Action>, ()> {
    match issue {
        Issue::SyntheticE1Page {
            stem,
            af,
            expected,
            observed,
        } => {
            let mut sol = get_e1_solutions(data, issue);
            if sol.len() == 1 && let Some(mut sol) = sol.pop() {
                for s in &mut sol {
                    match s {
                        Action::SetE1 { tag, torsion, proof } => {*proof = format!("Unique solution in stem {stem} af {af}. (auto)")},
                        _ => {unreachable!()}
                    }
                } 
                return Ok(sol);
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
                return Ok(vec![Action::SetInducedName {
                    name: original_name.clone(),
                    new_name: name.to_string(),
                    sphere: *sphere,
                    proof: format!("Only one choice which could represent this recursion. (auto)"),
                }]);
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

pub fn suggest_tau_solution_algebraic(data: &SyntheticSS, issues: &mut Vec<Issue>, top_trunc: i32, bot_trunc: i32, stem: i32) -> Option<ExtTauMult> {
    let (elements, _) = compute_pages(data, bot_trunc, top_trunc, stem, stem, false);
    
    issues.sort_by_key(|i| if let Issue::AlgebraicConvergence { bot_trunc, top_trunc, stem, af, expected, observed } = i {*af} else {unreachable!()});

    
    for i in issues {
        if let Issue::AlgebraicConvergence { bot_trunc, top_trunc, stem, af, expected, observed } = i {
            
            let t_ids: Vec<_> = data.model.gens_id_in_stem(*stem).iter().filter(|t_id| if let Some((t_af, t_torsion)) = elements.try_element_final(**t_id) {t_af == *af} else {false}).map(|x| *x).collect();
        
            let d = get_a_tau_for_t_ids(data,  &elements, &t_ids, *stem);
            if d.is_some() {
                return d;
            }
        }
    }

    None
}

pub fn suggest_tau_solution_generator_synthetic(data: &SyntheticSS, issues: &mut Vec<Issue>, top_trunc: i32, bot_trunc: i32, stem: i32) -> Option<ExtTauMult> {
    let (elements, _) = compute_pages(data, bot_trunc, top_trunc, stem, stem, false);
    
    issues.sort_by_key(|i| if let Issue::SyntheticConvergence { bot_trunc, top_trunc, stem, af, expected, observed } = i {*af} else {unreachable!()});

    let mut t_ids = vec![];
    // let mut s_ids = vec![];
    for i in issues {
        if let Issue::SyntheticConvergence { bot_trunc, top_trunc, stem, af, expected, observed } = i {
            for t_id in data.model.gens_id_in_stem(*stem).iter().filter(|t_id| if let Some((t_af, t_torsion)) = elements.try_element_final(**t_id) {t_af == *af} else {false}) {
                t_ids.push(*t_id);
            }
            // if expected.len() != observed.len() {
            // } else {
            //     for t_id in data.model.gens_id_in_stem(stem).iter().filter(|t_id| if let Some((t_af, t_torsion)) = elements.try_element_final(**t_id) {t_af == *af} else {false}) {
            //         t_ids.push(*t_id);
            //     }
            // }
        }
    }

    get_a_tau_for_t_ids_s_ids(data,  &elements, &t_ids, &t_ids, stem)
}

pub fn suggest_tau_solution_module_synthetic(data: &SyntheticSS, issues: &Vec<Issue>, top_trunc: i32, bot_trunc: i32, stem: i32) -> Option<ExtTauMult> {
    // let (elements, _) = compute_pages(data, bot_trunc, top_trunc, stem, stem, false);
    // let t_ids = vec![];
    // for i in issues {
    //     if let Issue::SyntheticConvergence { bot_trunc, top_trunc, stem, af, expected, observed } = i {
    //         for t_id in data.model.gens_id_in_stem(*stem).iter().filter(|t_id| if let Some((t_af, t_torsion)) = elements.try_element_final(**t_id) {t_af == *af} else {false}) {
    //             t_ids.push(t_id);
    //         }
        
    //     }
    // }
    // get_a_tau_for_t_ids_s_ids(data,  &elements, &t_ids, *stem)
    None
}