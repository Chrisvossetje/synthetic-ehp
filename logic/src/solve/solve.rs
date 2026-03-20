use std::collections::HashMap;

use crate::{MAX_STEM, data::{curtis::{DATA, DATA_PAGES}, naming::name_get_tag}, domain::{model::SyntheticSS, process::compute_pages, ss::{PagesGeneratorState, SSPages}}, solve::{action::Action, issues::Issue}, types::Torsion};


type Stack = Vec<(Change,PageChange)>;

// Truncation to Pages
type PagesLUT = HashMap<(i32,i32), SSPages>;

pub enum Change {
    // This is the easiest to revert ?
    // I think ?? Although we should also think on how to revert the 
    E1 { g: usize, new: Torsion, old: Torsion },
    Diff { from: usize, to: usize },
    IntTau { from: usize, to: usize },
    ExtTau { from: usize, to: usize },
}


// In general the PagesGeneratorState is quite small
pub struct PageChange {
    // (element, full old page data)
    old: Vec<(usize, PagesGeneratorState)>,
    
    // (element, full new page data)
    new: Vec<(usize, PagesGeneratorState)>,
}



// pub fn get_options_in_stem() -> Vec<Change> {

// }

// pub fn get_E1_options_in_stem() {

// }


pub fn induct(data: &mut SyntheticSS, pages: &mut SSPages, full_pages: &mut SSPages, issues: Vec<()>, stem: i32, depth: i32, max_depth: i32)  {
    // What we should do is get all options

    // The list of options should start with E1 stuff.
    // Then potential tau_mult stuff
    // Then potential differentials < this can be quite a long list ?
    // So probably this function should contain the possibility to look at certain truncations ?

    // We pick our first option and try to prove / disprove the option.
    

    // We do this by applying the option, getting all new issues, and induct until we find an error?
    // Probably we only want to "apply" the option to a small subset of truncations ?

    // In the induction step, if we have exhausted all options and still have issues in some stem. 
    // Then we actually have an unresolved stem error. 

    // Also; usually in AHSS, a single option has multiple applications due to James Periodicity. 

    // Also; if all options in a certain stem have been exhausted and no solution to the issue has been found we should quit.
    // It is only useful for A


}


pub fn try_and_get_as_far_as_possible(data: &mut SyntheticSS, bot_trunc: i32, top_trunc: i32) {
    // let pages = compute_pages(data, bot_trunc, top_trunc);
    

    // Sort issues by stem
    // Then we sort by first having E1 issues

    // Then, We try to prove/disprove E1 stuff



    // Then, try and (dis)prove certain tau-mults
    // Then, try and (dis)prove differentials
}



pub fn auto_deduce(data: &SyntheticSS, issue: &Issue) -> Result<Action, ()> {
    match issue {
        Issue::SyntheticE1Page { stem, af, expected, observed } => {
            if expected.len() == 1 {
                assert_eq!(observed.len(), 1);
                for id in data.model.gens_id_in_stem_af(*stem, *af) {
                    if data.model.y(*id) == 1 {
                        let name = data.model.name(*id);
                        return Ok(Action::SetE1 { 
                            tag: name_get_tag(name).to_string(), 
                            torsion: expected[0], 
                            proof: format!("Only one generator in stem {stem}, af {af}. (auto)")
                        });
                    }
                }
            }
            Err(())
        },
        Issue::InvalidName { original_name, unexpected_name, sphere, stem, af } => {
            let (pages, _) = compute_pages(data, 0, sphere-1, *stem, *stem);
            let (alg_pages, _) = compute_pages(&DATA, 0, sphere-1, *stem, *stem);

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
                    proof: format!("Only one choice which could represent this recursion. (auto)") });
            } if fil_alg.len() == 0 {
                println!("{} should be killed here. And might want to check algebraic convergence stuff here", original_name);
            }
            Err(())
        }
        _ => { Err(()) }
    }
}



pub fn solve_ahss(data: &mut SyntheticSS) {
    // let pages = compute_pages(data, 1, 256);
    // // let issues = ahss_issueus(data);

    // // let g_issues = vec![];
    // // let stem_af_issues = vec![];

    // for stem in 1..MAX_STEM {
    //     // Get all options in a stem



    //     // Start by resolving the E1 page.
    //     // I think this can be done completely independently ?
    //     // But is that needed ?

    //     // Then we will look at the convergence of truncations in order
        


    // }
}