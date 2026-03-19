use std::{process::exit, time::{self, Duration, Instant}};

use crate::{data::{compare::rp_truncations, curtis::{DATA, STABLE_DATA}}, domain::model::SyntheticSS, io::{cli::process_input, export::{get_log, write_all}}, solve::{action::{Action, process_action, revert_log_and_remake}, ahss::find_ahss_issues, ehp::{apply_ehp_recursively, find_ehp_issues, verify_geometric}, ehp_ahss::set_metastable_range, issues::compare_algebraic_spectral_sequence, solve::auto_deduce}};


mod types;
mod solve;
mod data;
mod io;
mod domain;





const MAX_STEM: i32 = 40;
// TODO: AHSS CURTIS DATA IS VALID UNTIL STEM 48
const MAX_VERIFY_STEM: i32 = 35;

// const MAX_VERIFY_SPHERE: i32 = MAX_VERIFY_STEM + 2;
// const MAX_UNEVEN_INPUT: i32 = (MAX_STEM + 1) * 2;



// pub fn add_final_diagonal(data: &mut OldSyntheticSS) {
//     // Generate the degree zero parts
//     for n in (3..MAX_UNEVEN_INPUT).step_by(4) {
//         let y = n / 2;
        
//         data.generators.push(Generator::new(format!("2(∞)[{}]", y), y, y, 2, 0, None));
//         data.generators.push(Generator::new(format!("1(∞)[{}]", y + 1), y + 1, y + 1, 1, 0, None));

//         data.differentials.push(Differential {
//             from: format!("1(∞)[{}]", y + 1),
//             to: format!("2(∞)[{}]", y),
//             coeff: 0,
//             d: 1,
//             proof: Some("Lifted AEHP differential.".to_string()),
//             synthetic: None,
//             kind: Kind::Real,
//         });
//     }
// }



fn ahss() -> (SyntheticSS, Duration) {
    let original_data = STABLE_DATA.clone();

    let mut log = get_log(true).unwrap_or(vec![]);
    let mut data = revert_log_and_remake(0, &mut log, &STABLE_DATA, true);
    
    write_all(&data, &log, true);

    let mut stem = 2;

    let mut total_input_time = Duration::ZERO;

    println!("");
    println!("----------------------------------------");
    println!("");
    println!("AHSS AHSS AHSS AHSS AHSS");
    println!("");
    println!("----------------------------------------");
    println!("");

    'outer: while stem <= MAX_VERIFY_STEM {
        println!("----------");
        println!("Stem {stem}");
        println!("----------");

        'middle: while let Err(issues) = find_ahss_issues(&data, stem) {
            for issue in &issues {
                // Automatic
                match auto_deduce(&data, &issue) {
                    Ok(action) => {
                        match process_action(&mut data, &action, true) {
                            Ok(_) => {
                                println!("\n{:?}\n", issue);
                                println!("\nAutomatically resolved the issue with the following action: {:?}\n", action);
                                log.push(action);
                                write_all(&data, &log, true);
                                continue 'middle;
                            },
                            Err(_) => {
                                panic!("Automated action was invalid ?? {action:?}");
                            },
                        }
                    },
                    Err(_) => {},
                }
            }

            // Manual
            loop {
                println!("\n\nIssues:");
                for issue in &issues {
                    println!("{:?}", issue);
                }
                println!("");
                let waited_on_input = Instant::now();
                match process_input(true) {
                    Ok(action) => {
                        total_input_time += waited_on_input.elapsed();
                        if let Action::Revert { times } = action {
                            data = revert_log_and_remake(times, &mut log, &original_data, true);
                            write_all(&data, &log, true);
                            stem = 2;
                            break;
                        } else {
                            match process_action(&mut data, &action, true) {
                                Ok(_) => {
                                    log.push(action);
                                    write_all(&data, &log, true);
                                    break;
                                },
                                Err(_) => {
                                    println!("\n---------------");
                                    println!("ACTION ISSUE!");
                                    println!("---------------\n");  
                                    println!("The following data was invalid {:?}. Please try again.", action);  
                                },
                            }
                        }
                    },
                    Err(_) => {
                        total_input_time += waited_on_input.elapsed();
                        println!("\ngoodbye!");  
                        break 'outer;
                    },
                }
            }
        }

        stem += 1;
    }
    
    write_all(&data, &log, true);
    return (data, total_input_time);
}

fn ehp(ahss: &SyntheticSS) -> (SyntheticSS, Duration) {
    let mut original_data = DATA.clone();
    set_metastable_range(&mut original_data, ahss).unwrap();
    
    let mut log = get_log(false).unwrap_or(vec![]);
    let mut data = revert_log_and_remake(0, &mut log, &original_data, false);
    
    write_all(&data, &log, false);

    let mut total_input_time = Duration::ZERO;

    println!("");
    println!("----------------------------------------");
    println!("");
    println!("EHP EHP EHP EHP EHP EHP");
    println!("");
    println!("----------------------------------------");
    println!("");

    let mut stem_minus_sphere = 2;

    'outer: while stem_minus_sphere <= (MAX_VERIFY_STEM + MAX_STEM) {
        print!("{stem_minus_sphere}-");
        
        'middle: while let Err(issues) = find_ehp_issues(&mut data, ahss, stem_minus_sphere) {
            println!("");
            write_all(&data, &log, false);
            for issue in &issues {
                // Automatic
                match auto_deduce(&data, &issue) {
                    Ok(action) => {
                        match process_action(&mut data, &action, false) {
                            Ok(_) => {
                                println!("\n{:?}\n", issue);
                                println!("\nAutomatically resolved the issue with the following action: {:?}\n", action);
                                log.push(action);
                                write_all(&data, &log, false);
                                continue 'middle;
                            },
                            Err(_) => {
                                panic!("Automated action was invalid ?? {action:?}");
                            },
                        }
                    },
                    Err(_) => {},
                }
            }

            // Manual
            loop {
                println!("\n\nIssues:");
                for issue in &issues {
                    println!("{:?}", issue);
                }
                println!("");
                let waited_on_input = Instant::now();
                match process_input(false) {
                    Ok(action) => {
                        total_input_time += waited_on_input.elapsed();
                        if let Action::Revert { times } = action {
                            data = revert_log_and_remake(times, &mut log, &original_data, false);
                            write_all(&data, &log, false);
                            stem_minus_sphere = 2;
                            break;
                        } else {
                            match process_action(&mut data, &action, false) {
                                Ok(_) => {
                                    stem_minus_sphere = 2;
                                    log.push(action);
                                    break;
                                },
                                Err(_) => {
                                    println!("\n---------------");
                                    println!("ACTION ISSUE!");
                                    println!("---------------\n");
                                    println!("The following data was invalid {:?}. Please try again.", action);  
                                },
                            }
                        }
                    },
                    Err(_) => {
                        total_input_time += waited_on_input.elapsed();
                        println!("\ngoodbye!");  
                        break 'outer;
                    },
                }
            }
            apply_ehp_recursively(&mut data, stem_minus_sphere);
            write_all(&data, &log, false);
        }

        write_all(&data, &log, false);
        stem_minus_sphere += 1;
    }
    // add_diffs(&mut data);
    // add_induced_names(&mut data);
    // add_tau_mults(&mut data);

    // data.differentials.sort();
    
    // compute_inductive_generators(&mut data);

    // // add_final_diagonal(&mut data);
    // write_typescript_file("../site/src/data.ts", "", &data).unwrap();
    // println!("\n-----\nTesting if data is well-defined, meaning differentials / multiplications understand have generators which exist.)\n-----\n");
    // if !verify_integrity(&data) {
    //     exit(1);
    // }

    // println!("\n-----\nTesting if Synthetic data is self coherent. (Rows coincide with convergence of SS)\n-----\n");
    // if !verify_self_coherence(&data, MAX_VERIFY_STEM) {
    //     exit(1);
    // }   

    // println!("\n-----\nTesting Geometric stable correctness\n-----\n");
    // if !verify_stable(&data) {
    //     // exit(1);
    // }

    // println!("\n-----\nTesting Geometric unstable correctness\n-----\n");
    // if !verify_geometric(&data) {
    //     // exit(1);
    // }

    // println!("\n-----\nTesting Algebraic correctness (Both stably and unstably) between the synthetic and the algebraic data\n-----\n");
    // if !verify_algebraic(&data) {
    //     // exit(1);
    // }

    // // TODO : Do a verify Hopf Inv One maps thing ?
    

    // add_final_diagonal(&mut data);
    // write_typescript_file("../site/src/data.ts", "", &data).unwrap();
    write_all(&data, &log, false);
    (data, total_input_time)
}

fn main() {
    let start = Instant::now();

    let (ahss, input_time_ahss) = ahss();
    let start_ehp = Instant::now();
    let (ehp, input_time_ehp) = ehp(&ahss);

    verify_geometric(&ehp);

    println!("\nAHSS Compute took: {:.2?}", start.elapsed() - input_time_ahss - start_ehp.elapsed());
    println!("EHP Compute took: {:.2?}", start_ehp.elapsed() - input_time_ehp);
    println!("Compute took: {:.2?}", start.elapsed() - input_time_ahss - input_time_ehp);
    println!("\nInput took: {:.2?}", input_time_ahss + input_time_ehp);
    println!("Program took: {:.2?}", start.elapsed());
}




