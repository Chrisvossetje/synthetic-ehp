//! Replay the recorded external relations on one truncation. Each move exchanges
//! two lower tower endpoints, leaving the AFs and the graded dimensions fixed.
use std::collections::{HashMap, HashSet};

use crate::{
    domain::{
        e1::E1,
        model::{ExtTauMult, SyntheticSS},
        ss::{GeneratorState, SSPages},
    },
    solve::issues::Issue,
    types::Torsion,
};

const FINAL_PAGE: i32 = 500;

// A tower occupies (end, top]. None orders below every finite end (free tower).
type End = Option<i32>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Move {
    Idle,
    Gap,
    Exchange,
}

fn classify(source: i32, s_end: End, target: i32, t_end: End) -> Move {
    if source < target || s_end == Some(source) || t_end == Some(target) || t_end >= s_end {
        Move::Idle
    } else if s_end > Some(target) {
        Move::Gap
    } else {
        Move::Exchange
    }
}

fn end((top, torsion): GeneratorState) -> End {
    torsion.0.map(|length| top - length)
}

fn state(top: i32, end: End) -> GeneratorState {
    (top, Torsion(end.map(|bottom| top - bottom)))
}

/// The one-edge fast path is also the pair operation for internal taus.
pub(super) fn apply_tau(
    model: &E1,
    pages: &mut SSPages,
    page: i32,
    gate: i32,
    from: usize,
    to: usize,
) -> Result<(), Issue> {
    let s = pages.element_final(from);
    let t = pages.element_final(to);
    if s.0 < gate {
        return Ok(());
    }
    match classify(s.0, end(s), t.0, end(t)) {
        Move::Idle => {}
        Move::Exchange => {
            pages.push(from, page, state(s.0, end(t)));
            pages.push(to, page, state(t.0, end(s)));
        }
        Move::Gap => {
            let (from_name, to_name) = model.get_names(from, to);
            return Err(Issue::InvalidTauMult {
                from,
                to,
                from_name,
                to_name,
            });
        }
    }
    Ok(())
}

struct Replay<'a> {
    tops: &'a [i32],
    edges: &'a [(usize, usize)],
    seen: HashSet<(Vec<bool>, Vec<End>)>,
    outcomes: Vec<Vec<End>>,
}

impl Replay<'_> {
    fn visit(&mut self, used: &mut [bool], ends: &mut [End]) {
        if !self.seen.insert((used.to_vec(), ends.to_vec())) {
            return;
        }
        let mut complete = true;
        for i in 0..self.edges.len() {
            if used[i] {
                continue;
            }
            let (s, t) = self.edges[i];
            match classify(self.tops[s], ends[s], self.tops[t], ends[t]) {
                Move::Idle => continue,
                Move::Gap => complete = false, // A different relation may bridge it.
                Move::Exchange => {
                    complete = false;
                    used[i] = true;
                    ends.swap(s, t);
                    self.visit(used, ends);
                    ends.swap(s, t);
                    used[i] = false;
                }
            }
        }
        if complete && !self.outcomes.iter().any(|outcome| outcome == ends) {
            self.outcomes.push(ends.to_vec());
        }
    }
}

/// Ranks can change only at tower tops and just above a finite end. A probe at
/// minus infinity distinguishes free tails, even below every finite endpoint.
fn ranks(tops: &[i32], ends: &[End], probes: &[(i32, i64)]) -> Vec<usize> {
    probes
        .iter()
        .map(|&(hi, lo)| {
            tops.iter()
                .zip(ends)
                .filter(|&(&top, &end)| top >= hi && end.is_none_or(|b| i64::from(b) < lo))
                .count()
        })
        .collect()
}

/// Select the first undominated assignment in recorded relation order.
fn maximal_outcome(tops: &[i32], mut ends: Vec<End>, edges: &[(usize, usize)]) -> Option<Vec<End>> {
    let mut search = Replay {
        tops,
        edges,
        seen: HashSet::new(),
        outcomes: vec![],
    };
    search.visit(&mut vec![false; edges.len()], &mut ends);
    let outcomes = search.outcomes;
    if outcomes.len() <= 1 {
        return outcomes.into_iter().next();
    }

    let mut lows = vec![i64::MIN];
    lows.extend(ends.iter().flatten().map(|&b| i64::from(b) + 1));
    let mut probes = vec![];
    for &hi in tops {
        probes.extend(
            lows.iter()
                .filter(|&&lo| lo <= i64::from(hi))
                .map(|&lo| (hi, lo)),
        );
    }
    probes.sort_unstable();
    probes.dedup();
    let profiles: Vec<_> = outcomes
        .iter()
        .map(|ends| ranks(tops, ends, &probes))
        .collect();
    let i = profiles.iter().position(|rank| {
        !profiles
            .iter()
            .any(|other| other != rank && other.iter().zip(rank).all(|(a, b)| a >= b))
    })?;
    outcomes.into_iter().nth(i)
}

fn apply_component(pages: &mut SSPages, model: &E1, edges: &[ExtTauMult]) -> Result<(), Issue> {
    if let [e] = edges {
        return apply_tau(model, pages, FINAL_PAGE, e.af, e.from, e.to);
    }
    let mut ids: Vec<_> = edges.iter().flat_map(|e| [e.from, e.to]).collect();
    ids.sort_unstable();
    ids.dedup();
    let tops: Vec<_> = ids.iter().map(|&id| pages.element_final(id).0).collect();
    let ends: Vec<_> = ids.iter().map(|&id| end(pages.element_final(id))).collect();
    let local: Vec<_> = edges
        .iter()
        .map(|e| {
            (
                ids.binary_search(&e.from).unwrap(),
                ids.binary_search(&e.to).unwrap(),
            )
        })
        .collect();
    let Some(selected) = maximal_outcome(&tops, ends, &local) else {
        return Err(Issue::IncompatibleTauChain {
            relations: edges
                .iter()
                .map(|e| model.get_names(e.from, e.to))
                .collect(),
        });
    };

    for (i, &id) in ids.iter().enumerate() {
        let result = state(tops[i], selected[i]);
        if pages.element_final(id) != result {
            pages.push(id, FINAL_PAGE, result);
        }
    }
    Ok(())
}

pub fn compute_ext_taus(
    pages: &mut SSPages,
    model: &E1,
    data: &SyntheticSS,
    _bot_trunc: i32,
    _top_trunc: i32,
    from_stem: i32,
    to_stem: i32,
) -> Vec<Issue> {
    let edges: Vec<_> = data
        .external_taus()
        .copied()
        .filter(|e| {
            (from_stem..=to_stem).contains(&model.stem(e.from))
                && pages
                    .try_element_final(e.from)
                    .is_some_and(|s| s.0 >= e.af && s.1.alive())
                && pages.try_element_final(e.to).is_some_and(|t| t.1.alive())
        })
        .collect();
    if edges.is_empty() {
        return vec![];
    }
    if edges.len() == 1 {
        return apply_component(pages, model, &edges)
            .err()
            .into_iter()
            .collect();
    }

    let mut incident = HashMap::<usize, smallvec::SmallVec<[usize; 2]>>::new();
    for (i, e) in edges.iter().enumerate() {
        incident.entry(e.from).or_default().push(i);
        incident.entry(e.to).or_default().push(i);
    }
    let mut seen = vec![false; edges.len()];
    let mut issues = vec![];
    for i in 0..edges.len() {
        if seen[i] {
            continue;
        }
        let edge = &edges[i];
        if incident[&edge.from].len() == 1 && incident[&edge.to].len() == 1 {
            if let Err(issue) = apply_tau(model, pages, FINAL_PAGE, edge.af, edge.from, edge.to) {
                issues.push(issue);
            }
            continue;
        }
        let mut component = vec![i];
        seen[i] = true;
        let mut cursor = 0;
        while cursor < component.len() {
            let e = edges[component[cursor]];
            for id in [e.from, e.to] {
                for j in incident.remove(&id).unwrap_or_default() {
                    if !seen[j] {
                        seen[j] = true;
                        component.push(j);
                    }
                }
            }
            cursor += 1;
        }
        component.sort_unstable();
        let group: Vec<_> = component.into_iter().map(|i| edges[i]).collect();
        if let Err(issue) = apply_component(pages, model, &group) {
            issues.push(issue);
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::process::compute_pages,
        types::{Generator, Kind},
    };

    #[test]
    fn a_pending_gap_can_be_closed_and_an_impossible_chain_is_atomic() {
        // The first relation starts with a gap. Taking the second first
        // lowers the source end from 5 to 1, and the first can then act.
        assert_eq!(
            maximal_outcome(&[6, 3, 5], vec![Some(5), None, Some(1)], &[(0, 1), (0, 2)]),
            Some(vec![None, Some(1), Some(5)])
        );
        let model = E1::new(
            (0..3)
                .map(|i| Generator::new(format!("g{i}"), 10, 4 - i, 6 - 2 * i, 0, None))
                .collect(),
        );
        let mut data = SyntheticSS::empty(model.clone());
        data.generators = vec![Torsion::new(2), Torsion::new(1), Torsion::default()];
        data.add_ext_tau(&model, 0, 1, 5, None, Kind::Real);
        data.add_ext_tau(&model, 1, 2, 3, None, Kind::Real);
        // b cannot reach c, unless a -> b kills b first.
        let (pages, issues) = compute_pages(&data, &model, 0, 4, 10, 10, true);
        assert!(issues.is_empty());
        assert_eq!(pages.element_final(0), (6, Torsion::new(3)));
        assert_eq!(pages.element_final(1), (4, Torsion::zero()));

        data.generators[0] = Torsion::new(3);
        let (pages, issues) = compute_pages(&data, &model, 0, 4, 10, 10, true);
        assert!(matches!(
            issues.as_slice(),
            [Issue::IncompatibleTauChain { .. }]
        ));
        assert_eq!(pages.element_final(0), (6, Torsion::new(3)));
        assert_eq!(pages.element_final(1), (4, Torsion::new(1)));
    }

    #[test]
    fn rank_beats_recorded_order_only_on_the_final_page() {
        let model = E1::new(vec![
            Generator::new("s".into(), 10, 3, 5, 0, None),
            Generator::new("near".into(), 10, 2, 4, 0, None),
            Generator::new("far".into(), 10, 1, 2, 0, None),
        ]);
        let mut data = SyntheticSS::empty(model.clone());
        data.generators = vec![Torsion::new(3), Torsion::default(), Torsion::default()];
        data.add_ext_tau(&model, 0, 1, 3, None, Kind::Real);
        data.add_ext_tau(&model, 0, 2, 4, None, Kind::Real);
        let (pages, issues) = compute_pages(&data, &model, 0, 3, 10, 10, true);
        assert!(issues.is_empty());
        // The second replay order preserves the additional free tail at AF 4.
        assert_eq!(pages.element_final(1), (4, Torsion::default()));
        assert_eq!(pages.element_final(2), (2, Torsion::zero()));
        assert_eq!(
            pages.element_at_page(FINAL_PAGE - 1, 0),
            (5, Torsion::new(3))
        );
        assert_eq!(
            pages.element_at_page(FINAL_PAGE, 0),
            (5, Torsion::default())
        );
        let (without_tau, issues) = compute_pages(&data, &model, 0, 3, 10, 10, false);
        assert!(issues.is_empty());
        for id in 0..3 {
            assert_eq!(
                pages.element_at_page(FINAL_PAGE - 1, id),
                without_tau.element_final(id)
            );
        }
    }

    #[test]
    fn recorded_order_breaks_ties_between_maximal_assignments() {
        assert_eq!(
            maximal_outcome(&[4, 4, 2], vec![Some(1), Some(1), None], &[(0, 2), (1, 2)]),
            Some(vec![None, Some(1), Some(1)])
        );
        // These two orders produce incomparable profiles. Each is maximal.
        let ends = vec![Some(2), Some(1), Some(0)];
        assert_eq!(
            maximal_outcome(&[5, 4, 2], ends.clone(), &[(1, 2), (0, 1)]),
            Some(vec![Some(0), Some(2), Some(1)])
        );
        assert_eq!(
            maximal_outcome(&[5, 4, 2], ends, &[(0, 1), (1, 2)]),
            Some(vec![Some(1), Some(0), Some(2)])
        );
    }

    #[test]
    fn gates_dead_towers_and_equal_af_are_shared_by_both_sequences() {
        assert_eq!(
            maximal_outcome(&[4, 4], vec![Some(2), None], &[(0, 1)]),
            Some(vec![None, Some(2)])
        );
        let model = E1::new(vec![
            Generator::new("s".into(), 10, 3, 4, 0, None),
            Generator::new("t".into(), 10, 2, 4, 0, None),
        ]);
        let mut data = SyntheticSS::empty(model.clone());
        data.generators[0] = Torsion::new(2);
        data.add_ext_tau(&model, 0, 1, 5, None, Kind::Real);
        let (pages, issues) = compute_pages(&data, &model, 0, 3, 10, 10, true);
        assert!(issues.is_empty());
        assert_eq!(pages.element_final(0).1, Torsion::new(2));
        assert!(matches!(classify(4, Some(4), 4, None), Move::Idle));
    }
}
