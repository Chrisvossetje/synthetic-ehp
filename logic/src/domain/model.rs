use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    MAX_STEM,
    domain::e1::E1,
    types::{Kind, Torsion},
};

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
    pub generators: Vec<Torsion>,
    pub induced_name: Option<Vec<Vec<(i32, String)>>>,

    // This should be indexed by page ??
    // Or should it be indexed by Gens
    // Length of this should equal max_stem + 1
    pub diffs_page: Vec<Vec<Diff>>,
    pub internal_tau_page: Vec<Vec<IntTauMult>>,

    // This happens at the "final" page
    // Indexed by the y coordinate of "from"
    // 2nd index is for the y difference
    // Third is by the AF thing (Meaning, the "better" the element fits onto the other the earlier it should be applied)
    pub external_tau_page: Vec<Vec<Vec<Vec<ExtTauMult>>>>,

    pub from_to: HashMap<FromTo, (Kind, Option<String>)>,

    // Remember incoming/outgoing stuff
    pub in_diffs: Vec<Vec<usize>>,
    pub out_diffs: Vec<Vec<usize>>,

    pub out_taus: Vec<Vec<usize>>,
}

impl SyntheticSS {
    pub fn empty(e1: E1) -> Self {
        let len = e1.gens().len();
        Self {
            generators: vec![ Torsion::default(); len],
            induced_name: None,
            diffs_page: vec![vec![]; (MAX_STEM + 1) as usize],
            internal_tau_page: vec![vec![]; (MAX_STEM + 1) as usize],
            external_tau_page: vec![
                vec![
                    vec![vec![]; (MAX_STEM + 1) as usize];
                    (MAX_STEM + 1) as usize
                ];
                (MAX_STEM + 1) as usize
            ],
            from_to: HashMap::default(),
            in_diffs: vec![vec![]; len],
            out_diffs: vec![vec![]; len],
            out_taus: vec![vec![]; len],
        }
    }

    pub fn add_diff(&mut self, model: &E1, from: usize, to: usize, proof: Option<String>, kind: Kind) {
        let d_y = model.y(from) - model.y(to);

        if !self.from_to.contains_key(&(from, to)) {
            self.from_to.insert((from, to), (kind, proof));
            match kind {
                Kind::Real | Kind::Algebraic => {
                    self.diffs_page[d_y as usize].push(Diff { from, to });
                    self.in_diffs[to].push(from);
                    self.out_diffs[from].push(to);
                },
                _ => {}
            }
        }
    }

    pub fn add_int_tau(
        &mut self,
        from: usize,
        to: usize,
        page: i32,
        proof: Option<String>,
        kind: Kind,
    ) {
        if !self.from_to.contains_key(&(from, to)) {
            self.from_to.insert((from, to), (kind, proof));
            match kind {
                Kind::Real => {
                    self.internal_tau_page[page as usize].push(IntTauMult { from, to });
                }
                _ => {}
            }
        }
    }

    pub fn add_ext_tau(
        &mut self,
        model: &E1,
        from: usize,
        to: usize,
        af: i32,
        proof: Option<String>,
        kind: Kind,
    ) {
        if !self.from_to.contains_key(&(from, to)) {
            self.from_to.insert((from, to), (kind, proof));
            match kind {
                Kind::Real => {
                    let y_from = model.y(from);
                    let y_to = model.y(to);
                    self.external_tau_page[y_from as usize][af as usize][(y_from - y_to) as usize]
                        .push(ExtTauMult { from, to, af });
                    self.out_taus[from].push(to);
                }
                _ => {}
            }
        }
    }

    pub fn add_diff_name(
        &mut self,
        model: &E1,
        from: String,
        to: String,
        proof: Option<String>,
        kind: Kind,
    ) -> Result<(), ()> {
        let from = model.try_index(&from).ok_or(())?;
        let to = model.try_index(&to).ok_or(())?;
        self.add_diff(model, from, to, proof, kind);
        Ok(())
    }

    pub fn add_int_tau_name(
        &mut self,
        model: &E1,
        from: String,
        to: String,
        page: i32,
        proof: Option<String>,
        kind: Kind,
    ) -> Result<(), ()> {
        let from = model.try_index(&from).ok_or(())?;
        let to = model.try_index(&to).ok_or(())?;
        self.add_int_tau(from, to, page, proof, kind);
        Ok(())
    }

    pub fn add_ext_tau_name(
        &mut self,
        model: &E1,
        from: String,
        to: String,
        af: i32,
        proof: Option<String>,
        kind: Kind,
    ) -> Result<(), ()> {
        let from = model.try_index(&from).ok_or(())?;
        let to = model.try_index(&to).ok_or(())?;
        self.add_ext_tau(model, from, to, af, proof, kind);
        Ok(())
    }

    pub fn set_generator(&mut self, model: &E1, name: &String, torsion: Torsion) -> Result<(), ()> {
        let id = model.try_index(name).ok_or(())?;
        self.generators[id] = torsion;
        Ok(())
    }


    pub fn get_name_at_sphere<'a>(&'a self, model: &'a E1, elt: usize, sphere: i32) -> &'a str {
        let l: &Vec<(i32, String)> = if let Some(v) = &self.induced_name
            && !v[elt].is_empty()
        {
            &v[elt]
        } else {
            &model.get(elt).induced_name
        };

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

    pub fn push_induced_name(&mut self, model: &E1, elt: usize, sphere: i32, new_name: String) {
        let len = model.gens().len();
        if self.induced_name.is_none() {
            self.induced_name = Some(vec![vec![]; len]);
        }
        let map = self.induced_name.as_mut().unwrap();
        if map[elt].is_empty() {
            map[elt] = model.get(elt).induced_name.clone();
        }
        map[elt].push((sphere, new_name));
    }
}
