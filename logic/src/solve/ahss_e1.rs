//! Enumerating candidate fixes for E1-page convergence issues in the AHSS.
//! [`get_e1_solutions`] lists the distinct ways the torsion in a (stem, AF)
//! cell could be assigned to match what's expected, and [`get_all_e1_solutions`]
//! takes the cartesian product across several issues. (The large commented-out
//! blocks are an earlier inductive solver, kept for reference.)

use itertools::{self, Itertools};

use crate::{
    data::naming::{generate_names_from_tag, name_get_tag},
    domain::{e1::E1, model::SyntheticSS},
    solve::{action::Action, issues::Issue},
    types::Torsion,
};

fn get_all_elts_belonging_to_this_e1(model: &E1, g: usize) -> Vec<usize> {
    let name = model.name(g);
    let tag = name_get_tag(&name);

    let mut idxs = vec![];
    for n in generate_names_from_tag(tag, 1, 1) {
        if let Some(id) = model.try_index(&n) {
            idxs.push(id);
        } else {
            break;
        };
    }
    idxs
}


