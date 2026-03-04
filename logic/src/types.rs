use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Torsion(pub Option<i32>);

impl Default for Torsion {
    fn default() -> Self {
        Self(None)
    }
}

impl Torsion {
    pub fn new(torsion: i32) -> Self {
        Self(Some(torsion))
    }

    pub fn zero() -> Self {
        Self(Some(0))
    }

    pub fn alive(&self) -> bool {
        self.0 != Some(0)
    }
}


// a <= b iff a can map to b
impl Ord for Torsion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.0 {
            Some(a) => match other.0 {
                Some(b) => b.cmp(&a),
                None => std::cmp::Ordering::Greater,
            },
            None => match other.0 {
                Some(_) => std::cmp::Ordering::Less,
                None => std::cmp::Ordering::Equal,
            },
        }
    }
}

impl PartialOrd for Torsion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Category {
    Synthetic,
    Algebraic,
    Geometric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Real,
    Fake,
    Unknown,
}


#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Generator {
    pub name: String,
    pub stem: i32,
    pub y: i32,
    pub af: i32,

    // As this is somewhat variable, do we want this information here ?
    // Yes as this generator should represent E1
    pub torsion: Torsion,

    // This is purely algebraic!
    pub born: i32,
    pub dies: Option<i32>,

    pub kind: Kind,
}

impl Generator {
    pub fn new(name: String, stem: i32, y: i32, af: i32, born: i32, dies: Option<i32>, kind: Kind) -> Generator {
        Generator {
            name: name.clone(),
            stem,
            y,
            af,
            torsion: Torsion::default(),
            born,
            dies,
            kind
        }
    }

    // pub fn get_induced_name(&self, sphere: i32) -> &str {
    //     // HERE I ASSUME THAT INDUCED NAME IS REVERSE SORTED!
    //     for (id, name) in &self.induced_name {
    //         if sphere >= *id {
    //             return &name;
    //         }
    //     }
    //     panic!("No element found?")
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Differential {
    pub from: String,
    pub to: String,
    pub coeff: i32,
    pub d: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>,

    pub kind: Kind,

    // #[serde(skip_serializing)]
    // pub diff_sources: Vec<Differential>,
    // #[serde(skip_serializing)]
    // pub diff_targets: Vec<Differential>,
}

impl PartialEq for Differential {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.coeff == other.coeff && self.d == other.d && self.proof == other.proof
    }
}

impl Eq for Differential { }

impl PartialOrd for Differential {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.d == other.d {
            return other.coeff.partial_cmp(&self.coeff);
        } else {
            return self.d.partial_cmp(&other.d);
        }
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
pub struct TauMult {
    pub from: String,
    pub to: String,
    pub kind: Kind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Multiplication {
    pub from: String,
    pub to: String,
    pub internal: bool,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OldSyntheticSS {
    pub generators: Vec<Generator>,
    pub differentials: Vec<Differential>,
    pub multiplications: Vec<Multiplication>,
    pub tau_mults: Vec<TauMult>,
    pub find_map: HashMap<String, usize>,
}


impl OldSyntheticSS {
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