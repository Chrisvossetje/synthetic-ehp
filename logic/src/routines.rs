//! Top-level entry points wiring the pieces together. The `interactive_*`
//! routines run the verify → resolve loop with a human at the keyboard
//! (auto-deducing what they can, prompting for the rest), while the
//! `automated_*` routines run the same loop fully unattended via the solvers.
//! Both replay a saved log first, so a session resumes where it left off.

use std::{collections::HashMap, format, println, time::{Duration, Instant}, todo, unreachable, vec};

use itertools::Itertools;

use crate::{
    MAX_STEM, MAX_VERIFY_STEM, data::{curtis::{DATA, MODEL, STABLE_DATA, STABLE_MODEL}, naming::{STABLE_NAMES, add_sphere_to_tag, generated_by_name, name_get_tag}}, domain::model::SyntheticSS, io::{
        cli::process_input,
        export::{write_all, write_tag_translation}, import::get_log,
    }, solve::{
        action::{Action, process_action, revert_log_and_remake}, ahss::find_ahss_issues, automated_ahss::ahss_solver, automated_ehp::ehp_solver, ehp::{apply_ehp_recursively, find_ehp_issues}, ehp_ahss::{ehp_to_ahss_map, set_metastable_range}, solve::auto_deduce
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
            vec![]
        }
    };

    let ahss = revert_log_and_remake(0, &mut ahss_log, &STABLE_MODEL, &STABLE_DATA, true);

    set_metastable_range(&mut original_data, &ahss).unwrap();

    let mut log = match get_log(false, false) {
        Ok(log) => log,
        Err(_) => {
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


type NameTranslate = (String, Vec<i32>, i32);

pub fn export_tag_translation() {
    // First we give names to all stably surviving elements.
    // Should be initialized manually whenever a bidegree has double elements
    let mut id_to_stable_name = HashMap::from([
        (159, "h_1d_0".to_owned()),
        (175, "h_0^4h_4 + h_1d_0".to_owned()),
        (255, "f_0".to_owned()),
        (262, "h_0^2h_2h_4".to_owned()),
        (484, "h_1Pd_0".to_owned()),
        (520, "h_0^2i + h_1Pd_0".to_owned()),
        
        // Stem 31
        (1373, "h_1h_4^2".to_owned()),
        (1420, "h_0^2h_5".to_owned()),
        (1357, "n".to_owned()),
        (1415, "h_0^4h_5".to_owned()),
        (1318, "d_0e_0".to_owned()),
        (1405, "d_0e_0 + h_0^7h_5".to_owned()),
        (1306, "h_0d_0e_0".to_owned()),
        (1397, "h_0^8h_5".to_owned()),
        (1300, "h_0^2d_0e_0".to_owned()),
        (1385, "h_0^9h_5".to_owned()),
        (1301, "h_1P^2d_0".to_owned()),
        (1354, "h_0^12h_5 + h_1P^2d_0".to_owned()),

        // Stem 37 and rest TODO        
    ]);

    for (index, m) in MODEL.enumerate() {
        if m.dies.is_none() && m.stem <= MAX_STEM {

            if !id_to_stable_name.contains_key(&index) {
                if let Some(l) = STABLE_NAMES.get(&(m.stem,m.af)) {
                    if l.len() > 1 {
                        println!("{}, index {index} in ({},{}) stable name could not be determined reliably. Please manually add to id_to_stable_name data,", m.name, m.stem, m.af)
                    } else {
                        id_to_stable_name.insert(index, l[0].to_owned());
                    }
                } else {
                    unreachable!("We shouldn't see an element in ({},{}) not visible in Lin's chart.", m.stem, m.af)
                }
            }
        }
    }


    // Unstable EHP to algebraic names
    let mut id_to_algebraic_name: Vec<Option<NameTranslate>> = vec![None; MODEL.gens().len()]; 

    id_to_algebraic_name[0] = Some(("1".to_owned(), vec![], 0));


    for iterate in 0..=(MAX_STEM + 1) {
        // We shouldn't reallly go by stem, but by diagonal
        // Else we can't guarantee that a name has been defined in time. 
        for y in 1..=(MAX_STEM+1) {
          
            let stem = iterate + y;

            if stem > MAX_STEM + 1 {
                continue;
            }


            for index in MODEL.gens_id_in_stem_y(stem, y) {
                let g = MODEL.get(*index);
                let tag = name_get_tag(&g.name);
                let (name, unadmissable) = if tag == "" {
                    ("1".to_owned(), vec![])
                } else {
                    let generator = generated_by_name(tag);
                    let gen_id = MODEL.get_index(&generator);
    
                    if MODEL.get(gen_id).dies.is_none() {
                        // Get the name of the stable element
                        let Some(name) = id_to_stable_name.get(&gen_id) else {
                            // An ambiguous stable basis still needs a manual choice.
                            continue;
                        };
                        (name.clone(), vec![])
                        
                    } else {
                        // Find element which killed this thing
                        let source_id = DATA.in_diffs[gen_id][0];

                        if let Some((tag, series, sphere)) = &id_to_algebraic_name[source_id] {
                            let mut new_series = series.clone();
                            new_series.push(*sphere);
                            (tag.clone(), new_series)
                        } else {
                            // This class depends on an unresolved stable basis choice.
                            continue;
                        }
                    }
                };
                id_to_algebraic_name[*index] = Some((name, unadmissable, y));
            }

        }
    }    

    let mut translations = std::collections::BTreeMap::new();
    for (index, name) in id_to_algebraic_name.into_iter().enumerate() {
        if let Some((tag, series, _sphere)) = name {
            let curtis_tag = name_get_tag(&MODEL.get(index).name).to_owned();
            let algebraic_tag =
            if series.len() > 0 {
                format!("{tag}<{}>", series.iter().map(|y| y.to_string()).join(", "))
            } else {
                tag
            };
            if let Some(previous) = translations.insert(curtis_tag.clone(), algebraic_tag.clone()) {
                assert_eq!(previous, algebraic_tag, "Inconsistent translation for Curtis tag {curtis_tag}");
            }
        }
    }
    // An AHSS tag is the Curtis word of a stable AEHP element. Read its name
    // directly from the stable assignments, without the unstable EHP recursion.
    let mut stable_translations = std::collections::BTreeMap::from([
        (String::new(), "1".to_owned()),
    ]);
    for generator in STABLE_MODEL.gens() {
        let tag = name_get_tag(&generator.name);
        if tag.is_empty() {
            continue;
        }
        if let Some(name) = MODEL.try_index(&generated_by_name(tag))
            .and_then(|index| id_to_stable_name.get(&index))
        {
            stable_translations.insert(tag.to_owned(), name.clone());
        }
    }
    write_tag_translation(&translations, &stable_translations)
        .expect("Could not export website tag translations");
    println!("Exported {} EHP and {} AHSS tag translations to site/src/tag_translation.ts",
        translations.len(), stable_translations.len());
}
