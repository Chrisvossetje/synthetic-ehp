//! Full discovery trees and reduced explanations from paired speculative worlds.
//!
//! Only conclusions established on BOTH mainlines can become common lemmas.
//! Nested guesses never count as facts. Only the refutation is exported, with
//! other differentials from its root source hidden. The full tree retains both
//! derivations and their earlier premises; reduction never changes solver data.

use crate::{
    domain::{
        e1::E1,
        model::{Diff, ExtTauMult},
    },
    types::{Kind, Torsion},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    sync::Arc,
};

const FULL_PROOF_TREE_HEADER: &str =
    "\nFull proof tree (references share subproofs; steps use earlier premises in their branch):\n";

/// Saved discovery proofs contain the compact chain followed by the full tree.
/// Export just the chain, preserving legacy proof strings as they are.
pub fn compact_proof_text(proof: &str) -> &str {
    proof
        .split_once(FULL_PROOF_TREE_HEADER)
        .map_or(proof, |(compact, _)| compact)
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum ProofChoice {
    Diff(Diff, bool),
    Tau(ExtTauMult, bool),
    FilteredDiff(Diff, Kind),
    FilteredTau(ExtTauMult, Kind),
    InducedName {
        from: usize,
        to: usize,
        sphere: i32,
    },
    E1 {
        stem: i32,
        assignments: Vec<(usize, Torsion)>,
    },
}

impl ProofChoice {
    fn differential_source(&self) -> Option<usize> {
        match self {
            Self::Diff(d, _) | Self::FilteredDiff(d, _) => Some(d.from),
            _ => None,
        }
    }
    fn opposite(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Diff(a, x), Self::Diff(b, y)) => a == b && x != y,
            (Self::Tau(a, x), Self::Tau(b, y)) => a == b && x != y,
            (
                Self::InducedName {
                    from: a,
                    to: x,
                    sphere: s,
                },
                Self::InducedName {
                    from: b,
                    to: y,
                    sphere: t,
                },
            ) => a == b && s == t && x != y,
            _ => false,
        }
    }
    fn describe(&self, model: &E1) -> String {
        match self {
            Self::Diff(d, exists) => format!(
                "diff {} -> {} {}",
                model.name(d.from),
                model.name(d.to),
                if *exists { "exists" } else { "absent" }
            ),
            Self::Tau(d, exists) => format!(
                "tau {} -> {} (af {}) {}",
                model.name(d.from),
                model.name(d.to),
                d.af,
                if *exists { "exists" } else { "absent" }
            ),
            Self::FilteredDiff(d, kind) => format!(
                "diff {} -> {}: {kind:?}",
                model.name(d.from),
                model.name(d.to)
            ),
            Self::FilteredTau(d, kind) => format!(
                "tau {} -> {} (af {}): {kind:?}",
                model.name(d.from),
                model.name(d.to),
                d.af
            ),
            Self::InducedName { from, to, sphere } => format!(
                "name {} = {} on S^{sphere}",
                model.name(*from),
                model.name(*to)
            ),
            Self::E1 { stem, assignments } => {
                let assignments = assignments
                    .iter()
                    .map(|(id, t)| format!("{}: {t:?}", model.name(*id)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("E1 in stem {stem} ({assignments})")
            }
        }
    }

    /// The compact chain shows asserted facts; exclusions stay in the detailed
    /// trees. Differential implications omit the repeated "diff" prefix.
    fn describe_implication(&self, model: &E1) -> Option<String> {
        match self {
            Self::Diff(d, true) => Some(format!(
                "{} -> {} exists",
                model.name(d.from),
                model.name(d.to)
            )),
            Self::Diff(_, false)
            | Self::Tau(_, false)
            | Self::FilteredDiff(_, _)
            | Self::FilteredTau(_, _) => None,
            _ => Some(self.describe(model)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Conclusion {
    choice: ProofChoice,
    rule: Arc<str>,
    proofs: Vec<SearchProof>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Ending {
    Contradiction(Arc<str>),
    Cases(Vec<SearchProof>),
    Open,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofTree {
    depth: i32,
    assumption: Option<ProofChoice>,
    conclusions: Vec<Conclusion>,
    ending: Ending,
    alternative: Option<SearchProof>,
    comparison: Option<SearchProof>,
}

/// Clones share subtrees. Bounds describe the successful refutation, excluding
/// unused speculative siblings and the bounded companion search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchProof {
    pub max_depth: i32,
    pub frontier: usize,
    tree: Arc<ProofTree>,
}

/// An owned view of the refutation alone. Shared conclusions and differentials
/// from the root's source are omitted along with their exclusive support.
/// Omitted common premises point back to the unchanged full proof. IDs in
/// `nodes` are local to this exported graph.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ReducedProofTree {
    pub root: usize,
    pub max_depth: i32,
    pub common: Vec<CommonConclusion>,
    pub nodes: Vec<ReducedProofNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FullProofLocation {
    pub node: usize,
    pub step: usize,
}

/// Only provenance is exported here. The shared fact and both original
/// derivations stay in the full tree at these one-based locations.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CommonConclusion {
    pub id: usize,
    pub left: FullProofLocation,
    pub right: FullProofLocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct OmittedPremise {
    pub original_step: usize,
    pub common: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ReducedConclusion {
    pub original_step: usize,
    pub choice: ProofChoice,
    pub rule: String,
    pub proofs: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", content = "value")]
pub enum ReducedEnding {
    Contradiction(String),
    Cases(Vec<usize>),
    Open,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ReducedProofNode {
    pub id: usize,
    pub full_tree_id: usize,
    pub depth: i32,
    pub assumption: Option<ProofChoice>,
    pub conclusions: Vec<ReducedConclusion>,
    pub omitted_premises: Vec<OmittedPremise>,
    pub ending: ReducedEnding,
}

/// Use the D/!D pair to find established common conclusions, then export only
/// the input root and its remaining support. Hide shared conclusions and other
/// differentials originating from the root assumption's source. Removing a
/// conclusion also removes support reachable only through it. Equal terminal
/// contradictions collapse across case splits; other endings remain explicit.
/// The input is unchanged; no search is rerun.
pub fn reduce_proof_tree(full: &SearchProof) -> ReducedProofTree {
    let mut ids = HashMap::new();
    let mut nodes = vec![];
    full.collect(&mut ids, &mut nodes);
    reduce_with_ids(full, &ids)
}

/// Export the actual reduced graph as JSON, rather than a formatted full tree.
pub fn export_reduced_proof_tree(
    full: &SearchProof,
    output: impl std::io::Write,
) -> serde_json::Result<()> {
    serde_json::to_writer_pretty(output, &reduce_proof_tree(full))
}

fn paired_branches(full: &SearchProof) -> Option<(&SearchProof, &SearchProof)> {
    let opposite = |a: &SearchProof, b: &SearchProof| {
        a.tree.depth == b.tree.depth
            && a.tree
                .assumption
                .as_ref()
                .zip(b.tree.assumption.as_ref())
                .is_some_and(|(a, b)| a.opposite(b))
    };
    for other in full
        .tree
        .comparison
        .iter()
        .chain(full.tree.alternative.iter())
    {
        if opposite(full, other) {
            return Some((full, other));
        }
    }
    match &full.tree.ending {
        Ending::Cases(branches) if branches.len() == 2 && opposite(&branches[0], &branches[1]) => {
            Some((&branches[0], &branches[1]))
        }
        _ => None,
    }
}

fn shared_conclusions(left: &SearchProof, right: &SearchProof) -> Vec<(usize, usize)> {
    let mut pairs: Vec<(usize, usize)> = vec![];
    for (i, c) in left.tree.conclusions.iter().enumerate() {
        if pairs
            .iter()
            .any(|&(earlier, _)| left.tree.conclusions[earlier].choice == c.choice)
        {
            continue;
        }
        if let Some(j) = right
            .tree
            .conclusions
            .iter()
            .position(|d| c.choice == d.choice)
        {
            pairs.push((i, j));
        }
    }
    pairs
}

fn reduce_with_ids(full: &SearchProof, ids: &HashMap<usize, usize>) -> ReducedProofTree {
    let pair = paired_branches(full);
    let mut shared = vec![];
    let mut common = vec![];
    if let Some((left, right)) = pair {
        for (i, j) in shared_conclusions(left, right) {
            shared.push(left.tree.conclusions[i].choice.clone());
            common.push(CommonConclusion {
                id: common.len() + 1,
                left: FullProofLocation {
                    node: ids[&left.key()],
                    step: i + 1,
                },
                right: FullProofLocation {
                    node: ids[&right.key()],
                    step: j + 1,
                },
            });
        }
    }
    let mut reducer = TreeReducer {
        full_ids: ids,
        shared,
        ignored_source: full
            .tree
            .assumption
            .as_ref()
            .and_then(ProofChoice::differential_source),
        scope_roots: pair.map(|(a, b)| [a.key(), b.key()]),
        visited: HashMap::new(),
        contradictions: HashMap::new(),
        nodes: vec![],
    };
    let root = reducer.visit(full, false);
    ReducedProofTree {
        root,
        max_depth: full.max_depth,
        common,
        nodes: reducer.nodes,
    }
}

struct TreeReducer<'a> {
    full_ids: &'a HashMap<usize, usize>,
    shared: Vec<ProofChoice>,
    ignored_source: Option<usize>,
    scope_roots: Option<[usize; 2]>,
    // Shared subproofs reached outside a paired scope must remain unfiltered.
    visited: HashMap<(usize, bool), usize>,
    contradictions: HashMap<usize, Option<Arc<str>>>,
    nodes: Vec<ReducedProofNode>,
}

impl TreeReducer<'_> {
    /// A case split can expose one common contradiction only if every terminal
    /// branch closes with that exact diagnostic. Never inspect supporting
    /// refutations or speculative siblings when deciding how a branch ends.
    fn uniform_contradiction(&mut self, full: &SearchProof) -> Option<Arc<str>> {
        if let Some(reason) = self.contradictions.get(&full.key()) {
            return reason.clone();
        }
        let reason = match &full.tree.ending {
            Ending::Contradiction(reason) => Some(reason.clone()),
            Ending::Cases(branches) if !branches.is_empty() => {
                self.uniform_contradiction(&branches[0]).filter(|reason| {
                    branches[1..]
                        .iter()
                        .all(|p| self.uniform_contradiction(p).as_ref() == Some(reason))
                })
            }
            _ => None,
        };
        self.contradictions.insert(full.key(), reason.clone());
        reason
    }

    fn visit(&mut self, full: &SearchProof, inherited_scope: bool) -> usize {
        let in_scope = inherited_scope
            || self
                .scope_roots
                .is_some_and(|roots| roots.contains(&full.key()));
        let key = (full.key(), in_scope);
        if let Some(&id) = self.visited.get(&key) {
            return id;
        }
        let id = self.nodes.len() + 1;
        self.visited.insert(key, id);
        // Reserve the ID before visiting children, retaining DAG sharing.
        self.nodes.push(ReducedProofNode {
            id,
            full_tree_id: self.full_ids[&full.key()],
            depth: full.tree.depth,
            assumption: full.tree.assumption.clone(),
            conclusions: vec![],
            omitted_premises: vec![],
            ending: ReducedEnding::Open,
        });
        let mut conclusions = vec![];
        let mut omitted_premises = vec![];
        for (i, c) in full.tree.conclusions.iter().enumerate() {
            if in_scope && let Some(j) = self.shared.iter().position(|choice| choice == &c.choice) {
                omitted_premises.push(OmittedPremise {
                    original_step: i + 1,
                    common: j + 1,
                });
                continue;
            }
            if self.ignored_source.is_some()
                && c.choice.differential_source() == self.ignored_source
            {
                continue;
            }
            conclusions.push(ReducedConclusion {
                original_step: i + 1,
                choice: c.choice.clone(),
                rule: c.rule.to_string(),
                proofs: c.proofs.iter().map(|p| self.visit(p, in_scope)).collect(),
            });
        }
        let ending = match &full.tree.ending {
            Ending::Contradiction(reason) => ReducedEnding::Contradiction(reason.to_string()),
            Ending::Cases(branches) => match self.uniform_contradiction(full) {
                Some(reason) => ReducedEnding::Contradiction(reason.to_string()),
                None => {
                    ReducedEnding::Cases(branches.iter().map(|p| self.visit(p, in_scope)).collect())
                }
            },
            Ending::Open => ReducedEnding::Open,
            Ending::Cancelled => ReducedEnding::Cancelled,
        };
        let node = &mut self.nodes[id - 1];
        node.conclusions = conclusions;
        node.omitted_premises = omitted_premises;
        node.ending = ending;
        id
    }
}

impl ReducedProofTree {
    /// A compact implication chain. Supporting refutations and routine
    /// exclusions remain in the graph and the unchanged full tree. Distinct
    /// terminal cases remain visible; equal contradictions have been folded.
    pub fn describe(&self, model: &E1) -> String {
        let mut out = String::new();
        self.describe_branch(self.root, model, "", &mut HashSet::new(), &mut out);
        out
    }

    fn describe_branch(
        &self,
        id: usize,
        model: &E1,
        indent: &str,
        seen: &mut HashSet<usize>,
        out: &mut String,
    ) {
        if !seen.insert(id) {
            writeln!(out, "{indent}-> see case R{id}").unwrap();
            return;
        }
        let node = &self.nodes[id - 1];
        if let Some(choice) = &node.assumption {
            writeln!(out, "{indent}assume {}", choice.describe(model)).unwrap();
        }
        for c in &node.conclusions {
            if let Some(choice) = c.choice.describe_implication(model) {
                writeln!(out, "{indent}-> {choice}").unwrap();
            }
        }
        match &node.ending {
            ReducedEnding::Contradiction(reason) => {
                writeln!(out, "{indent}-> contradiction: {reason}").unwrap();
            }
            ReducedEnding::Cases(branches) if branches.len() == 1 => {
                self.describe_branch(branches[0], model, indent, seen, out);
            }
            ReducedEnding::Cases(branches) => {
                for &branch in branches {
                    writeln!(out, "{indent}-> case R{branch}:").unwrap();
                    self.describe_branch(branch, model, &format!("{indent}  "), seen, out);
                }
            }
            ReducedEnding::Open => {
                writeln!(out, "{indent}-> open (search limit or unresolved choice)").unwrap();
            }
            ReducedEnding::Cancelled => {
                writeln!(out, "{indent}-> cancelled (partial search)").unwrap();
            }
        }
    }
}

impl SearchProof {
    pub fn depth(&self) -> i32 {
        self.tree.depth
    }
    pub fn assuming(mut self, depth: i32, choice: ProofChoice) -> Self {
        debug_assert_eq!(self.tree.depth, depth);
        Arc::make_mut(&mut self.tree).assumption = Some(choice);
        self
    }
    pub fn with_alternative(mut self, alternative: SearchProof) -> Self {
        Arc::make_mut(&mut self.tree).alternative = Some(alternative);
        self
    }
    pub fn with_comparison(mut self, comparison: SearchProof) -> Self {
        Arc::make_mut(&mut self.tree).comparison = Some(comparison);
        self
    }
    pub fn cases(depth: i32, frontier: usize, branches: Vec<SearchProof>) -> Self {
        let mut trace = ProofTrace::default();
        trace.at(frontier);
        trace.finish(depth, Ending::Cases(branches))
    }
    /// References name the fact's position in each root context, preserving all
    /// earlier premises. Matching never descends into conditional subproofs.
    fn common(&self) -> Vec<(usize, usize)> {
        paired_branches(self)
            .map(|(left, right)| shared_conclusions(left, right))
            .unwrap_or_default()
    }
    pub fn summary(&self) -> String {
        format!(
            "Search (max depth {}): {} conclusions; {} common lemmas",
            self.max_depth,
            self.tree.conclusions.len(),
            self.common().len()
        )
    }
    fn key(&self) -> usize {
        Arc::as_ptr(&self.tree) as usize
    }
    fn children(&self) -> Vec<&SearchProof> {
        let mut children: Vec<_> = self
            .tree
            .conclusions
            .iter()
            .flat_map(|c| &c.proofs)
            .collect();
        if let Ending::Cases(branches) = &self.tree.ending {
            children.extend(branches);
        }
        children.extend(self.tree.alternative.iter());
        children.extend(self.tree.comparison.iter());
        children
    }
    fn collect<'a>(&'a self, ids: &mut HashMap<usize, usize>, nodes: &mut Vec<&'a SearchProof>) {
        if ids.contains_key(&self.key()) {
            return;
        }
        ids.insert(self.key(), nodes.len() + 1);
        nodes.push(self);
        for child in self.children() {
            child.collect(ids, nodes);
        }
    }
    fn ending(&self) -> String {
        match &self.tree.ending {
            Ending::Contradiction(reason) => format!("contradiction: {reason}"),
            Ending::Cases(_) => "alternatives refuted by the listed cases".into(),
            Ending::Open => "open (search limit or unresolved choice)".into(),
            Ending::Cancelled => "cancelled (partial search)".into(),
        }
    }
    /// Both views are persisted in the existing string proof field. The full
    /// tree retains the original context and support of every summarized step.
    pub fn describe(&self, model: &E1) -> String {
        let mut ids = HashMap::new();
        let mut nodes = vec![];
        self.collect(&mut ids, &mut nodes);
        let mut out = reduce_with_ids(self, &ids).describe(model);
        out.push_str(FULL_PROOF_TREE_HEADER);
        for node in nodes {
            let id = ids[&node.key()];
            write!(out, "#{id} [{}]", node.tree.depth).unwrap();
            if let Some(choice) = &node.tree.assumption {
                write!(out, " assume {}", choice.describe(model)).unwrap();
            }
            out.push('\n');
            for (i, c) in node.tree.conclusions.iter().enumerate() {
                write!(
                    out,
                    "  step {}: {} ({})",
                    i + 1,
                    c.choice.describe(model),
                    c.rule
                )
                .unwrap();
                for proof in &c.proofs {
                    write!(out, " #{}", ids[&proof.key()]).unwrap();
                }
                out.push('\n');
            }
            if let Ending::Cases(branches) = &node.tree.ending {
                for branch in branches {
                    writeln!(out, "  case #{}", ids[&branch.key()]).unwrap();
                }
            }
            writeln!(out, "  {}", node.ending()).unwrap();
            if let Some(other) = &node.tree.alternative {
                writeln!(out, "  original alternative: #{}", ids[&other.key()]).unwrap();
            }
            if let Some(other) = &node.tree.comparison {
                writeln!(out, "  bounded companion: #{}", ids[&other.key()]).unwrap();
            }
        }
        out
    }
}

#[derive(Default)]
pub struct ProofTrace {
    conclusions: Vec<Conclusion>,
    frontier: usize,
}
impl ProofTrace {
    pub fn at(&mut self, frontier: usize) {
        self.frontier = frontier;
    }
    pub fn known(&mut self, depth: i32, choice: ProofChoice, rule: impl Into<Arc<str>>) {
        if depth != 0 {
            self.conclusions.push(Conclusion {
                choice,
                rule: rule.into(),
                proofs: vec![],
            });
        }
    }
    pub fn resolved(&mut self, depth: i32, choice: ProofChoice, proof: &SearchProof) {
        if depth != 0 {
            self.conclusions.push(Conclusion {
                choice,
                rule: "opposite alternative contradicted".into(),
                proofs: vec![proof.clone()],
            });
        }
    }
    fn finish(self, depth: i32, ending: Ending) -> SearchProof {
        let mut max_depth = depth;
        let mut frontier = self.frontier;
        let children = self
            .conclusions
            .iter()
            .flat_map(|c| &c.proofs)
            .chain(match &ending {
                Ending::Cases(branches) => branches.as_slice(),
                _ => &[],
            });
        for p in children {
            max_depth = max_depth.max(p.max_depth);
            frontier = frontier.max(p.frontier);
        }
        SearchProof {
            max_depth,
            frontier,
            tree: Arc::new(ProofTree {
                depth,
                assumption: None,
                conclusions: self.conclusions,
                ending,
                alternative: None,
                comparison: None,
            }),
        }
    }
    pub fn contradiction(self, depth: i32, reason: String) -> SearchProof {
        self.finish(depth, Ending::Contradiction(reason.into()))
    }
    pub fn contradiction_by(self, depth: i32, proof: SearchProof) -> SearchProof {
        self.finish(depth, Ending::Cases(vec![proof]))
    }
    pub fn open(self, depth: i32) -> SearchProof {
        self.finish(depth, Ending::Open)
    }
    pub fn cancelled(self, depth: i32) -> SearchProof {
        self.finish(depth, Ending::Cancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Generator;

    fn diff(from: usize, to: usize, exists: bool) -> ProofChoice {
        ProofChoice::Diff(Diff { from, to }, exists)
    }
    fn root(exists: bool) -> ProofChoice {
        diff(0, 1, exists)
    }
    fn leaf(depth: i32, frontier: usize, reason: &str) -> SearchProof {
        let mut trace = ProofTrace::default();
        trace.at(frontier);
        trace.contradiction(depth, reason.into())
    }
    fn model() -> E1 {
        E1::new(
            (0..8)
                .map(|i| Generator::new(format!("g{i}"), 10, 8 - i, 1, 0, None))
                .collect(),
        )
    }

    #[test]
    fn full_tree_keeps_all_discoveries_and_their_actual_supporting_proofs() {
        let mut trace = ProofTrace::default();
        trace.at(8);
        trace.resolved(1, diff(2, 3, true), &leaf(4, 12, "deepest"));
        trace.resolved(1, diff(4, 5, false), &leaf(2, 9, "later and shallower"));
        trace.known(1, diff(6, 7, false), "theory");
        let proof = trace.contradiction(1, "final contradiction".into());
        assert_eq!((proof.max_depth, proof.frontier), (4, 12));
        assert_eq!(proof.tree.conclusions.len(), 3);
        let text = proof.describe(&model());
        for reason in [
            "deepest",
            "later and shallower",
            "final contradiction",
            "theory",
        ] {
            assert!(text.contains(reason), "Missing {reason}");
        }
    }

    #[test]
    fn shared_conclusions_are_folded_but_both_derivations_and_contexts_survive() {
        let shared = diff(2, 3, true);
        let mut rejected = ProofTrace::default();
        rejected.known(1, shared.clone(), "left derivation");
        rejected.known(1, diff(4, 5, false), "branch-specific");
        let rejected = rejected
            .contradiction(1, "incompatible convergence".into())
            .assuming(1, root(false));
        let mut companion = ProofTrace::default();
        companion.known(1, shared.clone(), "right derivation");
        let proof = rejected.with_comparison(companion.open(1).assuming(1, root(true)));
        assert_eq!(proof.common(), [(0, 0)]);
        let text = proof.describe(&model());
        let (reduced, full) = text.split_once("\nFull proof tree").unwrap();
        assert!(reduced.starts_with("assume diff g0 -> g1 absent\n"));
        assert!(!reduced.contains("common"));
        assert!(!reduced.contains("assume diff g0 -> g1 exists"));
        assert!(!reduced.contains("diff g2 -> g3 exists"));
        assert!(!reduced.contains("choose diff g2 -> g3"));
        assert!(!reduced.contains("g4 -> g5"));
        assert_eq!(
            reduce_proof_tree(&proof).nodes[0].conclusions[0].choice,
            diff(4, 5, false)
        );
        assert!(full.contains("left derivation") && full.contains("right derivation"));
        assert!(full.contains("assume diff g0 -> g1 exists"));
        assert!(full.contains("assume diff g0 -> g1 absent"));
        // Formatting leaves the authoritative tree untouched.
        assert_eq!(proof.tree.conclusions[0].choice, shared);
    }

    #[test]
    fn shared_root_lemmas_are_factored_out_of_nested_refutations_too() {
        let shared = diff(2, 3, true);
        let mut nested = ProofTrace::default();
        nested.known(2, shared.clone(), "nested derivation");
        let nested = nested
            .contradiction(2, "nested contradiction".into())
            .assuming(2, diff(4, 5, false));
        let mut rejected = ProofTrace::default();
        rejected.known(1, shared.clone(), "left derivation");
        rejected.resolved(1, diff(4, 5, true), &nested);
        let mut companion = ProofTrace::default();
        companion.known(1, shared, "right derivation");
        let proof = rejected
            .contradiction(1, "root contradiction".into())
            .assuming(1, root(false))
            .with_comparison(companion.open(1).assuming(1, root(true)));
        let text = proof.describe(&model());
        let (reduced, full) = text.split_once("\nFull proof tree").unwrap();
        assert!(!reduced.contains("common conclusions:"));
        assert!(!reduced.contains("choose diff g2 -> g3 exists"));
        assert!(reduced.contains("-> g4 -> g5 exists"));
        assert!(!reduced.contains("assume diff g4 -> g5 absent"));
        assert!(full.contains("nested derivation"));
    }

    #[test]
    fn nested_conditional_facts_and_guesses_are_never_promoted_to_common_lemmas() {
        let fact = diff(2, 3, true);
        let mut rejected = ProofTrace::default();
        rejected.known(1, fact.clone(), "forced in rejected root");
        let rejected = rejected
            .contradiction(1, "bad".into())
            .assuming(1, root(false));
        let mut nested = ProofTrace::default();
        nested.known(2, fact.clone(), "only under an extra guess");
        let nested = nested.contradiction(2, "nested".into()).assuming(2, fact);
        let mut companion = ProofTrace::default();
        companion.resolved(1, diff(4, 5, true), &nested);
        let proof = rejected.with_comparison(companion.open(1).assuming(1, root(true)));
        assert!(proof.common().is_empty());
    }

    #[test]
    fn matching_respects_polarity_tau_filtration_and_induced_sphere() {
        let tau = |af| ProofChoice::Tau(ExtTauMult { from: 2, to: 3, af }, true);
        let name = |sphere| ProofChoice::InducedName {
            from: 4,
            to: 5,
            sphere,
        };
        let mut rejected = ProofTrace::default();
        let mut companion = ProofTrace::default();
        for choice in [diff(2, 3, true), tau(2), name(3)] {
            rejected.known(1, choice, "left");
        }
        for choice in [diff(2, 3, false), tau(3), name(5)] {
            companion.known(1, choice, "right");
        }
        let proof = rejected
            .contradiction(1, "bad".into())
            .assuming(1, root(false))
            .with_comparison(companion.open(1).assuming(1, root(true)));
        assert!(proof.common().is_empty());
    }

    #[test]
    fn unrelated_root_contexts_cannot_be_used_to_factor_out_a_fact() {
        let mut a = ProofTrace::default();
        let mut b = ProofTrace::default();
        a.known(1, diff(4, 5, true), "a");
        b.known(1, diff(4, 5, true), "b");
        let proof = a
            .contradiction(1, "bad".into())
            .assuming(1, root(false))
            .with_comparison(b.open(1).assuming(1, diff(2, 3, true)));
        assert!(proof.common().is_empty());
    }

    #[test]
    fn unused_alternatives_do_not_inflate_the_successful_depth_or_frontier() {
        let proof = leaf(2, 7, "useful")
            .with_alternative(leaf(9, 100, "unused"))
            .with_comparison(leaf(8, 90, "companion"));
        assert_eq!((proof.max_depth, proof.frontier), (2, 7));
        let text = proof.describe(&model());
        assert!(text.contains("unused") && text.contains("companion"));
    }

    #[test]
    fn repeated_subproofs_are_shared_and_emitted_once_in_the_full_tree() {
        let support = leaf(2, 5, "shared support");
        let mut trace = ProofTrace::default();
        trace.resolved(1, diff(2, 3, true), &support);
        trace.resolved(1, diff(4, 5, true), &support);
        let proof = trace.contradiction(1, "closed".into());
        let mut ids = HashMap::new();
        let mut nodes = vec![];
        proof.collect(&mut ids, &mut nodes);
        assert_eq!(nodes.len(), 2);
        let text = proof.describe(&model());
        let (_, full) = text.split_once("\nFull proof tree").unwrap();
        assert_eq!(full.matches("shared support").count(), 1);
    }

    #[test]
    fn independent_top_level_discoveries_do_not_accumulate_history() {
        let mut trace = ProofTrace::default();
        trace.resolved(0, diff(2, 3, true), &leaf(10, 99, "prior"));
        assert!(trace.open(0).tree.conclusions.is_empty());
    }

    #[test]
    fn compact_refutation_is_just_the_assumption_and_contradiction_when_only_same_source_choices_remain()
     {
        let shared = diff(2, 3, true);
        let reason = "Issue at S^3 | stem 21: [AlgebraicConvergence { bot_trunc: 0, top_trunc: 2, stem: 21, af: 7, expected: 0, observed: 1 }]";
        let mut trace = ProofTrace::default();
        for to in [2, 3, 4] {
            let support = leaf(2, 9, "other target is impossible").assuming(2, diff(0, to, true));
            trace.resolved(1, diff(0, to, false), &support);
        }
        trace.known(
            1,
            ProofChoice::FilteredDiff(Diff { from: 0, to: 5 }, Kind::Unnecessary),
            "another same-source choice",
        );
        trace.known(1, shared.clone(), "left common");
        let mut companion = ProofTrace::default();
        companion.known(1, shared, "right common");
        companion.known(1, diff(4, 5, true), "companion only");
        let proof = trace
            .contradiction(1, reason.into())
            .assuming(1, root(false))
            .with_comparison(companion.open(1).assuming(1, root(true)));

        let reduced = reduce_proof_tree(&proof);
        assert_eq!(reduced.nodes.len(), 1);
        assert!(reduced.nodes[0].conclusions.is_empty());
        assert_eq!(reduced.common.len(), 1);
        let expected = format!("assume diff g0 -> g1 absent\n-> contradiction: {reason}\n");
        assert_eq!(reduced.describe(&model()), expected);
        let text = proof.describe(&model());
        let (compact, full) = text.split_once("\nFull proof tree").unwrap();
        assert_eq!(compact, expected);
        assert!(full.contains("other target is impossible"));
        assert!(full.contains("another same-source choice"));
        assert!(full.contains("companion only"));
    }

    #[test]
    fn compact_chain_hides_exclusions_and_support_and_folds_the_same_terminal_contradiction() {
        let reason = "Issue at S^5 | stem 21: [AlgebraicConvergence { bot_trunc: 0, top_trunc: 4, stem: 21, af: 6, expected: 0, observed: 1 }]";
        let mut trace = ProofTrace::default();
        trace.resolved(
            1,
            diff(2, 3, true),
            &leaf(3, 9, "intermediate fact's proof"),
        );
        trace.resolved(1, diff(4, 5, false), &leaf(2, 8, "excluded alternative"));
        trace.known(
            1,
            ProofChoice::FilteredDiff(Diff { from: 6, to: 7 }, Kind::MinimalLength),
            "routine filter",
        );
        let cases = SearchProof::cases(
            1,
            9,
            vec![
                leaf(2, 9, reason).assuming(2, diff(6, 7, true)),
                leaf(2, 9, reason).assuming(2, diff(6, 7, false)),
            ],
        );
        let proof = trace.contradiction_by(1, cases).assuming(1, root(false));
        let before = proof.clone();
        let reduced = reduce_proof_tree(&proof);
        assert_eq!(reduced.nodes.len(), 3);
        assert_eq!(
            reduced.nodes[0].ending,
            ReducedEnding::Contradiction(reason.into())
        );
        assert_eq!(
            reduced.describe(&model()),
            format!(
                "assume diff g0 -> g1 absent\n-> g2 -> g3 exists\n-> contradiction: {reason}\n"
            )
        );
        assert_eq!(proof, before);
        let full = proof.describe(&model());
        let (_, full) = full.split_once("\nFull proof tree").unwrap();
        for detail in [
            "intermediate fact's proof",
            "excluded alternative",
            "routine filter",
            "assume diff g6 -> g7 exists",
            "assume diff g6 -> g7 absent",
        ] {
            assert!(full.contains(detail), "Missing full-tree detail: {detail}");
        }
    }
}

#[cfg(test)]
mod reduction_tests {
    use super::*;
    use std::collections::HashSet;

    fn fact(n: usize, exists: bool) -> ProofChoice {
        ProofChoice::Diff(Diff { from: n, to: n + 1 }, exists)
    }
    fn closed(depth: i32, reason: &str) -> SearchProof {
        ProofTrace::default().contradiction(depth, reason.into())
    }
    fn paired(left: ProofTrace, right: ProofTrace) -> SearchProof {
        left.contradiction(1, "left contradiction".into())
            .assuming(1, fact(0, false))
            .with_comparison(right.open(1).assuming(1, fact(0, true)))
    }
    fn check_graph(graph: &ReducedProofTree) {
        let mut seen = HashSet::new();
        let mut pending = vec![graph.root];
        while let Some(id) = pending.pop() {
            assert!(
                id > 0 && id <= graph.nodes.len(),
                "Dangling reduced reference R{id}"
            );
            if !seen.insert(id) {
                continue;
            }
            let node = &graph.nodes[id - 1];
            assert_eq!(node.id, id);
            assert!(node.full_tree_id > 0);
            for c in &node.conclusions {
                pending.extend(&c.proofs);
            }
            if let ReducedEnding::Cases(branches) = &node.ending {
                pending.extend(branches);
            }
            for p in &node.omitted_premises {
                assert!(p.common > 0 && p.common <= graph.common.len());
            }
        }
        assert_eq!(
            seen.len(),
            graph.nodes.len(),
            "Unreachable reduced nodes survived pruning"
        );
    }

    #[test]
    fn structural_reduction_keeps_only_the_root_branch_and_its_nonshared_support() {
        let mut left = ProofTrace::default();
        let mut right = ProofTrace::default();
        left.known(1, fact(2, true), "shared left");
        right.known(1, fact(2, true), "shared right");
        left.resolved(1, fact(4, false), &closed(2, "left support"));
        right.known(1, fact(4, true), "right-only conclusion");
        let full = paired(left, right);
        let before = full.clone();
        let reduced = reduce_proof_tree(&full);
        check_graph(&reduced);
        assert_eq!(full, before);
        assert_eq!(reduced.common.len(), 1);
        assert_eq!(
            reduced.nodes[reduced.root - 1].assumption,
            Some(fact(0, false))
        );
        assert_eq!(reduced.nodes.len(), 2);
        assert!(
            reduced
                .nodes
                .iter()
                .all(|n| n.assumption != Some(fact(0, true)))
        );
        assert!(
            reduced
                .nodes
                .iter()
                .flat_map(|n| &n.conclusions)
                .all(|c| c.choice != fact(4, true))
        );
        assert!(
            reduced
                .nodes
                .iter()
                .flat_map(|n| &n.conclusions)
                .all(|c| c.choice != fact(2, true))
        );
        assert!(
            reduced
                .nodes
                .iter()
                .flat_map(|n| &n.conclusions)
                .any(|c| c.choice == fact(4, false))
        );
        assert_eq!(
            reduced.nodes[reduced.root - 1].conclusions[0].original_step,
            2
        );
    }

    #[test]
    fn support_used_only_by_removed_conclusions_is_pruned_but_reused_support_survives() {
        let discarded = closed(2, "only supports common fact");
        let reused = closed(2, "supports common and unique facts");
        let mut left = ProofTrace::default();
        let mut right = ProofTrace::default();
        left.resolved(1, fact(2, true), &discarded);
        left.resolved(1, fact(4, true), &reused);
        left.resolved(1, fact(6, true), &reused);
        right.known(1, fact(2, true), "common");
        right.known(1, fact(4, true), "common");
        let full = paired(left, right);
        let graph = reduce_proof_tree(&full);
        check_graph(&graph);
        let reasons: Vec<_> = graph
            .nodes
            .iter()
            .filter_map(|n| match &n.ending {
                ReducedEnding::Contradiction(reason) => Some(reason.as_str()),
                _ => None,
            })
            .collect();
        assert!(!reasons.contains(&"only supports common fact"));
        assert_eq!(
            reasons
                .iter()
                .filter(|&&r| r == "supports common and unique facts")
                .count(),
            1
        );
        assert!(full.tree.conclusions[0].proofs[0] == discarded);
    }

    #[test]
    fn all_nonshared_subproofs_survive_instead_of_only_the_deepest_path() {
        let mut trace = ProofTrace::default();
        trace.resolved(1, fact(2, true), &closed(4, "deep"));
        trace.resolved(1, fact(4, true), &closed(2, "shallow"));
        let graph = reduce_proof_tree(&paired(trace, ProofTrace::default()));
        check_graph(&graph);
        assert_eq!(graph.max_depth, 4);
        for expected in ["deep", "shallow"] {
            assert!(graph.nodes.iter().any(|n| matches!(&n.ending,
                ReducedEnding::Contradiction(reason) if reason == expected)));
        }
    }

    #[test]
    fn source_filter_preserves_other_sources_tau_choices_and_reused_support() {
        let shared_support = closed(2, "retained support");
        let mut left = ProofTrace::default();
        left.resolved(
            1,
            ProofChoice::Diff(Diff { from: 0, to: 2 }, false),
            &shared_support,
        );
        left.resolved(
            1,
            ProofChoice::Diff(Diff { from: 2, to: 0 }, true),
            &shared_support,
        );
        left.known(
            1,
            ProofChoice::FilteredDiff(Diff { from: 3, to: 0 }, Kind::Unnecessary),
            "other source",
        );
        left.known(
            1,
            ProofChoice::Tau(
                ExtTauMult {
                    from: 0,
                    to: 4,
                    af: 2,
                },
                true,
            ),
            "tau",
        );
        let mut nested = ProofTrace::default();
        nested.resolved(
            2,
            ProofChoice::Diff(Diff { from: 0, to: 6 }, false),
            &closed(3, "discarded nested support"),
        );
        left.resolved(
            1,
            fact(4, true),
            &nested.contradiction(2, "nested contradiction".into()),
        );

        let graph = reduce_proof_tree(&paired(left, ProofTrace::default()));
        check_graph(&graph);
        let root = &graph.nodes[graph.root - 1];
        assert_eq!(root.conclusions.len(), 4);
        assert_eq!(
            root.conclusions[0].choice,
            ProofChoice::Diff(Diff { from: 2, to: 0 }, true)
        );
        assert!(matches!(
            root.conclusions[1].choice,
            ProofChoice::FilteredDiff(_, _)
        ));
        assert!(matches!(root.conclusions[2].choice, ProofChoice::Tau(_, _)));
        assert_eq!(graph.nodes.len(), 3);
        assert!(
            graph
                .nodes
                .iter()
                .any(|n| n.ending == ReducedEnding::Contradiction("retained support".into()))
        );
        assert!(
            graph.nodes.iter().all(
                |n| n.ending != ReducedEnding::Contradiction("discarded nested support".into())
            )
        );
        assert!(
            graph
                .nodes
                .iter()
                .flat_map(|n| &n.conclusions)
                .all(|c| c.choice.differential_source() != Some(0))
        );
    }

    #[test]
    fn nested_guesses_are_retained_and_do_not_create_global_common_facts() {
        let mut nested = ProofTrace::default();
        nested.known(2, fact(2, true), "conditional fact");
        let nested = nested
            .contradiction(2, "nested contradiction".into())
            .assuming(2, fact(6, false));
        let mut left = ProofTrace::default();
        left.resolved(1, fact(4, true), &nested);
        let mut right = ProofTrace::default();
        right.known(1, fact(2, true), "proved under the other root");
        let graph = reduce_proof_tree(&paired(left, right));
        check_graph(&graph);
        assert!(graph.common.is_empty());
        assert!(
            graph
                .nodes
                .iter()
                .any(|n| n.assumption == Some(fact(6, false)))
        );
        assert_eq!(
            graph
                .nodes
                .iter()
                .flat_map(|n| &n.conclusions)
                .filter(|c| c.choice == fact(2, true))
                .count(),
            1
        );
    }

    #[test]
    fn shared_facts_are_removed_from_nested_conclusions_but_never_from_assumptions() {
        let mut nested = ProofTrace::default();
        nested.known(2, fact(2, true), "nested copy");
        let nested = nested
            .contradiction(2, "nested contradiction".into())
            .assuming(2, fact(2, false));
        let mut left = ProofTrace::default();
        let mut right = ProofTrace::default();
        left.known(1, fact(2, true), "left");
        right.known(1, fact(2, true), "right");
        left.resolved(1, fact(4, true), &nested);
        let graph = reduce_proof_tree(&paired(left, right));
        check_graph(&graph);
        assert!(
            graph
                .nodes
                .iter()
                .flat_map(|n| &n.conclusions)
                .all(|c| c.choice != fact(2, true))
        );
        assert!(
            graph
                .nodes
                .iter()
                .any(|n| n.assumption == Some(fact(2, false)))
        );
    }

    #[test]
    fn both_refuted_cases_are_reduced_without_erasing_their_parent_premises() {
        let mut left = ProofTrace::default();
        let mut right = ProofTrace::default();
        let mut parent = ProofTrace::default();
        for trace in [&mut left, &mut right] {
            trace.known(2, fact(2, true), "shared");
        }
        parent.known(1, fact(2, true), "already proved before the case split");
        let full = parent.finish(
            1,
            Ending::Cases(vec![
                left.contradiction(2, "left".into())
                    .assuming(2, fact(0, true)),
                right
                    .contradiction(2, "right".into())
                    .assuming(2, fact(0, false)),
            ]),
        );
        let graph = reduce_proof_tree(&full);
        check_graph(&graph);
        assert_eq!(graph.common.len(), 1);
        assert_eq!(graph.nodes[0].conclusions[0].choice, fact(2, true));
        let ReducedEnding::Cases(branches) = &graph.nodes[0].ending else {
            panic!("Lost case split")
        };
        assert_eq!(branches.len(), 2);
        for &id in branches {
            assert!(graph.nodes[id - 1].conclusions.is_empty());
        }
    }

    #[test]
    fn equal_case_endings_fold_recursively_even_when_their_local_conclusions_differ() {
        let mut left = ProofTrace::default();
        left.known(2, fact(2, true), "left-only premise");
        let mut right = ProofTrace::default();
        right.known(2, fact(4, false), "right-only premise");
        let split = SearchProof::cases(
            1,
            5,
            vec![
                left.contradiction(2, "same contradiction".into())
                    .assuming(2, fact(6, true)),
                right
                    .contradiction(2, "same contradiction".into())
                    .assuming(2, fact(6, false)),
            ],
        );
        let mut parent = ProofTrace::default();
        parent.known(1, fact(2, true), "earlier premise");
        let full = parent
            .contradiction_by(1, split)
            .assuming(1, fact(0, false));
        let before = full.clone();
        let graph = reduce_proof_tree(&full);
        check_graph(&graph);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].conclusions[0].choice, fact(2, true));
        assert_eq!(
            graph.nodes[0].ending,
            ReducedEnding::Contradiction("same contradiction".into())
        );
        assert_eq!(full, before);
    }

    #[test]
    fn different_diagnostics_open_cancelled_and_empty_cases_never_fold_into_a_contradiction() {
        for other in [
            closed(2, "Issue at S^5 | stem 21: observed 2"),
            ProofTrace::default().open(2),
            ProofTrace::default().cancelled(2),
            SearchProof::cases(2, 5, vec![]),
        ] {
            let split = SearchProof::cases(
                1,
                5,
                vec![
                    closed(2, "Issue at S^5 | stem 21: observed 1").assuming(2, fact(6, true)),
                    other.assuming(2, fact(6, false)),
                ],
            );
            let graph = reduce_proof_tree(&split);
            check_graph(&graph);
            assert!(matches!(
                graph.nodes[graph.root - 1].ending,
                ReducedEnding::Cases(_)
            ));
        }
    }

    #[test]
    fn reused_nodes_remain_separate_when_only_one_occurrence_is_in_the_paired_scope() {
        let mut support = ProofTrace::default();
        support.known(3, fact(2, true), "shared source node");
        let support = support.contradiction(3, "support".into());
        let mut parent = ProofTrace::default();
        parent.resolved(1, fact(4, true), &support);
        let mut left = ProofTrace::default();
        let mut right = ProofTrace::default();
        for trace in [&mut left, &mut right] {
            trace.known(2, fact(2, true), "common");
        }
        left.resolved(2, fact(6, true), &support);
        let full = parent.finish(
            1,
            Ending::Cases(vec![
                left.contradiction(2, "left".into())
                    .assuming(2, fact(0, true)),
                right
                    .contradiction(2, "right".into())
                    .assuming(2, fact(0, false)),
            ]),
        );
        let graph = reduce_proof_tree(&full);
        check_graph(&graph);
        let copies: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.ending == ReducedEnding::Contradiction("support".into()))
            .collect();
        assert_eq!(copies.len(), 2);
        assert_eq!(copies[0].full_tree_id, copies[1].full_tree_id);
        assert!(copies.iter().any(|n| n.conclusions.len() == 1));
        assert!(
            copies
                .iter()
                .any(|n| n.conclusions.is_empty() && n.omitted_premises.len() == 1)
        );
    }

    #[test]
    fn json_export_is_deterministic_round_trippable_and_contains_only_reachable_nodes() {
        let mut left = ProofTrace::default();
        let mut right = ProofTrace::default();
        for trace in [&mut left, &mut right] {
            trace.known(1, fact(2, true), "common");
        }
        left.known(1, fact(4, true), "left only");
        let full = paired(left, right);
        let mut first = vec![];
        let mut second = vec![];
        export_reduced_proof_tree(&full, &mut first).unwrap();
        export_reduced_proof_tree(&full, &mut second).unwrap();
        assert_eq!(first, second);
        let restored: ReducedProofTree = serde_json::from_slice(&first).unwrap();
        assert_eq!(restored, reduce_proof_tree(&full));
        check_graph(&restored);
        assert_eq!(
            restored
                .nodes
                .iter()
                .map(|n| n.conclusions.len())
                .sum::<usize>(),
            1
        );
    }

    #[test]
    fn cancelled_sibling_is_usable_as_a_pair_but_its_guesses_are_not_common_facts() {
        let mut left = ProofTrace::default();
        let mut right = ProofTrace::default();
        for trace in [&mut left, &mut right] {
            trace.known(1, fact(2, true), "established before cancellation");
        }
        let full = left
            .contradiction(1, "bad".into())
            .assuming(1, fact(0, false))
            .with_alternative(right.cancelled(1).assuming(1, fact(0, true)));
        let graph = reduce_proof_tree(&full);
        check_graph(&graph);
        assert_eq!(graph.common.len(), 1);
        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.nodes[0].conclusions.is_empty());
        assert_eq!(
            graph.nodes[0].ending,
            ReducedEnding::Contradiction("bad".into())
        );
    }
}
