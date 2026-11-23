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

pub fn interactive_ahss(import_log: bool) -> (SyntheticSS, Duration) {
    let original_data = &STABLE_DATA.clone();

    let mut log = if import_log { match get_log(false, true) {
        Ok(log) => log,
        Err(_) => {
            panic!("Log importing was not succesful")
        }
    }} else { vec![] };

    println!("{:?}", log);

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

pub fn interactive_ehp(import_log: bool) -> (SyntheticSS, Duration) {

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

    let mut log = if import_log  { match get_log(false, false) {
        Ok(log) => log,
        Err(_) => {
            println!("Log importing was not succesful");
            vec![]
        }
    }} else { vec![] };
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
    // In bidegrees of dimension greater than one, compare stable AEHP products
    // with the S0 Adams E2 products in Lin's Kervaire chart.
    // https://waynelin92.github.io/ss/plot.html?diagram=kervaire&data=S0
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
        (1354, "h_0^{12}h_5 + h_1P^2d_0".to_owned()),
        
        // Stem 37,38,39
        (2205, "h_1t".to_owned()),
        (2253, "h_0^2x + h_1t".to_owned()),
        (2217, "e_0g + h_0^3x".to_owned()),
        (2231, "h_0^3x".to_owned()),
        (2510, "e_1".to_owned()),
        (2525, "h_0^2h_3h_5".to_owned()),
        (2383, "h_1x".to_owned()),
        (2449, "x_{38,6}".to_owned()),
        (2579, "h_1P^3d_0".to_owned()),
        (2658, "h_0^2P^2i + h_1P^3d_0".to_owned()),
        
        // Stem 40, 45
        (2896, "f_1".to_owned()),
        (2839, "h_0f_1".to_owned()),
        (2812, "h_0^2f_1".to_owned()),
        (2935, "h_1^2h_3h_5".to_owned()),
        (2931, "h_1c_0h_5".to_owned()),
        (2922, "Ph_1h_5".to_owned()),
        (4165, "h_1g_2".to_owned()),
        // d_0 times h_5 is the sum of IDs 4165 and 4206.
        (4206, "d_0h_5 + h_1g_2".to_owned()),

        // Stem 47. Multiplication by Ph_1 sends both 4671 and 4740 to 8283.
        (4671, "h_1M h_1".to_owned()),
        (4740, "Pc_0h_5 + h_1M h_1".to_owned()),
        (4651, r"P\Delta h_1d_0".to_owned()),
        (4896, r"\Delta h_0^2i".to_owned()),
        (4696, "d_0P^2e_0".to_owned()),
        (4872, r"h_0^3(\Delta h_0^2i+P\Delta h_1d_0)".to_owned()),
        (4667, "h_0d_0P^2e_0".to_owned()),
        (4850, r"h_0d_0P^2e_0 + h_0^4(\Delta h_0^2i+P\Delta h_1d_0)".to_owned()),
        (4655, "h_0^2d_0P^2e_0".to_owned()),
        (4819, r"h_0^2d_0P^2e_0 + h_0^5(\Delta h_0^2i+P\Delta h_1d_0)".to_owned()),
        (4656, "h_1x_{46,20}".to_owned()),
        (4753, r"h_1x_{46,20} + h_0^8(\Delta h_0^2i+P\Delta h_1d_0)".to_owned()),

        // Stem 48. Products through stem 60 fix 5002 + 5052 as h_0^2e_0h_5.
        // Use Lin's x_{48,7} for the earlier AEHP filtration.
        (5002, "x_{48,7}".to_owned()),
        (5052, "x_{48,7} + h_0^2e_0h_5".to_owned()),
        (4965, "h_0^2x_{48,7}".to_owned()),
        (4991, "h_1Pc_0h_5 + h_0^2x_{48,7}".to_owned()),

        // Stem 53
        (6866, r"d_0\Delta h_1d_0".to_owned()),
        (7040, r"h_0^3M\!P".to_owned()),
        (6904, r"d_0^2Pe_0 + h_0^6M\!P".to_owned()),
        (6929, "d_0^2Pe_0".to_owned()),

        // Stems 54 and 55. Products through stem 60 allow adding 7305 to 7584.
        (7305, "h_0^2ih_5".to_owned()),
        (7584, "x_{54,10}".to_owned()),
        (7314, r"h_1M\!P".to_owned()),
        (7529, r"h_1M\!P + h_0x_{54,10}".to_owned()),
        (7316, "d_0g^2".to_owned()),
        (7498, "d_0g^2 + h_0^2x_{54,10}".to_owned()),
        (7318, "d_0^2j + h_0^5x_{54,10}".to_owned()),
        (7369, "d_0^2j".to_owned()),
        (7797, "h_1x_{54,24}".to_owned()),
        (7941, "h_1x_{54,24} + h_0^2x_{55,23}".to_owned()),

        // Stems 56 and 57
        (8367, "gt".to_owned()),
        // Multiplication by g sends 8447 to e_0Md_0 in stem 76.
        (8447, "x_{56,10}".to_owned()),
        (8283, "h_0^3x_{56,10}".to_owned()),
        (8298, r"d_0\Delta h_1e_0".to_owned()),
        (8958, "h_5j".to_owned()),
        (9190, "h_5j + h_0Q_2".to_owned()),
        (8901, "h_0h_5j".to_owned()),
        (9080, "h_0^2Q_2".to_owned()),

        // Stem 60. Products through stem 60 allow adding 10675 to 10946.
        (10675, "h_0^2Mh_4".to_owned()),
        (10946, "[B_4]".to_owned()),
        (10650, "h_1Md_0".to_owned()),
        (10851, "h_0^2[B_4]".to_owned()),
        (10754, "g^3".to_owned()),
        (10780, "h_0^3[B_4]".to_owned()),
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
    // The checked AHSS transfers take precedence over matching Curtis words.
    let mut stable_translations = std::collections::BTreeMap::from([
        (String::new(), "1".to_owned()),
        ("1 6 2 3 3".to_owned(), id_to_stable_name[&159].clone()),
        ("1 6 2 3 4 4 1 1 1".to_owned(), id_to_stable_name[&484].clone()),
        ("1 6 2 3 4 4 1 1 2 4 1 1 1".to_owned(), id_to_stable_name[&1301].clone()),
        // In stem 37 the AHSS h_0-product of 7 12 4 5 3 3 3 is the sum
        // of these two words. Their transfers add to h_0^3x.
        ("4 5 5 3 6 6 5 3".to_owned(), id_to_stable_name[&2217].clone()),
        ("5 6 3 3 6 6 5 3".to_owned(), "e_0g".to_owned()),
        ("1 6 2 3 4 4 1 1 2 4 1 1 2 4 1 1 1".to_owned(), id_to_stable_name[&2579].clone()),
        ("2 17 7 7 7".to_owned(), id_to_stable_name[&2839].clone()),
        ("1 7 13 5 7 7".to_owned(), id_to_stable_name[&2812].clone()),
        // The AHSS h_1-product of 13 11 7[13] transfers to 4165.
        ("7 17 7 7 7".to_owned(), id_to_stable_name[&4165].clone()),
        ("1 8 3 5 9 7 7 7".to_owned(), id_to_stable_name[&4671].clone()),
        ("1 6 2 3 4 4 1 1 2 4 1 1 2 4 1 1 2 4 1 1 1".to_owned(), id_to_stable_name[&4656].clone()),
        ("8 5 5 9 7 7 7".to_owned(), "h_0^2e_0h_5".to_owned()),
        ("1 4 5 3 5 9 7 7 7".to_owned(), id_to_stable_name[&4965].clone()),
        ("1 8 3 5 9 3 5 7 7".to_owned(), id_to_stable_name[&4991].clone()),
        ("1 18 2 4 3 3 3 6 6 5 3".to_owned(), id_to_stable_name[&7314].clone()),
        ("13 2 2 4 7 3 3 6 6 5 3".to_owned(), "h_0x_{54,10}".to_owned()),
        ("11 6 2 2 4 3 3 3 6 6 5 3".to_owned(), "h_0^2x_{54,10}".to_owned()),
        ("1 6 2 3 4 4 1 1 2 4 1 1 2 4 1 1 2 4 1 1 2 4 1 1 1".to_owned(), id_to_stable_name[&7797].clone()),
        ("3 13 11 12 4 5 3 3 3".to_owned(), id_to_stable_name[&8901].clone()),
        ("2 25 5 5 3 6 6 5 3".to_owned(), id_to_stable_name[&10675].clone()),
        ("1 6 2 3 5 10 11 3 5 7 7".to_owned(), id_to_stable_name[&10650].clone()),
    ]);
    for generator in STABLE_MODEL.gens() {
        let tag = name_get_tag(&generator.name);
        if tag.is_empty() {
            continue;
        }
        if let Some(name) = MODEL.try_index(&generated_by_name(tag))
            .and_then(|index| id_to_stable_name.get(&index))
        {
            stable_translations.entry(tag.to_owned()).or_insert_with(|| name.clone());
        }
    }
    write_tag_translation(&translations, &stable_translations)
        .expect("Could not export website tag translations");
    println!("Exported {} EHP and {} AHSS tag translations to site/src/tag_translation.ts",
        translations.len(), stable_translations.len());
}
