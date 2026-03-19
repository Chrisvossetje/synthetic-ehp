use std::{collections::HashMap, convert::identity};
use itertools::{self, Itertools};

use crate::{MAX_VERIFY_STEM, issues::Issue, model::SyntheticSS, naming::{generate_names_from_tag, name_get_tag}, static_compare_data::{synthetic_s0, synthetic_s0_keys}, types::Torsion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GeneratorChange {
    g: usize,
    torsion: Torsion,
}


// fn apply_e1_change(data: &mut SyntheticSS, g: usize, torsion: Torsion) -> Result<(),String> {
//     if let Some(&source) = data.in_diffs.get(&g) {
//         let t = data.model.original_torsion(source);

//         // The only thing which is relevant to check here is if the source lives while the target is dead
//         // Torsion issues should and can be resolved later !
//         if t.alive() && !torsion.alive() {
//             let to_name = data.model.name(g);
//             let from_name = data.model.name(source);
//             return Err(format!("Tried to kill {}. But there is an algebraic differential from {} which has non zero torsion {:?}.", to_name, from_name, t));
//         }
//     }
    
//     data.model.get_mut(g).torsion = torsion;
//     Ok(())
// }

// fn full_apply_e1_change(data: &mut SyntheticSS, change: GeneratorChange) -> Result<Vec<usize>,String>  {
//     let idxs = get_all_elts_belonging_to_this_e1(data, change.g);
//     let name = data.model.name(change.g);
//     for i in &idxs {
//         let r = apply_e1_change(data, *i, change.torsion);
//         match r {
//             Ok(_) => {},
//             Err(e) => {
//                 reset_torsion_idxs(data, &idxs);
//                 return Err(e);
//             },
//         }
//     }

//     Ok(idxs)
// }

// fn apply_option(data: &mut SyntheticSS, option: &Vec<GeneratorChange>) -> Result<Vec<Vec<usize>>, String> {
//     let mut idxss = vec![]; 
//     if option.len() == 0 {
//         panic!("This option changes nothing, which is bogus");
//     }

//     for g in option {
//         match full_apply_e1_change(data, *g) {
//             Ok(idxs) => { idxss.push(idxs); },
//             Err(e) => {
//                 for idxs in idxss {
//                     reset_torsion_idxs(data, &idxs);
//                 }
//                 return Err(e)
//             },
//         }
//     }
//     Ok(idxss)
// }

// fn reset_torsion_idxs(data: &mut SyntheticSS, idxs: &Vec<usize>) {
//     for i in idxs {
//         data.model.get_mut(*i).torsion = Torsion::default();
//     }
// }

// fn revert_or_ok(data: &mut SyntheticSS, idxss: Vec<Vec<usize>>, res: Result<(), String>, revert_on_return: bool) -> Result<(), String> {
//     if revert_on_return {
//         for idxs in idxss {
//             reset_torsion_idxs(data, &idxs);
//         }
//         return res;
//     }
//     match res {
//         Ok(_) => {
//             Ok(())
//         },
//         Err(e) => {
//             for idxs in idxss {
//                 reset_torsion_idxs(data, &idxs);
//             }
//             Err(e)
//         },
//     }
// }

// fn get_all_elts_belonging_to_this_e1(data: &SyntheticSS, g: usize) -> Vec<usize> {
//     let name = data.model.name(g);
//     let tag = name_get_tag(&name);
    
//     let mut idxs = vec![];
//     for n in generate_names_from_tag(tag) {
//         if let Some(id) = data.model.try_index(&n) {
//             idxs.push(id);
//         } else {
//             break; 
//         };
//     }
//     idxs
// }

// fn induct(data: &mut SyntheticSS, issues: &Vec<SyntheticE1PageIssue>, issue: usize, depth: usize, max_depth: usize, revert_on_return: bool) -> Result<(), String> {
//     if issue >= issues.len() { return Ok(()) }
//     if depth > max_depth { return Ok(()) }



//     let options = get_e1_solutions(data, &issues[issue]);

//     // This would mean NO solutions were found, meaning that the algebraic data is WRONG
//     assert_ne!(options.len(), 0);

//     // Easy, no choice te make!
//     if options.len() == 1 {
//         // Error could very well occur here ! But at least i don't have to consider other options.
//         match apply_option(data, &options[0]) {
//             Ok(idxss) => {
//                 let ind = induct(data, issues, issue + 1, depth, max_depth, revert_on_return);
//                 return revert_or_ok(data, idxss, ind, revert_on_return);
//             }
//             Err(e) => {
//                 return Err(e)
//             },
//         }
    
//     // Shit, now we have to make choice and actually try :(
//     } else {
//         // We opt out slightly earlier if we have multiple options
//         if depth >= max_depth { return Ok(()) }

//         let mut succes_option = None;

//         for d in [1,3,7] {
//             if d >= max_depth {
//                 break;
//             }
//             let mut successes = vec![false; options.len()];
//             let mut errors = vec![];
//             for (index, option) in options.iter().enumerate() {
//                 match apply_option(data, option) {
//                     Ok(idxss) => {
//                         let ind = induct(data, issues, issue + 1, depth + 1, d, true);
//                         for idxs in idxss {
//                             reset_torsion_idxs(data, &idxs);
//                         }   
//                         match ind {
//                             Ok(_) => {
//                                 successes[index] = true;
//                             },
//                             Err(e) => {errors.push(e);},
//                         }
//                     },
//                     Err(_) => {},
//                 }
//             }

//             let succes_count = successes.iter().filter(|x| **x).count();
//             if succes_count == 0 {
//                 return Err(format!("There are no valid solutions sadddd. List of errors: {}", errors.join(" + ")));
//             } else if succes_count == 1 {
//                 let loc = successes.iter().find_position(|x| **x).unwrap();
//                 succes_option = Some(loc.0);
//                 break;
//             }
//         }

//         if let Some(o) = succes_option {
//             match apply_option(data, &options[o]) {
//                 Ok(idxss) => {
//                     let ind = induct(data, issues, issue + 1, depth, max_depth, revert_on_return);
//                     return revert_or_ok(data, idxss, ind, revert_on_return);
//                 }
//                 Err(e) => {
//                     return Err(e)
//                 },
//             }
//         } else {
//             println!("{:?}", &issues[issue]);
//             panic!("I should always had a succesful option or quit earlier ")
//         }
//     }

//     Ok(())
// }

// pub fn solve_e1_issues(data: &mut SyntheticSS, mut issues: Vec<SyntheticE1PageIssue>) {
//     // It is important that this list is sorted
//     // Because that way i can ALWAYS guarantee that i kill a source before i kill a potential target.
//     issues.sort_by_key(|x| (x.stem, x.af));

//     // Build some incoming diff map ?
//     // I hope that just using this map, i can resolve ALL synthetic E1 questions (at least up to stem 48)
    
//     let res = induct(data, &issues, 0, 0, 100, false);
//     println!("{:?}", res);
// }

// fn get_e1_solutions(data: &mut SyntheticSS, issue: &SyntheticE1PageIssue) -> Vec<Vec<GeneratorChange>> {
//     // Give a list of options which one could / should change
//     // This is the only time i (should) do this forward approach. Aka giving potential solutions.
//     // In other cases i should just "go" and see if some option resolves some issue

//     // The solutions should be some combinatorial thing 
//     // It should be "unique" permutations!

//     let stem = issue.stem + 1;

//     let stem_af_to_index = data.model.gens_id_in_stem_af(stem, issue.af);

//     let mut changes = vec![];

//     // TODO: This could be faster ?
//     for p in issue.expected.iter().permutations(issue.expected.len()).unique() {
//         let mut change = vec![];
//         for (id, t) in p.into_iter().enumerate() {
//             if *t != Torsion::default() {
//                 let real_id = stem_af_to_index[id];
//                 change.push(GeneratorChange {
//                     g: real_id,
//                     torsion: *t,
//                 });
//             }
//         }
//         changes.push(change);
//     } 
//     changes
// }