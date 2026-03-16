use std::time::{self, Duration, Instant};

use crate::{data::curtis::{generate_algebraic_data, generate_stable_algebraic_data}, domain::model::SyntheticSS, io::{cli::process_input, export::{get_log, write_all}}, solve::{action::{Action, process_action, revert_log_and_remake}, ehp_ahss::{self, set_metastable_range}, issues::find_ahss_issue, solve::auto_deduce}};


mod types;
mod solve;
mod data;
mod io;
mod domain;





const MAX_STEM: i32 = 35;
// TODO: AHSS CURTIS DATA IS VALID UNTIL STEM 48
const MAX_VERIFY_STEM: i32 = 25;

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
    let original_data = generate_stable_algebraic_data();

    let mut log = get_log(true).unwrap_or(vec![]);
    let mut data = revert_log_and_remake(0, &mut log, &original_data, true);
    
    write_all(&data, &log, true);

    let mut stem = 2;

    let mut total_input_time = Duration::ZERO;

    'outer: while stem <= MAX_VERIFY_STEM {
        println!("----------");
        println!("Stem {stem}");
        println!("----------");

        'middle: while let Err(issues) = find_ahss_issue(&data, stem) {
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

fn ehp(ahss: &SyntheticSS) -> SyntheticSS {
    let mut original_data = generate_algebraic_data(); 
    set_metastable_range(&mut original_data, ahss);

    let mut log = get_log(false).unwrap_or(vec![]);
    let mut data = revert_log_and_remake(0, &mut log, &original_data, false);
    
    write_all(&data, &log, false);

    let mut stem = 2;

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
    data
}

fn main() {
    let start = Instant::now();

    let (ahss, input_time) = ahss();
    let e = ehp(&ahss);

    // verify_ehp_to_ahss(&e, &a);
    
    println!("\nCompute took: {:.2?}", start.elapsed() - input_time);
    println!("Input took: {:.2?}", input_time);
    println!("Program took: {:.2?}", start.elapsed());
}




