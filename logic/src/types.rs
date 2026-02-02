use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Category {
    Synthetic,
    Algebraic,
    Classical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generator {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub adams_filtration: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub torsion: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alg_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hom_name: Option<String>,

    pub induced_name: String,
}

impl Generator {
    pub fn new(name: String, x: i32, y: i32, adams_filtration: i32) -> Generator {
        Generator {
            name: name.clone(),
            x,
            y,
            adams_filtration,
            torsion: None,
            alg_name: None,
            hom_name: None,
            induced_name: name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Differential {
    pub from: String,
    pub to: String,
    pub coeff: i32,
    pub d: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>,
}

impl PartialEq for Differential {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.coeff == other.coeff && self.d == other.d && self.proof == other.proof
    }
}

impl Eq for Differential { }

impl PartialOrd for Differential {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return self.d.partial_cmp(&other.d);
    }
}

impl Ord for Differential {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.d == other.d {
            return other.coeff.cmp(&self.coeff);
        } else {
            return self.d.cmp(&other.d);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Multiplication {
    pub from: String,
    pub to: String,
    pub internal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticEHP {
    pub generators: Vec<Generator>,
    pub differentials: Vec<Differential>,
    pub multiplications: Vec<Multiplication>,
    pub find_map: HashMap<String, usize>,
}


impl SyntheticEHP {
    /// Build the find_map from generators
    pub fn build_find_map(&mut self) {
        self.find_map = self.generators
            .iter()
            .enumerate()
            .map(|(i, g)| (g.name.clone(), i))
            .collect();
    }

    /// Insert a differential into the sorted differentials vector
    /// Maintains sort order by d value (differential page number)
    pub fn insert_diff(&mut self, diff: Differential) {
        let pos = self.differentials
            .binary_search(&diff)
            .unwrap_or_else(|e| e);

        self.differentials.insert(pos, diff);
    }

    /// Helper function to find a generator by name (O(1) lookup)
    pub fn find(&self, name: &str) -> Option<&Generator> {
        self.find_map.get(name).map(|&i| &self.generators[i])
    }

    /// Helper function to find a mutable generator by name (O(1) lookup)
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Generator> {
        self.find_map.get(name).map(|&i| &mut self.generators[i])
    }
}