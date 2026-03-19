use std::{cell::Cell, collections::{HashMap, HashSet}, iter::Enumerate, slice::Iter};

use serde::{Deserialize, Serialize};

use crate::{MAX_STEM, data::naming::name_get_tag, domain::e1::E1, types::{Generator, Torsion}};


pub type FromTo = (usize, usize);
pub type InducedName = Vec<(i32, String)>;

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

    // Remember incoming/outgoing stuff
    pub in_diffs: HashMap<usize, Vec<usize>>,
    pub out_diffs: HashMap<usize, Vec<usize>>,
}

impl SyntheticSS {
    pub fn empty(e1: E1) -> Self {
        Self {
            model: e1,
            diffs_page: vec![vec![]; (MAX_STEM+1) as usize],
            internal_tau_page: vec![vec![]; (MAX_STEM+1) as usize],
            external_tau_page: vec![],
            proven_from_to: HashMap::default(),
            disproven_from_to: HashMap::default(),
            temp_fakes: HashSet::default(),
            in_diffs: HashMap::default(),
            out_diffs: HashMap::default(),
        }
    }

    pub fn disprove_from_to(&mut self, from: usize, to: usize, proof: Option<String>) {
        self.disproven_from_to.insert((from, to), proof);
    }

    pub fn add_diff(&mut self, from: usize, to: usize, proof: Option<String>) {
        let d_y = self.model.y(from) - self.model.y(to);

        if !self.proven_from_to.contains_key(&(from, to)) {
            self.diffs_page[d_y as usize].push(Diff { from, to });
            self.in_diffs.entry(to).or_insert(vec![]).push(from);
            self.out_diffs.entry(from).or_insert(vec![]).push(to);
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

    pub fn get_name_at_sphere(&self, elt: usize, sphere: i32) -> &str {
        let l = &self.model.get(elt).induced_name;
        let mut id = 0; 

        loop {
            if id + 1 == l.len() {
                return &l[id].1;
            }
            if l[id+1].0 > sphere {
                return &l[id].1;
            } 
            id += 1;
            if id == 1 {

            }
        }
    }

    pub fn get_names(&self, from: usize, to: usize) -> (String, String) {
        (self.model.name(from).to_string(), self.model.name(to).to_string())
    }
}



// TODO : REMOVE!
// pub fn map_between_generators(from_g: GeneratorState, to_g: GeneratorState) -> Result<(GeneratorState, GeneratorState), (GeneratorState, GeneratorState)> {    
//     if from_g.1.alive() {
//         if !to_g.1.alive() {
//             return Err((from_g, to_g));
//         }

//         let coeff = to_g.0 - from_g.0 - 1;
//         if coeff < 0 {
//             // TODO: Figure out if this should be branching or not ?
//             return Err((from_g, to_g));
//             // panic!("We encountered a negative coefficient, such a differential should never have been suggested ? And is unfixable ?");
//         }
    
    
//         match to_g.1.0 {
//             Some(to_t) => match from_g.1.0 {
//                 Some(from_t) => {
//                     // Delta represents how much F2 generators are actually hit
//                     let delta = to_t - coeff; 
//                     if delta > from_t {
//                         // TODO: 
//                         Err((from_g, to_g))
//                     } else {
//                         let new_from_af =  from_g.0 - delta;
//                         let new_from_t = from_t - delta;
                        
//                         let from = (new_from_af, Torsion::new(new_from_t));
//                         let to = (to_g.0, Torsion::new(coeff));
//                         Ok((from, to))
//                     }
//                 },
//                 None => {
//                     let from = (from_g.0 - to_t + coeff, Torsion::default());
//                     let to = (to_g.0, Torsion::new(coeff));
//                     Ok((from, to))
//                 },
//             },
//             None => match from_g.1.0 {
//                 Some(_) => {
//                     // TODO: 
//                     Err((from_g, to_g))
//                 },
//                 None => {
//                     let from = (from_g.0, Torsion::zero());
//                     let to = (to_g.0, Torsion::new(coeff));
//                     Ok((from, to))
//                 },
//             },
//         }
//     } else {
//         Ok((from_g, to_g))
//     }
// }

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

