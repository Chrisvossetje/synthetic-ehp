//! [`SyntheticSS`]: the spectral-sequence facts asserted on top of an [`E1`]
//! page — differentials, internal tau-multiplications (same bidegree) and
//! external tau-multiplications (same stem). All facts are keyed in `from_to`
//! to dedupe, and additionally bucketed or sorted so that
//! [`crate::domain::process`] can apply them in the right order.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    MAX_STEM,
    domain::e1::E1,
    types::{Kind, Torsion},
};

pub type FromTo = (usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Diff {
    pub from: usize,
    pub to: usize,
}

// Here Tau Mult extension is probably not really correct ?
// It probably has more to do with choice of basis ?
// But i have to say something about convergence and how certain elements will lift :(
// Conclusion: This is the easiest workable method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct IntTauMult {
    pub from: usize,
    pub to: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExtTauMult {
    pub from: usize,
    pub to: usize,
    pub af: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct OrderedExtTauMult {
    // Matches the former bucket traversal: source y, AF, then y difference.
    key: (i32, i32, i32),
    tau: ExtTauMult,
}

// This should always implicitly reference some Model
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SyntheticSS {
    pub generators: Vec<Torsion>,
    pub induced_name: Option<Vec<Vec<(i32, String)>>>,

    // This should be indexed by page ??
    // Or should it be indexed by Gens
    // Length of this should equal max_stem + 1
    pub diffs_page: Vec<Vec<Diff>>,
    pub internal_tau_page: Vec<Vec<IntTauMult>>,

    // Applied at the final page. Keep only actual taus, in application order,
    // rather than allocating a mostly empty cubic array of buckets.
    external_taus: Vec<OrderedExtTauMult>,

    pub from_to: HashMap<FromTo, (Kind, Option<String>)>,

    // Remember incoming/outgoing stuff
    pub in_diffs: Vec<Vec<usize>>,
    pub out_diffs: Vec<Vec<usize>>,

    pub out_taus: Vec<Vec<usize>>,
}

impl SyntheticSS {
    pub fn empty(e1: E1) -> Self {
        let len = e1.gens().len();
        Self {
            generators: vec![ Torsion::default(); len],
            induced_name: None,
            diffs_page: vec![vec![]; (MAX_STEM + 1) as usize],
            internal_tau_page: vec![vec![]; (MAX_STEM + 1) as usize],
            external_taus: Vec::new(),
            from_to: HashMap::default(),
            in_diffs: vec![vec![]; len],
            out_diffs: vec![vec![]; len],
            out_taus: vec![vec![]; len],
        }
    }

    pub fn add_diff(&mut self, model: &E1, from: usize, to: usize, proof: Option<String>, kind: Kind) {
        let d_y = model.y(from) - model.y(to);

        if !self.from_to.contains_key(&(from, to)) {
            self.from_to.insert((from, to), (kind, proof));
            match kind {
                Kind::Real | Kind::Algebraic => {
                    self.diffs_page[d_y as usize].push(Diff { from, to });
                    self.in_diffs[to].push(from);
                    self.out_diffs[from].push(to);
                },
                _ => {}
            }
        }
    }

    pub fn add_int_tau(
        &mut self,
        from: usize,
        to: usize,
        page: i32,
        proof: Option<String>,
        kind: Kind,
    ) {
        if !self.from_to.contains_key(&(from, to)) {
            self.from_to.insert((from, to), (kind, proof));
            match kind {
                Kind::Real => {
                    self.internal_tau_page[page as usize].push(IntTauMult { from, to });
                }
                _ => {}
            }
        }
    }

    pub fn add_ext_tau(
        &mut self,
        model: &E1,
        from: usize,
        to: usize,
        af: i32,
        proof: Option<String>,
        kind: Kind,
    ) {
        if !self.from_to.contains_key(&(from, to)) {
            self.from_to.insert((from, to), (kind, proof));
            match kind {
                Kind::Real => {
                    let y_from = model.y(from);
                    let y_to = model.y(to);
                    let key = (y_from, af, y_from - y_to);
                    // Insert after equal keys: application mutates generator states,
                    // so the original insertion order within each bucket matters.
                    let index = self.external_taus.partition_point(|entry| entry.key <= key);
                    self.external_taus.insert(
                        index,
                        OrderedExtTauMult {
                            key,
                            tau: ExtTauMult { from, to, af },
                        },
                    );
                    self.out_taus[from].push(to);
                }
                _ => {}
            }
        }
    }

    /// Real external taus in application order, preserving insertion order for ties.
    pub fn external_taus(&self) -> impl Iterator<Item = &ExtTauMult> {
        self.external_taus.iter().map(|entry| &entry.tau)
    }

    pub fn add_diff_name(
        &mut self,
        model: &E1,
        from: String,
        to: String,
        proof: Option<String>,
        kind: Kind,
    ) -> Result<(), ()> {
        let from = model.try_index(&from).ok_or(())?;
        let to = model.try_index(&to).ok_or(())?;
        self.add_diff(model, from, to, proof, kind);
        Ok(())
    }

    pub fn add_int_tau_name(
        &mut self,
        model: &E1,
        from: String,
        to: String,
        page: i32,
        proof: Option<String>,
        kind: Kind,
    ) -> Result<(), ()> {
        let from = model.try_index(&from).ok_or(())?;
        let to = model.try_index(&to).ok_or(())?;
        self.add_int_tau(from, to, page, proof, kind);
        Ok(())
    }

    pub fn add_ext_tau_name(
        &mut self,
        model: &E1,
        from: String,
        to: String,
        af: i32,
        proof: Option<String>,
        kind: Kind,
    ) -> Result<(), ()> {
        let from = model.try_index(&from).ok_or(())?;
        let to = model.try_index(&to).ok_or(())?;
        self.add_ext_tau(model, from, to, af, proof, kind);
        Ok(())
    }

    pub fn set_generator(&mut self, model: &E1, name: &String, torsion: Torsion) -> Result<(), ()> {
        let id = model.try_index(name).ok_or(())?;
        self.generators[id] = torsion;
        Ok(())
    }


    pub fn get_name_at_sphere<'a>(&'a self, model: &'a E1, elt: usize, sphere: i32) -> &'a str {
        let l: &Vec<(i32, String)> = if let Some(v) = &self.induced_name
            && !v[elt].is_empty()
        {
            &v[elt]
        } else {
            &model.get(elt).induced_name
        };

        let mut id = 0;

        loop {
            if id + 1 == l.len() {
                return &l[id].1;
            }
            if l[id + 1].0 > sphere {
                return &l[id].1;
            }
            id += 1;
        }
    }

    pub fn push_induced_name(&mut self, model: &E1, elt: usize, sphere: i32, new_name: String) {
        let len = model.gens().len();
        if self.induced_name.is_none() {
            self.induced_name = Some(vec![vec![]; len]);
        }
        let map = self.induced_name.as_mut().unwrap();
        if map[elt].is_empty() {
            map[elt] = model.get(elt).induced_name.clone();
        }
        map[elt].push((sphere, new_name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::process::compute_pages, types::Generator};

    fn model(ys: &[i32]) -> E1 {
        E1::new(
            ys.iter()
                .enumerate()
                .map(|(id, &y)| Generator::new(format!("g{id}"), 10, y, 1, 0, None))
                .collect(),
        )
    }

    #[test]
    fn external_taus_preserve_bucket_order_and_insertion_order_within_ties() {
        let model = model(&[4, 1, 2, 6, 4, 6, 4]);
        let mut data = SyntheticSS::empty(model.clone());
        for (from, to, af) in [
            (3, 1, 3),
            (0, 1, 9),
            (3, 2, 2),
            (3, 4, 3),
            (5, 6, 3),
            (0, 2, 9),
            (5, 2, 2),
        ] {
            data.add_ext_tau(&model, from, to, af, None, Kind::Real);
        }

        let order: Vec<_> = data.external_taus().map(|t| (t.from, t.to, t.af)).collect();
        assert_eq!(
            order,
            [
                (0, 2, 9),
                (0, 1, 9),
                (3, 2, 2),
                (5, 2, 2),
                (3, 4, 3),
                (5, 6, 3),
                (3, 1, 3)
            ]
        );
    }

    #[test]
    fn external_tau_deduplication_and_non_real_facts_are_unchanged() {
        let model = model(&[4, 3, 2, 1]);
        let mut data = SyntheticSS::empty(model.clone());
        data.add_ext_tau(&model, 0, 1, 2, Some("original".into()), Kind::Real);
        data.add_ext_tau(&model, 0, 1, 3, Some("duplicate".into()), Kind::Real);
        data.add_ext_tau(&model, 0, 2, 2, None, Kind::Fake);
        data.add_ext_tau(&model, 0, 2, 2, None, Kind::Real);
        data.add_ext_tau(&model, 0, 3, 2, None, Kind::Unknown);

        assert_eq!(
            data.external_taus().copied().collect::<Vec<_>>(),
            [ExtTauMult {
                from: 0,
                to: 1,
                af: 2
            }]
        );
        assert_eq!(data.out_taus[0], [1]);
        assert_eq!(data.from_to[&(0, 1)], (Kind::Real, Some("original".into())));
        assert_eq!(data.from_to[&(0, 2)].0, Kind::Fake);
        assert_eq!(data.from_to[&(0, 3)].0, Kind::Unknown);
    }

    #[test]
    fn external_tau_chain_is_applied_in_filtration_order() {
        let model = E1::new(vec![
            Generator::new("a".into(), 10, 6, 6, 0, None),
            Generator::new("b".into(), 10, 3, 4, 0, None),
            Generator::new("c".into(), 10, 1, 2, 0, None),
        ]);
        let mut data = SyntheticSS::empty(model.clone());
        data.generators[0] = Torsion::new(3);
        data.generators[1] = Torsion::new(3);

        // Although recorded second, b -> c must run first. Applying a -> b
        // first leaves b with insufficient torsion and makes b -> c invalid.
        data.add_ext_tau(&model, 0, 1, 5, None, Kind::Real);
        data.add_ext_tau(&model, 1, 2, 3, None, Kind::Real);

        let (pages, issues) = compute_pages(&data, &model, 0, 6, 10, 10, true);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(pages.element_final(0), (6, Torsion::default()));
        assert_eq!(pages.element_final(1), (4, Torsion::new(1)));
        assert_eq!(pages.element_final(2), (2, Torsion::new(1)));
    }
}
