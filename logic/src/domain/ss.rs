//! [`SSPages`]: the computed state of every generator across the spectral
//! sequence's pages, produced by [`crate::domain::process`]. For each generator
//! we store the sequence of (page, state) it passes through; `None` means the
//! generator does not appear in this truncation.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{domain::e1::E1, types::Torsion};

/// A generator's (AF, torsion) on a given page.
pub type GeneratorState = (i32, Torsion);
// TODO: Smallvec performance check ?
/// The (page, state) checkpoints a generator passes through, in page order.
pub type PagesGeneratorState = Vec<(i32, GeneratorState)>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SSPages {
    pub bot_trunc: i32,
    pub top_trunc: i32,

    // Length of this should coincide with model and should always be filled
    // Should i also add another Vec around this to filter on Stem ?
    // TODO : SMALLVEC
    pub generators: Vec<Option<PagesGeneratorState>>,
}

impl SSPages {
    /// The generator's state as of `page`: the last checkpoint whose page is
    /// `<= page` (checkpoints are stored in increasing page order).
    pub fn element_at_page(&self, page: i32, elt: usize) -> GeneratorState {
        let l = self.generators[elt].as_ref().unwrap();
        let mut id = 0;

        loop {
            if id + 1 == l.len() {
                return l[id].1;
            }
            if l[id + 1].0 > page {
                return l[id].1;
            }
            id += 1;
        }
    }

    pub fn element_in_pages(&self, elt: usize) -> bool {
        self.generators[elt].is_some()
    }

    pub fn element_final(&self, elt: usize) -> GeneratorState {
        self.try_element_final(elt).unwrap()
    }

    pub fn try_element_final(&self, elt: usize) -> Option<GeneratorState> {
        let states = self.generators[elt].as_deref()?;
        Some(states.last().unwrap().1)
    }

    pub fn push(&mut self, elt: usize, page: i32, g: GeneratorState) {
        self.generators[elt].as_mut().unwrap().push((page, g));
    }

    pub fn convergence_at_stem(&self, model: &E1, stem: i32) -> HashMap<i32, Vec<Torsion>> {
        let mut m = HashMap::new();
        for id in model.gens_id_in_stem(stem) {
            if self.element_in_pages(*id) {
                let g = self.element_final(*id);
                if g.1.alive() {
                    m.entry(g.0).or_insert(vec![]).push(g.1);
                }
            }
        }

        for j in &mut m {
            j.1.sort();
        }
        m
    }

    pub fn algebraic_convergence_at_stem(
        &self,
        model: &E1,
        stem: i32,
    ) -> HashMap<i32, usize> {
        let observed_minus_one = self.convergence_at_stem(model, stem - 1);
        let observed = self.convergence_at_stem(model, stem);

        let mut observed: HashMap<_, _> = observed.iter().map(|(k, v)| (*k, v.len())).collect();
        for (j, l) in &observed_minus_one {
            for i in l {
                if let Some(torsion) = i.0 {
                    let b = j - torsion - 1;
                    *observed.entry(b).or_insert(0) += 1;
                }
            }
        }

        observed
    }
}
