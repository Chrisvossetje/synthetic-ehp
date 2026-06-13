//! The E1 page: the fixed list of [`Generator`]s together with lookup indices
//! by name, by stem, by (stem, AF), and by (stem, y). These never change during
//! a solve — only the spectral-sequence facts layered on top of them do.

use std::{collections::HashMap, iter::Enumerate, slice::Iter};

use serde::{Deserialize, Serialize};

use crate::{
    data::{r#static::EMPTY_LIST_USIZE, naming::name_get_tag},
    types::{Generator, Torsion},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct E1 {
    // Actual data
    generators: Vec<Generator>,

    // LUTS
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

    pub fn af(&self, elt: usize) -> i32 {
        self.generators[elt].af
    }

    #[allow(dead_code)]
    pub fn torsion(&self, elt: usize) -> Torsion {
        self.generators[elt].torsion
    }

    pub fn get(&self, id: usize) -> &Generator {
        &self.generators[id]
    }

    pub fn get_mut(&mut self, id: usize) -> &mut Generator {
        &mut self.generators[id]
    }

    pub fn get_index(&self, name: &str) -> usize {
        *self.index.get(name).unwrap()
    }

    pub fn try_index(&self, name: &str) -> Option<usize> {
        self.index.get(name).copied()
    }

    pub fn get_name(&self, name: &str) -> &Generator {
        self.get(self.get_index(name))
    }

    #[allow(unused)]
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
        self.stem.get(&stem).unwrap()
    }

    pub fn gens_id_in_stem_af(&self, stem: i32, af: i32) -> &Vec<usize> {
        self.stem_af.get(&(stem, af)).unwrap()
    }

    pub fn gens_id_in_stem_y(&self, stem: i32, y: i32) -> &Vec<usize> {
        self.stem_y.get(&(stem, y)).unwrap_or(&EMPTY_LIST_USIZE)
    }

    pub fn push(&mut self, g: Generator) {
        self.generators.push(g);
    }

    pub fn try_name_tag<'a>(&self, name: &'a str) -> Result<&'a str, ()> {
        self.try_index(name).ok_or(())?;
        Ok(name_get_tag(name))
    }

    pub fn get_names(&self, from: usize, to: usize) -> (String, String) {
        (
            self.name(from).to_string(),
            self.name(to).to_string(),
        )
    }
}
