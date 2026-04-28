use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use crate::solve::automated::PARALLEL_DEPTH;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchResult {
    Contradiction(String),
    Open,
    Cancelled,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceResult {
    Chosen,
    Open,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeculativeBranchOutcome {
    ChooseLeft(String),
    ChooseRight(String),
    BothOpen,
    Cancelled,
}


pub fn create_getout(
    getout: &[Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize],
    depth: i32
) -> [Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize] {
    let mut g = getout.clone();
    if depth < PARALLEL_DEPTH {
        g[depth as usize] = Some(Arc::new(AtomicBool::new(false)));
    }
    g
}

pub fn signal_parent_getout(
    getout: &mut [Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize],
    depth: i32,
) {
    if depth <= PARALLEL_DEPTH
        && depth != 0
        && let Some(flag) = &mut getout[(depth - 1) as usize]
    {
        flag.store(true, Ordering::Relaxed);
    }
}

pub fn check_getout(
    getout: &[Option<Arc<AtomicBool>>; PARALLEL_DEPTH as usize],
) -> bool {
    for g in getout {
        if let Some(g) = g {
            if g.load(Ordering::Relaxed) {
                return true;
            }
        }
    }
    return false;
}

pub fn resolve_speculative_branch_results(
    left_res: BranchResult,
    right_res: BranchResult,
) -> SpeculativeBranchOutcome {
    if let BranchResult::Contradiction(e) = left_res {
        SpeculativeBranchOutcome::ChooseRight(e)
    } else if let BranchResult::Contradiction(e) = right_res {
        SpeculativeBranchOutcome::ChooseLeft(e)
    } else if left_res == BranchResult::Cancelled
        && right_res == BranchResult::Cancelled
    {
        SpeculativeBranchOutcome::Cancelled
    } else {
        SpeculativeBranchOutcome::BothOpen
    }
}

pub fn branch_on_speculative_worlds<L, R>(
    depth: i32,
    left: L,
    right: R,
) -> SpeculativeBranchOutcome
where
    L: FnOnce() -> BranchResult + Send,
    R: FnOnce() -> BranchResult + Send,
{
    if depth < PARALLEL_DEPTH {
        let (left_res, right_res) = rayon::join(left, right);
        resolve_speculative_branch_results(left_res, right_res)
    } else {
        let left_res = left();

        if let BranchResult::Contradiction(e) = left_res {
            return SpeculativeBranchOutcome::ChooseRight(e);
        }
        if left_res == BranchResult::Cancelled {
            return SpeculativeBranchOutcome::Cancelled;
        }

        let right_res = right();

        match right_res {
            BranchResult::Contradiction(e) => SpeculativeBranchOutcome::ChooseLeft(e),
            BranchResult::Open => SpeculativeBranchOutcome::BothOpen,
            BranchResult::Cancelled => SpeculativeBranchOutcome::Cancelled,
        }
    }
}
