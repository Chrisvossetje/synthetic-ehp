use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{domain::model::SyntheticSS, types::Torsion};

pub type GeneratorState = (i32, Torsion);
// TODO: Smallvec performance check ?
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
        let a = &self.generators[elt];
        let b = a.as_deref().unwrap();
        b.last().unwrap().1.clone()
    }

    pub fn available_on_page(&self, elt: usize) -> i32 {
        self.generators[elt].as_deref().unwrap().last().unwrap().0
    }

    pub fn try_element_final(&self, elt: usize) -> Option<GeneratorState> {
        let a = &self.generators[elt];
        let b = a.as_deref()?;
        Some(b.last().unwrap().1.clone())
    }
    pub fn try_element_final_with_page(&self, elt: usize) -> Option<(i32, GeneratorState)> {
        let a = &self.generators[elt];
        let b = a.as_deref()?;
        Some(b.last().unwrap().clone())
    }

    
    pub fn push(&mut self, elt: usize, page: i32, g: GeneratorState) {
        self.generators[elt].as_mut().unwrap().push((page, g));
    }

    pub fn pop(&mut self, elt: usize, page: i32, g: GeneratorState) {
        self.generators[elt].as_mut().unwrap().push((page, g));
    }

    pub fn convergence_at_stem(&self, data: &SyntheticSS, stem: i32) -> HashMap<i32, Vec<Torsion>> {
        let mut m = HashMap::new();
        for id in data.model.gens_id_in_stem(stem) {
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
        data: &SyntheticSS,
        stem: i32,
    ) -> HashMap<i32, usize> {
        let observed_minus_one = self.convergence_at_stem(data, stem - 1);
        let observed = self.convergence_at_stem(data, stem);

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
