use itertools::Itertools;

use crate::{MAX_STEM, ahss, data::{compare::{algebraic_rp, rp_truncations, synthetic_rp}, curtis::STABLE_DATA}, domain::{model::{FromTo, SyntheticSS}, process::try_compute_pages}, solve::{action::{Action, revert_log_and_remake}, ahss::ahss_synthetic_e1_issue, generate::{get_a_diff, get_a_tau}, issues::{Issue, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic, synthetic_issue_is_tau_structure_issue}, solve::auto_deduce}, types::Kind};


fn check_issue(data: &SyntheticSS, stem: i32, bot_trunc: i32, top_trunc: i32) -> Result<(), Vec<Issue>> {
    for &(synthetic, bt, tt) in rp_truncations() {
        if (top_trunc == tt || (stem + 1 == top_trunc  && tt == 256)) && bot_trunc == bt {
            if synthetic {
                let pages = try_compute_pages(data, bt, tt, stem, stem)?;
                
                let observed = pages.convergence_at_stem(data, stem);

                compare_synthetic(
                    &observed,
                    synthetic_rp(bt, tt),
                    bt,
                    top_trunc,
                    stem,
                )?;
            } else {
                let pages = try_compute_pages(data, bt, tt, stem - 1, stem)?;
                
                let observed = pages.algebraic_convergence_at_stem(data, stem);

                compare_algebraic(
                    &observed,
                    algebraic_rp(bt, tt),
                    bt,
                    tt,
                    stem,
                )?;
            }
            compare_algebraic_spectral_sequence(data, stem, bt, tt, true)?;
        }
    }
    
    Ok(())
}

// TODO: Is a stem wise approach ENOUGH to conclude E1 stuff, lets hope E1 stuff can always be resolved on the current stem (sadly, i know this is not true :( )?
fn ahss_iterate(mut data: SyntheticSS, alg_data: &Vec<Vec<Vec<FromTo>>>, log: &mut Vec<Action>, stem: i32, top_trunc: i32, target_y: i32, depth: i32) -> Result<(), String> {    

    // The log here does nothing special, we do not rely on it.
    // This also means that we cannot reconstruct our data from this log
    // The reason this is fine is because we just look at a single stem
    // So any James periodicity additions will come later down the line

    let option = get_a_diff(&data, top_trunc, target_y, stem);

    // Should only need first option here
    if let Some(d) = option {
        let (from_name, to_name) = data.get_names(d.from, d.to);
        
        if depth == 0 {
            println!("Trying diff: {} | {}", from_name, to_name);
        }
        if depth >= 20 {
            println!("HOI!");
        }

        let mut with_data = data.clone();

        // // Check if diff option was algebraic
        // if alg_data.proven_from_to.contains_key(&(d.from, d.to)) {
        //     with_data.add_diff(d.from, d.to, None, Kind::Real);
        // } else {
        // 
        // }
        with_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Real);

        let with = ahss_iterate(with_data, alg_data, log, stem, top_trunc, target_y, depth + 1);

        if let Err(e) = with {
            // DISPROOF !
            let (from_name, to_name) = data.get_names(d.from, d.to);
            
            // Commit choice !
            if depth == 0 {
                log.push(Action::AddDiff { from: from_name, to: to_name, proof: Some(e.clone()), kind: Kind::Fake });
            }

            data.add_diff(d.from, d.to, Some(e), Kind::Fake);
            
            // And iterate further
            return ahss_iterate(data, alg_data, log, stem, top_trunc, target_y, depth)
        } else {
            let mut without_data = data.clone();
            without_data.disproven_from_to.insert((d.from, d.to), None);
            let without  = ahss_iterate(without_data, alg_data, log, stem, top_trunc, target_y, depth + 1);

            let (from_name, to_name) = data.get_names(d.from, d.to);
        
            if without.is_ok() && depth == 0 {
                println!("WITH OR WITHOUT ARE BOTH FINE FOR THE DIFFERENTIAL: {:?} | {:?}", d, data.get_names(d.from, d.to));
            }

            // If with and without are both Ok then we continue WITHOUT the differential
            // But we do remember that we don't know the result of this diff    
            let kind = if without.is_err() { Kind::Real } else { Kind::Unknown };
            let proof = if let Err(e) = without { Some(e) } else { None };
            
            data.add_diff(d.from, d.to, proof.clone(), kind);

            // Commit choice !
            if depth == 0 { 
                log.push(Action::AddDiff { from: from_name, to: to_name, proof, kind });
            }

            return ahss_iterate(data, alg_data, log, stem, top_trunc, target_y, depth)
        }
    }

    let option = get_a_tau(&data, top_trunc, target_y, stem);

    // Should only need first option here
    if let Some(d) = option {
        let (from_name, to_name) = data.get_names(d.from, d.to);
        
        if depth == 0 {
            println!("Trying tau: {} | {}", from_name, to_name);
        }

        let mut with_data = data.clone();

        with_data.add_ext_tau(d.from, d.to, d.af, Some("".to_string()), Kind::Real);

        let with = ahss_iterate(with_data, alg_data, log, stem, top_trunc, target_y, depth + 1);

        if let Err(e) = with {
            // DISPROOF !
            let (from_name, to_name) = data.get_names(d.from, d.to);
            
            // Commit choice !
            if depth == 0 {
                log.push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof: e.clone(), kind: Kind::Fake });
            }

            data.add_ext_tau(d.from, d.to, d.af, Some(e), Kind::Fake);
            
            // And iterate further
            return ahss_iterate(data, alg_data, log, stem, top_trunc, target_y, depth)
        } else {
            let mut without_data = data.clone();
            without_data.disproven_from_to.insert((d.from, d.to), None);
            let without = ahss_iterate(without_data, alg_data, log, stem, top_trunc, target_y, depth + 1);

            let (from_name, to_name) = data.get_names(d.from, d.to);
        
            if without.is_ok() && depth == 0 {
                println!("WITH OR WITHOUT ARE BOTH FINE FOR THE EXT TAU: {:?} | {:?}", d, data.get_names(d.from, d.to));
            }

            // If with and without are both Ok then we continue WITHOUT the differential
            // But we do remember that we don't know the result of this diff    
            let kind = if without.is_err() { Kind::Real } else { Kind::Unknown };
            let proof = if let Err(e) = without { e } else { "".to_string() };
            
            data.add_ext_tau(d.from, d.to, d.af, Some(proof.clone()), kind);

            // Commit choice !
            if depth == 0 { 
                log.push(Action::AddExt { from: from_name, to: to_name, af: d.af, proof, kind });
            }

            return ahss_iterate(data, alg_data, log, stem, top_trunc, target_y, depth)
        }
    }

    check_issue(&data, stem, target_y, top_trunc).map_err(|x| x.into_iter().map(|f| format!("{:?}", f)).join(", "))?;
    
    if target_y != 0 {
        // As we are moving up a page for possible diffs,
        // we should add all adams differentials which could arise from here
        
        let d_y = top_trunc - target_y + 1;
        for &(from, to) in &alg_data[stem as usize][d_y as usize]  {
            data.add_diff(from, to, None, Kind::Real);
        }
        
        return ahss_iterate(data, alg_data, log, stem, top_trunc, target_y-1, depth)
    }


    if top_trunc == stem + 1 {
        return Ok(());
    } else {
        ahss_iterate(data, alg_data, log, stem, top_trunc + 1, top_trunc, depth)
    }
}

fn e1_loop(ahss: SyntheticSS, partial_ahss: &mut SyntheticSS, alg_data: &Vec<Vec<Vec<FromTo>>>, log: &mut Vec<Action>, stem: i32) -> Result<(), String> {
    let mut proper_issues = vec![];
    match ahss_synthetic_e1_issue(&ahss, stem) {
        Ok(_) => {},
        Err(issues) => {
            for i in issues {
                // First we solve all the e1 issues we can resolve 
                match auto_deduce(&ahss, &i){
                    Ok(a) => log.push(a),
                    Err(_) => proper_issues.push(i),
                }
            }
        },
    }

    if proper_issues.len() != 0 {
        return Err("Can't decide on E1 stuff".to_string());
    }
    
    let ahss = revert_log_and_remake(0, log, &partial_ahss, true);

    let res = ahss_iterate(ahss, &alg_data, log, stem, 2, 1, 0);
    println!("Stem {stem}: {res:?}");

    res
}

pub fn ahss_solver() -> (Vec<Action>, SyntheticSS) {
    let alg_ahss = STABLE_DATA.clone();
    let mut partial_ahss = SyntheticSS::empty(alg_ahss.model.clone());
    // We should add all d1's from the algebraic data
    
    let mut alg_data = vec![vec![vec![]; (MAX_STEM + 1) as usize]; (MAX_STEM + 1) as usize];

    for (&(from, to), _) in &alg_ahss.proven_from_to {
        let d_y = alg_ahss.model.y(from) - alg_ahss.model.y(to);
        if d_y == 1 {
            partial_ahss.add_diff(from, to, None, Kind::Real);
        } else {
            let stem = alg_ahss.model.stem(to);
            alg_data[stem as usize][d_y as usize].push((from, to));
        }
    }

    let mut log = vec![
        Action::AddInt { 
            from: "6 5 3[2]".to_string(), 
            to: "6 2 3 3[2]".to_string(), 
            page: 2, 
            proof: "Unique solution to RP1_2".to_string(), 
            kind: Kind::Real 
        }
    ]; 
    
    for stem in 2..=15 {
        let ahss = revert_log_and_remake(0, &mut log, &partial_ahss, true);
        match e1_loop(ahss, &mut partial_ahss, &alg_data, &mut log, stem) {
            Ok(_) => {},
            Err(_) => break,
        }
        for ds in &alg_data[stem as usize] {
            for &(from, to) in ds {
                partial_ahss.add_diff(from, to, None, Kind::Real);
            }
        }
    }

    let ahss = revert_log_and_remake(0, &mut log, &partial_ahss, true);
    (log, ahss)
}