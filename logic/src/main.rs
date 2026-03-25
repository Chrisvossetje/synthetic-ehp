use std::time::{Duration, Instant};

use crate::{
    data::curtis::{DATA, STABLE_DATA},
    domain::{model::SyntheticSS, process::compute_pages},
    io::{
        cli::process_input,
        export::{get_log, write_all},
    },
    solve::{
        action::{Action, process_action, revert_log_and_remake},
        ahss::find_ahss_issues,
        ehp::{apply_ehp_recursively, find_ehp_issues, verify_geometric},
        ehp_ahss::{ehp_to_ahss_map, set_metastable_range},
        solve::auto_deduce,
    },
};

mod data;
mod domain;
mod io;
mod solve;
mod types;

const MAX_STEM: i32 = 48;
// TODO: AHSS CURTIS DATA IS VALID UNTIL STEM 48
// TODO: It seems EHP curtis data is also valid until +- STEM 48
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

fn print_convergence_of_stem(data: &SyntheticSS) {
    let (p, _) = compute_pages(data, 0, 6, 22, 22, true);

    println!("{:?}", p.convergence_at_stem(data, 22));
}

fn ahss() -> (SyntheticSS, Duration) {
    let original_data = STABLE_DATA.clone();

    let mut log = match get_log(true) {
        Ok(log) => log,
        Err(_) => {
            println!("Log importing was not succesful");
            println!("Log importing was not succesful");
            println!("Log importing was not succesful");
            println!("Log importing was not succesful");
            println!("Log importing was not succesful");
            vec![]
        }
    };
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

    // get_all_diffs(&mut data, 7, 13);
    // write_all(&data, &log, true);
    // todo!();

    'outer: while stem <= MAX_VERIFY_STEM {
        print!("{stem}-");

        'middle: while let Err(issues) = find_ahss_issues(&data, stem) {
            println!("");
            for issue in &issues {
                // Automatic
                match auto_deduce(&data, &issue) {
                    Ok(action) => match process_action(&mut data, &action, true) {
                        Ok(_) => {
                            println!("\n{:?}\n", issue);
                            println!(
                                "\nAutomatically resolved the issue with the following action: {:?}\n",
                                action
                            );
                            log.push(action);
                            write_all(&data, &log, true);
                            continue 'middle;
                        }
                        Err(_) => {
                            panic!("Automated action was invalid ?? {action:?}");
                        }
                    },
                    Err(_) => {}
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
                                }
                                Err(_) => {
                                    println!("\n---------------");
                                    println!("ACTION ISSUE!");
                                    println!("---------------\n");
                                    println!(
                                        "The following data was invalid {:?}. Please try again.",
                                        action
                                    );
                                }
                            }
                        }
                    }
                    Err(_) => {
                        total_input_time += waited_on_input.elapsed();
                        println!("\ngoodbye!");
                        break 'outer;
                    }
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

    let mut log = match get_log(false) {
        Ok(log) => log,
        Err(_) => {
            println!("Log importing was not succesful");
            println!("Log importing was not succesful");
            println!("Log importing was not succesful");
            println!("Log importing was not succesful");
            println!("Log importing was not succesful");
            vec![]
        }
    };
    let mut data = revert_log_and_remake(0, &mut log, &original_data, false);

    write_all(&data, &log, false);

    let map = ehp_to_ahss_map(&data, ahss);

    let mut total_input_time = Duration::ZERO;

    println!("");
    println!("----------------------------------------");
    println!("");
    println!("EHP EHP EHP EHP EHP EHP");
    println!("");
    println!("----------------------------------------");
    println!("");

    let mut stem_minus_sphere = 2;
    let slanted = true;

    'outer: while stem_minus_sphere <= (MAX_VERIFY_STEM + MAX_STEM) {
        print!("{stem_minus_sphere}-");

        'middle: while let Err(issues) =
            find_ehp_issues(&mut data, ahss, &map, stem_minus_sphere, slanted)
        {
            println!("");

            write_all(&data, &log, false);
            for issue in &issues {
                // Automatic
                match auto_deduce(&data, &issue) {
                    Ok(action) => match process_action(&mut data, &action, false) {
                        Ok(_) => {
                            println!("\n{:?}\n", issue);
                            println!(
                                "\nAutomatically resolved the issue with the following action: {:?}\n",
                                action
                            );
                            log.push(action);
                            write_all(&data, &log, false);
                            continue 'middle;
                        }
                        Err(_) => {
                            panic!("Automated action was invalid ?? {action:?}");
                        }
                    },
                    Err(_) => {}
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
                                }
                                Err(_) => {
                                    println!("\n---------------");
                                    println!("ACTION ISSUE!");
                                    println!("---------------\n");
                                    println!(
                                        "The following data was invalid {:?}. Please try again.",
                                        action
                                    );
                                }
                            }
                        }
                    }
                    Err(_) => {
                        total_input_time += waited_on_input.elapsed();
                        println!("\ngoodbye!");
                        break 'outer;
                    }
                }
            }
        }
        stem_minus_sphere += 1;
    }

    // // TODO : Do a verify Hopf Inv One maps thing ?
    // THIS CAN BE VERIFIED BY JUST NOT ALLOWING DIFF / THINGS FROM y = 3 / 7

    for i in 2..=MAX_STEM {
        apply_ehp_recursively(&mut data, i, false);
    }

    // add_final_diagonal(&mut data);
    write_all(&data, &log, false);
    (data, total_input_time)
}

// This check whether the data maps its original AF to the original AF target when both source and target between a AEHP diff exist.
// BUT THIS IS NOT IMPORTANT SOMEHOW !
fn temp_lol(data: &SyntheticSS) {
    let (page, _) = compute_pages(data, 0, 256, 0, MAX_STEM, true);

    for (&(from, to), proof) in &data.proven_from_to {
        if proof.is_none() {
            if page.element_in_pages(from) && page.element_in_pages(to) {
                let d_y = data.model.y(from) - data.model.y(to);
                let from_af = data.model.original_af(from);
                let to_af = data.model.original_af(to);

                if page.element_at_page(d_y, from).0 != from_af
                    || page.element_at_page(d_y, to).0 != to_af
                {
                    if data.model.stem(from) <= MAX_VERIFY_STEM {
                        println!(
                            "STEM: {} | Issue from: {:?} \n to: {:?}\n\n",
                            data.model.stem(to),
                            data.model.get(from),
                            data.model.get(to)
                        );
                    }
                }
            }
        }
    }
}

fn main() {
    let start = Instant::now();

    let (ahss, input_time_ahss) = ahss();


    let start_ehp = Instant::now();
    let (ehp, input_time_ehp) = ehp(&ahss);

    verify_geometric(&ehp);

    println!(
        "\nAHSS Compute took: {:.2?}",
        start.elapsed() - input_time_ahss - start_ehp.elapsed()
    );
    println!(
        "EHP Compute took: {:.2?}",
        start_ehp.elapsed() - input_time_ehp
    );
    println!(
        "Compute took: {:.2?}",
        start.elapsed() - input_time_ahss - input_time_ehp
    );
    println!("\nInput took: {:.2?}", input_time_ahss + input_time_ehp);
    println!("Program took: {:.2?}", start.elapsed());
}
