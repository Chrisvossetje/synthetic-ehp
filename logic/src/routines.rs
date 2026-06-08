use std::time::{Duration, Instant};

use crate::{
    MAX_STEM, MAX_VERIFY_STEM, data::curtis::{DATA, MODEL, STABLE_DATA, STABLE_MODEL}, domain::model::SyntheticSS, io::{
        cli::process_input,
        export::write_all, import::get_log,
    }, solve::{
        action::{Action, process_action, revert_log_and_remake}, ahss::find_ahss_issues, automated::ahss_solver, automated_ehp::ehp_solver, ehp::{apply_ehp_recursively, find_ehp_issues}, ehp_ahss::{ehp_to_ahss_map, set_metastable_range}, solve::auto_deduce
    }
};

pub fn interactive_ahss() -> (SyntheticSS, Duration) {
    let original_data = &STABLE_DATA.clone();

    let mut log = match get_log(false, true) {
        Ok(log) => log,
        Err(_) => {
            panic!("Log importing was not succesful");
        }
    };
    let mut data = revert_log_and_remake(0, &mut log, &STABLE_MODEL, &STABLE_DATA, true);

    write_all(&data, &STABLE_MODEL, &log, true);

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
        print!("{stem}-");

        'middle: while let Err(issues) = find_ahss_issues(&data, &STABLE_MODEL, stem) {
            println!("");
            for issue in &issues {
                // Automatic
                match auto_deduce(&data, &STABLE_MODEL, &issue) {
                    Ok(actions) => {
                        for action in actions {
                            match process_action(&mut data, &STABLE_MODEL, &action, false) {
                                Ok(_) => {
                                    println!("\n{:?}\n", issue);
                                    println!(
                                        "\nAutomatically resolved the issue with the following action: {:?}\n",
                                        action
                                    );
                                    log.push(action);
                                    write_all(&data, &STABLE_MODEL, &log, false);
                                }
                                Err(_) => {
                                    panic!("Automated action was invalid ?? {action:?}");
                                }
                            }
                        }
                        continue 'middle;
                    }
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
                            data = revert_log_and_remake(times, &mut log, &STABLE_MODEL, &original_data, true);
                            write_all(&data, &STABLE_MODEL, &log, true);
                            stem = 2;
                            break;
                        } else {
                            match process_action(&mut data, &STABLE_MODEL, &action, true) {
                                Ok(_) => {
                                    log.push(action);
                                    write_all(&data, &STABLE_MODEL, &log, true);
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

    write_all(&data, &STABLE_MODEL, &log, true);
    return (data, total_input_time);
}

pub fn interactive_ehp() -> (SyntheticSS, Duration) {

    let mut original_data = DATA.clone();

    let mut ahss_log = match get_log(false, true) {
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

    let ahss = revert_log_and_remake(0, &mut ahss_log, &STABLE_MODEL, &STABLE_DATA, true);

    set_metastable_range(&mut original_data, &ahss).unwrap();

    let mut log = match get_log(false, false) {
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
    let mut data = revert_log_and_remake(0, &mut log, &MODEL, &original_data, false);

    write_all(&data, &MODEL, &log, false);

    let map = ehp_to_ahss_map();

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
            find_ehp_issues(&mut data, &MODEL, &ahss, &STABLE_MODEL, &map, stem_minus_sphere, slanted)
        {
            println!("");

            write_all(&data, &MODEL, &log, false);
            for issue in &issues {
                // Automatic
                match auto_deduce(&data, &MODEL, &issue) {
                    Ok(actions) => {
                        for action in actions {
                            match process_action(&mut data, &MODEL, &action, false) {
                                Ok(_) => {
                                    println!("\n{:?}\n", issue);
                                    println!(
                                        "\nAutomatically resolved the issue with the following action: {:?}\n",
                                        action
                                    );
                                    log.push(action);
                                    write_all(&data, &MODEL, &log, false);
                                }
                                Err(_) => {
                                    panic!("Automated action was invalid ?? {action:?}");
                                }
                            }
                        }
                        continue 'middle;
                    }
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
                            data = revert_log_and_remake(times, &mut log, &MODEL, &original_data, false);
                            write_all(&data, &MODEL, &log, false);
                            stem_minus_sphere = 2;
                            break;
                        } else {
                            match process_action(&mut data, &MODEL, &action, false) {
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

    for i in 2..=MAX_STEM {
        let _ = apply_ehp_recursively(&mut data, &MODEL, i, false);
    }

    write_all(&data, &MODEL, &log, false);
    (data, total_input_time)
}

pub fn automated_ahss(minimal: bool) {
    let start = Instant::now();

    let ahss_log = match get_log(minimal, true) {
        Ok(log) => Some(log),
        Err(_) => None,
    };

    let (ahss_log, ahss) = ahss_solver(ahss_log);
    write_all(&ahss, &STABLE_MODEL, &ahss_log, true);

    println!("\nProgram took: {:.2?}\n", start.elapsed());
}

pub fn automated_ehp(minimal: bool) -> SyntheticSS {
    let start = Instant::now();

    let mut ahss_log = match get_log(false, true) {
        Ok(log) => log,
        Err(_) => vec![],
    };

    let ahss = revert_log_and_remake(0, &mut ahss_log, &STABLE_MODEL, &STABLE_DATA, true);

    let ehp_log = match get_log(minimal, false) {
        Ok(log) => log,
        Err(_) => {
            panic!("Log importing was not succesful");
        }
    };

    let (ehp_log, ehp) = ehp_solver(&ahss, Some(ehp_log));

    write_all(&ehp, &MODEL, &ehp_log, false);

    println!("\nProgram took: {:.2?}\n", start.elapsed());

    ehp
}
