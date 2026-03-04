use serde::{Deserialize, Serialize};

use crate::types::Torsion;



pub struct Issue {
    issue: Issues,
    still_valid: bool,
}

pub enum Issues {
    
}

// We should distinguish between fixable and unfixable issues.
// Aka, Issues and Errors ?


#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SyntheticE1PageIssue {
    stem: i32,
    af: i32,

    expected: Vec<Torsion>,
    observed: Vec<Torsion>,
}

// Synthetically these groups do not converge to the same thing
// Stem Issue
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConvergenceIssue {
    bot_trunc: i32,
    top_trunc: i32,
    stem: i32,

    expected: Vec<Torsion>,
    observed: Vec<Torsion>,
}


// Mapping lower torision to higher torsion
// Generator Issue
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct InvalidTorsionIssue {
    from: usize,

    // Only to can be the problem, as it must be lower by some page
    to: usize,
    to_needed: Torsion,
}
