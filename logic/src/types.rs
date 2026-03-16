use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
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

}

impl Generator {
    pub fn new(name: String, stem: i32, y: i32, af: i32, born: i32, dies: Option<i32>) -> Generator {
        Generator {
            name: name.clone(),
            stem,
            y,
            af,
            torsion: Torsion::default(),
            born,
            dies,
        }
    }
}

