use crate::{MAX_STEM, model::{PagesGeneratorState, SSPages, SyntheticSS}, process::compute_pages, types::Torsion};


type Stack = Vec<(Change,PageChange)>;

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



// pub fn get_options() -> Vec<i32> {

// }


pub fn induct(data: &mut SyntheticSS, pages: &mut SSPages, stem: i32, depth: i32)  {
    // What we should do is get all options

    // The list of options should start with E1 stuff.
    // Then potential tau_mult stuff
    // Then potential differentials < this can be quite a long list ?
    // So probably this function should contain the possibility to look at certain truncations ?

    // We pick our first option and try to prove / disprove the option.
    
    // We do this by applying the option, getting all new issues, and induct until we find an error?
    
    // In the induction step, if we have exhausted all options and still have issues in some stem. 
    // Then we actually have an unresolved stem error. 

    // Also; usually in AHSS, a single option has multiple applications due to James Periodicity. 
}

pub fn solve_ahss(data: &mut SyntheticSS) {
    let pages = compute_pages(data, 1, 256);
    let issues = ahss_issueus(data);

    let g_issues = vec![];
    let stem_af_issues = vec![];

    for stem in 1..MAX_STEM {
        // Get all options in a stem



        // Start by resolving the E1 page.
        // I think this can be done completely independently ?
        // But is that needed ?

        // Then we will look at the convergence of truncations in order
        


    }
}