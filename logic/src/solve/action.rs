use core::panic;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::{
    MAX_STEM,
    data::naming::{generate_names_from_tag_special, name_to_sphere},
    domain::model::SyntheticSS,
    types::{Kind, Torsion},
};

pub static D_R_REPEATS: LazyLock<Vec<usize>> = LazyLock::new(|| {
    let mut r = vec![];
    for i in 0..=MAX_STEM {
        let mut c = 0;
        for j in 1..=i {
            let m = j % 8;
            if m == 0 || m == 1 || m == 2 || m == 4 {
                c += 1;
            }
        }
        r.push(2_usize.pow(c));
    }
    r
});

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Action {
    AddDiff {
        from: String,
        to: String,
        proof: String,
        kind: Kind,
    },
    AddInt {
        from: String,
        to: String,
        page: i32,
        proof: String,
        kind: Kind,
    },
    AddExt {
        from: String,
        to: String,
        af: i32,
        proof: String,
        kind: Kind,
    },
    SetE1 {
        tag: String,
        torsion: Torsion,
        proof: String,
    },
    SetInducedName {
        name: String,
        new_name: String,
        sphere: i32,
        proof: String,
    },
    Revert {
        times: i32,
    },
}

pub fn process_action(data: &mut SyntheticSS, action: &Action, ahss: bool) -> Result<i32, ()> {
    match action {
        Action::AddDiff { from, to, proof, kind } => {
            let from_tag = data.try_name_tag(&from)?;
            let to_tag = data.try_name_tag(&to)?;

            let x_from = data.model.get_name(from).stem;
            let x_to = data.model.get_name(to).stem;

            if x_from - x_to != 1 {
                println!("Tried to add differential between two stems not 1 apart.");
                return Err(());
            }

            let d_y = data.model.get_name(&from).y - data.model.get_name(&to).y;

            if d_y <= 0 {
                println!("Tried to add a diff on a non positive page");
                return Err(());
            }
            if ahss {
                let from_start = name_to_sphere(&from);
                let to_start = name_to_sphere(&to);

                let repeats = D_R_REPEATS[d_y as usize];

                // TODO:
                let a = (to_start - 1) / (repeats as i32);
                let from_start = from_start - a * (repeats as i32);
                let to_start = to_start - a * (repeats as i32);

                for (f, t) in generate_names_from_tag_special(from_tag, from_start, repeats)
                    .zip(generate_names_from_tag_special(to_tag, to_start, repeats))
                {
                    let p = if &f == from {
                        proof.clone()
                    } else {
                        format!(
                            "By James periodicity it follows from the external tau from {from} to {to}"
                        )
                    };
                    if data.add_diff_name(f, t, Some(p), *kind).is_err() {
                        break;
                    }
                }
                Ok(to_start)
            } else {
                data.add_diff_name(from.clone(), to.clone(), Some(proof.clone()), *kind)?;
                Ok(2)
            }
        }
        Action::AddInt {
            from,
            to,
            page,
            proof,
            kind,
        } => {
            let from_tag = data.try_name_tag(&from)?;
            let to_tag = data.try_name_tag(&to)?;

            let x_from = data.model.get_name(from).stem;
            let x_to = data.model.get_name(to).stem;

            if x_from != x_to {
                println!("Tried to add an internal tau between different stems");
                return Err(());
            }

            let d_y = data.model.get_name(&from).y - data.model.get_name(&to).y;

            if d_y != 0 {
                println!("Tried to add an internal tau between different filtrations");
                return Err(());
            }

            if ahss {
                let from_start = name_to_sphere(&from);
                let to_start = name_to_sphere(&to);

                let repeats = D_R_REPEATS[(*page - 1) as usize];

                // TODO:
                let a = (*page - 1) / (repeats as i32);
                let from_start = from_start - a * (repeats as i32);
                let to_start = to_start - a * (repeats as i32);

                for (f, t) in generate_names_from_tag_special(from_tag, from_start, repeats)
                    .zip(generate_names_from_tag_special(to_tag, to_start, repeats))
                {
                    let p = if &f == from {
                        proof.clone()
                    } else {
                        format!(
                            "By James periodicity it follows from the internal tau from {from} to {to}"
                        )
                    };
                    if data.add_int_tau_name(f, t, *page, Some(p), *kind).is_err() {
                        break;
                    }
                }
                Ok(to_start)
            } else {
                data.add_int_tau_name(from.clone(), to.clone(), *page, Some(proof.clone()), *kind)?;
                Ok(2)
            }
        }
        Action::AddExt {
            from,
            to,
            af,
            proof, 
            kind } => {
            let from_tag = data.try_name_tag(&from)?;
            let to_tag = data.try_name_tag(&to)?;

            let x_from = data.model.get_name(from).stem;
            let x_to = data.model.get_name(to).stem;

            if x_from != x_to {
                println!("Tried to add an external tau between different stems");
                return Err(());
            }

            let d_y = data.model.get_name(&from).y - data.model.get_name(&to).y;

            if d_y <= 0 {
                println!("Tried to add an external tau between wrong filtrations");
                return Err(());
            }

            // If on E1 these already have valid source and target torsion / af.
            // Then we apply James periodicity
            if ahss {
                if let Some(source_torsion) = data.model.get_name(from).torsion.0 {
                    if data.model.get_name(from).af - source_torsion == data.model.get_name(to).af {
                        let from_start = name_to_sphere(&from);
                        let to_start = name_to_sphere(&to);

                        let mut repeats = D_R_REPEATS[d_y as usize];

                        let a = (to_start - 1) / (repeats as i32);
                        if a > 0 {
                            repeats *= 2_usize.pow(a as u32);
                        }

                        for (f, t) in generate_names_from_tag_special(from_tag, from_start, repeats)
                            .zip(generate_names_from_tag_special(to_tag, to_start, repeats))
                        {
                            let p = if &f == from {
                                proof.clone()
                            } else {
                                format!(
                                    "By James periodicity it follows from the external tau from {from} to {to}"
                                )
                            };

                            if data.add_ext_tau_name(f, t, *af, Some(p), *kind).is_err() {
                                break;
                            }
                        }
                        return Ok(to_start);
                    }
                }
            }

            data.add_ext_tau_name(from.clone(), to.clone(), *af, Some(proof.clone()), *kind)?;
            Ok(2)
        }
        Action::SetE1 {
            tag,
            torsion,
            proof: _,
        } => {
            if !ahss {
                panic!("We can't Set E1 in EHP mode")
            }
            let mut to_start = 0;
            for g in generate_names_from_tag_special(tag, 1, 1) {
                if data.set_generator(&g, *torsion).is_err() {
                    break;
                }
                to_start = to_start.min(data.model.get_name(&g).stem);
            }

            Ok(to_start)
        }
        Action::SetInducedName {
            name,
            new_name,
            sphere,
            proof: _,
        } => {
            if ahss {
                panic!("We have no induced names in AHSS mode")
            }
            let original_id = data.model.try_index(name).ok_or(())?;

            // This is not completely necessary but we do want the induced thing to be valid
            let _ = data.model.try_index(new_name).ok_or(())?;

            data.model
                .get_mut(original_id)
                .induced_name
                .push((*sphere, new_name.clone()));
            Ok(2)
        }
        Action::Revert { times: _ } => Err(()),
    }
}

pub fn revert_log_and_remake(
    times: i32,
    log: &mut Vec<Action>,
    original_data: &SyntheticSS,
    ahss: bool,
) -> SyntheticSS {
    for _ in 0..times {
        log.pop();
    }

    let mut data = original_data.clone();
    for action in log {
        process_action(&mut data, action, ahss)
            .expect("There was an invalid action in the log. That should not be possible :(");
    }

    data
}
