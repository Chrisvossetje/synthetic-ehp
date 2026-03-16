use std::{cell::Cell, collections::{HashMap, HashSet}, iter::Enumerate, slice::Iter};

use serde::{Deserialize, Serialize};

use crate::{domain::e1::E1, data::naming::name_get_tag, types::{Generator, Torsion}};


pub type FromTo = (usize, usize);

// This should always implicitly reference some Model
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SyntheticSS {
    pub model: E1,

    // This should be indexed by page ??
    // Or should it be indexed by Gens
    // Length of this should equal max_stem + 1
    pub diffs_page: Vec<Vec<Diff>>,
    pub internal_tau_page: Vec<Vec<IntTauMult>>,

    // This happens at the "final" page
    pub external_tau_page: Vec<ExtTauMult>,

    // Index on the two generors, then a potential proof / disproof ?
    pub proven_from_to: HashMap<FromTo, Option<String>>,
    pub disproven_from_to: HashMap<FromTo, Option<String>>,

    pub temp_fakes: HashSet<FromTo>,

    // Remember all incoming stuff
    pub in_diffs: HashMap<usize, usize>,
    pub out_ext_tau: HashMap<usize, usize>,
}

impl SyntheticSS {
    pub fn disprove_from_to(&mut self, from: usize, to: usize, proof: Option<String>) {
        self.disproven_from_to.insert((from, to), proof);
    }

    pub fn add_diff(&mut self, from: usize, to: usize, proof: Option<String>) {
        let d_y = self.model.y(from) - self.model.y(to);

        if !self.proven_from_to.contains_key(&(from, to)) {
            self.diffs_page[d_y as usize].push(Diff { from, to });
            self.in_diffs.insert(to, from);
            self.proven_from_to.insert((from, to), proof);
        }
        
    }
    
    pub fn add_int_tau(&mut self, from: usize, to: usize, page: i32, proof: Option<String>) {
        if !self.proven_from_to.contains_key(&(from, to)) {
            self.internal_tau_page[page as usize].push(IntTauMult { from, to });
            self.proven_from_to.insert((from, to), proof);
        }
    }
    
    pub fn add_ext_tau(&mut self, from: usize, to: usize, proof: Option<String>) {
        if !self.proven_from_to.contains_key(&(from, to)) {
            self.external_tau_page.push(ExtTauMult { from, to });
            self.proven_from_to.insert((from, to), proof);
            self.out_ext_tau.insert(from, to);
        }
    }

    pub fn add_diff_name(&mut self, from: String, to: String, proof: Option<String>) -> Result<(),()> {
        let from = self.model.try_index(&from).ok_or(())?;
        let to = self.model.try_index(&to).ok_or(())?;
        self.add_diff(from, to, proof);
        Ok(())
    }

    pub fn add_int_tau_name(&mut self, from: String, to: String, page: i32, proof: Option<String>) -> Result<(),()> {
        let from = self.model.try_index(&from).ok_or(())?;
        let to = self.model.try_index(&to).ok_or(())?;
        self.add_int_tau(from, to, page, proof);
        Ok(())
    }

    pub fn add_ext_tau_name(&mut self, from: String, to: String, proof: Option<String>) -> Result<(),()> {
        let from = self.model.try_index(&from).ok_or(())?;
        let to = self.model.try_index(&to).ok_or(())?;
        self.add_ext_tau(from, to, proof);
        Ok(())
    }

    pub fn set_generator(&mut self, name: &String, torsion: Torsion) -> Result<(),()> {
        let id = self.model.try_index(name).ok_or(())?;
        self.model.get_mut(id).torsion = torsion;
        Ok(())
    }

    pub fn try_name_tag<'a>(&self, name: &'a str) -> Result<&'a str, ()> {
        self.model.try_index(name).ok_or(())?;
        Ok(name_get_tag(name))
    }
}

pub type GeneratorState = (i32, Torsion);
// TODO: Smallvec performance check ?
pub type PagesGeneratorState = Vec<(i32, GeneratorState)>;

pub fn map_between_generators(from: GeneratorState, to: GeneratorState) -> Result<(GeneratorState, GeneratorState), (GeneratorState, GeneratorState)> {    
    if from.1.alive() {
        if !to.1.alive() {
            return Err((from, to));
        }

        let coeff = to.0 - from.0 - 1;
        if coeff < 0 {
            // TODO: Figure out if this should be branching or not ?
            return Err((from, to));
            // panic!("We encountered a negative coefficient, such a differential should never have been suggested ? And is unfixable ?");
        }
    
    
        match to.1.0 {
            Some(to_t) => match from.1.0 {
                Some(from_t) => {
                    // Delta represents how much F2 generators are actually hit
                    let delta = to_t - coeff; 
                    if delta > from_t {
                        // TODO: 
                        Err((from, to))
                    } else {
                        let new_from_af =  from.0 - delta;
                        let new_from_t = from_t - delta;
                        
                        let from = (new_from_af, Torsion::new(new_from_t));
                        let to = (to.0, Torsion::new(coeff));
                        Ok((from, to))
                    }
                },
                None => {
                    let from = (from.0 - to_t + coeff, Torsion::default());
                    let to = (to.0, Torsion::new(coeff));
                    Ok((from, to))
                },
            },
            None => match from.1.0 {
                Some(_) => {
                    // TODO: 
                    Err((from, to))
                },
                None => {
                    let from = (from.0, Torsion::zero());
                    let to = (to.0, Torsion::new(coeff));
                    Ok((from, to))
                },
            },
        }
    } else {
        Ok((from, to))
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

        loop {
            if id + 1 == l.len() {
                return l[id].1;
            }
            if l[id+1].0 > page {
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

    pub fn try_element_final(&self, elt: usize) -> Option<GeneratorState> {
        let a = &self.generators[elt];
        let b = a.as_deref()?;
        Some(b.last().unwrap().1.clone())
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
}