use std::collections::HashMap;

use itertools::Diff;

use crate::{MAX_STEM, processor::{find_possible_sources_for_differentials_in_stem, get_filtered_data}, stable_verification::read_rp_csv, types::{Category, Differential, Kind, SyntheticSS, TauMult}};


#[derive(Debug, Clone, PartialEq)]
struct Issue {
    bot_trunc: i32,
    top_trunc: i32,
    stem: i32,
    af: i32,
    obs: Vec<(Option<i32>, String)>,
    exp: Vec<Option<i32>>,
}

#[derive(Debug, Clone, PartialEq)]
enum Solution {
    Diff(DiffSolution),
    DoubleDiff(DiffSolution, DiffSolution),
    Tau(TauMultSolution),
}

impl Solution {
    pub fn names_occur_in_both(&self, other: &Solution) -> bool {
        let (from, to) = match other {
            Solution::Diff(diff_solution) => (&diff_solution.from, &diff_solution.to),
            Solution::Tau(tau_mult_solution) => (&tau_mult_solution.from, &tau_mult_solution.to),
        };
        match self {
            Solution::Diff(diff_solution) => {
                &diff_solution.from == from || &diff_solution.from == to ||
                &diff_solution.to == from || &diff_solution.to == to
            },
            Solution::Tau(tau_mult_solution) => {
                &tau_mult_solution.from == from || &tau_mult_solution.from == to ||
                &tau_mult_solution.to == from || &tau_mult_solution.to == to
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffSolution {
    from: String,
    to: String,
    coeff: i32,
    d: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TauMultSolution {
    from: String,
    to: String,
}


fn find_issues(data: &SyntheticSS, bot_trunc: i32, top_trunc: i32, stem: i32) -> Vec<Issue> {
    let mut issues = vec![];

    let exp = read_rp_csv(bot_trunc, top_trunc);
    let gens = get_filtered_data(data, Category::Synthetic, bot_trunc, top_trunc + 1, 1000, Some(stem)).get_final_page();
    
    let mut compare = HashMap::new();
    for n in gens {
        let g = data.find(&n.0).unwrap();
        let (stem, af, dr) = (g.x, n.1.1, n.1.0);
        if g.x == stem && n.1.0 != Some(0) {
            compare.entry((stem, af)).or_insert(vec![]).push((dr, n.0));
        }
    }

    for j in &mut compare {
        j.1.sort();
    }

    for e in &exp {
        if e.0.0 == stem {
            assert!(compare.contains_key(e.0));
        }
    }

    for (stem_af, gens) in compare {
        if let Some(exp) = exp.get(&stem_af) {
            if gens.iter().map(|x| x.0).collect::<Vec<_>>() != *exp {
                issues.push(Issue {
                    bot_trunc,
                    top_trunc,
                    stem,
                    af: stem_af.1,
                    obs: gens,
                    exp: exp.clone(),
                });
            }
        } else {
            issues.push(Issue {
                bot_trunc,
                top_trunc,
                stem,
                af: stem_af.1,
                obs: gens,
                exp: vec![],
            });
        }
    }

    issues
}


pub fn diff_sorted(
    a: &[(Option<i32>, String)],
    b: &[Option<i32>],
) -> (Vec<(Option<i32>, String)>, Vec<Option<i32>>) {
    let mut only_a = Vec::new();
    let mut only_b = Vec::new();

    let mut i = 0;
    let mut j = 0;

    while i < a.len() && j < b.len() {
        match a[i].0.cmp(&b[j]) {
            std::cmp::Ordering::Less => {
                only_a.push(a[i].clone());
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                only_b.push(b[j].clone());
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
            }
        }
    }

    // remaining tail
    only_a.extend_from_slice(&a[i..]);
    only_b.extend_from_slice(&b[j..]);

    (only_a, only_b)
}

fn find_diffs_for_target(
    data: &SyntheticSS, 
    sources: &HashMap<String, (i32, Option<i32>, i32)>, 
    t_name: &String, 
    t_torsion: Option<i32>, 
    t_af: i32, 
    t_y: i32, 
    exp_torsion: i32) -> Vec<Solution> {
        
    let needed_af = t_af - exp_torsion - 1;
    let mut sols = vec![];

    for (src, (p, tor, af)) in sources {
        if *af == needed_af {
            let g_src = data.find(&src).unwrap();
            let d_r = g_src.y - t_y;
            let mut works = false;
            if d_r > *p {

                if let Some(obs_torsion) = t_torsion {
                    if let Some(src_torsion) = tor {
                        if obs_torsion - exp_torsion <= *src_torsion {
                            works = true;
                        }
                    } else {
                        works = true;
                    }
                } else {
                    if tor.is_none() {
                        works = true;
                    }
                }
            }
            if works {
                sols.push(Solution::Diff(
                    DiffSolution {
                        from: src.clone(),
                        to: t_name.clone(),
                        coeff: exp_torsion,
                        d: d_r,
                    }
                ));
            }
        }
    }
    sols
}

// This function need not neccesarily give a solution to ALL issues
fn find_solutions(data: &SyntheticSS, bottom_trunc: i32, top_trunc: i32, stem: i32, issues: Vec<Issue>) -> Vec<(Issue, Vec<Solution>)> {
    let mut sols = vec![];
    
    let sources = find_possible_sources_for_differentials_in_stem(data, stem + 1);
    let target = get_filtered_data(data, Category::Synthetic, 0, top_trunc + 1, 1000, Some(stem)).get_final_page();

    // If there is a generator somewhere which is too much then it must be some tau mult ??
    // Yes, if and only if the generator therre is a generator some AF higher which must not be torsion
    for i in &issues {
        if i.obs.len() > i.exp.len() {
            // either TAU MULT or COEFF 1 DIFF
            // First TAU MULT
            // Find potential sources
            let diff = diff_sorted(&i.obs, &i.exp);
            if diff.1.len() != 0 {
                eprintln!("There is also another problem in this AF, whoops ?");
            }
            let f = diff.0.iter().find(|x| x.0 == None);
            
            if let Some(target) = f {
                let t_y = data.find(&target.1).unwrap().y;
                for j in &issues {
                    let d = diff_sorted(&j.obs, &j.exp);
                    
                    let stem_diff = j.af - i.af;
                    if stem_diff > 0 {
                        for source in d.0 {
                            let s_y = data.find(&source.1).unwrap().y;
                            if source.0 == Some(stem_diff) && t_y <= s_y {
                                let sol = Solution::Tau(
                                    TauMultSolution {
                                        from: source.1.clone(),
                                        to: target.1.clone(),
                                    }
                                );
                                sols.push((i.clone(), vec![sol]));
                            }
                        }
                    }
                }
            }
        } else if i.obs.len() == i.exp.len() {
            // Some differential should target something ?
            let (obs, exp) = diff_sorted(&i.obs, &i.exp);
            if obs.len() == 1 && exp.len() == 1 {
                if let Some(exp_torsion) = exp[0] {
                    let mut pot_sols = vec![];
                    for (t_name, (t_torsion, t_af)) in &target {
                        if *t_af == i.af && *t_torsion != Some(0) {
                            // Try to find source to do this
                            // We also need to respect module structure
                            let t_y = data.find(&t_name).unwrap().y;
                            let mut d = find_diffs_for_target(data, &sources, t_name, *t_torsion, *t_af, t_y, exp_torsion);
                            pot_sols.append(&mut d);
                        }
                    }
                    sols.push((
                        i.clone(),
                        pot_sols
                    ));
                }
            }
        } else {
            println!("{:?}", i);
            panic!("Wtf was the problem?")
        }
    }

    sols
}

fn filter_multiple_solutions(data: &SyntheticSS, sols: Vec<(Issue, Vec<Solution>)>) -> Vec<(Issue, Solution)> {
    let mut real_sols = vec![];
    for i in 0..sols.len() {
        let (first, last) = sols.split_at(i);
        let ((issue, sol), last) = last.split_first().unwrap();
        if sol.len() == 1 {
            for (_, s) in first.iter().chain(last.iter()) {
                for s in s {
                    if s.names_occur_in_both(&sol[0]) {
                        continue;
                    }
                }
            }
            
            real_sols.push((issue.clone(), sol[0].clone()));
        }
    }
    real_sols
}

fn apply_solutions(data: &mut SyntheticSS, sols: Vec<(Issue, Solution)>) {
    for (i, sol) in sols {
        println!("{:?}", sol);
        match sol {
            Solution::Diff(diff_solution) => {
                data.insert_diff(Differential {
                    from: diff_solution.from,
                    to: diff_solution.to,
                    coeff: diff_solution.coeff,
                    d: diff_solution.d,
                    synthetic: Some(()),
                    proof: None,
                    kind: Kind::Real,
                });
            },
            Solution::Tau(tau_mult_solution) => {
                data.tau_mults.push(TauMult {
                    from: tau_mult_solution.from,
                    to: tau_mult_solution.to,
                    kind: Kind::Real,
                });
            },
        }
    }
}



// This diff_sources might be incorrect as it can change depending on which page a generator lives ?
fn fix_in_trunc_stem(data: &mut SyntheticSS, bot_trunc: i32, top_trunc: i32, stem: i32) -> Result<bool, ()> {
    println!("\nRP{:?}_{:?}:\n", bot_trunc, top_trunc);
    let top_trunc = top_trunc;
    
    let issues = find_issues(data, bot_trunc, top_trunc, stem); 
    let no_issues = issues.len() == 0;

    let sols = find_solutions(data, bot_trunc, top_trunc, stem, issues.clone());    
    
    let filt_sols = filter_multiple_solutions(data, sols.clone());
    
    if filt_sols.is_empty() {
        if no_issues {
            Ok(false)
        } else {
            println!("{:?}", issues);
            Err(())
        }
    } else {
        apply_solutions(data, filt_sols);
        Ok(true)
    }
}

pub fn fix_correctness_by_stem(data: &mut SyntheticSS) {
    for stem in 1..=(MAX_STEM - 1) {
        println!("-----------");
        println!("Stem: {stem}");
        println!("-----------");
        let mut applied_change = true;
        let mut is_error = false;
        
        while applied_change {
            applied_change = false;
            is_error = false;
            for top_trunc in 1..=(stem + 3) {
                let bot_trunc = stem + 3 - top_trunc;
                if bot_trunc & 1 == 1 {
                    match fix_in_trunc_stem(data, bot_trunc, 256, stem) {
                        Ok(r) => applied_change |= r,
                        Err(_) => is_error = true,
                    }
                }
                if top_trunc & 1 == 0 {
                    match fix_in_trunc_stem(data, 1, top_trunc, stem) {
                        Ok(r) => applied_change |= r,
                        Err(_) => is_error = true,
                    }
                }
            }  

            
            if !applied_change && is_error {
                println!("{} {}", applied_change, is_error);
                eprintln!("We could not find a solution to every issue in stem {stem}.");
                return
            }  
        }
    }
}