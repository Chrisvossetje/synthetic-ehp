use std::collections::HashMap;

use crate::{curtis::{Tagged, Untagged}, model::{Diff, E1, SyntheticSS}, types::{Differential, Generator, Kind, OldSyntheticSS}};




// fn parse_stable_algebraic_data(
//     untagged: &[Untagged],
//     tagged: &[Tagged],
// ) -> SyntheticSS {
//     let mut generators = Vec::new();
//     let mut differentials = Vec::new();

//     for unt in untagged {
//         if unt.stem <= MAX_STEM {
//             let (first, second) = unt.tag.strip_prefix('(').unwrap().split_once(')').unwrap();
//             let y: i32 = first.parse().unwrap();

//             generators.push(Generator::new(
//                 format!("{}[{}]", second.trim(), first.trim()),
//                 unt.stem,
//                 y,
//                 unt.filt,
//                 y + 1,
//                 None,
//             ));
//         }
//     }

//     for tag in tagged {
//         if tag.stem <= MAX_STEM {
//             let (first_l, second_l) = tag.left_tag.strip_prefix('(').unwrap().split_once(')').unwrap();
//             let y: i32 = first_l.parse().unwrap();

//             let to = format!("{}[{}]", second_l.trim(), first_l.trim());

//             let (first_r, second_r) = tag.right_tag.strip_prefix('(').unwrap().split_once(')').unwrap();
//             let y_2: i32 = first_r.parse().unwrap();
            
//             let from = format!("{}[{}]", second_r.trim(), first_r.trim());
            
//             generators.push(Generator::new(
//                 to.clone(),
//                 tag.stem,
//                 y,
//                 tag.filt,
//                 y + 1,
//                 Some(y_2 + 1),
//             ));
            
//             generators.push(Generator::new(
//                 from.clone(),
//                 tag.stem + 1,
//                 y_2,
//                 tag.filt - 1,
//                 y_2 + 1,
//                 Some(y_2 + 1),
//             ));

//             differentials.push(Differential {
//                 from,
//                 to,
//                 coeff: 0,
//                 d: (y_2 - y),
//                 proof: None,
//                 kind: Kind::Real,
//             });

//         }
//     }

//     generators.sort_by_key(|x| x.af);
//     generators.sort_by_key(|x| x.y);
//     generators.sort_by_key(|x| x.stem);

//     let mut diffs_page = vec![vec![]; (MAX_STEM + 1) as usize];
//     let internal_tau_page = vec![vec![]; (MAX_STEM + 1) as usize];
//     let external_tau_page = vec![];
//     let mut proven_from_to = HashMap::new();

//     let model = E1::new(generators);


//     for d in differentials {
//         let from = model.get_index(&d.from);
//         let to = model.get_index(&d.to);
//         diffs_page[d.d as usize].push(Diff { from, to });
//         proven_from_to.insert((from, to), None);
//     }

//     let in_diffs = proven_from_to.iter().map(|x| (x.0.1, x.0.0)).collect();

//     let data = SyntheticSS {
//         model,
//         diffs_page,
//         internal_tau_page,
//         external_tau_page,
//         proven_from_to,
//         disproven_from_to: HashMap::default(),
//         in_diffs,
//     };

//     data
// }


