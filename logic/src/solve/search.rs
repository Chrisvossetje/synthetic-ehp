//! Speculative search with shared cancellation and bounded companion searches.

use crate::solve::{
    automated::{MAX_DEPTH, PARALLEL_DEPTH},
    proof::{ProofChoice, SearchProof},
};
use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchResult {
    Contradiction(SearchProof),
    Open(SearchProof),
    Cancelled(SearchProof),
}
impl BranchResult {
    pub fn assuming(self, depth: i32, choice: ProofChoice) -> Self {
        match self {
            Self::Contradiction(p) => Self::Contradiction(p.assuming(depth, choice)),
            Self::Open(p) => Self::Open(p.assuming(depth, choice)),
            Self::Cancelled(p) => Self::Cancelled(p.assuming(depth, choice)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoiceResult {
    Chosen,
    Contradiction(SearchProof),
    Open,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeculativeBranchOutcome {
    ChooseLeft(SearchProof),
    ChooseRight(SearchProof),
    Contradiction(SearchProof),
    BothOpen,
    Cancelled,
}

#[derive(Clone)]
pub struct GetOut {
    flags: [Option<Arc<AtomicI32>>; PARALLEL_DEPTH as usize],
    /// Absolute traversal cell number, inherited by cloned worlds. Resolving
    /// more choices within the same cell does not advance this cursor.
    pub position: usize,
    max_depth: i32,
    stop_after: Option<usize>,
}
impl GetOut {
    pub fn max_depth(&self) -> i32 {
        self.max_depth
    }
    pub fn past_frontier(&self) -> bool {
        self.stop_after.is_some_and(|end| self.position > end)
    }
    fn companion(&self, proof: &SearchProof) -> Self {
        Self {
            flags: [const { None }; PARALLEL_DEPTH as usize],
            position: self.position,
            max_depth: proof.max_depth,
            stop_after: Some(proof.frontier),
        }
    }
}

pub fn empty_getout() -> GetOut {
    GetOut {
        flags: [const { None }; PARALLEL_DEPTH as usize],
        position: 0,
        max_depth: MAX_DEPTH,
        stop_after: None,
    }
}
pub fn create_getout(getout: &GetOut, choices: i32, depth: i32) -> GetOut {
    let mut g = getout.clone();
    if depth < PARALLEL_DEPTH {
        g.flags[depth as usize] = Some(Arc::new(AtomicI32::new(choices - 1)));
    }
    g
}
pub fn signal_parent_getout(getout: &mut GetOut, depth: i32) {
    if depth <= PARALLEL_DEPTH
        && depth != 0
        && let Some(flag) = &getout.flags[(depth - 1) as usize]
    {
        flag.fetch_sub(1, Ordering::Relaxed);
    }
}
pub fn check_getout(getout: &GetOut) -> bool {
    getout
        .flags
        .iter()
        .flatten()
        .any(|g| g.load(Ordering::Relaxed) <= 0)
}

pub fn resolve_speculative_branch_results(
    left: BranchResult,
    right: BranchResult,
) -> SpeculativeBranchOutcome {
    use BranchResult::*;
    match (left, right) {
        (Contradiction(left), Contradiction(right)) => {
            SpeculativeBranchOutcome::Contradiction(SearchProof::cases(
                left.depth() - 1,
                left.frontier.max(right.frontier),
                vec![left, right],
            ))
        }
        (Contradiction(p), Open(other) | Cancelled(other)) => {
            SpeculativeBranchOutcome::ChooseRight(p.with_alternative(other))
        }
        (Open(other) | Cancelled(other), Contradiction(p)) => {
            SpeculativeBranchOutcome::ChooseLeft(p.with_alternative(other))
        }
        (Cancelled(_), _) | (_, Cancelled(_)) => SpeculativeBranchOutcome::Cancelled,
        _ => SpeculativeBranchOutcome::BothOpen,
    }
}

pub fn branch_on_speculative_worlds<L, R>(depth: i32, left: L, right: R) -> SpeculativeBranchOutcome
where
    L: FnOnce() -> BranchResult + Send,
    R: FnOnce() -> BranchResult + Send,
{
    if depth < PARALLEL_DEPTH {
        let (left, right) = rayon::join(left, right);
        resolve_speculative_branch_results(left, right)
    } else {
        let left = left();
        // A single refutation suffices here; the top-level companion is run
        // separately with fresh cancellation flags and explicit bounds.
        match left {
            BranchResult::Contradiction(p) => SpeculativeBranchOutcome::ChooseRight(p),
            BranchResult::Cancelled(_) => SpeculativeBranchOutcome::Cancelled,
            left => resolve_speculative_branch_results(left, right()),
        }
    }
}

/// Companion searches are performed only for actual, depth-zero discoveries.
/// Nested discoveries retain their full proofs without spawning companions.
/// The companion assumes the winning literal; it never uses its just-proven
/// verdict as a premise, and cannot reuse cancellation from the rejected world.
pub fn discover_choice<R>(
    depth: i32,
    getout: &GetOut,
    choices: [ProofChoice; 2],
    run: R,
) -> SpeculativeBranchOutcome
where
    R: Fn(usize, GetOut) -> BranchResult + Sync,
{
    let g = create_getout(getout, 2, depth);
    let outcome = branch_on_speculative_worlds(
        depth,
        || run(0, g.clone()).assuming(depth + 1, choices[0].clone()),
        || run(1, g.clone()).assuming(depth + 1, choices[1].clone()),
    );
    if depth != 0 {
        return outcome;
    }
    let (winner, proof) = match outcome {
        SpeculativeBranchOutcome::ChooseLeft(p) => (0, p),
        SpeculativeBranchOutcome::ChooseRight(p) => (1, p),
        other => return other,
    };
    let companion =
        run(winner, getout.companion(&proof)).assuming(depth + 1, choices[winner].clone());
    let proof = match companion {
        BranchResult::Contradiction(other) => {
            return SpeculativeBranchOutcome::Contradiction(SearchProof::cases(
                depth,
                getout.position,
                vec![proof, other],
            ));
        }
        BranchResult::Open(other) | BranchResult::Cancelled(other) => proof.with_comparison(other),
    };
    if winner == 0 {
        SpeculativeBranchOutcome::ChooseLeft(proof)
    } else {
        SpeculativeBranchOutcome::ChooseRight(proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::model::Diff, solve::proof::ProofTrace};
    use std::sync::atomic::AtomicUsize;

    fn choices() -> [ProofChoice; 2] {
        [
            ProofChoice::Diff(Diff { from: 0, to: 1 }, true),
            ProofChoice::Diff(Diff { from: 0, to: 1 }, false),
        ]
    }
    fn leaf(depth: i32, reason: &str) -> SearchProof {
        ProofTrace::default().contradiction(depth, reason.into())
    }

    #[test]
    fn two_refuted_alternatives_propagate_a_contradiction_instead_of_choosing_one() {
        let outcome = resolve_speculative_branch_results(
            BranchResult::Contradiction(leaf(2, "left")),
            BranchResult::Contradiction(leaf(2, "right")),
        );
        let SpeculativeBranchOutcome::Contradiction(proof) = outcome else {
            panic!("Must refute the parent")
        };
        assert_eq!(proof.depth(), 1);
        assert_eq!(proof.max_depth, 2);
    }

    #[test]
    fn companion_restarts_the_winner_with_fresh_flags_and_the_refutations_bounds() {
        for rejected in [0, 1] {
            let calls = AtomicUsize::new(0);
            let mut control = empty_getout();
            control.position = 5;
            let outcome = discover_choice(0, &control, choices(), |world, mut control| {
                calls.fetch_add(1, Ordering::Relaxed);
                let mut trace = ProofTrace::default();
                trace.at(control.position);
                if control.stop_after.is_some() {
                    assert_eq!(world, 1 - rejected);
                    assert!(!check_getout(&control));
                    assert_eq!(control.max_depth(), 4);
                    assert_eq!(control.stop_after, Some(11));
                    assert_eq!(control.position, 5);
                    trace.known(1, choices()[0].clone(), "companion fact");
                    return BranchResult::Open(trace.open(1));
                }
                if world == rejected {
                    signal_parent_getout(&mut control, 1);
                    let mut support = ProofTrace::default();
                    support.at(11);
                    trace.resolved(
                        1,
                        choices()[0].clone(),
                        &support.contradiction(4, "support".into()),
                    );
                    BranchResult::Contradiction(trace.contradiction(1, "rejected".into()))
                } else {
                    BranchResult::Cancelled(trace.cancelled(1))
                }
            });
            assert_eq!(calls.load(Ordering::Relaxed), 3);
            match outcome {
                SpeculativeBranchOutcome::ChooseLeft(p) if rejected == 1 => {
                    assert_eq!(p.max_depth, 4)
                }
                SpeculativeBranchOutcome::ChooseRight(p) if rejected == 0 => {
                    assert_eq!(p.max_depth, 4)
                }
                other => panic!("Unexpected outcome: {other:?}"),
            }
        }
    }

    #[test]
    fn nested_discoveries_do_not_spawn_companion_searches() {
        let calls = AtomicUsize::new(0);
        let outcome = discover_choice(1, &empty_getout(), choices(), |world, control| {
            calls.fetch_add(1, Ordering::Relaxed);
            assert!(control.stop_after.is_none());
            if world == 0 {
                BranchResult::Contradiction(leaf(2, "bad"))
            } else {
                BranchResult::Open(ProofTrace::default().open(2))
            }
        });
        assert!(matches!(outcome, SpeculativeBranchOutcome::ChooseRight(_)));
        assert_eq!(calls.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn contradiction_in_the_companion_is_propagated_without_committing_a_verdict() {
        let outcome = discover_choice(0, &empty_getout(), choices(), |world, control| {
            if world == 0 || control.stop_after.is_some() {
                BranchResult::Contradiction(leaf(1, "bad in this world"))
            } else {
                BranchResult::Cancelled(ProofTrace::default().cancelled(1))
            }
        });
        assert!(matches!(
            outcome,
            SpeculativeBranchOutcome::Contradiction(_)
        ));
    }

    #[test]
    fn open_and_cancelled_searches_never_supply_a_discovery() {
        assert_eq!(
            resolve_speculative_branch_results(
                BranchResult::Open(ProofTrace::default().open(1)),
                BranchResult::Open(ProofTrace::default().open(1))
            ),
            SpeculativeBranchOutcome::BothOpen
        );
        assert_eq!(
            resolve_speculative_branch_results(
                BranchResult::Open(ProofTrace::default().open(1)),
                BranchResult::Cancelled(ProofTrace::default().cancelled(1))
            ),
            SpeculativeBranchOutcome::Cancelled
        );
    }

    #[test]
    fn sequential_search_stops_at_a_refutation_and_keeps_its_evidence() {
        let outcome = branch_on_speculative_worlds(
            PARALLEL_DEPTH,
            || BranchResult::Contradiction(leaf(PARALLEL_DEPTH + 1, "bad")),
            || panic!("This sibling is unnecessary in the sequential search"),
        );
        assert!(matches!(outcome, SpeculativeBranchOutcome::ChooseRight(_)));
    }

    #[test]
    fn companion_frontier_allows_the_last_relevant_cell_but_not_the_next() {
        let mut trace = ProofTrace::default();
        trace.at(7);
        let proof = trace.contradiction(3, "bad".into());
        let mut control = empty_getout().companion(&proof);
        control.position = 7;
        assert!(!control.past_frontier());
        control.position += 1;
        assert!(control.past_frontier());
    }
}
