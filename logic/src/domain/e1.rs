use std::{collections::HashMap, iter::Enumerate, slice::Iter};

use serde::{Deserialize, Serialize};

use crate::{
    data::compare::EMPTY_LIST_USIZE,
    types::{Generator, Torsion},
};

// This should represent the E1 page ? Is this necessary ?
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct E1 {
    generators: Vec<Generator>,
    index: HashMap<String, usize>,
    stem: HashMap<i32, Vec<usize>>,
    stem_af: HashMap<(i32, i32), Vec<usize>>,
    stem_y: HashMap<(i32, i32), Vec<usize>>,
}

impl E1 {
    pub fn new(generators: Vec<Generator>) -> Self {
        let mut index = HashMap::new();
        let mut stem = HashMap::new();
        let mut stem_af = HashMap::new();
        let mut stem_y = HashMap::new();

        for (i, g) in generators.iter().enumerate() {
            index.insert(g.name.clone(), i);
            stem.entry(g.stem).or_insert(vec![]).push(i);
            stem_af.entry((g.stem, g.af)).or_insert(vec![]).push(i);
            stem_y.entry((g.stem, g.y)).or_insert(vec![]).push(i);
        }

        Self {
            generators,
            index,
            stem,
            stem_af,
            stem_y,
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

    pub fn get_mut(&mut self, id: usize) -> &mut Generator {
        &mut self.generators[id]
    }

    pub fn get_index(&self, name: &String) -> usize {
        *self.index.get(name).unwrap()
    }

    pub fn try_index(&self, name: &str) -> Option<usize> {
        self.index.get(name).map(|x| *x)
    }

    pub fn get_name(&self, name: &String) -> &Generator {
        self.get(self.get_index(name))
    }

    pub fn get_name_mut(&mut self, name: &String) -> &mut Generator {
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

    pub fn gens_id_in_stem_y(&self, stem: i32, y: i32) -> &Vec<usize> {
        &self.stem_y.get(&(stem, y)).unwrap_or(&EMPTY_LIST_USIZE)
    }
}
