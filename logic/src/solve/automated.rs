use itertools::Itertools;

use crate::{ahss, data::{compare::{algebraic_rp, rp_truncations, synthetic_rp}, curtis::STABLE_DATA}, domain::{model::SyntheticSS, process::try_compute_pages}, solve::{action::{Action, revert_log_and_remake}, generate::get_all_diffs, issues::{Issue, compare_algebraic, compare_algebraic_spectral_sequence, compare_synthetic, synthetic_issue_is_tau_structure_issue}}, types::Kind};


fn check_issues(data: &SyntheticSS, stem: i32, super_top_trunc: i32) -> Result<(), Vec<Issue>> {
    for &(synthetic, bot_trunc, top_trunc) in rp_truncations() {
        if top_trunc == super_top_trunc || (stem + 1 == super_top_trunc  && top_trunc == 256) {
            if synthetic {
                let pages = try_compute_pages(data, bot_trunc, top_trunc, stem, stem)?;
                
                let observed = pages.convergence_at_stem(data, stem);

                compare_synthetic(
                    &observed,
                    synthetic_rp(bot_trunc, top_trunc),
                    bot_trunc,
                    top_trunc,
                    stem,
                ).map_err(|x| {
                    println!("Tau issues: {}", synthetic_issue_is_tau_structure_issue(&x));
                    x
                })?;
            } else {
                let pages = try_compute_pages(data, bot_trunc, top_trunc, stem - 1, stem)?;
                
                let observed = pages.algebraic_convergence_at_stem(data, stem);

                compare_algebraic(
                    &observed,
                    algebraic_rp(bot_trunc, top_trunc),
                    bot_trunc,
                    top_trunc,
                    stem,
                )?;
            }
            compare_algebraic_spectral_sequence(data, stem, bot_trunc, top_trunc, true)?;
        }
    }
    
    Ok(())
}

// TODO: Is a stem wise approach ENOUGH to conclude E1 stuff, lets hope E1 stuff can always be resolved on the current stem (sadly, i know this is not true :( )?
fn ahss_iterate(mut data: SyntheticSS, alg_data: &SyntheticSS, log: &mut Vec<Action>, stem: i32, top_trunc: i32, depth: i32) -> Result<(), String> {    

    // The log here does nothing special, we do not rely on it.
    // This also means that we cannot reconstruct our data from this log
    // The reason this is fine is because we just look at a single stem
    // So any James periodicity additions will come later down the line

    let options = get_all_diffs(&data, top_trunc, stem);

    for d in options {
        let (from_name, to_name) = data.get_names(d.from, d.to);

        let mut with_data = data.clone();

        // Check if diff option was algebraic
        if alg_data.proven_from_to.contains_key(&(d.from, d.to)) {
            with_data.add_diff(d.from, d.to, None, Kind::Real);
        } else {
            with_data.add_diff(d.from, d.to, Some("".to_string()), Kind::Real);
        }

        let with = ahss_iterate(with_data, alg_data, log, stem, top_trunc, depth + 1);

        if let Err(e) = with && depth == 0 {
            // DISPROOF !
            let (from_name, to_name) = data.get_names(d.from, d.to);
            // let proof = if alg_data.proven_from_to.contains_key(&(d.from, d.to)) {
            //     None
            // } else {
            //     Some(e)
            // };
            log.push(Action::AddDiff { from: from_name, to: to_name, proof: e.clone(), kind: Kind::Fake });
            data.add_diff(d.from, d.to, Some(e), Kind::Fake);
            
            // And iterate further
            return ahss_iterate(data, alg_data, log, stem, top_trunc, 0)
        } else {
            let mut without_data = data.clone();
            without_data.disproven_from_to.insert((d.from, d.to), None);
            let without  = ahss_iterate(without_data, alg_data, log, stem, top_trunc, depth + 1);
            if let Err(e) = without {
                if depth == 0 {
                    // PROOF !
                    let (from_name, to_name) = data.get_names(d.from, d.to);
                    let proof = if alg_data.proven_from_to.contains_key(&(d.from, d.to)) {
                        None
                    } else {
                        log.push(Action::AddDiff { from: from_name, to: to_name, proof: e.clone(), kind: Kind::Real });
                        Some(e)
                    };
                    data.add_diff(d.from, d.to, proof, Kind::Real);

                    // And iterate
                    return ahss_iterate(data, alg_data, log, stem, top_trunc, 0)
                }
            }
    
            println!("WITH OR WITHOUT ARE BOTH FINE FOR THE DIFFERENTIAL : {:?} | {:?}", d, data.get_names(d.from, d.to));
            // Continuing without, adding unknown to the diff ?
        }
    }

    check_issues(&data, stem, top_trunc).map_err(|x| x.into_iter().map(|f| format!("{:?}", f)).join(", "))?;

    if top_trunc == stem + 1 {
        return Ok(());
    } else {
        ahss_iterate(data, alg_data, log, stem, top_trunc + 1, depth)
    }
}


pub fn ahss_solver() -> (Vec<Action>, SyntheticSS) {
    let alg_ahss = STABLE_DATA.clone();
    let empty_ahss = SyntheticSS::empty(alg_ahss.model.clone());

    let mut ahss = empty_ahss.clone();
    

    let mut log = vec![]; 

    for stem in 2..=4 {
        let res = ahss_iterate(ahss, &alg_ahss, &mut log, stem, 2, 0);
        println!("Stem {stem}: {res:?}");
    
        ahss = revert_log_and_remake(0, &mut log, &empty_ahss, true);
    }


    (log, ahss)
}