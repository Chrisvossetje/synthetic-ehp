use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{MAX_STEM, data::naming::name_get_tag, domain::e1::E1, types::{Kind, Torsion}};

pub type FromTo = (usize, usize);

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
    pub to: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExtTauMult {
    pub from: usize,
    pub to: usize,
    pub af: i32,
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

    // This happens at the "final" page
    // Indexed by the y coordinate of "from"
    // 2nd index is by difference in y coordinates
    pub external_tau_page: Vec<Vec<Vec<ExtTauMult>>>,

    // Index on the two generors, then a potential proof / disproof ?
    pub proven_from_to: HashMap<FromTo, Option<String>>,
    pub disproven_from_to: HashMap<FromTo, Option<String>>,

    // Remember incoming/outgoing stuff
    pub in_diffs: Vec<Vec<usize>>,
    pub out_diffs: Vec<Vec<usize>>,

    pub out_taus: Vec<Vec<usize>>,
}

impl SyntheticSS {
    pub fn empty(e1: E1) -> Self {
        let len = e1.gens().len();
        Self {
            model: e1,
            diffs_page: vec![vec![]; (MAX_STEM + 1) as usize],
            internal_tau_page: vec![vec![]; (MAX_STEM + 1) as usize],
            external_tau_page: vec![vec![vec![]; (MAX_STEM + 1) as usize]; (MAX_STEM + 1) as usize],
            proven_from_to: HashMap::default(),
            disproven_from_to: HashMap::default(),
            in_diffs: vec![vec![]; len],
            out_diffs: vec![vec![]; len],
            out_taus: vec![vec![]; len],
        }
    }

    pub fn add_diff(&mut self, from: usize, to: usize, proof: Option<String>, kind: Kind,) {
        let d_y = self.model.y(from) - self.model.y(to);
        
        if !self.proven_from_to.contains_key(&(from, to)) && !self.disproven_from_to.contains_key(&(from, to)) {
            match kind {
                Kind::Real => {
                    self.diffs_page[d_y as usize].push(Diff { from, to });
                    self.in_diffs[to].push(from);
                    self.out_diffs[from].push(to);
                    self.proven_from_to.insert((from, to), proof);
                },
                _ => {
                    self.disproven_from_to.insert((from, to), proof);
                },
            }
        }
    }

    pub fn add_int_tau(&mut self, from: usize, to: usize, page: i32, proof: Option<String>, kind: Kind,) {
        if !self.proven_from_to.contains_key(&(from, to)) && !self.disproven_from_to.contains_key(&(from, to)) {
            match kind {
                Kind::Real => {
                    self.internal_tau_page[page as usize].push(IntTauMult { from, to });
                    self.proven_from_to.insert((from, to), proof);
                },
                _ => {
                    self.disproven_from_to.insert((from, to), proof); 
                },
            }
        }
    }

    pub fn add_ext_tau(&mut self, from: usize, to: usize, af: i32, proof: Option<String>, kind: Kind,) {
        if !self.proven_from_to.contains_key(&(from, to)) && !self.disproven_from_to.contains_key(&(from, to)) {
            match kind {
                Kind::Real => {
                    let y_from = self.model.y(from);
                    let y_diff = self.model.y(from) - self.model.y(to);
                    self.external_tau_page[y_from as usize][y_diff as usize].push(ExtTauMult { from, to, af });
                    self.out_taus[from].push(to);
                    self.proven_from_to.insert((from, to), proof);
                },
                _ => {
                    self.disproven_from_to.insert((from, to), proof);
                    
                },
            }
        }
    }

    pub fn add_diff_name(
        &mut self,
        from: String,
        to: String,
        proof: Option<String>,
        kind: Kind,
    ) -> Result<(), ()> {
        let from = self.model.try_index(&from).ok_or(())?;
        let to = self.model.try_index(&to).ok_or(())?;
        self.add_diff(from, to, proof, kind);
        Ok(())
    }

    pub fn add_int_tau_name(
        &mut self,
        from: String,
        to: String,
        page: i32,
        proof: Option<String>,
        kind: Kind,
    ) -> Result<(), ()> {
        let from = self.model.try_index(&from).ok_or(())?;
        let to = self.model.try_index(&to).ok_or(())?;
        self.add_int_tau(from, to, page, proof, kind);
        Ok(())
    }

    pub fn add_ext_tau_name(
        &mut self,
        from: String,
        to: String,
        af: i32,
        proof: Option<String>,
        kind: Kind,
    ) -> Result<(), ()> {
        let from = self.model.try_index(&from).ok_or(())?;
        let to = self.model.try_index(&to).ok_or(())?;
        self.add_ext_tau(from, to, af, proof, kind);
        Ok(())
    }

    pub fn set_generator(&mut self, name: &String, torsion: Torsion) -> Result<(), ()> {
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
            if l[id + 1].0 > sphere {
                return &l[id].1;
            }
            id += 1;
        }
    }

    pub fn get_names(&self, from: usize, to: usize) -> (String, String) {
        (
            self.model.name(from).to_string(),
            self.model.name(to).to_string(),
        )
    }
}
