use std::{cell::Cell, collections::HashMap, iter::Enumerate, slice::Iter};

use serde::{Deserialize, Serialize};

use crate::{issues::InvalidTorsionIssue, types::{Differential, Generator, TauMult, Torsion}};



// This should represent the E1 page ? Is this necessary ?
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct E1 {
    generators: Vec<Generator>,
    index: HashMap<String, usize>,
    stem: HashMap<i32, Vec<usize>>,
    stem_af: HashMap<(i32, i32), Vec<usize>>,
}

impl E1 {
    pub fn new(generators: Vec<Generator>) -> Self {
        let mut index = HashMap::new();
        let mut stem = HashMap::new();
        let mut stem_af = HashMap::new();

        for (i, g) in generators.iter().enumerate() {
            index.insert(g.name.clone(), i);
            stem.entry(g.stem).or_insert(vec![]).push(i);
            stem_af.entry((g.stem, g.af)).or_insert(vec![]).push(i);
        }
        
        Self {
            generators,
            index,
            stem,
            stem_af,
        }
    }

    pub fn name(&self, elt: usize) -> &str {
        &self.generators[elt].name
    }

    pub fn y(&self, elt: usize) -> i32 {
        self.generators[elt].y
    }

    pub fn stem(&self, elt: usize) -> i32 {
        self.generators[elt].stem
    }

    pub fn original_af(&self, elt: usize) -> i32 {
        self.generators[elt].af
    }

    pub fn original_torsion(&self, elt: usize) -> Torsion {
        self.generators[elt].torsion
    }

    pub fn get(&self, id: usize) -> &Generator {
        &self.generators[id]
    }

    pub fn get_mut(&mut self, id: usize) -> &Generator {
        &mut self.generators[id]
    }

    pub fn get_index(&self, name: &String) -> usize {
        *self.index.get(name).unwrap()
    }

    pub fn get_name(&self, name: &String) -> &Generator {
        self.get(self.get_index(name))
    }

    pub fn get_name_mut(&mut self, name: &String) -> &Generator {
        self.get_mut(self.get_index(name))
    }

    pub fn gens(&self) -> &Vec<Generator> {
        &self.generators
    } 
    
    pub fn enumerate(&self) -> Enumerate<Iter<'_, Generator>> {
        self.generators.iter().enumerate()
    } 
    
    pub fn gens_id_in_stem(&self, stem: i32) -> &Vec<usize> {
        &self.stem.get(&stem).unwrap()
    } 
    
    pub fn gens_id_in_stem_af(&self, stem: i32, af: i32) -> &Vec<usize> {
        &self.stem_af.get(&(stem, af)).unwrap()
    } 
}

// This should always implicitly reference some Model
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SyntheticSS {
    pub model: E1,

    // This should be indexed by page ??
    // Or should it be indexed by Gens
    // Length of this should equal max_stem + 1
    pub diffs_page: Vec<Vec<Diff>>,
    pub internal_tau_page: Vec<Vec<IntTauMult>>,


    pub external_tau_page: Vec<ExtTauMult>,

    // // Maybe we will want to have both options ?
    // pub diffs_in: Vec<Vec<Differential>>,
    // pub diffs_out: Vec<Vec<Differential>>,
    // pub tau_in: Vec<Vec<TauMult>>,
    // pub tau_out: Vec<Vec<TauMult>>,
}

pub type GeneratorState = (i32, Torsion);
// TODO: Smallvec performance check ?
pub type PagesGeneratorState = Vec<(i32, GeneratorState)>;

pub fn map_between_generators(from: GeneratorState, to: GeneratorState) -> Result<(GeneratorState, GeneratorState), ()> {
    let coeff = to.0 - from.0 - 1;
    if coeff < 0 {
        // TODO: Figure out if this should be branching or not ?
        panic!("We encountered a negative coefficient, such a differential should never have been suggested ? And is unfixable ?");
    }

    match to.1.0 {
        Some(to_t) => match from.1.0 {
            Some(from_t) => {
                // Delta represents how much F2 generators are actually hit
                let delta = to_t - coeff; 
                if delta > from_t {
                    Err(())
                } else {
                    let new_from_af =  from.0 - delta;
                    let new_from_t = from_t - delta;
                    
                    let from = (new_from_af, Torsion::new(new_from_t));
                    let to = (to.0, Torsion::new(coeff));
                    Ok((from, to))
                }
            },
            None => {
                let from = (from.0 - coeff, Torsion::default());
                let to = (to.0, Torsion::new(coeff));
                Ok((from, to))
            },
        },
        None => match from.1.0 {
            Some(from_t) => {
                Err(())
            },
            None => {
                let from = (from.0, Torsion::zero());
                let to = (to.0, Torsion::new(coeff));
                Ok((from, to))
            },
        },
    }
}

// A diff could also be defined just by where it is from to where it goes ?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Diff {
    pub from: usize,
    pub to: usize,
}

// Here Tau Mult extension is probably not really correct ?
// It probably has more to do with choice of basis ?
// But i have to say something about convergence and how certain elements will lift :(
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct IntTauMult {
    pub from: usize,
    pub to: usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExtTauMult {
    pub from: usize,
    pub to: usize,
}


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

        while l[id].0 > page {
            id += 1;
            
        }
        l.last().unwrap().1
    }

    pub fn element_final(&self, elt: usize) -> GeneratorState {
        let a = &self.generators[elt];
        let b = a.as_deref().unwrap();
        b.last().unwrap().1.clone()
    }

    pub fn push(&mut self, elt: usize, page: i32, g: GeneratorState) {
        self.generators[elt].as_mut().unwrap().push((page, g));
    }

    pub fn pop(&mut self, elt: usize, page: i32, g: GeneratorState) {
        self.generators[elt].as_mut().unwrap().push((page, g));
    }
}