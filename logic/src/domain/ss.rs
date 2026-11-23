//! [`SSPages`]: the computed state of every generator across the spectral
//! sequence's pages, produced by [`crate::domain::process`]. Store histories in
//! the requested window and keep short histories inline.

use std::collections::HashMap;

use smallvec::SmallVec;

use crate::{domain::e1::E1, types::Torsion};

/// A generator's (AF, torsion) on a given page.
pub type GeneratorState = (i32, Torsion);
type Checkpoint = (i32, GeneratorState);
pub(super) type PageHistory = SmallVec<[Checkpoint; 2]>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SSPages {
    pub bot_trunc: i32,
    pub top_trunc: i32,

    // An empty history marks a generator excluded by the truncation or E1 torsion.
    first_id: usize,
    histories: Vec<PageHistory>,
}

impl SSPages {
    pub(super) fn new(
        bot_trunc: i32,
        top_trunc: i32,
        first_id: usize,
        histories: Vec<PageHistory>,
    ) -> Self {
        Self {
            bot_trunc,
            top_trunc,
            first_id,
            histories,
        }
    }

    pub fn checkpoints(&self, elt: usize) -> Option<&[(i32, GeneratorState)]> {
        let history = self.histories.get(elt.checked_sub(self.first_id)?)?;
        (!history.is_empty()).then_some(history.as_slice())
    }

    /// The generator's state as of `page`: the last checkpoint whose page is
    /// `<= page` (checkpoints are stored in increasing page order).
    pub fn element_at_page(&self, page: i32, elt: usize) -> GeneratorState {
        let l = self.checkpoints(elt).unwrap();
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
        self.checkpoints(elt).is_some()
    }

    pub fn element_final(&self, elt: usize) -> GeneratorState {
        self.try_element_final(elt).unwrap()
    }

    pub fn try_element_final(&self, elt: usize) -> Option<GeneratorState> {
        self.checkpoints(elt)?.last().map(|&(_, state)| state)
    }

    pub fn push(&mut self, elt: usize, page: i32, g: GeneratorState) {
        let history = &mut self.histories[elt - self.first_id];
        debug_assert!(!history.is_empty());
        history.push((page, g));
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
